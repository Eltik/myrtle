// node --test disk-guard.test.mjs
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, it } from "node:test";

import {
	DEFAULT_MIN_FREE_GB,
	LOCK_STALE_MS,
	acquireExtractLock,
	checkDiskBudget,
	freeBytes,
	lockIsStale,
	parseMinFreeGb,
	pendingRewriteBytes,
	readExtractLock,
	startDiskWatchdog,
} from "./disk-guard.mjs";

const GIB = 2 ** 30;
let dir;
beforeEach(() => {
	dir = mkdtempSync(join(tmpdir(), "disk-guard-"));
});
afterEach(() => {
	rmSync(dir, { recursive: true, force: true });
});

describe("parseMinFreeGb", () => {
	it("takes the default when unset or blank", () => {
		for (const raw of [undefined, "", "   "]) {
			assert.deepEqual(parseMinFreeGb(raw), {
				gb: DEFAULT_MIN_FREE_GB,
				bytes: DEFAULT_MIN_FREE_GB * GIB,
				invalid: false,
			});
		}
	});
	it("reads a number, fractions included", () => {
		assert.equal(parseMinFreeGb("12").bytes, 12 * GIB);
		assert.equal(parseMinFreeGb("0.5").bytes, GIB / 2);
	});
	it("turns off only on an explicit 0", () => {
		assert.deepEqual(parseMinFreeGb("0"), { gb: 0, bytes: 0, invalid: false });
	});
	it("falls back to the default, flagged, on an unusable value", () => {
		for (const raw of ["5GB", "-1", "NaN"]) {
			const parsed = parseMinFreeGb(raw);
			assert.equal(parsed.gb, DEFAULT_MIN_FREE_GB);
			assert.equal(parsed.invalid, true);
		}
	});
});

describe("checkDiskBudget", () => {
	const minFreeBytes = 5 * GIB;
	it("passes when the rewrite fits with the margin to spare", () => {
		assert.deepEqual(checkDiskBudget({ free: 36 * GIB, rewriteBytes: 25 * GIB, minFreeBytes }), {
			ok: true,
			shortBy: 0,
		});
	});
	it("refuses the 2026-11-11 case: 21 GiB free, 25 GiB to put back", () => {
		const budget = checkDiskBudget({ free: 21 * GIB, rewriteBytes: 25 * GIB, minFreeBytes });
		assert.equal(budget.ok, false);
		assert.equal(budget.shortBy, 9 * GIB);
	});
	it("refuses when the margin alone is already gone", () => {
		assert.equal(checkDiskBudget({ free: 2 * GIB, minFreeBytes }).ok, false);
	});
	it("fails open when free space is unknown or the check is off", () => {
		assert.equal(checkDiskBudget({ free: null, rewriteBytes: 99 * GIB, minFreeBytes }).ok, true);
		assert.equal(checkDiskBudget({ free: 1, rewriteBytes: 99 * GIB, minFreeBytes: 0 }).ok, true);
	});
});

describe("freeBytes", () => {
	it("reads the disk of a path that does not exist yet", () => {
		const free = freeBytes(join(dir, "not", "here", "yet"));
		assert.equal(typeof free, "number");
		assert.ok(free > 0);
	});
});

describe("pendingRewriteBytes", () => {
	/** A region tree with a manifest; `files` maps rel path to on-disk bytes. */
	function tree({ prefix = "myrtle", objects, files }) {
		const out = join(dir, "en");
		for (const [rel, bytes] of Object.entries(files)) {
			mkdirSync(join(out, rel, ".."), { recursive: true });
			writeFileSync(join(out, rel), "x".repeat(bytes));
		}
		mkdirSync(out, { recursive: true });
		writeFileSync(join(out, ".asset-sync.json"), JSON.stringify({ version: 2, bucket: "assets", prefix, objects }));
		return out;
	}
	const rec = (size, evicted) => ({ size, md5: "m", etag: "e", ...(evicted ? { evicted: true } : {}) });

	it("counts only files that are evicted AND still 0 bytes on disk", async () => {
		const out = tree({
			objects: {
				"myrtle/en/textures/a.png": rec(1000, true), // stub: counts
				"myrtle/en/textures/b.png": rec(2000, true), // flagged, already rewritten
				"myrtle/en/textures/c.png": rec(4000, false), // never evicted
				"myrtle/en/audio/gone.m4a": rec(8000, true), // flagged, swept away
				"myrtle/cn/textures/a.png": rec(16000, true), // another region's key
			},
			files: { "textures/a.png": 0, "textures/b.png": 2000, "textures/c.png": 4000 },
		});
		assert.deepEqual(await pendingRewriteBytes(out), { known: true, bytes: 1000, stubs: 1 });
	});
	it("handles an empty prefix", async () => {
		const out = tree({
			prefix: "",
			objects: { "en/spine/x.skel": rec(512, true) },
			files: { "spine/x.skel": 0 },
		});
		assert.deepEqual(await pendingRewriteBytes(out), { known: true, bytes: 512, stubs: 1 });
	});
	it("batches past the concurrency limit", async () => {
		const objects = {};
		const files = {};
		for (let i = 0; i < 25; i++) {
			objects[`myrtle/en/t/${i}.png`] = rec(10, true);
			files[`t/${i}.png`] = 0;
		}
		const out = tree({ objects, files });
		assert.deepEqual(await pendingRewriteBytes(out, { concurrency: 4 }), { known: true, bytes: 250, stubs: 25 });
	});
	it("is unknown without a readable manifest", async () => {
		assert.deepEqual(await pendingRewriteBytes(join(dir, "nope")), { known: false, bytes: 0, stubs: 0 });
		mkdirSync(join(dir, "jp"));
		writeFileSync(join(dir, "jp", ".asset-sync.json"), "{ not json");
		assert.deepEqual(await pendingRewriteBytes(join(dir, "jp")), { known: false, bytes: 0, stubs: 0 });
	});
});

describe("startDiskWatchdog", () => {
	it("calls onLow on every low poll and stops when told to", async () => {
		const readings = [9, 3, 9, 2, 1];
		const seen = [];
		const stop = startDiskWatchdog({
			path: dir,
			floorBytes: 5,
			intervalMs: 5,
			free: () => readings.shift() ?? 9,
			onLow: (free) => seen.push(free),
		});
		await new Promise((resolve) => setTimeout(resolve, 80));
		stop();
		assert.deepEqual(seen, [3, 2, 1]);
	});
	it("does nothing with a zero floor or an unreadable disk", async () => {
		let calls = 0;
		const off = startDiskWatchdog({ path: dir, floorBytes: 0, intervalMs: 5, onLow: () => calls++ });
		const blind = startDiskWatchdog({ path: dir, floorBytes: 5, intervalMs: 5, free: () => null, onLow: () => calls++ });
		await new Promise((resolve) => setTimeout(resolve, 40));
		off();
		blind();
		assert.equal(calls, 0);
	});
});

describe("extract lock", () => {
	const lock = () => join(dir, "output", ".extract.lock");
	const fake = (fields, ageMs = 0) => {
		mkdirSync(join(dir, "output"), { recursive: true });
		writeFileSync(lock(), typeof fields === "string" ? fields : JSON.stringify(fields));
		const at = new Date(Date.now() - ageMs);
		utimesSync(lock(), at, at);
	};

	it("is taken, read back, refused to a second caller, and released", async () => {
		const first = await acquireExtractLock(lock(), { server: "en" });
		assert.equal(first.ok, true);
		const holder = readExtractLock(lock());
		assert.equal(holder.pid, process.pid);
		assert.equal(holder.server, "en");

		// Same process, second caller: must not treat its own pid as stale.
		const second = await acquireExtractLock(lock(), { server: "cn" });
		assert.equal(second.ok, false);
		assert.equal(second.holder.server, "en");

		first.release();
		assert.equal(readExtractLock(lock()), null);
		first.release(); // idempotent

		const third = await acquireExtractLock(lock(), { server: "cn" });
		assert.equal(third.ok, true);
		third.release();
	});

	it("is refused while another live watcher holds it", async () => {
		fake({ pid: 424242, server: "en", token: "theirs" });
		const got = await acquireExtractLock(lock(), { server: "cn", alive: () => true });
		assert.equal(got.ok, false);
		assert.equal(got.holder.server, "en");
		assert.equal(JSON.parse(readFileSync(lock(), "utf-8")).token, "theirs");
	});

	it("is taken over from a dead holder", async () => {
		fake({ pid: 424242, server: "en", token: "theirs" });
		const got = await acquireExtractLock(lock(), { server: "cn", settleMs: 5, alive: () => false });
		assert.equal(got.ok, true);
		assert.equal(readExtractLock(lock()).server, "cn");
		got.release();
	});

	it("is taken over from a live pid that stopped touching it", async () => {
		fake({ pid: 424242, server: "en", token: "theirs" }, LOCK_STALE_MS + 1000);
		const got = await acquireExtractLock(lock(), { server: "cn", settleMs: 5, alive: () => true });
		assert.equal(got.ok, true);
		got.release();
	});

	it("leaves a fresh unparseable lock alone, and takes an old one", async () => {
		fake("");
		assert.equal((await acquireExtractLock(lock(), { settleMs: 5 })).ok, false);
		fake("", LOCK_STALE_MS + 1000);
		const got = await acquireExtractLock(lock(), { settleMs: 5 });
		assert.equal(got.ok, true);
		got.release();
	});

	it("does not remove a lock that was taken over while it was held", async () => {
		const got = await acquireExtractLock(lock(), { server: "en" });
		fake({ pid: 424242, server: "cn", token: "theirs" });
		got.release();
		assert.equal(readExtractLock(lock()).token, "theirs");
	});

	it("staleness: dead pid, own pid and an old heartbeat are stale; a live fresh one is not", () => {
		const now = Date.now();
		const base = { server: "en", token: "t", at: null, mtimeMs: now };
		assert.equal(lockIsStale({ ...base, pid: 7 }, now, () => true), false);
		assert.equal(lockIsStale({ ...base, pid: 7 }, now, () => false), true);
		assert.equal(lockIsStale({ ...base, pid: process.pid }, now, () => true), true);
		assert.equal(lockIsStale({ ...base, pid: 7, mtimeMs: now - LOCK_STALE_MS - 1 }, now, () => true), true);
		assert.equal(lockIsStale({ ...base, pid: null }, now, () => true), false);
	});
});
