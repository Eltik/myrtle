// One queue, one lock, one resource guard for every region the watcher serves.
//
// The VPS (3 cores, 10 GiB, one SATA disk under a ~113 GB tree) has logged WRITE
// DMA timeouts, writeback soft lockups and RCU stalls when two extracts overlapped.
// Four processes phased apart on the clock kept the SCHEDULED checks apart, but a
// manual force_update bypassed the phase and a long extract could run into the next
// region's slot. Everything that downloads or extracts now goes through `submit`,
// which runs exactly one job at a time, in FIFO order, under a lockfile that a
// legacy single-region process honours too.

import { execFileSync } from "node:child_process";
import {
	closeSync,
	existsSync,
	openSync,
	readFileSync,
	statSync,
	unlinkSync,
	utimesSync,
	writeSync,
} from "node:fs";
import { freemem } from "node:os";

const noop = () => {};

/**
 * Parse an env value that must be a finite, non-negative number. A MISSING or
 * blank value takes the default; only an explicit number, including "0", is
 * honoured. `Number("")` is 0 and finite, which is the trap this guards.
 */
export function envNumber(env, name, fallback) {
	const raw = env[name];
	if (raw === undefined || String(raw).trim() === "") return fallback;
	const n = Number(raw);
	return Number.isFinite(n) && n >= 0 ? n : fallback;
}

// ─── Cross-process lock ─────────────────────────────────────────────────────

function pidAlive(pid) {
	if (!Number.isInteger(pid) || pid <= 0) return false;
	try {
		process.kill(pid, 0);
		return true;
	} catch (err) {
		// EPERM: the pid exists and belongs to someone else, so it is alive.
		return err.code === "EPERM";
	}
}

/**
 * An advisory lockfile created with O_EXCL. The holder rewrites its mtime every
 * `heartbeatMs`; a file whose pid is dead, or whose mtime is older than `staleMs`,
 * is a crashed holder and is taken over. Returns null when `path` is falsy, which
 * is the WS_LOCK=0 kill switch.
 */
export function createFileLock(
	path,
	{ staleMs = 10 * 60_000, heartbeatMs = 30_000, pollMs = 5_000 } = {},
) {
	if (!path) return null;
	let held = false;

	const readHolder = () => {
		try {
			const info = JSON.parse(readFileSync(path, "utf8"));
			return { ...info, mtimeMs: statSync(path).mtimeMs };
		} catch {
			// Unreadable or half-written: treat it as present but anonymous so the
			// staleness test below decides by age alone.
			try {
				return { pid: null, mtimeMs: statSync(path).mtimeMs };
			} catch {
				return null;
			}
		}
	};

	const tryCreate = (owner) => {
		try {
			const fd = openSync(path, "wx");
			writeSync(fd, JSON.stringify({ pid: process.pid, owner, since: Date.now() }));
			closeSync(fd);
			return true;
		} catch (err) {
			if (err.code === "EEXIST") return false;
			throw err;
		}
	};

	const releaseSync = () => {
		if (!held) return;
		held = false;
		try {
			const info = JSON.parse(readFileSync(path, "utf8"));
			if (info.pid === process.pid) unlinkSync(path);
		} catch {}
	};
	process.on("exit", releaseSync);

	return {
		path,
		holder: () => (existsSync(path) ? readHolder() : null),
		/**
		 * Resolves with a release function once the lock is ours. `onWait` is called
		 * once, with the holder, if the lock was taken by someone else.
		 */
		async acquire(owner, { onWait = noop } = {}) {
			let announced = false;
			for (;;) {
				if (tryCreate(owner)) break;
				const h = readHolder();
				if (h) {
					const stale =
						h.pid === process.pid || // left behind by this pid, never by a live job of ours
						(h.pid != null && !pidAlive(h.pid)) ||
						Date.now() - h.mtimeMs > staleMs;
					if (stale) {
						try {
							unlinkSync(path);
						} catch {}
						continue;
					}
					if (!announced) {
						announced = true;
						onWait(h);
					}
				}
				await new Promise((r) => setTimeout(r, pollMs));
			}
			held = true;
			const beat = setInterval(() => {
				try {
					const now = new Date();
					utimesSync(path, now, now);
				} catch {}
			}, heartbeatMs);
			beat.unref?.();
			return () => {
				clearInterval(beat);
				releaseSync();
			};
		},
	};
}

// ─── Resource guard ─────────────────────────────────────────────────────────

/**
 * MiB the kernel says a new workload can take without swapping. MemAvailable, NOT
 * MemFree: on a box serving a 113 GB tree the page cache holds most of RAM, so
 * MemFree is small all the time and a MemFree threshold would never let an extract
 * start. Falls back to os.freemem() where /proc/meminfo is absent.
 */
export function readAvailableMb() {
	try {
		const m = readFileSync("/proc/meminfo", "utf8").match(/^MemAvailable:\s+(\d+)\s+kB/m);
		if (m) return Number(m[1]) / 1024;
	} catch {}
	return freemem() / (1024 * 1024);
}

/**
 * `minFreeMb` 0 disables the guard. `waitMs` is how long `waitFor` holds an extract
 * for memory to come back before giving up.
 */
export function createResourceGuard({
	minFreeMb = 0,
	readMb = readAvailableMb,
	waitMs = 10 * 60_000,
	pollMs = 30_000,
} = {}) {
	const check = () => {
		if (!minFreeMb) return { ok: true, availableMb: null, minFreeMb };
		const availableMb = Math.floor(readMb());
		return { ok: availableMb >= minFreeMb, availableMb, minFreeMb };
	};
	return {
		minFreeMb,
		check,
		/** Waits up to `waitMs` for the threshold; throws if it never clears. */
		async waitFor({ onWait = noop } = {}) {
			let c = check();
			if (c.ok) return c;
			onWait(c);
			const deadline = Date.now() + waitMs;
			while (!c.ok && Date.now() < deadline) {
				await new Promise((r) => setTimeout(r, Math.min(pollMs, Math.max(1, deadline - Date.now()))));
				c = check();
			}
			if (!c.ok) {
				throw new Error(
					`only ${c.availableMb} MiB available, need ${c.minFreeMb} MiB (WS_MIN_FREE_MB); extract deferred`,
				);
			}
			return c;
		},
	};
}

// ─── Low-priority spawn ─────────────────────────────────────────────────────

const haveCache = new Map();
function have(cmd) {
	if (!haveCache.has(cmd)) {
		let ok = false;
		if (process.platform !== "win32") {
			try {
				execFileSync("sh", ["-c", `command -v ${cmd}`], { stdio: "ignore" });
				ok = true;
			} catch {}
		}
		haveCache.set(cmd, ok);
	}
	return haveCache.get(cmd);
}

/**
 * Wrap `bin args` in `ionice -c <class> -n <level> nice -n <nice>` where those
 * exist. Both exec the next program, so the spawned pid IS the unpacker and a
 * SIGKILL or an OOM signal still reads straight off the child. `nice` is applied
 * through the wrapper rather than os.setPriority after spawn because Linux niceness
 * is per thread: rayon threads created before a late setpriority would keep the
 * old value. `priority` null, or nice 0 and ioniceClass 0, returns the command
 * unchanged.
 */
export function lowPriorityCommand(bin, args, priority, hasCmd = have) {
	if (!priority) return [bin, args];
	let cmd = bin;
	let argv = args;
	if (priority.nice > 0 && hasCmd("nice")) {
		argv = ["-n", String(priority.nice), cmd, ...argv];
		cmd = "nice";
	}
	if (priority.ioniceClass > 0 && hasCmd("ionice")) {
		const io = ["-c", String(priority.ioniceClass)];
		if (priority.ioniceClass === 2) io.push("-n", String(priority.ioniceLevel ?? 7));
		argv = [...io, cmd, ...argv];
		cmd = "ionice";
	}
	return [cmd, argv];
}

// ─── Scheduler ──────────────────────────────────────────────────────────────

/**
 * A FIFO queue drained by ONE worker. A region has at most one job pending
 * (queued or running); `submit` refuses a second. Before a job starts the guard is
 * consulted: below the threshold the job is DEFERRED, put to the back of the queue
 * with a retry time, and dropped (its `onSkip` called) once it has waited longer
 * than `maxDeferMs`. A dropped job does not touch the region's backoff; the next
 * scheduled tick submits it again.
 *
 * @param {object} opts
 * @param {ReturnType<typeof createFileLock>} [opts.fileLock]
 * @param {ReturnType<typeof createResourceGuard>} [opts.guard]
 */
export function createScheduler({
	fileLock = null,
	guard = null,
	deferRetryMs = 2 * 60_000,
	maxDeferMs = 30 * 60_000,
	onEvent = noop,
} = {}) {
	const queue = [];
	let running = null;
	let pumping = false;
	let wakeTimer = null;
	const history = [];

	const pending = (region) =>
		running?.region === region || queue.some((j) => j.region === region);

	const snapshot = () => ({
		running: running ? { region: running.region, kind: running.kind } : null,
		queued: queue.map((j) => ({ region: j.region, kind: j.kind })),
	});

	async function runJob(job) {
		running = job;
		history.push({ region: job.region, kind: job.kind });
		onEvent("start", job, snapshot());
		let release = null;
		try {
			if (fileLock) {
				release = await fileLock.acquire(`${job.region}:${job.kind}`, {
					onWait: (holder) => job.hooks.onLockWait?.(holder),
				});
			}
			job.hooks.onStart?.();
			await job.run();
		} catch (err) {
			// A job reports its own failures; anything that escapes is a bug in the
			// job, and must not stop the queue from draining.
			job.hooks.onError?.(err);
		} finally {
			release?.();
			running = null;
			onEvent("end", job, snapshot());
			job.resolve();
		}
	}

	async function pump() {
		if (pumping) return;
		pumping = true;
		try {
			while (queue.length > 0) {
				const now = Date.now();
				const idx = queue.findIndex((j) => j.notBefore <= now);
				if (idx === -1) {
					const next = Math.min(...queue.map((j) => j.notBefore));
					clearTimeout(wakeTimer);
					wakeTimer = setTimeout(() => void pump(), Math.max(1, next - now));
					return;
				}
				const [job] = queue.splice(idx, 1);
				if (guard) {
					const c = guard.check();
					if (!c.ok) {
						job.deferredSince ??= now;
						if (now - job.deferredSince >= maxDeferMs) {
							onEvent("skip", job, snapshot());
							job.hooks.onSkip?.(c);
							job.resolve();
							continue;
						}
						job.notBefore = now + deferRetryMs;
						queue.push(job);
						onEvent("defer", job, snapshot());
						job.hooks.onDefer?.(c);
						continue;
					}
				}
				for (const j of queue) j.hooks.onQueued?.(queue.indexOf(j) + 1, job.region);
				await runJob(job);
			}
		} finally {
			pumping = false;
		}
	}

	return {
		pending,
		busy: () => running !== null,
		snapshot,
		history,
		/**
		 * Queue `run` for `region`. Returns { accepted, position, done }: position 0
		 * means the job starts now, n > 0 that n jobs are ahead of it. `done`
		 * resolves when the job has run, been skipped, or was refused.
		 */
		submit(region, kind, run, hooks = {}) {
			if (pending(region)) {
				return { accepted: false, position: -1, done: Promise.resolve() };
			}
			let resolve;
			const done = new Promise((r) => {
				resolve = r;
			});
			const job = { region, kind, run, hooks, notBefore: 0, deferredSince: null, resolve };
			queue.push(job);
			const position = running || pumping ? queue.length : 0;
			onEvent("submit", job, snapshot());
			if (position > 0) hooks.onQueued?.(position, running?.region ?? null);
			void pump();
			return { accepted: true, position, done };
		},
		/** Resolves once the queue is empty and nothing runs. */
		async idle() {
			while (running || queue.length > 0 || pumping) {
				await new Promise((r) => setTimeout(r, 5));
			}
		},
		stop() {
			clearTimeout(wakeTimer);
		},
	};
}
