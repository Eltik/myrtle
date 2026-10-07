#!/usr/bin/env node

import { execFileSync, spawn } from "node:child_process";
import {
	copyFileSync,
	existsSync,
	mkdirSync,
	readdirSync,
	readFileSync,
	statSync,
	unlinkSync,
	writeFileSync,
} from "node:fs";
import { readdir, stat } from "node:fs/promises";
import { dirname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

import boxen from "boxen";
import chalk from "chalk";
import inquirer from "inquirer";
import ora from "ora";
import { WebSocketServer } from "ws";
import { sweepOrphans } from "./orphans.mjs";
import { parseServers, resolveSettings } from "./ws/config.mjs";
import { makeLogger, createRegionWatcher } from "./ws/region.mjs";
import { createScheduler, lowPriorityCommand } from "./ws/scheduler.mjs";

// ─── Constants ──────────────────────────────────────────────────────────────

const __dirname = dirname(fileURLToPath(import.meta.url));
const isWindows = process.platform === "win32";
const isMac = process.platform === "darwin";
const exe = isWindows ? ".exe" : "";
const BINARIES_DIR = join(__dirname, "binaries");
const DOWNLOADER_BIN = join(BINARIES_DIR, `downloader${exe}`);
const UNPACKER_BIN = join(BINARIES_DIR, `unpacker${exe}`);
const DOWNLOADER_BUILD = join(
	__dirname,
	"downloader",
	"target",
	"release",
	`downloader${exe}`,
);
const UNPACKER_BUILD = join(
	__dirname,
	"unpacker",
	"target",
	"release",
	`unpacker${exe}`,
);
const DEFAULT_THREADS = 2;

// WS_NO_TRUNCATION_CHECK=1 disables the post-extract file-count comparison in
// `outputLooksTruncated`. It is an escape hatch for stopping a re-extract loop
// while the cause is diagnosed, not a setting: with the check off a genuinely
// truncated tree is undetectable again, which is the failure it exists to catch.
// Compared against "1" rather than tested for truthiness, so that writing "0"
// turns it OFF, which is what anyone setting it to "0" expects.
const TRUNCATION_CHECK_OFF = process.env.WS_NO_TRUNCATION_CHECK === "1";

// How often the truncation walk may actually run, per output tree, and when each
// tree was last walked. A tree does not spontaneously lose files between two
// 30-minute ticks: the things that truncate one are an OOM-killed unpacker or a
// bad sweep, both of which happen during an extract, not while the watcher idles.
// Walking every tick bought nothing and cost a full-tree traversal every 30
// minutes per region. Throttled, the check still catches a truncated tree within
// WS_TRUNCATION_CHECK_MIN of it happening, which is well inside the weeks that
// went unnoticed before the check existed at all. Parsed like the backoff cap: an
// empty or unusable value takes the default rather than becoming 0.
const rawTruncMin = process.env.WS_TRUNCATION_CHECK_MIN;
const parsedTruncMin =
	rawTruncMin === undefined || rawTruncMin.trim() === ""
		? 360
		: Number(rawTruncMin);
const TRUNCATION_WALK_EVERY_MS =
	(Number.isFinite(parsedTruncMin) && parsedTruncMin >= 0
		? parsedTruncMin
		: 360) *
	60 *
	1000;
const lastTruncationWalk = new Map();
// Built via RegExp constructor to avoid a literal ESC control character in source
const ANSI_RE = new RegExp(`${String.fromCharCode(0x1b)}\\[[0-9;]*m`, "g");

// ─── Server Version URL Map ─────────────────────────────────────────────────
// Source: downloader/src/server.rs

const SERVERS = {
	en: {
		label: "Global/EN (Yostar)",
		versionUrl:
			"https://ark-us-static-online.yo-star.com/assetbundle/official/Android/version",
		cdnBaseUrl:
			"https://ark-us-static-online.yo-star.com/assetbundle/official/Android/assets",
	},
	jp: {
		label: "Japan (Yostar)",
		versionUrl:
			"https://ark-jp-static-online.yo-star.com/assetbundle/official/Android/version",
		cdnBaseUrl:
			"https://ark-jp-static-online.yo-star.com/assetbundle/official/Android/assets",
	},
	kr: {
		label: "Korea (Yostar)",
		// The bare `ark-kr-static-online.yo-star.com` stopped resolving (NXDOMAIN on
		// 1.1.1.1, 2026-10-06). This host is the `hu` entry of
		// https://ak-conf.arknights.kr/config/prod/official/network_config; read it
		// there again if KR's version check starts failing with "fetch failed".
		versionUrl:
			"https://ark-kr-static-online-1300509597.yo-star.com/assetbundle/official/Android/version",
		cdnBaseUrl:
			"https://ark-kr-static-online-1300509597.yo-star.com/assetbundle/official/Android/assets",
	},
	tw: {
		label: "Taiwan (Gryphline)",
		versionUrl:
			"https://ark-tw-static-online.yo-star.com/assetbundle/official/Android/version",
		cdnBaseUrl:
			"https://ark-tw-static-online.yo-star.com/assetbundle/official/Android/assets",
	},
	cn: {
		label: "CN Official (Hypergryph)",
		versionUrl:
			"https://ak-conf.hypergryph.com/config/prod/official/Android/version",
		cdnBaseUrl: "https://ak.hycdn.cn/assetbundle/official/Android/assets",
	},
	bilibili: {
		label: "CN Bilibili",
		versionUrl: "https://ak-conf.hypergryph.com/config/prod/b/Android/version",
		cdnBaseUrl: "https://ak.hycdn.cn/assetbundle/bilibili/Android/assets",
	},
};

// ─── Shared Utilities ───────────────────────────────────────────────────────

function tryExec(cmd, args) {
	try {
		return execFileSync(cmd, args, {
			encoding: "utf-8",
			stdio: ["pipe", "pipe", "pipe"],
		}).trim();
	} catch {
		return null;
	}
}

function parseVersion(versionStr) {
	const match = versionStr?.match(/(\d+)\.(\d+)\.(\d+)/);
	if (!match) return null;
	return { major: +match[1], minor: +match[2], patch: +match[3] };
}

function versionGte(v, min) {
	if (v.major !== min.major) return v.major > min.major;
	if (v.minor !== min.minor) return v.minor > min.minor;
	return v.patch >= min.patch;
}

async function fetchServerVersion(serverKey) {
	const server = SERVERS[serverKey];
	if (!server) throw new Error(`Unknown server: ${serverKey}`);
	const res = await fetch(server.versionUrl);
	if (!res.ok) throw new Error(`HTTP ${res.status} from ${server.versionUrl}`);
	const data = await res.json();
	return { resVersion: data.resVersion, clientVersion: data.clientVersion };
}

async function fetchHotUpdateList(serverKey, resVersion) {
	const server = SERVERS[serverKey];
	if (!server) throw new Error(`Unknown server: ${serverKey}`);
	const url = `${server.cdnBaseUrl}/${resVersion}/hot_update_list.json`;
	const res = await fetch(url);
	if (!res.ok) throw new Error(`HTTP ${res.status} from ${url}`);
	return res.json();
}

/**
 * Delete .bin files in savedir/anon/ and .idx files at savedir root that
 * aren't referenced by the given hot_update_list.json. Orphans accumulate
 * across version bumps and silently corrupt gamedata extraction because the
 * unpacker walks all .bin files and writes by TextAsset m_Name — when two
 * bundles share an m_Name (the suffix is stable across versions), last write
 * wins and old content can clobber new.
 */
/**
 * Drop entries from persistent_res_list.json whose file is no longer on disk.
 *
 * The downloader's `Manifest::filter_needed` keeps a file only when the server's
 * md5 differs from the md5 recorded here - it never stats the file. So a record
 * left behind for a bundle we deleted makes that bundle permanently
 * un-redownloadable: the downloader believes it already has it, forever. Since
 * `anon/` names are content hashes, a bundle re-referenced later at the same
 * name is skipped and its gamedata table silently goes missing.
 *
 * Sweeps the whole record rather than just this run's deletions, so it also
 * heals entries orphaned by earlier prunes.
 *
 * @param {string} savedir
 * @returns {number} records dropped
 */
function reconcileManifest(savedir) {
	const manifestPath = join(savedir, "persistent_res_list.json");
	if (!existsSync(manifestPath)) return 0;

	let manifest;
	try {
		manifest = JSON.parse(readFileSync(manifestPath, "utf-8"));
	} catch (err) {
		console.warn(`  prune: cannot read ${manifestPath}: ${err.message}`);
		return 0;
	}

	const entries = manifest?.entries ?? manifest;
	if (!entries || typeof entries !== "object") return 0;

	let dropped = 0;
	for (const name of Object.keys(entries)) {
		if (existsSync(join(savedir, name))) continue;
		delete entries[name];
		dropped++;
	}

	if (dropped > 0) {
		try {
			writeFileSync(manifestPath, JSON.stringify(manifest), "utf-8");
		} catch (err) {
			console.warn(`  prune: cannot write ${manifestPath}: ${err.message}`);
			return 0;
		}
	}
	return dropped;
}

function pruneOrphans(savedir, hotList) {
	const keepBin = new Set();
	for (const a of hotList.abInfos ?? []) {
		if (a.name?.startsWith("anon/")) {
			keepBin.add(a.name.slice("anon/".length));
		}
	}
	const keepIdx = new Set([hotList.manifestName].filter(Boolean));

	// An EMPTY keep-set is not an instruction to delete everything, it is evidence
	// that the hot-update list did not parse into what we expected. `fetchHotUpdateList`
	// only checks the HTTP status, so a 200 carrying an error page, a truncated body
	// or a changed schema all arrive here as an object with no `abInfos`, and the two
	// loops below would then unlink every bundle in the cache and every .idx at the
	// root. The next run re-downloads tens of GB onto a disk that may be the reason
	// the response was bad in the first place. There is no legitimate resVersion with
	// zero bundles, so treat it as a refusal rather than a sweep.
	if (keepBin.size === 0) {
		console.warn(
			chalk.yellow(
				`  prune: hot-update list yielded 0 bundles to keep; refusing to prune (would have deleted the whole cache)`,
			),
		);
		return { deleted: 0, freedBytes: 0 };
	}

	let deleted = 0;
	let freedBytes = 0;

	const anonDir = join(savedir, "anon");
	if (existsSync(anonDir)) {
		for (const f of readdirSync(anonDir)) {
			if (!f.endsWith(".bin") || keepBin.has(f)) continue;
			const path = join(anonDir, f);
			try {
				freedBytes += statSync(path).size;
				unlinkSync(path);
				deleted++;
			} catch (err) {
				console.warn(`  prune: failed to delete ${path}: ${err.message}`);
			}
		}
	}

	for (const f of readdirSync(savedir)) {
		if (!f.endsWith(".idx") || keepIdx.has(f)) continue;
		const path = join(savedir, f);
		try {
			freedBytes += statSync(path).size;
			unlinkSync(path);
			deleted++;
		} catch (err) {
			console.warn(`  prune: failed to delete ${path}: ${err.message}`);
		}
	}

	const reconciled = reconcileManifest(savedir);
	if (reconciled > 0) {
		console.log(`  prune: dropped ${reconciled} stale manifest record(s)`);
	}

	return { deleted, freedBytes, reconciled };
}

function readStoredVersion(savedir) {
	const versionFile = join(savedir, ".version");
	try {
		return readFileSync(versionFile, "utf-8").trim();
	} catch {
		return null;
	}
}

function writeStoredVersion(savedir, resVersion) {
	mkdirSync(savedir, { recursive: true });
	writeFileSync(join(savedir, ".version"), resVersion, "utf-8");
}

/**
 * Check if the unpacker binary is newer than the last extraction timestamp.
 * Returns true if re-extraction is needed (binary was rebuilt since last extract).
 */
function unpackerIsNewer(savedir) {
	const stampFile = join(savedir, ".last_extract");
	try {
		const binMtime = statSync(UNPACKER_BIN).mtimeMs;
		const stampMtime = statSync(stampFile).mtimeMs;
		return binMtime > stampMtime;
	} catch {
		// Stamp doesn't exist → never extracted, or binary missing
		return existsSync(UNPACKER_BIN);
	}
}

/**
 * Check whether the extraction output is missing/empty enough to require a
 * re-extract. Returns true if outputDir is missing or empty, or if the
 * `gamedata/` subdir is missing — the backend depends on it, and an admin
 * who deletes only that subdir to force a re-extract (a deliberate path for
 * recovering from orphan-bundle corruption) should be honored.
 */
function outputMissingOrEmpty(outputDir) {
	try {
		const entries = readdirSync(outputDir);
		if (entries.length === 0) return true;
	} catch {
		return true;
	}
	return !existsSync(join(outputDir, "gamedata"));
}

/**
 * Record a successful unpack: timestamp plus the number of files the unpacker
 * said it exported.
 *
 * The count is what makes a TRUNCATED extract detectable. `outputMissingOrEmpty`
 * only checks that the output dir is non-empty and `gamedata/` exists, so a run
 * killed partway (the unpacker has been OOM-killed on a 10 GiB box more than
 * once) leaves a half-written tree that looks complete forever — that is how one
 * region sat for weeks missing thousands of textures and audio files with no
 * error anywhere.
 *
 * `onDisk` is the baseline the check compares against; `exported` is kept for
 * diagnostics only. They are NOT interchangeable, and the stamps that carried
 * only `exported` are why. The unpacker's figure is a sum of its per-bundle
 * "Exported N" lines, taken before `sweepOrphans` deletes anything, so it counts
 * assets the exporter wrote and then removed, and it counts a path twice when two
 * bundles write it under last-write-wins. It is biased HIGH against the tree it
 * was compared to, permanently: a region whose sweep clears more than the 2%
 * tolerance re-extracted on every check, having just succeeded.
 *
 * @param {string} savedir
 * @param {number} [exported] assets the unpacker reported, PRE-sweep, diagnostic only
 * @param {number} [onDisk] files counted under the output tree AFTER the orphan sweep
 */
function touchExtractStamp(savedir, exported, onDisk) {
	mkdirSync(savedir, { recursive: true });
	const payload = {
		at: new Date().toISOString(),
		exported: exported ?? null,
		onDisk: onDisk ?? null,
	};
	writeFileSync(join(savedir, ".last_extract"), JSON.stringify(payload), "utf-8");
}

/**
 * Read the persisted backoff state, or a cleared state when there is none.
 *
 * The backoff HAS to outlive the process. pm2 restarts this watcher at
 * max_memory_restart, and the OOM killer takes node itself when the box is under
 * the memory pressure a runaway extract creates, so the process holding an
 * in-memory counter is precisely the one that dies. Kept in memory only, three
 * failures would arm a 2 hour wait, the fourth would kill node, and the restart
 * would re-extract immediately with the counter back at zero: a box crash-looping
 * every 40 minutes re-extracts MORE often than the 30 minute interval it had
 * before the backoff existed.
 *
 * Every field is range-checked rather than trusted: a truncated or hand-edited
 * `.backoff` must degrade to "no backoff", never to NaN, which would make
 * `Date.now() < nextAttemptAt` false forever and silently disable the guard.
 *
 * @param {string} savedir
 * @returns {{ consecutiveFailures: number, nextAttemptAt: number }}
 */
function readBackoffState(savedir) {
	try {
		const raw = JSON.parse(readFileSync(join(savedir, ".backoff"), "utf-8"));
		const failures = Number(raw?.consecutiveFailures);
		const until = Number(raw?.nextAttemptAt);
		return {
			consecutiveFailures:
				Number.isFinite(failures) && failures > 0 ? Math.floor(failures) : 0,
			nextAttemptAt: Number.isFinite(until) && until > 0 ? until : 0,
		};
	} catch {
		return { consecutiveFailures: 0, nextAttemptAt: 0 };
	}
}

/**
 * Persist the backoff, or remove the file when the count is back to zero, so a
 * healthy region leaves no state behind. Best effort: a watcher that cannot write
 * this must keep working, and the in-memory copy still holds for this process.
 *
 * @param {string} savedir
 * @param {number} consecutiveFailures
 * @param {number} nextAttemptAt epoch ms
 */
function writeBackoffState(savedir, consecutiveFailures, nextAttemptAt) {
	const path = join(savedir, ".backoff");
	try {
		if (consecutiveFailures <= 0) {
			if (existsSync(path)) unlinkSync(path);
			return;
		}
		mkdirSync(savedir, { recursive: true });
		writeFileSync(
			path,
			JSON.stringify({
				consecutiveFailures,
				nextAttemptAt,
				at: new Date().toISOString(),
			}),
			"utf-8",
		);
	} catch (err) {
		console.log(
			chalk.dim(`Could not persist backoff state: ${err.message}`),
		);
	}
}

/**
 * True when a stamp exists but carries no baseline `outputLooksTruncated` can use,
 * which is every stamp written before `onDisk` existed. The check is inert on
 * those by design, and nothing re-arms it except a successful extract, which only
 * happens if some OTHER trigger fires: on a region whose version is current and
 * whose unpacker is unchanged, that can be weeks. So the condition is announced at
 * startup rather than left to be discovered later by noticing missing textures.
 *
 * @param {string} savedir
 * @returns {boolean}
 */
function stampBaselineMissing(savedir) {
	const path = join(savedir, ".last_extract");
	try {
		const raw = JSON.parse(readFileSync(path, "utf-8"));
		return typeof raw?.onDisk !== "number" || raw.onDisk <= 0;
	} catch {
		// Unparseable or the old plain-timestamp format. Missing entirely is not a
		// warning: a first run has nothing to be inert about.
		return existsSync(path);
	}
}

/**
 * Count files actually present under `outputDir`, for comparison against the
 * `exported` figure recorded by the last successful unpack.
 *
 * ASYNC because this walks the whole extracted tree, which on the VPS is ~113 GB
 * and several hundred thousand files. Done with readdirSync it blocked the event
 * loop for the length of the walk, so the WebSocket server answered nothing while
 * it ran: no status, no pings, and a `force_update` a human had just sent sitting
 * unread in the socket. On a box whose failure mode IS disk starvation that was a
 * self-inflicted stall on the queue this whole change exists to unload.
 *
 * @param {string} dir
 * @returns {Promise<number>}
 */
async function countFiles(dir) {
	let total = 0;
	const stack = [dir];
	while (stack.length > 0) {
		const current = stack.pop();
		let entries;
		try {
			entries = await readdir(current, { withFileTypes: true });
		} catch {
			continue;
		}
		for (const entry of entries) {
			if (entry.isDirectory()) stack.push(join(current, entry.name));
			else total++;
		}
	}
	return total;
}

/**
 * True when the tree holds materially fewer files than the last successful
 * extract reported. Tolerates a small shortfall so a handful of pruned or
 * renamed outputs doesn't trigger a pointless multi-hour re-extract.
 *
 * @param {string} savedir
 * @param {string} outputDir
 * @returns {Promise<boolean>}
 */
async function outputLooksTruncated(savedir, outputDir) {
	if (TRUNCATION_CHECK_OFF) return false;

	let expected;
	try {
		const raw = JSON.parse(readFileSync(join(savedir, ".last_extract"), "utf-8"));
		expected = typeof raw?.onDisk === "number" ? raw.onDisk : null;
	} catch {
		return false; // no stamp, or the old plain-timestamp format
	}
	// A stamp with no `onDisk` predates the post-sweep count. Its `exported`
	// baseline is biased high (see touchExtractStamp) and decides nothing, so the
	// check stays INERT until the next successful unpack writes a comparable
	// number. Tested against null and <= 0 rather than for truthiness: an onDisk of
	// 0 is a real reading, and `outputMissingOrEmpty` already covers an empty tree.
	if (expected === null || expected <= 0) return false;

	// Throttled here, after the cheap baseline checks and before the expensive
	// walk, so a tick that is going to decide nothing costs nothing. Returning
	// false when throttled errs toward NOT re-extracting, which is the safe
	// direction: the cost of a late detection is stale files, the cost of a false
	// positive is the ~113 GB re-extract this change exists to prevent.
	const lastWalk = lastTruncationWalk.get(outputDir) ?? 0;
	if (Date.now() - lastWalk < TRUNCATION_WALK_EVERY_MS) return false;

	const actual = await countFiles(outputDir);
	lastTruncationWalk.set(outputDir, Date.now());
	const TOLERANCE = 0.98;
	if (actual >= Math.floor(expected * TOLERANCE)) return false;

	console.log(
		chalk.yellow(
			`Output looks truncated: ${actual} files on disk vs ${expected} exported at the last successful unpack`,
		),
	);
	return true;
}

function binariesExist() {
	return existsSync(DOWNLOADER_BIN) && existsSync(UNPACKER_BIN);
}

function formatBytes(bytes) {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	if (bytes < 1024 * 1024 * 1024)
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

/**
 * Recursively walk a directory, summing the byte size and count of every nested
 * file. Unreadable subdirectories are skipped rather than aborting the walk.
 * @param {string} dir - Absolute directory path
 * @returns {Promise<{ size: number, fileCount: number }>}
 */
async function dirStats(dir) {
	let size = 0;
	let fileCount = 0;
	let entries;
	try {
		entries = await readdir(dir, { withFileTypes: true });
	} catch {
		return { size, fileCount };
	}
	for (const entry of entries) {
		const fullPath = join(dir, entry.name);
		if (entry.isDirectory()) {
			const sub = await dirStats(fullPath);
			size += sub.size;
			fileCount += sub.fileCount;
		} else if (entry.isFile()) {
			try {
				const st = await stat(fullPath);
				size += st.size;
				fileCount++;
			} catch {
				// skip unreadable file
			}
		}
	}
	return { size, fileCount };
}

// ─── Progress Bar ───────────────────────────────────────────────────────────

function createProgressBar(label, { width = 30 } = {}) {
	let finished = false;
	let spinnerActive = false;
	let spinner = null;

	function startSpinner(text) {
		if (finished) return;
		spinner = ora(text).start();
		spinnerActive = true;
	}

	function render(completed, total) {
		if (finished) return;
		// Stop spinner on first real progress render
		if (spinnerActive && spinner) {
			spinner.stop();
			spinnerActive = false;
		}
		const percent = total > 0 ? completed / total : 0;
		const filled = Math.round(width * percent);
		const empty = width - filled;
		const bar =
			chalk.cyan("\u2588".repeat(filled)) + chalk.dim("\u2591".repeat(empty));
		const pct = (percent * 100).toFixed(1).padStart(5);
		const counts = `${String(completed).padStart(String(total).length)}/${total}`;
		process.stdout.write(
			`\r  ${chalk.bold(label)}  ${bar}  ${counts}  ${pct}%`,
		);
	}

	function clearLine() {
		process.stdout.write(`\r${" ".repeat(process.stdout.columns || 120)}\r`);
	}

	function succeed(message) {
		if (finished) return;
		finished = true;
		if (spinnerActive && spinner) spinner.stop();
		clearLine();
		console.log(`  ${chalk.green("\u2713")} ${message}`);
	}

	function fail(message) {
		if (finished) return;
		finished = true;
		if (spinnerActive && spinner) spinner.stop();
		clearLine();
		console.log(`  ${chalk.red("\u2717")} ${message}`);
	}

	return { startSpinner, render, succeed, fail };
}

// ─── Binary Progress Helpers ────────────────────────────────────────────────

/**
 * Spawn the downloader binary and track progress.
 *
 * indicatif suppresses all stderr output when not connected to a TTY,
 * so we track progress by polling the persistent_res_list.json manifest
 * which the downloader updates after each file completes.
 *
 * @param {object} opts
 * @param {string} opts.serverKey - Server region key
 * @param {string} opts.savedir - Save directory
 * @param {number} [opts.threads] - Concurrent download threads
 * @param {(p: {completed: number, total: number, percent: number}) => void} [opts.onProgress]
 * @returns {Promise<{downloaded: number, failed: number, totalBytes: number}>}
 */
function runDownload({
	serverKey,
	savedir,
	threads = DEFAULT_THREADS,
	onProgress,
	profile,
	priority = null,
}) {
    const args = ["--server", serverKey, "-d", savedir, "-t", String(threads), "download", "--all"];
    if (profile) args.push("--profile", profile);
	// `priority` (ws mode only) wraps the binary in nice/ionice; null spawns it bare.
	const [cmd, cmdArgs] = lowPriorityCommand(DOWNLOADER_BIN, args, priority);
	return new Promise((resolve, reject) => {
		const child = spawn(
			cmd,
			cmdArgs,
			{
				cwd: __dirname,
				stdio: ["ignore", "pipe", "pipe"],
			},
		);

		activeChildren.add(child);
		child.on("close", () => activeChildren.delete(child));

		let stdoutBuf = "";
		let stderrRaw = "";
		let totalFiles = 0;
		let initialManifestCount = 0;
		let pollTimer = null;

		// Count entries in the manifest file to track progress
		function countManifestEntries() {
			try {
				const manifestPath = join(savedir, "persistent_res_list.json");
				const content = readFileSync(manifestPath, "utf-8");
				const entries = JSON.parse(content);
				return Object.keys(entries).length;
			} catch {
				return 0;
			}
		}

		// Snapshot the manifest before download starts
		initialManifestCount = countManifestEntries();

		child.stdout.on("data", (chunk) => {
			stdoutBuf += chunk.toString();
			// Parse total from "N files to download (M skipped)"
			if (totalFiles === 0) {
				const m = stdoutBuf.match(/(\d+) files to download/);
				if (m) {
					totalFiles = parseInt(m[1], 10);
					onProgress?.({ completed: 0, total: totalFiles, percent: 0 });

					// Start polling manifest for progress
					pollTimer = setInterval(() => {
						const currentCount = countManifestEntries();
						const completed = currentCount - initialManifestCount;
						if (completed > 0 && totalFiles > 0) {
							onProgress?.({
								completed: Math.min(completed, totalFiles),
								total: totalFiles,
								percent: (Math.min(completed, totalFiles) / totalFiles) * 100,
							});
						}
					}, 1000);
				}
			}
		});

		child.stderr.on("data", (chunk) => {
			stderrRaw += chunk.toString();
		});

		child.on("close", (code) => {
			if (pollTimer) clearInterval(pollTimer);

			if (code !== 0) {
				const clean = `${stdoutBuf}\n${stderrRaw}`
					.replace(ANSI_RE, "")
					.replace(/\r/g, "\n");
				const lines = clean
					.split("\n")
					.map((l) => l.trim())
					.filter(Boolean);
				const errOutput = lines.slice(-10).join("\n");
				reject(new Error(`Downloader exited with code ${code}\n${errOutput}`));
				return;
			}
			// Parse "Done: X downloaded, Y failed, Z bytes"
			const doneMatch = stdoutBuf.match(
				/Done:\s*(\d+)\s*downloaded,\s*(\d+)\s*failed,\s*(\d+)\s*bytes/,
			);
			resolve({
				downloaded: doneMatch ? parseInt(doneMatch[1], 10) : 0,
				failed: doneMatch ? parseInt(doneMatch[2], 10) : 0,
				totalBytes: doneMatch ? parseInt(doneMatch[3], 10) : 0,
			});
		});

		child.on("error", (err) => {
			if (pollTimer) clearInterval(pollTimer);
			reject(err);
		});
	});
}

/**
 * Spawn the unpacker binary and track progress.
 *
 * indicatif suppresses all stderr output when not connected to a TTY.
 * The unpacker prints "Exported N ..." lines to stdout as each phase completes.
 * We parse stdout for these lines to track progress.
 *
 * @param {object} opts
 * @param {string} opts.inputDir - Input directory with bundles
 * @param {string} opts.outputDir - Output directory for extracted assets
 * @param {number} [opts.jobs] - Parallel extraction threads
 * @param {(p: {completed: number, total: number, percent: number}) => void} [opts.onProgress]
 * @returns {Promise<{exported: number}>}
 */
function runUnpackOnce({
	inputDir,
	outputDir,
	jobs = DEFAULT_THREADS,
	onProgress,
	priority = null,
}) {
	const [cmd, cmdArgs] = lowPriorityCommand(
		UNPACKER_BIN,
		["extract", "-i", inputDir, "-o", outputDir, "-j", String(jobs)],
		priority,
	);
	return new Promise((resolve, reject) => {
		const child = spawn(
			cmd,
			cmdArgs,
			{
				cwd: __dirname,
				stdio: ["ignore", "pipe", "pipe"],
			},
		);

		activeChildren.add(child);
		child.on("close", () => activeChildren.delete(child));

		const MAX_TAIL = 256 * 1024; // bounded rolling buffer for error diagnostics
		let stdoutTail = "";
		let stderrTail = "";
		let stdoutPartial = ""; // incomplete trailing line between chunks
		let stderrPartial = "";
		let totalExported = 0;
		let exportedSum = 0;

		const appendTail = (tail, s) => {
			const combined = tail + s;
			return combined.length > MAX_TAIL ? combined.slice(-MAX_TAIL) : combined;
		};

		child.stdout.on("data", (chunk) => {
			const text = chunk.toString();
			stdoutTail = appendTail(stdoutTail, text);

			stdoutPartial += text;
			const lines = stdoutPartial.split("\n");
			stdoutPartial = lines.pop() ?? "";

			let latestProgress = totalExported;
			for (const line of lines) {
				const mp = line.match(/progress:\s*(\d+)\s+assets/);
				if (mp) {
					const n = parseInt(mp[1], 10);
					if (n > latestProgress) latestProgress = n;
				}
				const me = line.match(/Exported\s+(\d+)\s+/);
				if (me) {
					const n = parseInt(me[1], 10);
					exportedSum += n;
					if (n > latestProgress) latestProgress = n;
				}
			}
			if (latestProgress > totalExported) {
				totalExported = latestProgress;
				onProgress?.({ completed: totalExported, total: 0, percent: -1 });
			}
		});

		child.stderr.on("data", (chunk) => {
			stderrTail = appendTail(stderrTail, chunk.toString());
			stderrPartial += chunk.toString();
			if (stderrPartial.length > MAX_TAIL) {
				stderrPartial = stderrPartial.slice(-MAX_TAIL);
			}
		});

		child.on("close", (code, signal) => {
			if (code !== 0) {
				const clean = `${stdoutTail}\n${stderrTail}`
					.replace(ANSI_RE, "")
					.replace(/\r/g, "\n");
				const lines = clean
					.split("\n")
					.map((l) => l.trim())
					.filter(Boolean);
				const errOutput = lines.slice(-10).join("\n");
				// code is null when the process was killed by a signal (e.g.
				// SIGKILL from the OOM killer) rather than exiting on its own.
				const how =
					code === null
						? `killed by signal ${signal}${signal === "SIGKILL" ? " (likely out of memory — lower -j jobs)" : ""}`
						: `exited with code ${code}`;
				const err = new Error(`Unpacker ${how}\n${errOutput}`);
				err.signal = signal; // null for a clean exit; set when signal-killed
				reject(err);
				return;
			}
			// Process any final line without trailing newline
			if (stdoutPartial) {
				const me = stdoutPartial.match(/Exported\s+(\d+)\s+/);
				if (me) exportedSum += parseInt(me[1], 10);
			}
			resolve({ exported: exportedSum });
		});

		child.on("error", reject);
	});
}

/**
 * Run the unpacker, retrying once single-threaded if the first attempt is
 * OOM-killed (SIGKILL). Each extraction job holds whole bundles in RAM, so
 * -j 1 roughly halves peak memory and lets a memory-starved box finish where
 * the parallel run got killed. A crash (SIGSEGV) or a clean non-zero exit is
 * not retried — those are not memory problems and re-running won't help.
 *
 * @param {Parameters<typeof runUnpackOnce>[0] & {onNotice?: (message: string) => void}} opts
 * @returns {Promise<{exported: number}>}
 */
async function runUnpack(opts) {
	const startedAt = Date.now();
	let stats;
	try {
		stats = await runUnpackOnce(opts);
	} catch (err) {
		const jobs = opts.jobs ?? DEFAULT_THREADS;
		if (err?.signal === "SIGKILL" && jobs > 1) {
			const msg = `Unpacker ran out of memory at -j ${jobs}; retrying single-threaded (-j 1)…`;
			console.log(chalk.yellow(`[${new Date().toLocaleTimeString()}] ${msg}`));
			opts.onNotice?.(msg);
			stats = await runUnpackOnce({ ...opts, jobs: 1 });
		} else {
			throw err;
		}
	}
	// REPLACE, not overlay: the exporter rewrites every file it produces, so a file in a
	// subtree this run wrote into that predates the run is one the exporter no longer
	// produces, still served and referenced by nothing. Remove it and log it, as the local
	// install does (see orphans.mjs). `--keep-orphans` (WS_KEEP_ORPHANS=1) skips the sweep,
	// `--dry-orphans` (WS_DRY_ORPHANS=1) logs what it would remove and removes nothing.
	const keep = !!(cliArgs["keep-orphans"] ?? process.env.WS_KEEP_ORPHANS);
	const dry = !!(cliArgs["dry-orphans"] ?? process.env.WS_DRY_ORPHANS);
	if (!keep) {
		const swept = sweepOrphans(opts.outputDir, startedAt, { dry });
		const verb = dry ? "would remove" : "removed";
		const msg = `Orphan sweep ${verb} ${swept.files} file(s), ${formatBytes(swept.bytes)}${swept.log ? `, logged at ${swept.log}` : ""}${swept.skipped.length ? ` (untouched subtrees left alone: ${swept.skipped.join(", ")})` : ""}`;
		console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] ${msg}`));
		opts.onNotice?.(msg);
	}
	// Counted HERE, after the sweep, so the number written to the stamp and the
	// number a later check measures are the same measurement of the same tree.
	// Taken before the sweep it is a different quantity, and comparing the two is
	// what made a successful extract look truncated.
	return { ...stats, onDisk: await countFiles(opts.outputDir) };
}

// ─── Option 1: Setup ───────────────────────────────────────────────────────

async function runSetup() {
	console.log(chalk.bold("\n─── Prerequisite Checks ───\n"));

	const RUST_MIN = { major: 1, minor: 85, patch: 0 };
	const checks = [];

	// Git
	const gitOut = tryExec("git", ["--version"]);
	checks.push({
		name: "Git",
		found: !!gitOut,
		version: gitOut ?? null,
		install: {
			darwin: "xcode-select --install  (or)  brew install git",
			linux: "sudo apt install git  (or)  sudo dnf install git",
			win32: "Download from https://git-scm.com/download/win",
		},
	});

	// Rust compiler
	const rustcOut = tryExec("rustc", ["--version"]);
	const rustcVer = parseVersion(rustcOut);
	const rustcOk = rustcVer ? versionGte(rustcVer, RUST_MIN) : false;
	checks.push({
		name: "Rust (rustc)",
		found: !!rustcOut && rustcOk,
		version: rustcOut ?? null,
		detail: rustcOut && !rustcOk ? "Need >= 1.85.0 for edition 2024" : null,
		install: {
			darwin: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
			linux: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
			win32: "Download rustup-init.exe from https://rustup.rs",
		},
	});

	// Cargo
	const cargoOut = tryExec("cargo", ["--version"]);
	checks.push({
		name: "Cargo",
		found: !!cargoOut,
		version: cargoOut ?? null,
		install: {
			darwin: "Installed with Rust (rustup)",
			linux: "Installed with Rust (rustup)",
			win32: "Installed with Rust (rustup)",
		},
	});

	// C compiler
	let ccFound = false;
	let ccVersion = null;
	if (isMac) {
		const xcodeOut = tryExec("xcode-select", ["-p"]);
		ccFound = !!xcodeOut;
		ccVersion = xcodeOut ? `Xcode CLT: ${xcodeOut}` : null;
	} else if (isWindows) {
		// Check which Rust toolchain is active (gnu vs msvc)
		const rustcHost = tryExec("rustc", ["-vV"]);
		const isMsvcToolchain = rustcHost?.includes("x86_64-pc-windows-msvc");

		if (isMsvcToolchain) {
			// For MSVC toolchain, check if cl.exe exists (even if not in PATH)
			// Try common MSVC installation paths
			const clOut = tryExec("where", ["cl"]);
			if (clOut) {
				ccFound = true;
				ccVersion = clOut.split("\n")[0];
			} else {
				// Check if MSVC is installed even if not in PATH
				const vswhereOut = tryExec("C:\\Program Files (x86)\\Microsoft Visual Studio\\Installer\\vswhere.exe",
					["-latest", "-requires", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"]);
				if (vswhereOut) {
					ccFound = true;
					ccVersion = "MSVC (found via vswhere, not in PATH)";
				}
			}
		} else {
			// For GNU toolchain, check for gcc
			const gccOut = tryExec("gcc", ["--version"]);
			ccFound = !!gccOut;
			ccVersion = gccOut ? gccOut.split("\n")[0] : null;
		}
	} else {
		const gccOut = tryExec("gcc", ["--version"]);
		ccFound = !!gccOut;
		ccVersion = gccOut ? gccOut.split("\n")[0] : null;
	}
	checks.push({
		name: "C Compiler",
		found: ccFound,
		version: ccVersion ?? null,
		install: {
			darwin: "xcode-select --install",
			linux: "sudo apt install build-essential  (or)  sudo dnf install gcc",
			win32: "Install Visual Studio Build Tools (C++ workload)",
		},
	});

	// Display results
	const missing = [];
	for (const c of checks) {
		const icon = c.found ? chalk.green("✓") : chalk.red("✗");
		const ver = c.version ? chalk.dim(` (${c.version})`) : "";
		const detail = c.detail ? chalk.yellow(` — ${c.detail}`) : "";
		console.log(`  ${icon} ${c.name}${ver}${detail}`);
		if (!c.found) missing.push(c);
	}
	console.log();

	if (missing.length > 0) {
		const instructions = missing
			.map((c) => {
				const cmd = c.install[process.platform] ?? c.install.linux;
				return `${chalk.bold(c.name)}\n  ${cmd}`;
			})
			.join("\n\n");

		console.log(
			boxen(instructions, {
				title: "Missing Prerequisites",
				titleAlignment: "center",
				padding: 1,
				margin: { top: 0, bottom: 1, left: 1, right: 1 },
				borderStyle: "round",
				borderColor: "yellow",
			}),
		);
		console.log(
			chalk.yellow("Install the missing tools above, then re-run this script."),
		);
		return;
	}

	const { proceed } = await inquirer.prompt([
		{
			type: "confirm",
			name: "proceed",
			message: "All prerequisites found. Proceed with building?",
			default: true,
		},
	]);
	if (!proceed) return;

	// Check OpenArknightsFBS
	const fbsDir = join(__dirname, "OpenArknightsFBS", "FBS");
	let hasFbs = false;
	try {
		const files = readdirSync(fbsDir);
		hasFbs = files.some((f) => f.endsWith(".fbs"));
	} catch {
		// directory doesn't exist
	}
	if (!hasFbs) {
		console.log(
			chalk.yellow(
				"\n⚠  OpenArknightsFBS/FBS/ has no .fbs files.\n" +
					"   This is only needed for the generate-fbs binary.\n" +
					"   If you need it, clone the OpenArknightsFBS repo into this directory.\n",
			),
		);
	}

	// Build crates
	for (const crate of ["downloader", "unpacker"]) {
		await buildCrate(crate);
	}

	// Copy binaries to /binaries
	mkdirSync(BINARIES_DIR, { recursive: true });
	const copySpinner = ora("Copying binaries to ./binaries/…").start();
	copyFileSync(DOWNLOADER_BUILD, DOWNLOADER_BIN);
	copyFileSync(UNPACKER_BUILD, UNPACKER_BIN);
	copySpinner.succeed("Binaries copied to ./binaries/");

	// Results summary
	const displayDownloader = `.${sep}binaries${sep}downloader${exe}`;
	const displayUnpacker = `.${sep}binaries${sep}unpacker${exe}`;

	const summary = [
		chalk.bold.green("Build complete!\n"),
		`${chalk.bold("Downloader:")} ${displayDownloader}`,
		`${chalk.bold("Unpacker:")}   ${displayUnpacker}`,
		"",
		chalk.dim("Quick start:"),
		`  ${displayDownloader} --server en download --all`,
		`  ${displayUnpacker} extract -i ./ArkAssets -o ./output`,
	].join("\n");

	console.log(
		boxen(summary, {
			padding: 1,
			margin: { top: 1, bottom: 0, left: 1, right: 1 },
			borderStyle: "round",
			borderColor: "green",
		}),
	);
}

async function buildCrate(name) {
	const crateDir = join(__dirname, name);
	const spinner = ora(`Building ${name} (release)…`).start();
	const startTime = Date.now();
	let compiledCount = 0;

	return new Promise((resolve, reject) => {
		const child = spawn("cargo", ["build", "--release"], {
			cwd: crateDir,
			stdio: ["ignore", "pipe", "pipe"],
		});

		let stderr = "";
		child.stderr.on("data", (data) => {
			stderr += data.toString();
			const lines = data.toString().trim().split("\n");
			for (const line of lines) {
				if (line.includes("Compiling")) compiledCount++;
			}
			const last = lines[lines.length - 1].trim();
			if (last) {
				spinner.text = `Building ${name}: ${last}${compiledCount > 0 ? chalk.dim(` (${compiledCount} crates)`) : ""}`;
			}
		});

		child.on("close", (code) => {
			const elapsed = ((Date.now() - startTime) / 1000).toFixed(1);
			if (code === 0) {
				const detail =
					compiledCount > 0 ? `${compiledCount} crates compiled` : "up to date";
				spinner.succeed(`${name} built successfully (${elapsed}s, ${detail})`);
				resolve();
			} else {
				spinner.fail(`${name} build failed (${elapsed}s)`);
				console.error(chalk.red(stderr.slice(-500)));
				reject(new Error(`${name} build failed with code ${code}`));
			}
		});

		child.on("error", reject);
	});
}

// ─── Option 2: Check & Update ──────────────────────────────────────────────

async function runUpdate() {
	if (!binariesExist()) {
		console.log(chalk.red("\nBinaries not found. Run Setup first.\n"));
		return;
	}

	// Prompt for configuration
	let { serverKey, savedir, outputDir, threads, profile } = await inquirer.prompt([
		{
			type: "list",
			name: "serverKey",
			message: "Server region:",
			choices: Object.entries(SERVERS).map(([key, s]) => ({
				name: `${key} — ${s.label}`,
				value: key,
			})),
			default: "en",
		},
		{
			type: "input",
			name: "savedir",
			message: "Asset download directory:",
			default: "./ArkAssets",
		},
		{
			type: "input",
			name: "outputDir",
			message: "Extraction output directory:",
			default: "./output",
		},
		{
			type: "number",
			name: "threads",
			message: "Concurrent threads (download & unpack):",
			default: DEFAULT_THREADS,
		},
		{
			type: "list",
			name: "profile",
			message: "Content profile:",
			choices: [
				{ name: "full — everything (default)", value: "full" },
				{ name: "operators — gamedata + operator assets only", value: "operators" },
				{ name: "stages — stage-viewer level scenes + preview/banner art", value: "stages" },
				{ name: "gamedata — only the anon/ bundles + .idx (what `unpacker extract --gamedata` reads)", value: "gamedata" },
				{ name: "release — event / banner / skin-brand art for the Release Planner", value: "release" },
			],
			default: "full",
		},
	]);

	savedir = join(savedir, serverKey);
	outputDir = join(outputDir, serverKey);

	// Check version
	const versionSpinner = ora("Checking server version…").start();
	let serverVer;
	try {
		serverVer = await fetchServerVersion(serverKey);
		versionSpinner.succeed(
			`Server: client=${serverVer.clientVersion}  resources=${serverVer.resVersion}`,
		);
	} catch (err) {
		versionSpinner.fail(`Failed to fetch version: ${err.message}`);
		return;
	}

	const storedVer = readStoredVersion(savedir);
	if (storedVer) {
		console.log(chalk.dim(`  Local version: ${storedVer}`));
	} else {
		console.log(chalk.dim("  No local version found (first run)"));
	}

	const assetsUpToDate = storedVer === serverVer.resVersion;
	const needsReextract =
		assetsUpToDate &&
		(unpackerIsNewer(savedir) ||
			outputMissingOrEmpty(outputDir) ||
			(await outputLooksTruncated(savedir, outputDir)));

	if (assetsUpToDate && !needsReextract) {
		console.log(
			boxen(chalk.green("Assets are up to date!"), {
				padding: 1,
				margin: { top: 1, bottom: 0, left: 1, right: 1 },
				borderStyle: "round",
				borderColor: "green",
			}),
		);
		return;
	}

	if (needsReextract) {
		const reason = outputMissingOrEmpty(outputDir)
			? "but the output directory is missing or empty."
			: "but the unpacker binary has been rebuilt since last extraction.";
		console.log(
			boxen(
				[
					chalk.yellow.bold("Re-extraction needed"),
					"",
					`Assets version ${chalk.green(storedVer)} is current,`,
					reason,
				].join("\n"),
				{
					padding: 1,
					margin: { top: 1, bottom: 0, left: 1, right: 1 },
					borderStyle: "round",
					borderColor: "yellow",
				},
			),
		);

		const { proceed } = await inquirer.prompt([
			{
				type: "confirm",
				name: "proceed",
				message: "Re-extract with updated unpacker?",
				default: true,
			},
		]);
		if (!proceed) return;
	} else {
		// New assets available from server
		console.log(
			boxen(
				[
					chalk.yellow.bold("Update Available"),
					"",
					`Current: ${storedVer ?? chalk.dim("(none)")}`,
					`Server:  ${chalk.green(serverVer.resVersion)}`,
				].join("\n"),
				{
					padding: 1,
					margin: { top: 1, bottom: 0, left: 1, right: 1 },
					borderStyle: "round",
					borderColor: "yellow",
				},
			),
		);

		const { proceed } = await inquirer.prompt([
			{
				type: "confirm",
				name: "proceed",
				message: "Download and extract updates?",
				default: true,
			},
		]);
		if (!proceed) return;
	}

	// Download phase (skip if only re-extracting)
	if (!needsReextract) {
		console.log();
		const dlBar = createProgressBar("Downloading");
		dlBar.startSpinner("Fetching asset manifest…");
		try {
			const dlStats = await runDownload({
				serverKey,
				savedir,
				threads,
				profile,
				onProgress: ({ completed, total }) => dlBar.render(completed, total),
			});
			dlBar.succeed(
				`Download complete: ${dlStats.downloaded} files, ${dlStats.failed} failed, ${formatBytes(dlStats.totalBytes)}`,
			);
			// Same refusal as the watcher path: an incomplete bundle set must not be
			// unpacked, because the sweep afterwards deletes what this run did not write.
			if (dlStats.failed > 0 && process.env.WS_ALLOW_PARTIAL_DOWNLOAD !== "1") {
				console.log(
					chalk.red(
						`  ${dlStats.failed} bundle(s) failed to download. Refusing to unpack an incomplete set; the existing output is left alone. Re-run, or set WS_ALLOW_PARTIAL_DOWNLOAD=1 to override.`,
					),
				);
				return;
			}
		} catch (err) {
			dlBar.fail(`Download failed: ${err.message}`);
			return;
		}

		// Prune orphans before unpack so stale .bin bundles can't clobber
		// fresh ones via last-write-wins on shared TextAsset m_Names.
		try {
			const hotList = await fetchHotUpdateList(serverKey, serverVer.resVersion);
			const { deleted, freedBytes } = pruneOrphans(savedir, hotList);
			if (deleted > 0) {
				console.log(
					chalk.dim(
						`  Pruned ${deleted} orphan file(s), freed ${formatBytes(freedBytes)}`,
					),
				);
			}
		} catch (err) {
			console.warn(chalk.yellow(`  Orphan prune skipped: ${err.message}`));
		}
	}

	// Unpack phase
	const upSpinner = ora("Extracting assets…").start();
	let upStats;
	try {
		upStats = await runUnpack({
			inputDir: savedir,
			outputDir,
			jobs: threads,
			onProgress: ({ completed }) => {
				upSpinner.text = `Extracting assets… (${completed.toLocaleString()} exported so far)`;
			},
		});
		upSpinner.succeed(
			`Extraction complete: ${upStats.exported.toLocaleString()} assets exported`,
		);
	} catch (err) {
		upSpinner.fail(`Extraction failed: ${err.message}`);
		return;
	}

	// Save version & extraction timestamp
	if (!assetsUpToDate) {
		writeStoredVersion(savedir, serverVer.resVersion);
	}
	touchExtractStamp(savedir, upStats?.exported, upStats?.onDisk);

	const msg = needsReextract
		? `Re-extracted with updated unpacker (${serverVer.resVersion})`
		: `Updated to ${serverVer.resVersion}`;
	console.log(
		boxen(chalk.green.bold(msg), {
			padding: 1,
			margin: { top: 1, bottom: 0, left: 1, right: 1 },
			borderStyle: "round",
			borderColor: "green",
		}),
	);
}

// ─── Option 3: WebSocket Server ─────────────────────────────────────────────

// `run.mjs ws` serves one region per process (legacy: `--server`, or WS_SERVER with
// no WS_SERVERS) or every region from one process (WS_SERVERS / `--servers`, a JSON
// array of {server, port, profile, startDelayMin}). Both modes build the same
// region watchers from ws/region.mjs; in the single process they share ONE
// scheduler, so only one download or extract runs at a time on the box, a manual
// force_update included. The lockfile (WS_LOCK_FILE, default <savedir>/.watcher.lock)
// extends that across processes, so a legacy process left running beside the
// single one waits its turn instead of overlapping.
async function runWebSocketServer({ nonInteractive = false, cliArgs = {} } = {}) {
	if (!binariesExist()) {
		console.log(chalk.red("\nBinaries not found. Run Setup first.\n"));
		return;
	}

	const savedirRoot = cliArgs.savedir ?? process.env.WS_SAVEDIR ?? "./ArkAssets";
	const outputRoot = cliArgs.output ?? process.env.WS_OUTPUT ?? "./output";
	const defaults = {
		serverKey: cliArgs.server ?? process.env.WS_SERVER ?? "en",
		savedir: savedirRoot,
		outputDir: outputRoot,
		threads: Number(cliArgs.threads ?? process.env.WS_THREADS ?? DEFAULT_THREADS),
		profile: cliArgs.profile ?? process.env.WS_PROFILE ?? "full",
		port: Number(cliArgs.port ?? process.env.WS_PORT ?? 9160),
		intervalMin: Number(cliArgs.interval ?? process.env.WS_INTERVAL ?? 30),
		// Minutes past each interval boundary of this region's checks (its PHASE).
		startDelayMin: Number(
			cliArgs["start-delay"] ?? process.env.WS_START_DELAY_MIN ?? 0,
		),
	};

	// An explicit `--server` always means the legacy single-region mode, whatever
	// WS_SERVERS says: that is the kill switch back to one process per region.
	const serversSpec =
		cliArgs.server === undefined ? (cliArgs.servers ?? process.env.WS_SERVERS) : undefined;
	const multi = nonInteractive && typeof serversSpec === "string" && serversSpec.trim() !== "";

	let regions;
	if (multi) {
		try {
			regions = parseServers(serversSpec, defaults, Object.keys(SERVERS));
		} catch (err) {
			console.log(chalk.red(`\n${err.message}\n`));
			process.exitCode = 1;
			return;
		}
	} else {
		const config = nonInteractive
			? { ...defaults }
			: await inquirer.prompt([
				{
					type: "list",
					name: "serverKey",
					message: "Server region:",
					choices: Object.entries(SERVERS).map(([key, s]) => ({
						name: `${key} — ${s.label}`,
						value: key,
					})),
					default: defaults.serverKey,
				},
				{ type: "input", name: "savedir", message: "Asset download directory:", default: defaults.savedir },
				{ type: "input", name: "outputDir", message: "Extraction output directory:", default: defaults.outputDir },
				{ type: "number", name: "threads", message: "Concurrent threads (download & unpack):", default: defaults.threads },
				{ type: "list", name: "profile", message: "Content profile:", choices: [{ name: "full — everything", value: "full" }, { name: "operators — gamedata + operator assets only", value: "operators" }, { name: "stages — stage-viewer level scenes + preview/banner art", value: "stages" }, { name: "gamedata — only the anon/ bundles + .idx", value: "gamedata" }, { name: "release — event / banner / skin-brand art for the Release Planner", value: "release" }], default: defaults.profile },
				{ type: "number", name: "port", message: "WebSocket port:", default: defaults.port },
				{ type: "number", name: "intervalMin", message: "Check interval (minutes):", default: defaults.intervalMin },
			]);
		if (config.startDelayMin === undefined) config.startDelayMin = defaults.startDelayMin;

		if (!SERVERS[config.serverKey]) {
			console.log(chalk.red(`\nUnknown server: ${config.serverKey}. Valid: ${Object.keys(SERVERS).join(", ")}\n`));
			return;
		}
		config.savedir = join(config.savedir, config.serverKey);
		config.outputDir = join(config.outputDir, config.serverKey);

		// Clamped: Number("30m") is NaN, setInterval coerces NaN to 1 ms and the
		// backoff guard `Date.now() < NaN` never holds, so one typo would spin the
		// check loop with the backoff disabled. Number("") is 0, the same failure.
		const rawIntervalMin = Number(config.intervalMin);
		const effectiveIntervalMin =
			Number.isFinite(rawIntervalMin) && rawIntervalMin > 0 ? rawIntervalMin : 30;
		if (effectiveIntervalMin !== rawIntervalMin) {
			console.log(
				chalk.yellow(
					`Check interval "${config.intervalMin}" is not a usable number of minutes; using ${effectiveIntervalMin}`,
				),
			);
		}
		config.intervalMin = effectiveIntervalMin;
		regions = [config];
	}

	// The backoff cap (WS_MAX_BACKOFF_MIN, 360 by default, "0" disables it) and the
	// resource settings; see ws/config.mjs for every variable and its kill switch.
	// The 6 hour cap is a TRADE: long enough that a wedged box stops driving the
	// queue, short enough that a new resVersion still lands the same day.
	const settings = resolveSettings(process.env, {
		savedirRoot: multi ? savedirRoot : dirname(regions[0].savedir),
		log: (m) => console.log(chalk.yellow(m)),
	});

	const scheduler = createScheduler({
		fileLock: settings.fileLock,
		guard: settings.guard,
		deferRetryMs: settings.deferRetryMs,
		maxDeferMs: settings.maxDeferMs,
	});

	const deps = {
		runDownload,
		runUnpack,
		fetchServerVersion,
		fetchHotUpdateList,
		pruneOrphans,
		readStoredVersion,
		writeStoredVersion,
		unpackerIsNewer,
		outputMissingOrEmpty,
		outputLooksTruncated,
		touchExtractStamp,
		readBackoffState,
		writeBackoffState,
		formatBytes,
		dirStats,
	};

	const watchers = regions.map((config) =>
		createRegionWatcher({
			config,
			deps,
			scheduler,
			settings,
			log: makeLogger(multi ? `[${config.serverKey}]` : ""),
		}),
	);
	const servers = await Promise.all(watchers.map((w) => w.listen()));

	// Listen for CTRL+C directly on raw stdin: signal-exit (used by inquirer/ora)
	// patches process.emit and can swallow SIGINT.
	if (process.stdin.isTTY && process.stdin.setRawMode) {
		process.stdin.setRawMode(true);
		process.stdin.resume();
		process.stdin.on("data", (key) => {
			if (key[0] === 0x03) shutdown(scheduler.busy(), servers);
		});
	}

	const lines = [chalk.bold.green(multi ? "WebSocket Servers Running (one process)" : "WebSocket Server Running"), ""];
	for (const c of regions) {
		if (multi) {
			lines.push(
				`${chalk.bold(c.serverKey.padEnd(3))} ws://localhost:${c.port}  ${c.profile}  -t ${c.threads}  every ${c.intervalMin} min at +${c.startDelayMin}`,
			);
		} else {
			lines.push(
				`${chalk.bold("Address:")}  ws://localhost:${c.port}`,
				`${chalk.bold("Server:")}   ${c.serverKey} — ${SERVERS[c.serverKey].label}`,
				`${chalk.bold("Profile:")}  ${c.profile}`,
				`${chalk.bold("Savedir:")}  ${c.savedir}`,
				`${chalk.bold("Output:")}   ${c.outputDir}`,
				`${chalk.bold("Threads:")}  ${c.threads}`,
				`${chalk.bold("Interval:")} ${c.intervalMin} minutes`,
			);
		}
	}
	const p = settings.priority;
	lines.push(
		"",
		`${chalk.bold("Guard:")}    ${p ? `nice ${p.nice}, ionice class ${p.ioniceClass}` : "normal priority"}; unpack -j cap ${settings.maxUnpackJobs || "none"}; min free ${settings.guard ? `${settings.guard.minFreeMb} MiB` : "off"}; lock ${settings.fileLock?.path ?? "off"}`,
		"",
		chalk.dim("Press Ctrl+C to stop"),
	);
	console.log(
		boxen(lines.join("\n"), {
			padding: 1,
			margin: { top: 1, bottom: 0, left: 1, right: 1 },
			borderStyle: "round",
			borderColor: "green",
		}),
	);

	for (const c of regions) {
		if (stampBaselineMissing(c.savedir)) {
			console.log(
				chalk.yellow(
					`${multi ? `[${c.serverKey}] ` : ""}Truncation check is INERT for this region: .last_extract carries no onDisk baseline, so a truncated tree will not be detected. It re-arms on the next successful extract.`,
				),
			);
		}
	}

	// `shutdown`'s first argument means "an update is running". The previous code
	// passed `!updating`, so an idle SIGTERM logged "during an active update" and
	// exited 1 while a busy one claimed it was safe; fixed here.
	process.on("SIGTERM", () => shutdown(scheduler.busy(), servers));
	await Promise.all(watchers.map((w) => w.startSchedule()));
}


// ─── Global SIGINT ──────────────────────────────────────────────────────────
// Track active child processes so CTRL+C/SIGTERM/etc. can kill them and exit cleanly
const activeChildren = new Set();

const shutdown = (updateRunning, ws_servers) => {
	for (const wss of [ws_servers ?? []].flat()) {
		wss.close();
	}

	for (const child of activeChildren) {
		try {
			child.kill("SIGKILL");
		} catch {}
	}

	if (updateRunning) {
		console.log(chalk.red("Shut down called for during an active update, assets may be incomplete, stale, or corrupted."));
		process.exit(1);
	} else {
		console.log(chalk.dim("No active update running, shutting down safely."));
		process.exit(0);
	}
}

// NOTE: Do NOT use readline.createInterface here — it puts stdin into raw mode
// which intercepts CTRL+C (0x03) and prevents the process-level SIGINT from
// firing. Instead, rely on process.on("SIGINT") which works when stdin is in
// normal (cooked) mode.
process.on("SIGINT", () => shutdown(true));

// ─── Main Menu ──────────────────────────────────────────────────────────────

// ─── CLI argv parsing (non-interactive entry) ──────────────────────────────
// Usage: node run.mjs ws [--server en] [--savedir ./ArkAssets] [--output ./output]
//        [--keep-orphans] [--dry-orphans]   (the post-extract sweep, see runUnpack)
//                        [--threads N] [--port 9160] [--interval 30]
const argv = process.argv.slice(2);
const cliAction = argv[0] && !argv[0].startsWith("--") ? argv[0] : null;
const cliArgs = {};
for (let i = cliAction ? 1 : 0; i < argv.length; i++) {
	const a = argv[i];
	if (!a.startsWith("--")) continue;
	const key = a.slice(2);
	const next = argv[i + 1];
	if (next !== undefined && !next.startsWith("--")) {
		cliArgs[key] = next;
		i++;
	} else {
		cliArgs[key] = true;
	}
}

if (cliAction) {
	const platformLabel = isMac ? "macOS" : isWindows ? "Windows" : "Linux";
	console.log(chalk.dim(`Platform: ${platformLabel} (${process.arch})`));
	switch (cliAction) {
		case "setup":
			await runSetup();
			break;
		case "update":
			await runUpdate();
			break;
		case "ws":
			await runWebSocketServer({ nonInteractive: true, cliArgs });
			break;
		default:
			console.log(chalk.red(`Unknown command: ${cliAction}`));
			console.log("Valid commands: setup, update, ws");
			process.exit(1);
	}
} else {
	console.log(
		boxen(chalk.bold.cyan("Arknights Asset Pipeline"), {
			padding: 1,
			margin: 1,
			borderStyle: "round",
			borderColor: "cyan",
		}),
	);

	const platformLabel = isMac ? "macOS" : isWindows ? "Windows" : "Linux";
	console.log(chalk.dim(`Platform: ${platformLabel} (${process.arch})\n`));

	const hasBinaries = binariesExist();
	if (!hasBinaries) {
		console.log(
			chalk.yellow("Binaries not found. Run Setup first to build them.\n"),
		);
	}

	const choices = [
		{ name: "Setup (build binaries)", value: "setup" },
		...(hasBinaries
			? [
					{ name: "Check for Updates & Update", value: "update" },
					{ name: "WebSocket Server (continuous updates)", value: "ws" },
				]
			: []),
	];

	const { action } = await inquirer.prompt([
		{
			type: "list",
			name: "action",
			message: "What would you like to do?",
			choices,
		},
	]);

	switch (action) {
		case "setup":
			await runSetup();
			break;
		case "update":
			await runUpdate();
			break;
		case "ws":
			await runWebSocketServer();
			break;
	}
}
