// Storage safeguards for the asset watchers (`run.mjs ws`).
//
// Every extract rewrites its region's output tree at full size, and only the
// object storage sync AFTER it turns the served files back into 0-byte stubs
// (README, "Freeing local disk"). So an extract needs free space for the whole
// tree while it runs, and two of them at once need room for both. The watchers
// were only kept apart by a clock offset, which holds while an extract is
// shorter than the offset. On 2026-11-11 CN (37 min) and EN (57 min) overlapped
// on a disk with 21 GB free, filled it, and took PostgreSQL down with it.
//
// Three guards, each usable without the others:
//
// - an extract lock, so one region at a time downloads, extracts and syncs,
//   however long each takes;
// - a free-space budget, checked before the unpacker starts: the bytes the
//   extract is about to put back on disk must fit with WS_MIN_FREE_GB to spare;
// - a watchdog that stops a running download or extract when something else
//   eats that margin anyway.
//
// Every guard FAILS OPEN: a lock file that cannot be created, a manifest that
// cannot be read or a statfs that throws leaves the watcher doing what it did
// before these existed. They are here to prevent an outage, not to become a new
// way for assets to stop updating.

import {
	closeSync,
	existsSync,
	mkdirSync,
	openSync,
	readFileSync,
	statSync,
	statfsSync,
	unlinkSync,
	utimesSync,
	writeSync,
} from "node:fs";
import { readFile, stat } from "node:fs/promises";
import { basename, dirname, join, resolve as resolvePath } from "node:path";

/** `asset-sync`'s manifest, one per region (backend/crates/storage, `MANIFEST_NAME`). */
const SYNC_MANIFEST = ".asset-sync.json";

/** Shared by every watcher writing under one output root. */
export const EXTRACT_LOCK_NAME = ".extract.lock";

/** How often the holder touches the lock, and how old a touch may get. */
export const LOCK_HEARTBEAT_MS = 60 * 1000;
export const LOCK_STALE_MS = 10 * 60 * 1000;

export const DEFAULT_MIN_FREE_GB = 5;

const GIB = 2 ** 30;

// ─── Free space ─────────────────────────────────────────────────────────────

/**
 * Bytes an unprivileged process can still write on the filesystem holding
 * `path`, or `null` when that cannot be read (the caller then skips the check).
 * `path` need not exist yet: the nearest existing parent is on the same disk.
 *
 * @param {string} path
 * @returns {number | null}
 */
export function freeBytes(path) {
	try {
		let dir = resolvePath(path);
		while (!existsSync(dir)) {
			const parent = dirname(dir);
			if (parent === dir) return null;
			dir = parent;
		}
		const fs = statfsSync(dir);
		const free = Number(fs.bavail) * Number(fs.bsize);
		return Number.isFinite(free) && free >= 0 ? free : null;
	} catch {
		return null;
	}
}

/**
 * Parse WS_MIN_FREE_GB: the free space an extract must leave on the disk.
 *
 * Read like the watcher's other numeric settings: unset or blank takes the
 * default, an unusable value (NaN, negative) takes the default and is reported
 * as `invalid` so the caller can say so, and only an explicit `0` turns the
 * budget and the watchdog off.
 *
 * @param {string | undefined} raw
 * @returns {{ gb: number, bytes: number, invalid: boolean }}
 */
export function parseMinFreeGb(raw) {
	const blank = raw === undefined || String(raw).trim() === "";
	const parsed = blank ? DEFAULT_MIN_FREE_GB : Number(raw);
	const usable = Number.isFinite(parsed) && parsed >= 0;
	const gb = usable ? parsed : DEFAULT_MIN_FREE_GB;
	return { gb, bytes: Math.round(gb * GIB), invalid: !blank && !usable };
}

/**
 * Bytes the next extract will put back on disk for one region: the recorded
 * size of every file that is a 0-byte stub right now.
 *
 * The manifest alone is not enough. Its `evicted` flag is only updated by a
 * sync, so after an extract that was stopped halfway it still flags files the
 * unpacker already rewrote in full. Counting those twice would refuse the retry
 * for space that is in fact already spent, so each flagged file is stat'ed and
 * counted only while it is still empty.
 *
 * An upper bound: the unpacker leaves some stubs alone (video, music, the
 * `tex/` pool), and those are counted too.
 *
 * `known: false` means there is nothing to estimate from (no manifest, so no
 * eviction, so an extract overwrites files in place and adds little).
 *
 * @param {string} outputDir the region's tree, `<output>/<server>`
 * @param {{ concurrency?: number }} [opts]
 * @returns {Promise<{ known: boolean, bytes: number, stubs: number }>}
 */
export async function pendingRewriteBytes(outputDir, { concurrency = 128 } = {}) {
	const unknown = { known: false, bytes: 0, stubs: 0 };
	let manifest;
	try {
		manifest = JSON.parse(await readFile(join(outputDir, SYNC_MANIFEST), "utf-8"));
	} catch {
		return unknown;
	}
	const objects = manifest?.objects;
	if (!objects || typeof objects !== "object") return unknown;

	// Key = `[prefix/]<server>/<rel>` (backend/crates/storage/src/key.rs).
	const prefix = typeof manifest.prefix === "string" ? manifest.prefix : "";
	const head = `${prefix ? `${prefix}/` : ""}${basename(outputDir)}/`;

	const flagged = [];
	for (const [key, record] of Object.entries(objects)) {
		if (record?.evicted !== true || !key.startsWith(head)) continue;
		const size = Number(record.size);
		if (Number.isFinite(size) && size > 0) flagged.push([key.slice(head.length), size]);
	}

	let bytes = 0;
	let stubs = 0;
	for (let i = 0; i < flagged.length; i += concurrency) {
		const batch = await Promise.all(
			flagged.slice(i, i + concurrency).map(async ([rel, size]) => {
				try {
					return (await stat(join(outputDir, rel))).size === 0 ? size : 0;
				} catch {
					// Gone: the orphan sweep removed it, and nothing brings it back.
					return 0;
				}
			}),
		);
		for (const size of batch) {
			if (size > 0) {
				bytes += size;
				stubs += 1;
			}
		}
	}
	return { known: true, bytes, stubs };
}

/**
 * Whether an extract that adds `rewriteBytes` fits with `minFreeBytes` to spare.
 * An unreadable free-space figure passes: see "FAILS OPEN" above.
 *
 * @param {{ free: number | null, rewriteBytes?: number, minFreeBytes: number }} budget
 * @returns {{ ok: boolean, shortBy: number }}
 */
export function checkDiskBudget({ free, rewriteBytes = 0, minFreeBytes }) {
	if (free === null || !(minFreeBytes > 0)) return { ok: true, shortBy: 0 };
	const shortBy = rewriteBytes + minFreeBytes - free;
	return shortBy > 0 ? { ok: false, shortBy } : { ok: true, shortBy: 0 };
}

/**
 * Poll free space while a download or extract runs, and call `onLow` on every
 * poll that finds it under `floorBytes`. Every poll, not the first only: the
 * caller stops whatever child is running at that moment, and a dip between two
 * phases (nothing running, nothing to stop) must not disarm it for the next
 * one. Returns the function that stops the polling.
 *
 * @param {{ path: string, floorBytes: number, onLow: (free: number) => void, intervalMs?: number, free?: (path: string) => number | null }} opts
 * @returns {() => void}
 */
export function startDiskWatchdog({ path, floorBytes, onLow, intervalMs = 10 * 1000, free = freeBytes }) {
	if (!(floorBytes > 0)) return () => {};
	const timer = setInterval(() => {
		const now = free(path);
		if (now !== null && now < floorBytes) onLow(now);
	}, intervalMs);
	timer.unref?.();
	return () => clearInterval(timer);
}

// ─── Extract lock ───────────────────────────────────────────────────────────

/** Whether `pid` names a live process. EPERM means it exists and is not ours. */
function pidAlive(pid) {
	try {
		process.kill(pid, 0);
		return true;
	} catch (err) {
		return err?.code === "EPERM";
	}
}

/**
 * Read the lock, or `null` when there is none. A lock whose content cannot be
 * parsed still comes back, with its fields nulled: it may be one another
 * watcher is in the middle of writing.
 *
 * @param {string} lockPath
 * @returns {{ pid: number | null, server: string | null, token: string | null, at: string | null, mtimeMs: number } | null}
 */
export function readExtractLock(lockPath) {
	let mtimeMs;
	try {
		mtimeMs = statSync(lockPath).mtimeMs;
	} catch {
		return null;
	}
	let raw = {};
	try {
		raw = JSON.parse(readFileSync(lockPath, "utf-8")) ?? {};
	} catch {}
	const str = (v) => (typeof v === "string" ? v : null);
	return {
		pid: Number.isInteger(raw.pid) && raw.pid > 0 ? raw.pid : null,
		server: str(raw.server),
		token: str(raw.token),
		at: str(raw.at),
		mtimeMs,
	};
}

/**
 * A lock is stale when its holder is gone (killed, OOM-killed, pm2-restarted:
 * none of those run the release) or has stopped touching it. The heartbeat is
 * what covers a reused pid after a reboot, where the "holder" is alive and is
 * some other program entirely.
 *
 * @param {NonNullable<ReturnType<typeof readExtractLock>>} holder
 * @param {number} [now]
 * @param {(pid: number) => boolean} [alive]
 */
export function lockIsStale(holder, now = Date.now(), alive = pidAlive) {
	if (now - holder.mtimeMs > LOCK_STALE_MS) return true;
	// Unparseable and fresh: being written right now. Not ours to take.
	if (holder.pid === null) return false;
	// Our own pid cannot be holding it: acquire is only called when we are not.
	return holder.pid === process.pid || !alive(holder.pid);
}

/** Locks this process holds, removed on exit so a restart leaves nothing behind. */
const heldLocks = new Map();
let exitHookInstalled = false;

function releaseHeldLock(lockPath) {
	const held = heldLocks.get(lockPath);
	if (!held) return;
	heldLocks.delete(lockPath);
	clearInterval(held.heartbeat);
	try {
		// Only remove a lock that is still ours: one that went stale under us
		// (a suspended machine) now belongs to whoever took it.
		if (readExtractLock(lockPath)?.token === held.token) unlinkSync(lockPath);
	} catch {}
}

/**
 * Take the box-wide extract lock, or report who holds it.
 *
 * The lock is a file created with `wx`, which is atomic, holding the holder's
 * pid, region and a token. Taking over a STALE lock is not atomic (two waiters
 * can both remove it and both create one), so a watcher that took one over
 * waits a moment and reads it back: only the one whose token is still in the
 * file holds it.
 *
 * Throws only when the lock file cannot be created or read for a reason other
 * than "someone has it" (a read-only or full disk); the caller carries on
 * without a lock.
 *
 * @param {string} lockPath
 * @param {{ server?: string, settleMs?: number, alive?: (pid: number) => boolean }} [opts]
 * @returns {Promise<{ ok: true, release: () => void } | { ok: false, holder: ReturnType<typeof readExtractLock> }>}
 */
export async function acquireExtractLock(lockPath, { server = null, settleMs = 150, alive = pidAlive } = {}) {
	const token = `${process.pid}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
	const body = JSON.stringify({ pid: process.pid, server, token, at: new Date().toISOString() });

	const create = () => {
		let fd;
		try {
			fd = openSync(lockPath, "wx");
		} catch (err) {
			if (err?.code === "EEXIST") return false;
			throw err;
		}
		try {
			writeSync(fd, body);
		} finally {
			closeSync(fd);
		}
		return true;
	};

	// A second caller in the SAME process (a scheduled check racing a
	// force_update) must not read its own pid as a stale holder and take the
	// lock from itself.
	if (heldLocks.has(lockPath)) return { ok: false, holder: readExtractLock(lockPath) };

	mkdirSync(dirname(lockPath), { recursive: true });
	if (!create()) {
		const holder = readExtractLock(lockPath);
		if (holder && !lockIsStale(holder, Date.now(), alive)) return { ok: false, holder };
		try {
			unlinkSync(lockPath);
		} catch (err) {
			if (err?.code !== "ENOENT") throw err;
		}
		if (!create()) return { ok: false, holder: readExtractLock(lockPath) };
		await new Promise((resolve) => setTimeout(resolve, settleMs + Math.random() * settleMs));
		const now = readExtractLock(lockPath);
		if (now?.token !== token) return { ok: false, holder: now };
	}

	const heartbeat = setInterval(() => {
		try {
			const at = new Date();
			utimesSync(lockPath, at, at);
		} catch {}
	}, LOCK_HEARTBEAT_MS);
	heartbeat.unref?.();
	heldLocks.set(lockPath, { token, heartbeat });
	if (!exitHookInstalled) {
		exitHookInstalled = true;
		process.on("exit", () => {
			for (const path of [...heldLocks.keys()]) releaseHeldLock(path);
		});
	}
	return { ok: true, release: () => releaseHeldLock(lockPath) };
}
