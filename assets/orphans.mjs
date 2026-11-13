// The orphan sweep that makes an in-place extract behave like a replacement.
//
// The unpacker is non-incremental: every file it produces is written on every run. It
// writes straight into the served output tree, so a file the exporter no longer produces
// (a texture pool replaced 2,781 per-call PNGs, a page rename left 21 old pages) stays on
// disk and keeps being served, unreferenced, after every re-extract. The output tree is
// tens of gigabytes, so extracting into a staging copy and swapping is not an option; the
// export is swept instead: after a run, every file in a subtree the run wrote into whose
// mtime predates the run's start is an orphan. Subtrees the run did not touch (a filtered
// extract, `--spine` only) are left alone, so a partial run never deletes what it did not
// re-produce. Every removed path is logged next to the output tree, the way the local
// install (`install_spine.sh`, rsync --delete --itemize-changes) records its deletions.
//
// `derived/` is never swept. The unpacker writes nothing there: it holds the backend's own
// caches and archives (story art, release art, farm stages, the gacha pool-detail sidecar),
// written at request or job time, so "older than the run" says nothing about whether a file
// is stale. A backend write landing during an extract used to mark the subtree as written,
// and the sweep then deleted everything else in it, archives that cannot be rebuilt included.
//
// The sweep is also bounded. "Older than the run" means "no longer produced" only when the
// run rewrote everything it should have; an extract that lost its disk part-way (read
// errors, a full disk) rewrites a fraction and leaves the rest looking like orphans, and
// sweeping them deletes the last good copy of every file the run failed to rewrite. A real
// orphan set is a small share of a subtree (2,781 pooled PNGs, 21 renamed pages), so a
// subtree where more than `maxFraction` of the files would go is left alone and reported:
// the S3 sync refuses its deletes the same way when the local tree is under 90% of the
// remote. `allowShrink` (run.mjs: `--allow-sweep-shrink`, WS_ALLOW_SWEEP_SHRINK=1) is the way through
// for an export that really did drop that much.
import { readdirSync, rmdirSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import { join, relative, sep } from "node:path";

/** Clock slack between "the run started" and the first file's mtime, in ms. */
const MTIME_SLACK_MS = 2000;

/** The share of a subtree's files a sweep may remove before it is refused. */
export const DEFAULT_SWEEP_MAX_FRACTION = 0.1;

/**
 * Below this many orphans a subtree is always swept: a handful of renamed files in a
 * small subtree is a large share of it and no sign of a failed run.
 */
export const SWEEP_GUARD_MIN_FILES = 25;

/**
 * The sweep limit from its env spelling (`WS_SWEEP_MAX_FRACTION`): a number above 0 and
 * at most 1, where 1 never refuses. Unset or blank takes the default; anything else takes
 * the default too and says so, because a typo must not turn the guard off.
 * @param {string | undefined} raw
 * @returns {{fraction: number, invalid: boolean}}
 */
export function parseSweepMaxFraction(raw) {
	if (raw === undefined || raw.trim() === "") return { fraction: DEFAULT_SWEEP_MAX_FRACTION, invalid: false };
	const n = Number(raw);
	if (!Number.isFinite(n) || n <= 0 || n > 1) return { fraction: DEFAULT_SWEEP_MAX_FRACTION, invalid: true };
	return { fraction: n, invalid: false };
}

/**
 * Whether removing `orphans` of a subtree's `files` is more than a sweep may take.
 * @param {number} orphans
 * @param {number} files
 * @param {number} maxFraction
 */
export function sweepWouldShrink(orphans, files, maxFraction) {
	return orphans > SWEEP_GUARD_MIN_FILES && orphans > files * maxFraction;
}

/** Top-level subtrees the extract does not own, so the sweep never enters them. */
const NOT_EXTRACT_OUTPUT = new Set(["derived"]);

/**
 * Walk `dir` depth first, yielding every regular file's path and stat.
 * @param {string} dir
 * @returns {Generator<{path: string, mtimeMs: number, size: number, name: string}>}
 */
function* walk(dir) {
	let entries;
	try {
		entries = readdirSync(dir, { withFileTypes: true });
	} catch {
		return;
	}
	for (const e of entries) {
		const p = join(dir, e.name);
		if (e.isSymbolicLink()) continue;
		if (e.isDirectory()) {
			yield* walk(p);
		} else if (e.isFile()) {
			let st;
			try {
				st = statSync(p);
			} catch {
				continue;
			}
			yield { path: p, mtimeMs: st.mtimeMs, size: st.size, name: e.name };
		}
	}
}

/**
 * Whether any file under `dir` was written at or after `startedAt`.
 * @param {string} dir
 * @param {number} startedAt
 */
function subtreeWasWritten(dir, startedAt) {
	for (const f of walk(dir)) {
		if (f.mtimeMs >= startedAt - MTIME_SLACK_MS) return true;
	}
	return false;
}

/**
 * Remove (or list) every file under `outputDir` that the extract which began at
 * `startedAt` did not write, within the top-level subtrees it did write into.
 *
 * @param {string} outputDir - The tree the extract wrote into (e.g. `./output/en`).
 * @param {number} startedAt - `Date.now()` taken just before the unpacker was spawned.
 * @param {{dry?: boolean, logDir?: string, label?: string, maxFraction?: number, allowShrink?: boolean}} [opts] -
 *   `dry` lists without removing; `logDir` is where the deletion log lands (default: the
 *   output tree's parent); `label` names the log (default: the output dir's basename);
 *   `maxFraction` is the share of a subtree's files the sweep may remove (default
 *   `DEFAULT_SWEEP_MAX_FRACTION`); `allowShrink` sweeps whatever the share.
 * @returns {{files: number, bytes: number, skipped: string[], refused: {name: string, orphans: number, files: number, bytes: number}[], log: string | null, dry: boolean}}
 *   `refused` names each subtree left alone because the sweep would have removed more
 *   than `maxFraction` of it; its files are counted in neither `files` nor `bytes`.
 */
export function sweepOrphans(outputDir, startedAt, opts = {}) {
	const dry = !!opts.dry;
	const maxFraction = opts.maxFraction ?? DEFAULT_SWEEP_MAX_FRACTION;
	const result = { files: 0, bytes: 0, skipped: [], refused: [], log: null, dry };
	let top;
	try {
		top = readdirSync(outputDir, { withFileTypes: true });
	} catch {
		return result;
	}
	const orphans = [];
	const kept = [];
	for (const e of top) {
		if (!e.isDirectory()) continue;
		const sub = join(outputDir, e.name);
		if (NOT_EXTRACT_OUTPUT.has(e.name) || !subtreeWasWritten(sub, startedAt)) {
			result.skipped.push(e.name);
			continue;
		}
		let files = 0;
		const found = [];
		for (const f of walk(sub)) {
			files++;
			if (f.name === ".DS_Store" || f.mtimeMs < startedAt - MTIME_SLACK_MS) found.push(f);
		}
		if (!opts.allowShrink && sweepWouldShrink(found.length, files, maxFraction)) {
			result.refused.push({ name: e.name, orphans: found.length, files, bytes: found.reduce((sum, f) => sum + f.size, 0) });
			kept.push(...found);
			continue;
		}
		orphans.push(...found);
	}
	orphans.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
	const stamp = new Date().toISOString().replace(/[-:]/g, "").replace(/\.\d+Z$/, "");
	const label = opts.label ?? outputDir.split(sep).filter(Boolean).at(-1) ?? "output";
	const logDir = opts.logDir ?? join(outputDir, "..");
	const logPath = join(logDir, `orphans_${label}_${stamp}${dry ? ".dry" : ""}.log`);
	kept.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
	const lines = [
		...orphans.map((f) => `${dry ? "would delete" : "deleting"} ${relative(outputDir, f.path)} (${f.size} bytes)`),
		// What a refused subtree would have lost, so the log shows whether it was a failed run.
		...kept.map((f) => `kept (sweep refused) ${relative(outputDir, f.path)} (${f.size} bytes)`),
	];
	try {
		writeFileSync(logPath, `${lines.join("\n")}${lines.length ? "\n" : ""}`);
		result.log = logPath;
	} catch {
		result.log = null;
	}
	const emptied = new Set();
	for (const f of orphans) {
		result.files++;
		result.bytes += f.size;
		if (dry) continue;
		try {
			unlinkSync(f.path);
			emptied.add(join(f.path, ".."));
		} catch {
			// Reported by the count of what remains on the next run rather than aborting a sweep.
		}
	}
	// Directories the sweep emptied are removed too, so a renamed skin leaves no shell.
	if (!dry) {
		for (const d of [...emptied].sort((a, b) => b.length - a.length)) {
			try {
				if (readdirSync(d).length === 0) rmdirSync(d);
			} catch {
				// Not empty, or gone already.
			}
		}
	}
	return result;
}
