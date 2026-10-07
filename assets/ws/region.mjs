// One region's WebSocket endpoint and update cycle. The legacy single-region
// process (`run.mjs ws --server en`) and the single all-regions process build the
// same object; the only difference is how many of them share one scheduler. The
// message shapes on the socket are the ones the backend's asset_watcher.rs reads:
// status, update_available, download_progress, download_complete, prune_complete,
// unpack_progress, update_complete, error, resource_list.

import { existsSync } from "node:fs";
import { readdir, stat } from "node:fs/promises";
import { join } from "node:path";
import chalk from "chalk";
import { WebSocketServer } from "ws";

const ts = () => `[${new Date().toLocaleTimeString()}]`;

/** Logger that prefixes the region tag in multi-region mode and nothing in legacy mode. */
export function makeLogger(tag) {
	const pre = tag ? `${tag} ` : "";
	return {
		info: (msg, color = chalk.dim) => console.log(color(`${ts()} ${pre}${msg}`)),
		warn: (msg) => console.warn(chalk.yellow(`${ts()} ${pre}${msg}`)),
	};
}

export const silentLogger = { info: () => {}, warn: () => {} };

/**
 * @param {object} p
 * @param {{serverKey: string, savedir: string, outputDir: string, threads: number,
 *   profile: string, port: number, intervalMin: number, startDelayMin: number}} p.config
 *   savedir/outputDir already carry the region.
 * @param {object} p.deps the side-effecting helpers from run.mjs (or fakes in tests)
 * @param {ReturnType<import("./scheduler.mjs").createScheduler>} p.scheduler
 * @param {object} p.settings
 * @param {number} p.settings.maxBackoffMs
 * @param {boolean} [p.settings.alignToClock]
 * @param {boolean} [p.settings.allowPartialDownload]
 * @param {number} [p.settings.maxUnpackJobs] 0 = no cap
 * @param {object|null} [p.settings.priority] see lowPriorityCommand
 * @param {ReturnType<import("./scheduler.mjs").createResourceGuard>|null} [p.settings.guard]
 */
export function createRegionWatcher({ config, deps, scheduler, settings, log = makeLogger("") }) {
	const key = config.serverKey;
	const intervalMs = config.intervalMin * 60 * 1000;
	const startDelayMs = Number.isFinite(config.startDelayMin) ? config.startDelayMin * 60 * 1000 : 0;
	const { maxBackoffMs } = settings;

	/** Delay before the next attempt, after `failures` consecutive failures. */
	const backoffMs = (failures) => Math.min(intervalMs * 2 ** Math.max(0, failures - 1), maxBackoffMs);

	// The thread count the unpacker gets: the region's own, clamped by the global cap.
	const unpackJobs =
		settings.maxUnpackJobs > 0 ? Math.max(1, Math.min(config.threads, settings.maxUnpackJobs)) : config.threads;

	let currentState = "idle";
	let currentVersion = deps.readStoredVersion(config.savedir);
	let queueInfo = null;
	// Loaded from disk so a pm2 or OOM restart does not clear a backoff that the
	// restart itself is evidence for. Cleared by a run whose extract succeeds.
	const persisted = deps.readBackoffState(config.savedir);
	let consecutiveFailures = persisted.consecutiveFailures;
	// A stored wait further out than the cap can only come from a clock that moved.
	let nextAttemptAt = Math.min(persisted.nextAttemptAt, Date.now() + maxBackoffMs);
	let nextCheckAt = 0;
	if (consecutiveFailures > 0) {
		log.warn(
			`Resuming backoff from disk: ${consecutiveFailures} consecutive failure(s), next attempt ${
				nextAttemptAt > Date.now() ? `in ${Math.ceil((nextAttemptAt - Date.now()) / 60000)} minute(s)` : "now"
			}`,
		);
	}

	const clients = new Set();
	let wss = null;
	const timers = new Set();

	function broadcast(msg) {
		const data = JSON.stringify(msg);
		for (const ws of clients) {
			if (ws.readyState === ws.OPEN) ws.send(data);
		}
	}

	function sendTo(ws, msg) {
		if (ws.readyState === ws.OPEN) ws.send(JSON.stringify(msg));
	}

	// The legacy shape, plus `queue` ONLY while the state is "queued". The backend
	// reads `type` and nothing else from a status message.
	function statusMessage() {
		const msg = {
			type: "status",
			state: currentState,
			version: { current: currentVersion ?? null },
			nextCheckAt: nextCheckAt || null,
			backoff: { consecutiveFailures, nextAttemptAt: nextAttemptAt || null },
		};
		if (currentState === "queued" && queueInfo) msg.queue = queueInfo;
		return msg;
	}

	function setState(state) {
		currentState = state;
		if (state !== "queued") queueInfo = null;
		broadcast(statusMessage());
	}

	async function listResources(ws) {
		const dir = config.outputDir;
		if (!existsSync(dir)) {
			sendTo(ws, { type: "resource_list", files: [], totalSize: 0 });
			return;
		}
		const files = [];
		let totalSize = 0;
		try {
			const entries = await readdir(dir, { withFileTypes: true });
			for (const entry of entries) {
				const fullPath = join(dir, entry.name);
				const st = await stat(fullPath);
				if (entry.isDirectory()) {
					const { size: dirSize, fileCount } = await deps.dirStats(fullPath);
					files.push({
						name: entry.name,
						path: entry.name,
						size: dirSize,
						fileCount,
						modified: st.mtime.toISOString(),
						created: st.birthtime.toISOString(),
						type: "directory",
					});
					totalSize += dirSize;
				} else {
					files.push({
						name: entry.name,
						path: entry.name,
						size: st.size,
						modified: st.mtime.toISOString(),
						created: st.birthtime.toISOString(),
						type: "file",
					});
					totalSize += st.size;
				}
			}
		} catch (err) {
			sendTo(ws, { type: "error", message: `Failed to list resources: ${err.message}` });
			return;
		}
		sendTo(ws, { type: "resource_list", files, totalSize, totalSizeFormatted: deps.formatBytes(totalSize) });
	}

	// Download + unpack. `knownVer` is the version checkAndUpdate already fetched,
	// reused so a CDN blip at the END of hours of work cannot discard it. `manual`
	// marks a force_update: it ignores `nextAttemptAt` and its failures do not touch
	// the counter, but its success still clears the backoff. Runs only as a
	// scheduler job, so it never overlaps another region's download or extract.
	async function performUpdate(knownVer, { manual = false } = {}) {
		try {
			setState("downloading");
			log.info("Downloading assets...", chalk.blue);

			const dlStats = await deps.runDownload({
				serverKey: key,
				savedir: config.savedir,
				threads: config.threads,
				profile: config.profile,
				priority: settings.priority ?? null,
				onProgress: (p) => broadcast({ type: "download_progress", ...p }),
			});

			log.info(
				`Download complete: ${dlStats.downloaded} files, ${dlStats.failed} failed, ${deps.formatBytes(dlStats.totalBytes)}`,
				chalk.blue,
			);

			// A partial download must NOT reach the unpacker: its orphan sweep would
			// delete every good output the short run did not re-produce.
			if (dlStats.failed > 0 && !settings.allowPartialDownload) {
				throw new Error(
					`${dlStats.failed} bundle(s) failed to download; refusing to unpack an incomplete set (WS_ALLOW_PARTIAL_DOWNLOAD=1 to override)`,
				);
			}
			broadcast({
				type: "download_complete",
				downloaded: dlStats.downloaded,
				failed: dlStats.failed,
				totalBytes: dlStats.totalBytes,
				totalBytesFormatted: deps.formatBytes(dlStats.totalBytes),
			});

			// Prune orphans before unpack: stale .bin bundles sharing a TextAsset
			// m_Name with a current one clobber it via last-write-wins.
			try {
				const ver = await deps.fetchServerVersion(key);
				const hotList = await deps.fetchHotUpdateList(key, ver.resVersion);
				const { deleted, freedBytes } = deps.pruneOrphans(config.savedir, hotList);
				if (deleted > 0) log.info(`Pruned ${deleted} orphan file(s), freed ${deps.formatBytes(freedBytes)}`);
				broadcast({ type: "prune_complete", deleted, freedBytes });
			} catch (err) {
				log.warn(`Orphan prune skipped: ${err.message}`);
			}

			// Memory guard, second gate: the download may have run for hours and the
			// box may be in a different state than when the job started.
			if (settings.guard) {
				await settings.guard.waitFor({
					onWait: (c) => {
						const message = `Waiting for memory before extracting: ${c.availableMb} MiB available, need ${c.minFreeMb} MiB`;
						log.warn(message);
						broadcast({ type: "status", state: "downloading", message });
					},
				});
			}

			setState("unpacking");
			log.info(`Unpacking assets (-j ${unpackJobs})...`, chalk.blue);

			const upStats = await deps.runUnpack({
				inputDir: config.savedir,
				outputDir: config.outputDir,
				jobs: unpackJobs,
				priority: settings.priority ?? null,
				onProgress: (p) => broadcast({ type: "unpack_progress", ...p }),
				onNotice: (message) => broadcast({ type: "status", state: "unpacking", message }),
			});

			// The extract SUCCEEDED; nothing below may fall into the catch.
			consecutiveFailures = 0;
			nextAttemptAt = 0;
			deps.writeBackoffState(config.savedir, 0, 0);

			let serverVer = knownVer;
			try {
				if (!serverVer) serverVer = await deps.fetchServerVersion(key);
				deps.writeStoredVersion(config.savedir, serverVer.resVersion);
				currentVersion = serverVer.resVersion;
			} catch (err) {
				log.warn(
					`Extract succeeded but the version could not be recorded: ${err.message}. The next check will see a version mismatch and repeat the download.`,
				);
			}
			try {
				deps.touchExtractStamp(config.savedir, upStats.exported, upStats.onDisk);
			} catch (err) {
				log.warn(`Extract succeeded but the stamp could not be written: ${err.message}`);
			}

			currentState = "idle";
			log.info(
				`Update complete: v${currentVersion}, ${dlStats.downloaded} downloaded, ${upStats.exported} exported, ${upStats.onDisk} on disk`,
				chalk.green,
			);
			broadcast({
				type: "update_complete",
				version: currentVersion,
				downloaded: dlStats.downloaded,
				failed: dlStats.failed,
				exported: upStats.exported,
			});
			broadcast(statusMessage());
		} catch (err) {
			currentState = "idle";
			log.info(`Update failed: ${err.message}`, chalk.red);
			if (manual) {
				log.info(
					`Manual update, so the automatic backoff is unchanged (${consecutiveFailures} consecutive failure(s) on record)`,
				);
			} else {
				consecutiveFailures += 1;
				const wait = backoffMs(consecutiveFailures);
				nextAttemptAt = Date.now() + wait;
				deps.writeBackoffState(config.savedir, consecutiveFailures, nextAttemptAt);
				log.info(`Failure ${consecutiveFailures}; next attempt in ${Math.round(wait / 60000)} minute(s)`);
			}
			broadcast({ type: "error", message: `Update failed: ${err.message}`, consecutiveFailures, nextAttemptAt });
			broadcast(statusMessage());
		}
	}

	async function checkAndUpdate() {
		try {
			setState("checking");
			log.info("Checking for updates...");

			const serverVer = await deps.fetchServerVersion(key);
			const storedVer = deps.readStoredVersion(config.savedir);
			currentState = "idle";

			const needsReextract =
				deps.unpackerIsNewer(config.savedir) ||
				deps.outputMissingOrEmpty(config.outputDir) ||
				(await deps.outputLooksTruncated(config.savedir, config.outputDir));
			if (storedVer === serverVer.resVersion && !needsReextract) {
				log.info(`Up to date (${storedVer})`);
				broadcast(statusMessage());
				return;
			}
			if (needsReextract) {
				log.info("Re-extraction needed (assets current but output stale)", chalk.yellow);
			} else {
				log.info(`Update available: ${storedVer ?? "(none)"} → ${serverVer.resVersion}`, chalk.yellow);
			}
			broadcast({
				type: "update_available",
				currentVersion: storedVer ?? null,
				newVersion: serverVer.resVersion,
				clientVersion: serverVer.clientVersion,
			});
			await performUpdate(serverVer);
		} catch (err) {
			currentState = "idle";
			log.info(`Version check failed: ${err.message}`, chalk.red);
			broadcast({ type: "error", message: `Version check failed: ${err.message}` });
			broadcast(statusMessage());
		}
	}

	// Hooks that keep this region's socket honest while its job waits on others.
	const jobHooks = {
		onQueued: (position, behind) => {
			currentState = "queued";
			queueInfo = { position, behind };
			broadcast(statusMessage());
			log.info(`Queued at position ${position}${behind ? ` behind ${behind}` : ""}`);
		},
		onLockWait: (holder) => {
			currentState = "queued";
			queueInfo = { position: 1, behind: holder.owner ?? `pid ${holder.pid}` };
			broadcast(statusMessage());
			log.info(`Waiting for the watcher lock held by ${holder.owner ?? "?"} (pid ${holder.pid ?? "?"})`);
		},
		onDefer: (c) => {
			currentState = "queued";
			const message = `Deferred: ${c.availableMb} MiB available, need ${c.minFreeMb} MiB (WS_MIN_FREE_MB)`;
			queueInfo = { position: 0, behind: null, reason: "memory" };
			broadcast({ ...statusMessage(), message });
			log.warn(message);
		},
		onSkip: (c) => {
			currentState = "idle";
			const message = `Skipped this check: memory stayed below the threshold (${c.availableMb} MiB available, need ${c.minFreeMb} MiB)`;
			broadcast({ type: "error", message });
			broadcast(statusMessage());
			log.warn(message);
		},
		onError: (err) => {
			currentState = "idle";
			log.warn(`Job threw: ${err?.message ?? err}`);
			broadcast(statusMessage());
		},
	};

	/** One scheduled tick: the backoff guard runs here so a backed-off region never takes a queue slot. */
	function tick() {
		if (scheduler.pending(key)) {
			log.info("Previous check still queued or running; skipping this tick");
			return null;
		}
		if (Date.now() < nextAttemptAt) {
			const mins = Math.ceil((nextAttemptAt - Date.now()) / 60000);
			log.info(`Backing off after ${consecutiveFailures} failed update(s); next attempt in ${mins} minute(s)`);
			setState("backing_off");
			return null;
		}
		return scheduler.submit(key, "scheduled", checkAndUpdate, jobHooks);
	}

	/** A force_update from a client. Same refusal text as the legacy watcher. */
	function forceUpdate(ws) {
		if (scheduler.pending(key)) {
			if (ws) sendTo(ws, { type: "error", message: "Update already in progress" });
			return null;
		}
		// Deliberately not gated on `nextAttemptAt`: it is the hatch out of a backoff.
		return scheduler.submit(key, "manual", () => performUpdate(undefined, { manual: true }), jobHooks);
	}

	function onConnection(ws) {
		log.info(`Client connected (${clients.size + 1} total)`);
		clients.add(ws);
		sendTo(ws, statusMessage());
		ws.on("message", async (raw) => {
			let msg;
			try {
				msg = JSON.parse(raw.toString());
			} catch {
				sendTo(ws, { type: "error", message: "Invalid JSON" });
				return;
			}
			switch (msg.type) {
				case "force_update":
					forceUpdate(ws);
					break;
				case "list_resources":
					await listResources(ws);
					break;
				default:
					sendTo(ws, { type: "error", message: `Unknown command: ${msg.type}` });
			}
		});
		ws.on("close", () => {
			clients.delete(ws);
			log.info(`Client disconnected (${clients.size} remaining)`);
		});
		ws.on("error", () => clients.delete(ws));
	}

	function setTimer(fn, ms) {
		const t = setTimeout(() => {
			timers.delete(t);
			fn();
		}, ms);
		timers.add(t);
		return t;
	}

	/**
	 * Arm the check schedule. Aligned to the WALL clock by default (WS_ALIGN=0 is
	 * the old relative stagger): checks land at `startDelayMin` past each interval
	 * boundary whenever the process last restarted, so EN :00/:30, JP :07/:37, CN
	 * :15/:45, KR :22/:52 exactly as the four processes had them. The TRADE: no
	 * immediate check at startup, so a deploy can wait one interval before a new
	 * resVersion is noticed; force_update says "go now". Ruled out: checking
	 * immediately and then aligning, which collides regions on every restart.
	 */
	async function startSchedule() {
		if (!settings.alignToClock) {
			if (startDelayMs > 0) {
				log.info(`Staggered start: waiting ${config.startDelayMin} minute(s) before the first check`);
				await new Promise((resolve) => setTimer(resolve, startDelayMs));
			}
			nextCheckAt = Date.now() + intervalMs;
			const iv = setInterval(() => {
				nextCheckAt = Date.now() + intervalMs;
				tick();
			}, intervalMs);
			timers.add(iv);
			tick();
			return;
		}
		const offsetMs = ((startDelayMs % intervalMs) + intervalMs) % intervalMs;
		const scheduleNext = () => {
			const now = Date.now();
			// floor(..) + 1, not ceil(..): a slot landing exactly on `now` must
			// schedule the NEXT one, or the zero-delay timer re-enters itself forever.
			nextCheckAt = (Math.floor((now - offsetMs) / intervalMs) + 1) * intervalMs + offsetMs;
			setTimer(() => {
				scheduleNext();
				tick();
			}, nextCheckAt - now);
		};
		scheduleNext();
		log.info(
			`Checks aligned to the clock: every ${config.intervalMin} min at offset ${offsetMs / 60000} min; first check at ${new Date(nextCheckAt).toLocaleTimeString()}`,
		);
	}

	return {
		key,
		config,
		statusMessage,
		tick,
		forceUpdate,
		checkAndUpdate,
		/** Open the WebSocket port. Resolves once it is listening. */
		listen() {
			return new Promise((resolve, reject) => {
				wss = new WebSocketServer({ port: config.port });
				wss.on("connection", onConnection);
				wss.once("listening", () => resolve(wss));
				wss.once("error", reject);
			});
		},
		startSchedule,
		get wss() {
			return wss;
		},
		get state() {
			return currentState;
		},
		async close() {
			for (const t of timers) {
				clearTimeout(t);
				clearInterval(t);
			}
			timers.clear();
			for (const ws of clients) ws.terminate();
			await new Promise((r) => (wss ? wss.close(() => r()) : r()));
		},
	};
}
