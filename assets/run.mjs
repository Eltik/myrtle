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
		versionUrl:
			"https://ark-kr-static-online.yo-star.com/assetbundle/official/Android/version",
		cdnBaseUrl:
			"https://ark-kr-static-online.yo-star.com/assetbundle/official/Android/assets",
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
	profile
}) {
    const args = ["--server", serverKey, "-d", savedir, "-t", String(threads), "download", "--all"];
    if (profile) args.push("--profile", profile);
	return new Promise((resolve, reject) => {
		const child = spawn(
			DOWNLOADER_BIN,
			args,
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
}) {
	return new Promise((resolve, reject) => {
		const child = spawn(
			UNPACKER_BIN,
			["extract", "-i", inputDir, "-o", outputDir, "-j", String(jobs)],
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

async function runWebSocketServer({ nonInteractive = false, cliArgs = {} } = {}) {
	if (!binariesExist()) {
		console.log(chalk.red("\nBinaries not found. Run Setup first.\n"));
		return;
	}

	const defaults = {
		serverKey: cliArgs.server ?? process.env.WS_SERVER ?? "en",
		savedir: cliArgs.savedir ?? process.env.WS_SAVEDIR ?? "./ArkAssets",
		outputDir: cliArgs.output ?? process.env.WS_OUTPUT ?? "./output",
		threads: Number(cliArgs.threads ?? process.env.WS_THREADS ?? DEFAULT_THREADS),
		profile: cliArgs.profile ?? process.env.WS_PROFILE ?? "full",
		port: Number(cliArgs.port ?? process.env.WS_PORT ?? 9160),
		intervalMin: Number(cliArgs.interval ?? process.env.WS_INTERVAL ?? 30),
		// Minutes to wait before the FIRST check, and so the phase of every check
		// after it. Two watchers on one box otherwise wake on the same boundary and
		// their extracts overlap. Default 0 leaves the behaviour as it was.
		startDelayMin: Number(
			cliArgs["start-delay"] ?? process.env.WS_START_DELAY_MIN ?? 0,
		),
	};

	const config = nonInteractive
		? defaults
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

	if (!SERVERS[config.serverKey]) {
		console.log(chalk.red(`\nUnknown server: ${config.serverKey}. Valid: ${Object.keys(SERVERS).join(", ")}\n`));
		return;
	}

	config.savedir = join(config.savedir, config.serverKey);
	config.outputDir = join(config.outputDir, config.serverKey);

	// Clamped for exactly the reason the backoff cap is, and it matters more now
	// that the backoff is a multiple of it. Number("30m") is NaN: setInterval
	// coerces NaN to 1 ms, so the check loop spins, while `Date.now() < NaN` is
	// always false, so the backoff guard can never hold. One typo'd WS_INTERVAL
	// would reproduce the original incident and disable the fix for it in the same
	// stroke. Number("") is 0, which is the same failure with a busier loop.
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
	const intervalMs = effectiveIntervalMin * 60 * 1000;
	// NaN degrades to 0, which is the previous behaviour, so a garbled value cannot
	// wedge the watcher in a delay it never leaves.
	const startDelayMs = Number.isFinite(config.startDelayMin)
		? config.startDelayMin * 60 * 1000
		: 0;

	// A failed update leaves every trigger that caused it STILL TRUE: `.version`
	// and `.last_extract` are written only on success, so the next tick re-runs the
	// same download and the same extract. With no backoff that is a full re-download
	// and re-extract every intervalMin for as long as the failure lasts, which on a
	// 3-core box against a ~113 GB tree is enough to keep the kernel in writeback
	// and time out the disk.
	//
	// The cap is a TRADE, shipped knowingly, not a derived value: 6 hours is long
	// enough that a wedged box stops driving the queue and short enough that a
	// genuinely new resVersion still lands the same day. Ruled out on the way there:
	// no backoff at all, which is the bug; and a hard attempt cap, which leaves a box
	// stale with no retry once a human has fixed the cause.
	//
	// WS_MAX_BACKOFF_MIN=0 restores the previous behaviour EXACTLY: the delay becomes
	// 0 ms and the `Date.now() < nextAttemptAt` guard can never hold. Parsed for
	// finiteness rather than truthiness, because Number("") is 0 and Number("abc") is
	// NaN, and NaN would silently disable the guard as well.
	//
	// An empty value takes the DEFAULT rather than 0. `process.env` holds strings,
	// so a blanked pm2 entry or a bare `export WS_MAX_BACKOFF_MIN=` reaches
	// Number("") === 0, which is finite and would turn the whole mechanism off with
	// no log line saying so. A negative value is rejected the same way: it makes
	// nextAttemptAt a time in the past, which is 0 wearing a disguise. Only an
	// explicit "0" disables the backoff, and the resolved cap is logged at startup
	// so an operator can read what actually took effect instead of inferring it.
	const rawBackoffMin = process.env.WS_MAX_BACKOFF_MIN;
	const parsedBackoffMin =
		rawBackoffMin === undefined || rawBackoffMin.trim() === ""
			? 360
			: Number(rawBackoffMin);
	const backoffCapMin =
		Number.isFinite(parsedBackoffMin) && parsedBackoffMin >= 0
			? parsedBackoffMin
			: 360;
	if (rawBackoffMin !== undefined && backoffCapMin !== parsedBackoffMin) {
		console.log(
			chalk.yellow(
				`WS_MAX_BACKOFF_MIN="${rawBackoffMin}" is not a usable number of minutes; using ${backoffCapMin}`,
			),
		);
	}
	const maxBackoffMs = backoffCapMin * 60 * 1000;

	/** Delay before the next attempt, after `failures` consecutive failures. */
	const backoffMs = (failures) =>
		Math.min(intervalMs * 2 ** Math.max(0, failures - 1), maxBackoffMs);

	// State
	let currentState = "idle";
	let updating = false;
	let currentVersion = readStoredVersion(config.savedir);
	// Consecutive failed `performUpdate` runs, and the earliest time the next
	// attempt may start. Loaded from disk so a pm2 or OOM restart does not clear a
	// backoff that the restart itself is evidence for. Cleared by a run whose
	// extract succeeds.
	const persistedBackoff = readBackoffState(config.savedir);
	let consecutiveFailures = persistedBackoff.consecutiveFailures;
	// A stored wait further out than the cap can only come from a clock that moved,
	// so it is trimmed rather than honoured. Left alone, one bad clock reading
	// would park a region past any horizon a human would think to look at.
	let nextAttemptAt = Math.min(
		persistedBackoff.nextAttemptAt,
		Date.now() + maxBackoffMs,
	);
	// When the scheduler will next call `checkAndUpdate`. Published so a client can
	// tell a watcher that is between checks from one that has stopped checking.
	let nextCheckAt = 0;
	if (consecutiveFailures > 0) {
		console.log(
			chalk.yellow(
				`Resuming backoff from disk: ${consecutiveFailures} consecutive failure(s), next attempt ${
					nextAttemptAt > Date.now()
						? `in ${Math.ceil((nextAttemptAt - Date.now()) / 60000)} minute(s)`
						: "now"
				}`,
			),
		);
	}

	// WebSocket server
	const wss = new WebSocketServer({ port: config.port });

	// Listen for CTRL+C directly on raw stdin. Libraries like signal-exit
	// (used by inquirer/ora) patch process.emit and install their own SIGINT
	// handlers, which can swallow the signal. Reading raw stdin for 0x03 is
	// the most reliable way to detect CTRL+C regardless of what other
	// libraries do.
	if (process.stdin.isTTY && process.stdin.setRawMode) {
		process.stdin.setRawMode(true);
		process.stdin.resume();
		process.stdin.on("data", (key) => {
			// 0x03 = CTRL+C
			if (key[0] === 0x03) {
				shutdown(!updating, wss);
			}
		});
	}
	const clients = new Set();

	function broadcast(msg) {
		const data = JSON.stringify(msg);
		for (const ws of clients) {
			if (ws.readyState === ws.OPEN) ws.send(data);
		}
	}

	function sendTo(ws, msg) {
		if (ws.readyState === ws.OPEN) ws.send(JSON.stringify(msg));
	}

	// A watcher in a six-hour backoff was INDISTINGUISHABLE from a healthy one over
	// this socket: the backoff guard returned before touching `currentState`, so a
	// client connecting after the one-shot failure broadcast saw `state: "idle"`
	// and a version read off disk, with no way to learn the watcher would not check
	// again for hours. The only trace was a line in the pm2 log. For a mechanism
	// whose whole purpose is to stop work for up to six hours, that state belongs
	// here, or the first symptom anyone gets is stale assets and a green dashboard.
	function statusMessage() {
		return {
			type: "status",
			state: currentState,
			version: { current: currentVersion ?? null },
			nextCheckAt: nextCheckAt || null,
			backoff: {
				consecutiveFailures,
				nextAttemptAt: nextAttemptAt || null,
			},
		};
	}

	// List resources in output directory
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
					// Summarize directory: recursively count every nested file and its bytes.
					const { size: dirSize, fileCount } = await dirStats(fullPath);
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
			sendTo(ws, {
				type: "error",
				message: `Failed to list resources: ${err.message}`,
			});
			return;
		}

		sendTo(ws, {
			type: "resource_list",
			files,
			totalSize,
			totalSizeFormatted: formatBytes(totalSize),
		});
	}

	// Perform download + unpack cycle.
	//
	// `knownVer` is the version `checkAndUpdate` already fetched. Fetching it again
	// after the work is what made a CDN blip expensive: the download can run for
	// hours and the extract for hours more, and a 503 on a fresh version call at
	// the END of that threw into the catch, so neither `.version` nor
	// `.last_extract` was written and the next tick repeated the whole thing. The
	// manual force_update path passes nothing and still fetches, which is fine: it
	// has no earlier fetch to reuse.
	//
	// `manual` marks an operator-initiated force_update. Such a run DELIBERATELY
	// ignores `nextAttemptAt`, because an escape hatch is what makes a six-hour cap
	// tolerable, and its failures do not touch the counter. Sharing one counter
	// meant an operator debugging a broken CDN pushed the automatic retry out with
	// every click: five attempts reached backoffMs(5), pinning the scheduled
	// watcher at the 6 hour cap because a human tried to help, with nothing in the
	// logs connecting the two. A manual SUCCESS still clears the backoff, because
	// what the counter counts is failed extracts and that extract did not fail.
	async function performUpdate(knownVer, { manual = false } = {}) {
		if (updating) return;
		updating = true;

		try {
			// Download phase
			currentState = "downloading";
			broadcast(statusMessage());
			console.log(chalk.blue(`[${new Date().toLocaleTimeString()}] Downloading assets...`));

			const dlStats = await runDownload({
				serverKey: config.serverKey,
				savedir: config.savedir,
				threads: config.threads,
				profile: config.profile,
				onProgress: (p) => broadcast({ type: "download_progress", ...p }),
				onStatus: (msg) =>
					broadcast({ type: "status", state: "downloading", message: msg }),
			});

			console.log(chalk.blue(`[${new Date().toLocaleTimeString()}] Download complete: ${dlStats.downloaded} files, ${dlStats.failed} failed, ${formatBytes(dlStats.totalBytes)}`));

			// A partial download must NOT reach the unpacker. The downloader exits 0
			// whatever its failure count, so this was the only thing standing between a
			// stalled disk and an extract built from an incomplete bundle set, whose
			// orphan sweep then deletes every previously-good output the short run did
			// not re-produce, and whose result is recorded as the new baseline.
			// Throwing here routes into the existing catch: no unpack, no sweep, no
			// stamp, and the backoff arms. WS_ALLOW_PARTIAL_DOWNLOAD=1 is the escape
			// hatch for a region where some bundles are permanently 404.
			if (dlStats.failed > 0 && process.env.WS_ALLOW_PARTIAL_DOWNLOAD !== "1") {
				throw new Error(
					`${dlStats.failed} bundle(s) failed to download; refusing to unpack an incomplete set (WS_ALLOW_PARTIAL_DOWNLOAD=1 to override)`,
				);
			}
			broadcast({
				type: "download_complete",
				downloaded: dlStats.downloaded,
				failed: dlStats.failed,
				totalBytes: dlStats.totalBytes,
				totalBytesFormatted: formatBytes(dlStats.totalBytes),
			});

			// Prune orphans before unpack. Without this, stale .bin bundles from
			// prior versions sharing a TextAsset m_Name with the current bundle
			// can clobber the new content via last-write-wins in the unpacker.
			try {
				const ver = await fetchServerVersion(config.serverKey);
				const hotList = await fetchHotUpdateList(config.serverKey, ver.resVersion);
				const { deleted, freedBytes } = pruneOrphans(config.savedir, hotList);
				if (deleted > 0) {
					console.log(
						chalk.dim(
							`[${new Date().toLocaleTimeString()}] Pruned ${deleted} orphan file(s), freed ${formatBytes(freedBytes)}`,
						),
					);
				}
				broadcast({ type: "prune_complete", deleted, freedBytes });
			} catch (err) {
				console.warn(
					chalk.yellow(
						`[${new Date().toLocaleTimeString()}] Orphan prune skipped: ${err.message}`,
					),
				);
			}

			// Unpack phase
			currentState = "unpacking";
			broadcast(statusMessage());
			console.log(chalk.blue(`[${new Date().toLocaleTimeString()}] Unpacking assets...`));

			const upStats = await runUnpack({
				inputDir: config.savedir,
				outputDir: config.outputDir,
				jobs: config.threads,
				onProgress: (p) => broadcast({ type: "unpack_progress", ...p }),
				onNotice: (message) =>
					broadcast({ type: "status", state: "unpacking", message }),
			});

			// The extract SUCCEEDED at this point, and everything below is recording
			// that. None of it may fall into the catch: doing so discards hours of
			// completed work, leaves every trigger true, and re-runs the whole cycle on
			// the next tick, which is the loop this whole change exists to close.
			//
			// The backoff clears first, before any call that can throw, because it
			// counts failed extracts and this extract did not fail.
			consecutiveFailures = 0;
			nextAttemptAt = 0;
			writeBackoffState(config.savedir, 0, 0);

			// Update stored version and extraction timestamp
			let serverVer = knownVer;
			try {
				if (!serverVer) serverVer = await fetchServerVersion(config.serverKey);
				writeStoredVersion(config.savedir, serverVer.resVersion);
				currentVersion = serverVer.resVersion;
			} catch (err) {
				console.log(
					chalk.yellow(
						`[${new Date().toLocaleTimeString()}] Extract succeeded but the version could not be recorded: ${err.message}. The next check will see a version mismatch and repeat the download.`,
					),
				);
			}
			// Written even when the version could not be: it records the tree that is
			// now on disk, which is true regardless of what the CDN just said.
			try {
				touchExtractStamp(config.savedir, upStats.exported, upStats.onDisk);
			} catch (err) {
				console.log(
					chalk.yellow(
						`[${new Date().toLocaleTimeString()}] Extract succeeded but the stamp could not be written: ${err.message}`,
					),
				);
			}

			currentState = "idle";
			console.log(chalk.green(`[${new Date().toLocaleTimeString()}] Update complete: v${currentVersion}, ${dlStats.downloaded} downloaded, ${upStats.exported} exported, ${upStats.onDisk} on disk`));
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
			console.log(chalk.red(`[${new Date().toLocaleTimeString()}] Update failed: ${err.message}`));
			if (manual) {
				console.log(
					chalk.dim(
						`[${new Date().toLocaleTimeString()}] Manual update, so the automatic backoff is unchanged (${consecutiveFailures} consecutive failure(s) on record)`,
					),
				);
			} else {
				consecutiveFailures += 1;
				const wait = backoffMs(consecutiveFailures);
				nextAttemptAt = Date.now() + wait;
				writeBackoffState(config.savedir, consecutiveFailures, nextAttemptAt);
				console.log(
					chalk.dim(
						`[${new Date().toLocaleTimeString()}] Failure ${consecutiveFailures}; next attempt in ${Math.round(wait / 60000)} minute(s)`,
					),
				);
			}
			broadcast({
				type: "error",
				message: `Update failed: ${err.message}`,
				consecutiveFailures,
				nextAttemptAt,
			});
			broadcast(statusMessage());
		} finally {
			updating = false;
		}
	}

	// Check for updates and trigger download if needed
	async function checkAndUpdate() {
		if (updating) return;

		if (Date.now() < nextAttemptAt) {
			const mins = Math.ceil((nextAttemptAt - Date.now()) / 60000);
			console.log(
				chalk.dim(
					`[${new Date().toLocaleTimeString()}] Backing off after ${consecutiveFailures} failed update(s); next attempt in ${mins} minute(s)`,
				),
			);
			currentState = "backing_off";
			broadcast(statusMessage());
			return;
		}

		try {
			currentState = "checking";
			broadcast(statusMessage());
			console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] Checking for updates...`));

			const serverVer = await fetchServerVersion(config.serverKey);
			const storedVer = readStoredVersion(config.savedir);

			currentState = "idle";

			const needsReextract =
				unpackerIsNewer(config.savedir) ||
				outputMissingOrEmpty(config.outputDir) ||
				(await outputLooksTruncated(config.savedir, config.outputDir));
			if (storedVer === serverVer.resVersion && !needsReextract) {
				console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] Up to date (${storedVer})`));
				broadcast(statusMessage());
				return;
			}

			if (needsReextract) {
				console.log(chalk.yellow(`[${new Date().toLocaleTimeString()}] Re-extraction needed (assets current but output stale)`));
			} else {
				console.log(chalk.yellow(`[${new Date().toLocaleTimeString()}] Update available: ${storedVer ?? "(none)"} → ${serverVer.resVersion}`));
			}

			// Update available (new assets or unpacker rebuild)
			broadcast({
				type: "update_available",
				currentVersion: storedVer ?? null,
				newVersion: serverVer.resVersion,
				clientVersion: serverVer.clientVersion,
			});

			await performUpdate(serverVer);
		} catch (err) {
			currentState = "idle";
			console.log(chalk.red(`[${new Date().toLocaleTimeString()}] Version check failed: ${err.message}`));
			broadcast({
				type: "error",
				message: `Version check failed: ${err.message}`,
			});
			broadcast(statusMessage());
		}
	}

	// Handle client connections
	wss.on("connection", (ws) => {
		console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] Client connected (${clients.size + 1} total)`));
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
					if (updating) {
						sendTo(ws, {
							type: "error",
							message: "Update already in progress",
						});
					} else {
						// Deliberately not gated on `nextAttemptAt`: this is the hatch
						// out of a long backoff, and a person is asking for it.
						performUpdate(undefined, { manual: true });
					}
					break;

				case "list_resources":
					await listResources(ws);
					break;

				default:
					sendTo(ws, {
						type: "error",
						message: `Unknown command: ${msg.type}`,
					});
			}
		});

		ws.on("close", () => {
			clients.delete(ws);
			console.log(chalk.dim(`[${new Date().toLocaleTimeString()}] Client disconnected (${clients.size} remaining)`));
		});
		ws.on("error", () => clients.delete(ws));
	});

	console.log(
		boxen(
			[
				chalk.bold.green("WebSocket Server Running"),
				"",
				`${chalk.bold("Address:")}  ws://localhost:${config.port}`,
				`${chalk.bold("Server:")}   ${config.serverKey} — ${SERVERS[config.serverKey].label}`,
				`${chalk.bold("Profile:")}  ${config.profile}`,
				`${chalk.bold("Savedir:")}  ${config.savedir}`,
				`${chalk.bold("Output:")}   ${config.outputDir}`,
				`${chalk.bold("Threads:")}  ${config.threads}`,
				`${chalk.bold("Interval:")} ${config.intervalMin} minutes`,
				"",
				chalk.dim("Press Ctrl+C to stop"),
			].join("\n"),
			{
				padding: 1,
				margin: { top: 1, bottom: 0, left: 1, right: 1 },
				borderStyle: "round",
				borderColor: "green",
			},
		),
	);

	if (stampBaselineMissing(config.savedir)) {
		console.log(
			chalk.yellow(
				`Truncation check is INERT for this region: .last_extract carries no onDisk baseline, so a truncated tree will not be detected. It re-arms on the next successful extract.`,
			),
		);
	}

	// WS_ALIGN=0 restores the previous scheduling exactly: sleep the stagger, arm
	// the interval from that moment, check immediately.
	const alignToClock = process.env.WS_ALIGN !== "0";

	if (!alignToClock) {
		if (startDelayMs > 0) {
			console.log(
				chalk.dim(
					`[${new Date().toLocaleTimeString()}] Staggered start: waiting ${config.startDelayMin} minute(s) before the first check`,
				),
			);
			await new Promise((resolve) => setTimeout(resolve, startDelayMs));
		}
		nextCheckAt = Date.now() + intervalMs;
		setInterval(() => {
			nextCheckAt = Date.now() + intervalMs;
			void checkAndUpdate();
		}, intervalMs);
		await checkAndUpdate();
		return;
	}

	// Anchored to the WALL clock rather than to process start. A relative offset
	// only holds while both watchers keep the start times they happened to get:
	// pm2 restarting one of them re-phases that one to its own restart moment, and
	// the two drift back onto the same boundary with nothing left to separate
	// them. Anchored to the clock, en fires at :00 and :30 and cn at :15 and :45
	// whenever either process last came up, so a restart cannot collide them.
	//
	// The TRADE, shipped knowingly: there is no immediate check at startup any
	// more, so a deploy can wait up to one interval before the new resVersion is
	// noticed. force_update is how to say "go now" and WS_ALIGN=0 is the way back.
	// Ruled out on the way here: checking immediately and then aligning, which
	// reintroduces the collision on the first restart and so buys nothing.
	const offsetMs = ((startDelayMs % intervalMs) + intervalMs) % intervalMs;
	const scheduleNext = () => {
		const now = Date.now();
		// floor(..) + 1, not ceil(..): a slot landing exactly on `now` must schedule
		// the NEXT one, or the zero-delay timer re-enters itself forever.
		nextCheckAt =
			(Math.floor((now - offsetMs) / intervalMs) + 1) * intervalMs + offsetMs;
		setTimeout(() => {
			scheduleNext();
			void checkAndUpdate();
		}, nextCheckAt - now);
	};
	scheduleNext();
	console.log(
		chalk.dim(
			`[${new Date().toLocaleTimeString()}] Checks aligned to the clock: every ${config.intervalMin} min at offset ${offsetMs / 60000} min; first check at ${new Date(nextCheckAt).toLocaleTimeString()}`,
		),
	);

	process.on("SIGTERM", () => shutdown(!updating, wss));
}

// ─── Global SIGINT ──────────────────────────────────────────────────────────
// Track active child processes so CTRL+C/SIGTERM/etc. can kill them and exit cleanly
const activeChildren = new Set();

const shutdown = (graceful, ws_server) => {
	if (ws_server) {
		ws_server.close()
	}

	for (const child of activeChildren) {
		try {
			child.kill("SIGKILL");
		} catch {}
	}

	if (graceful) {
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
