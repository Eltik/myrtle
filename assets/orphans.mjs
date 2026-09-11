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
import { readdirSync, rmdirSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import { join, relative, sep } from "node:path";

/** Clock slack between "the run started" and the first file's mtime, in ms. */
const MTIME_SLACK_MS = 2000;

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
 * @param {{dry?: boolean, logDir?: string, label?: string}} [opts] - `dry` lists without
 *   removing; `logDir` is where the deletion log lands (default: the output tree's parent);
 *   `label` names the log (default: the output dir's basename).
 * @returns {{files: number, bytes: number, skipped: string[], log: string | null, dry: boolean}}
 */
export function sweepOrphans(outputDir, startedAt, opts = {}) {
	const dry = !!opts.dry;
	const result = { files: 0, bytes: 0, skipped: [], log: null, dry };
	let top;
	try {
		top = readdirSync(outputDir, { withFileTypes: true });
	} catch {
		return result;
	}
	const orphans = [];
	for (const e of top) {
		if (!e.isDirectory()) continue;
		const sub = join(outputDir, e.name);
		if (!subtreeWasWritten(sub, startedAt)) {
			result.skipped.push(e.name);
			continue;
		}
		for (const f of walk(sub)) {
			if (f.name === ".DS_Store" || f.mtimeMs < startedAt - MTIME_SLACK_MS) orphans.push(f);
		}
	}
	orphans.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
	const stamp = new Date().toISOString().replace(/[-:]/g, "").replace(/\.\d+Z$/, "");
	const label = opts.label ?? outputDir.split(sep).filter(Boolean).at(-1) ?? "output";
	const logDir = opts.logDir ?? join(outputDir, "..");
	const logPath = join(logDir, `orphans_${label}_${stamp}${dry ? ".dry" : ""}.log`);
	const lines = orphans.map((f) => `${dry ? "would delete" : "deleting"} ${relative(outputDir, f.path)} (${f.size} bytes)`);
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
