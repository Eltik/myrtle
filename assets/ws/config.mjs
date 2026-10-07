// Configuration for `run.mjs ws`: which regions, and the shared settings.
//
// WS_SERVERS (or `--servers`) holds a JSON array of region entries and selects the
// single all-regions process. Without it, or with an explicit `--server`, the
// legacy one-region-per-process mode runs exactly as before.

import { cpus } from "node:os";
import { join } from "node:path";
import { createFileLock, createResourceGuard, envNumber } from "./scheduler.mjs";

/**
 * @param {string} raw JSON array, e.g. [{"server":"en","port":9160,"profile":"full","startDelayMin":0}]
 * @param {object} defaults { savedir, outputDir, threads, intervalMin } applied where an entry omits them
 * @param {string[]} knownServers
 */
export function parseServers(raw, defaults, knownServers) {
	let list;
	try {
		list = JSON.parse(raw);
	} catch (err) {
		throw new Error(`WS_SERVERS is not valid JSON: ${err.message}`);
	}
	if (!Array.isArray(list) || list.length === 0) {
		throw new Error("WS_SERVERS must be a non-empty JSON array of {server, port, profile, startDelayMin}");
	}
	const seenServers = new Set();
	const seenPorts = new Set();
	return list.map((e, i) => {
		const serverKey = e.server;
		if (!knownServers.includes(serverKey)) {
			throw new Error(`WS_SERVERS[${i}]: unknown server "${serverKey}". Valid: ${knownServers.join(", ")}`);
		}
		const port = Number(e.port);
		if (!Number.isInteger(port) || port <= 0) throw new Error(`WS_SERVERS[${i}] (${serverKey}): port is required`);
		if (seenServers.has(serverKey)) throw new Error(`WS_SERVERS: server "${serverKey}" listed twice`);
		if (seenPorts.has(port)) throw new Error(`WS_SERVERS: port ${port} listed twice`);
		seenServers.add(serverKey);
		seenPorts.add(port);
		const intervalRaw = Number(e.intervalMin ?? defaults.intervalMin);
		return {
			serverKey,
			port,
			profile: e.profile ?? "full",
			threads: Number(e.threads ?? defaults.threads),
			intervalMin: Number.isFinite(intervalRaw) && intervalRaw > 0 ? intervalRaw : 30,
			startDelayMin: Number(e.startDelayMin ?? 0),
			savedir: join(e.savedir ?? defaults.savedir, serverKey),
			outputDir: join(e.outputDir ?? defaults.outputDir, serverKey),
		};
	});
}

/**
 * Shared settings, every one with a kill switch whose value restores the old
 * behaviour exactly:
 *
 *   WS_MAX_BACKOFF_MIN   360   "0" = no backoff. A blank or negative value takes 360,
 *                             because Number("") is 0 and would disable it silently.
 *                             The cap is a TRADE; ruled out on the way: no backoff at
 *                             all (the re-download loop that timed out the disk) and a
 *                             hard attempt cap (leaves a box stale after a human fixes it).
 *   WS_LOCK              1     0 = no lockfile (multi mode still runs one job at a time in-process)
 *   WS_LOCK_FILE         <savedir>/.watcher.lock
 *   WS_NICE              10    0 = normal CPU priority
 *   WS_IONICE_CLASS      2     0 = no ionice (2 = best-effort, 3 = idle)
 *   WS_IONICE_LEVEL      7     best-effort level, 7 is the lowest
 *   WS_MAX_UNPACK_JOBS   cores-1 (min 1)   0 = no cap, -j is the region's own threads
 *   WS_MIN_FREE_MB       1536 on Linux, 0 elsewhere   0 = no memory guard
 *   WS_MEM_WAIT_MIN      10    how long an extract waits for memory before failing
 *   WS_MEM_RETRY_MIN     2     how often a deferred job re-checks memory
 *   WS_MEM_MAX_DEFER_MIN 30    after this a deferred job is skipped until its next tick
 */
export function resolveSettings(env, { savedirRoot, log = console.log, platform = process.platform } = {}) {
	const rawBackoffMin = env.WS_MAX_BACKOFF_MIN;
	const backoffCapMin = envNumber(env, "WS_MAX_BACKOFF_MIN", 360);
	if (rawBackoffMin !== undefined && rawBackoffMin.trim() !== "" && Number(rawBackoffMin) !== backoffCapMin) {
		log(`WS_MAX_BACKOFF_MIN="${rawBackoffMin}" is not a usable number of minutes; using ${backoffCapMin}`);
	}

	const minFreeMb = envNumber(env, "WS_MIN_FREE_MB", platform === "linux" ? 1536 : 0);
	const guard = minFreeMb > 0
		? createResourceGuard({ minFreeMb, waitMs: envNumber(env, "WS_MEM_WAIT_MIN", 10) * 60_000 })
		: null;

	const lockOn = env.WS_LOCK !== "0";
	const lockPath = lockOn ? (env.WS_LOCK_FILE || join(savedirRoot, ".watcher.lock")) : null;

	const nice = envNumber(env, "WS_NICE", 10);
	const ioniceClass = envNumber(env, "WS_IONICE_CLASS", 2);
	const priority = nice > 0 || ioniceClass > 0
		? { nice, ioniceClass, ioniceLevel: envNumber(env, "WS_IONICE_LEVEL", 7) }
		: null;

	return {
		maxBackoffMs: backoffCapMin * 60_000,
		backoffCapMin,
		alignToClock: env.WS_ALIGN !== "0",
		allowPartialDownload: env.WS_ALLOW_PARTIAL_DOWNLOAD === "1",
		maxUnpackJobs: envNumber(env, "WS_MAX_UNPACK_JOBS", Math.max(1, cpus().length - 1)),
		priority,
		guard,
		fileLock: createFileLock(lockPath),
		deferRetryMs: envNumber(env, "WS_MEM_RETRY_MIN", 2) * 60_000,
		maxDeferMs: envNumber(env, "WS_MEM_MAX_DEFER_MIN", 30) * 60_000,
	};
}
