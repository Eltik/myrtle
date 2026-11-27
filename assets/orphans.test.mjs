// node --test orphans.test.mjs
import assert from "node:assert/strict";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, it } from "node:test";

import { DEFAULT_SWEEP_MAX_FRACTION, SWEEP_GUARD_MIN_FILES, parseSweepMaxFraction, sweepOrphans, sweepWouldShrink } from "./orphans.mjs";

let root;
let out;
/** The moment the pretend extract started; files dated before it are what it did not rewrite. */
let startedAt;
beforeEach(() => {
	root = mkdtempSync(join(tmpdir(), "orphans-"));
	out = join(root, "en");
	mkdirSync(out);
	startedAt = Date.now();
});
afterEach(() => {
	rmSync(root, { recursive: true, force: true });
});

/** Write `count` files under `out/<sub>`, dated an hour before the run (`old`) or just after it began. */
function write(sub, prefix, count, old) {
	const dir = join(out, sub);
	mkdirSync(dir, { recursive: true });
	const at = new Date(old ? startedAt - 3_600_000 : startedAt + 1_000);
	const paths = [];
	for (let i = 0; i < count; i++) {
		const path = join(dir, `${prefix}_${i}.png`);
		writeFileSync(path, "x");
		utimesSync(path, at, at);
		paths.push(path);
	}
	return paths;
}

const present = (paths) => paths.filter((p) => existsSync(p)).length;

describe("parseSweepMaxFraction", () => {
	it("takes the default when unset or blank", () => {
		for (const raw of [undefined, "", "  "]) {
			assert.deepEqual(parseSweepMaxFraction(raw), { fraction: DEFAULT_SWEEP_MAX_FRACTION, invalid: false });
		}
	});
	it("reads a share above 0 and at most 1", () => {
		assert.deepEqual(parseSweepMaxFraction("0.25"), { fraction: 0.25, invalid: false });
		assert.deepEqual(parseSweepMaxFraction("1"), { fraction: 1, invalid: false });
	});
	it("falls back to the default, flagged, on anything else", () => {
		for (const raw of ["0", "-0.1", "1.5", "10%", "half"]) {
			assert.deepEqual(parseSweepMaxFraction(raw), { fraction: DEFAULT_SWEEP_MAX_FRACTION, invalid: true }, raw);
		}
	});
});

describe("sweepWouldShrink", () => {
	it("never refuses a handful of orphans, whatever share they are", () => {
		assert.equal(sweepWouldShrink(SWEEP_GUARD_MIN_FILES, SWEEP_GUARD_MIN_FILES, 0.1), false);
	});
	it("refuses only above the share", () => {
		assert.equal(sweepWouldShrink(100, 1000, 0.1), false);
		assert.equal(sweepWouldShrink(101, 1000, 0.1), true);
		assert.equal(sweepWouldShrink(1000, 1000, 1), false);
	});
});

describe("sweepOrphans", () => {
	it("removes the files a full run did not rewrite", () => {
		const rewritten = write("textures", "kept", 400, false);
		const stale = write("textures", "renamed", 30, true);
		const swept = sweepOrphans(out, startedAt);
		assert.equal(swept.files, 30);
		assert.deepEqual(swept.refused, []);
		assert.equal(present(stale), 0);
		assert.equal(present(rewritten), 400);
	});

	it("leaves a subtree alone when the run rewrote only a fraction of it", () => {
		// What a disk that failed part-way leaves: 40 files rewritten, 360 not.
		const rewritten = write("textures", "written", 40, false);
		const unwritten = write("textures", "unwritten", 360, true);
		const swept = sweepOrphans(out, startedAt);
		assert.equal(swept.files, 0);
		assert.equal(swept.bytes, 0);
		assert.deepEqual(swept.refused, [{ name: "textures", orphans: 360, files: 400, bytes: 360 }]);
		assert.equal(present(unwritten), 360, "the only good copies are still there");
		assert.equal(present(rewritten), 40);
		const log = readFileSync(swept.log, "utf8");
		assert.match(log, /kept \(sweep refused\) textures[/\\]unwritten_0\.png/);
		assert.doesNotMatch(log, /deleting/);
	});

	it("judges each subtree on its own", () => {
		write("textures", "written", 40, false);
		const unwritten = write("textures", "unwritten", 360, true);
		write("spine", "kept", 300, false);
		const staleSpine = write("spine", "renamed", 10, true);
		const swept = sweepOrphans(out, startedAt);
		assert.deepEqual(
			swept.refused.map((r) => r.name),
			["textures"],
		);
		assert.equal(swept.files, 10);
		assert.equal(present(staleSpine), 0);
		assert.equal(present(unwritten), 360);
	});

	it("sweeps a large share when told the export really shrank", () => {
		write("textures", "written", 40, false);
		const unwritten = write("textures", "unwritten", 360, true);
		const swept = sweepOrphans(out, startedAt, { allowShrink: true });
		assert.equal(swept.files, 360);
		assert.deepEqual(swept.refused, []);
		assert.equal(present(unwritten), 0);
	});

	it("takes the share it is given", () => {
		write("textures", "written", 40, false);
		const unwritten = write("textures", "unwritten", 360, true);
		assert.equal(sweepOrphans(out, startedAt, { maxFraction: 0.95, dry: true }).refused.length, 0);
		assert.equal(sweepOrphans(out, startedAt, { maxFraction: 0.5, dry: true }).refused.length, 1);
		assert.equal(present(unwritten), 360, "a dry run removes nothing");
	});

	it("still skips subtrees the run did not touch, and never enters derived/", () => {
		const untouched = write("audio", "old", 50, true);
		write("derived", "fresh", 1, false);
		const archives = write("derived", "archive", 50, true);
		write("textures", "kept", 100, false);
		const swept = sweepOrphans(out, startedAt);
		assert.deepEqual(swept.skipped.sort(), ["audio", "derived"]);
		assert.equal(present(untouched), 50);
		assert.equal(present(archives), 50);
	});
});
