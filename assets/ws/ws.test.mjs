// node --test ws/   (from assets/). Fakes stand in for the downloader, the unpacker
// and the CDN; nothing here touches a game server or the real asset tree.

import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import WebSocket from "ws";
import { parseServers, resolveSettings } from "./config.mjs";
import { createRegionWatcher, silentLogger } from "./region.mjs";
import {
	createFileLock,
	createResourceGuard,
	createScheduler,
	lowPriorityCommand,
} from "./scheduler.mjs";

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Fake side effects shared by every region; `active` counts download+extract in flight. */
function fakeWorld({ workMs = 25, failDownload = new Set() } = {}) {
	const world = {
		active: 0,
		maxActive: 0,
		extractsActive: 0,
		maxExtracts: 0,
		events: [],
		stored: new Map(),
		backoff: new Map(),
	};
	const enter = (kind, region) => {
		world.active++;
		world.maxActive = Math.max(world.maxActive, world.active);
		if (kind === "unpack") {
			world.extractsActive++;
			world.maxExtracts = Math.max(world.maxExtracts, world.extractsActive);
		}
		world.events.push(`${kind}:start:${region}`);
	};
	const leave = (kind, region) => {
		world.active--;
		if (kind === "unpack") world.extractsActive--;
		world.events.push(`${kind}:end:${region}`);
	};
	const regionOf = (dir) => dir.split(/[\\/]/).pop();
	world.deps = {
		async runDownload({ serverKey, onProgress, priority }) {
			world.lastDownloadPriority = priority;
			enter("download", serverKey);
			await sleep(workMs);
			onProgress?.({ completed: 1, total: 2, percent: 50 });
			await sleep(workMs);
			leave("download", serverKey);
			if (failDownload.has(serverKey)) throw new Error("fake download failure");
			return { downloaded: 2, failed: 0, totalBytes: 2048 };
		},
		async runUnpack({ outputDir, jobs, onProgress, priority }) {
			const region = regionOf(outputDir);
			world.lastUnpack = { jobs, priority };
			enter("unpack", region);
			await sleep(workMs);
			onProgress?.({ completed: 10, total: 0, percent: -1 });
			await sleep(workMs);
			leave("unpack", region);
			return { exported: 10, onDisk: 10 };
		},
		fetchServerVersion: async () => ({ resVersion: "v2", clientVersion: "c2" }),
		fetchHotUpdateList: async () => ({ abInfos: [] }),
		pruneOrphans: () => ({ deleted: 0, freedBytes: 0 }),
		readStoredVersion: (savedir) => world.stored.get(savedir) ?? "v1",
		writeStoredVersion: (savedir, v) => world.stored.set(savedir, v),
		unpackerIsNewer: () => false,
		outputMissingOrEmpty: () => false,
		outputLooksTruncated: async () => false,
		touchExtractStamp: () => {},
		readBackoffState: (savedir) => world.backoff.get(savedir) ?? { consecutiveFailures: 0, nextAttemptAt: 0 },
		writeBackoffState: (savedir, consecutiveFailures, nextAttemptAt) =>
			world.backoff.set(savedir, { consecutiveFailures, nextAttemptAt }),
		formatBytes: (b) => `${b} B`,
		dirStats: async () => ({ size: 5, fileCount: 1 }),
	};
	return world;
}

const REGIONS = [
	{ server: "en", profile: "full", startDelayMin: 0 },
	{ server: "cn", profile: "operators,release", startDelayMin: 15 },
	{ server: "jp", profile: "gamedata", startDelayMin: 7 },
	{ server: "kr", profile: "gamedata", startDelayMin: 22 },
];

const baseSettings = {
	maxBackoffMs: 360 * 60_000,
	alignToClock: true,
	allowPartialDownload: false,
	maxUnpackJobs: 0,
	priority: null,
	guard: null,
};

async function buildRegions(world, { scheduler = createScheduler(), settings = {}, root } = {}) {
	root ??= mkdtempSync(join(tmpdir(), "ws-test-"));
	const watchers = REGIONS.map(({ server, profile, startDelayMin }) =>
		createRegionWatcher({
			config: {
				serverKey: server,
				savedir: join(root, "ArkAssets", server),
				outputDir: join(root, "output", server),
				threads: 1,
				profile,
				port: 0, // ephemeral, read back from the server
				intervalMin: 30,
				startDelayMin,
			},
			deps: world.deps,
			scheduler,
			settings: { ...baseSettings, ...settings },
			log: silentLogger,
		}),
	);
	await Promise.all(watchers.map((w) => w.listen()));
	return { watchers, scheduler, root };
}

/** A client that records every message; `next(pred)` waits for a matching one. */
async function connect(watcher) {
	const ws = new WebSocket(`ws://127.0.0.1:${watcher.wss.address().port}`);
	const msgs = [];
	const waiters = [];
	ws.on("message", (raw) => {
		const m = JSON.parse(raw.toString());
		msgs.push(m);
		for (const w of [...waiters]) {
			if (w.pred(m)) {
				waiters.splice(waiters.indexOf(w), 1);
				w.resolve(m);
			}
		}
	});
	await new Promise((r) => ws.once("open", r));
	return {
		ws,
		msgs,
		send: (o) => ws.send(typeof o === "string" ? o : JSON.stringify(o)),
		next: (pred, ms = 3000) => {
			const hit = msgs.find(pred);
			if (hit) return Promise.resolve(hit);
			return new Promise((resolve, reject) => {
				const w = { pred, resolve };
				waiters.push(w);
				setTimeout(() => reject(new Error("timed out waiting for message")), ms);
			});
		},
	};
}

async function closeAll(watchers, clients = []) {
	for (const c of clients) c.ws.close();
	await Promise.all(watchers.map((w) => w.close()));
}

test("four regions in one process never run two downloads or extracts at once", async () => {
	const world = fakeWorld();
	const { watchers, scheduler } = await buildRegions(world);
	// Every region wakes at once (what the stagger used to prevent), and every
	// region also gets a manual force_update while its scheduled job is pending.
	const subs = watchers.map((w) => w.tick());
	const refused = watchers.map((w) => w.forceUpdate(null));
	assert.ok(subs.every((s) => s?.accepted));
	assert.ok(refused.every((r) => r === null), "a second job for a pending region is refused");
	await scheduler.idle();
	// Then four manual force_updates at once, the case the old phase never covered.
	watchers.forEach((w) => w.forceUpdate(null));
	await scheduler.idle();

	assert.equal(world.maxActive, 1, "download/extract concurrency");
	assert.equal(world.maxExtracts, 1, "extract concurrency");
	assert.equal(world.events.filter((e) => e.startsWith("unpack:start")).length, 8);
	// Strictly alternating start/end: nothing begins before the previous thing ends.
	for (let i = 0; i < world.events.length; i += 2) {
		const [kind, , region] = world.events[i].split(":");
		assert.equal(world.events[i + 1], `${kind}:end:${region}`);
	}
	await closeAll(watchers);
});

test("queue order is FIFO by submission; duplicates are refused; a manual job waits its turn", async () => {
	const world = fakeWorld();
	const { watchers, scheduler } = await buildRegions(world);
	const [en, cn, jp, kr] = watchers;
	const jpClient = await connect(jp);

	en.tick();
	cn.tick();
	assert.equal(en.tick(), null, "en is already pending");
	const manual = jp.forceUpdate(null);
	assert.equal(manual.accepted, true);
	assert.equal(manual.position, 2, "two jobs ahead of jp: en running, cn queued");
	kr.tick();

	// jp's socket says it is queued, and behind whom.
	const queued = await jpClient.next((m) => m.type === "status" && m.state === "queued");
	assert.deepEqual(queued.queue, { position: 2, behind: "en" });

	await scheduler.idle();
	assert.deepEqual(
		scheduler.history.map((h) => `${h.region}:${h.kind}`),
		["en:scheduled", "cn:scheduled", "jp:manual", "kr:scheduled"],
	);
	const unpackOrder = world.events.filter((e) => e.startsWith("unpack:start")).map((e) => e.split(":")[2]);
	assert.deepEqual(unpackOrder, ["en", "cn", "jp", "kr"]);
	await closeAll(watchers, [jpClient]);
});

test("socket protocol is the legacy one: message types, order, fields, command replies", async () => {
	const world = fakeWorld();
	const { watchers, scheduler, root } = await buildRegions(world);
	const en = watchers[0];
	mkdirSync(join(root, "output", "en", "gamedata"), { recursive: true });
	writeFileSync(join(root, "output", "en", "a.json"), "{}");
	const c = await connect(en);

	const hello = await c.next((m) => m.type === "status");
	assert.deepEqual(Object.keys(hello), ["type", "state", "version", "nextCheckAt", "backoff"]);
	assert.equal(hello.state, "idle");
	assert.deepEqual(hello.version, { current: "v1" });
	assert.deepEqual(Object.keys(hello.backoff), ["consecutiveFailures", "nextAttemptAt"]);

	c.send({ type: "force_update" });
	await c.next((m) => m.type === "status" && m.state === "downloading");
	c.send({ type: "force_update" });
	const busy = await c.next((m) => m.type === "error");
	assert.deepEqual(busy, { type: "error", message: "Update already in progress" });

	const done = await c.next((m) => m.type === "update_complete");
	assert.deepEqual(done, { type: "update_complete", version: "v2", downloaded: 2, failed: 0, exported: 10 });
	await scheduler.idle();
	await sleep(20);

	const seq = c.msgs
		.slice(1)
		.filter((m) => m.type !== "error")
		.map((m) => (m.type === "status" ? `status:${m.state}` : m.type));
	assert.deepEqual(seq, [
		"status:downloading",
		"download_progress",
		"download_complete",
		"prune_complete",
		"status:unpacking",
		"unpack_progress",
		"update_complete",
		"status:idle",
	]);
	const dc = c.msgs.find((m) => m.type === "download_complete");
	assert.deepEqual(Object.keys(dc), ["type", "downloaded", "failed", "totalBytes", "totalBytesFormatted"]);
	assert.deepEqual(c.msgs.find((m) => m.type === "download_progress"), {
		type: "download_progress",
		completed: 1,
		total: 2,
		percent: 50,
	});

	c.send({ type: "list_resources" });
	const list = await c.next((m) => m.type === "resource_list");
	assert.deepEqual(Object.keys(list), ["type", "files", "totalSize", "totalSizeFormatted"]);
	assert.deepEqual(list.files.map((f) => `${f.name}:${f.type}`).sort(), ["a.json:file", "gamedata:directory"]);

	c.send({ type: "nope" });
	assert.deepEqual(await c.next((m) => m.message?.startsWith("Unknown")), {
		type: "error",
		message: "Unknown command: nope",
	});
	c.send("{not json");
	assert.deepEqual(await c.next((m) => m.message === "Invalid JSON"), { type: "error", message: "Invalid JSON" });
	await closeAll(watchers, [c]);
});

test("each region keeps its own port and only hears its own messages", async () => {
	const world = fakeWorld();
	const { watchers, scheduler } = await buildRegions(world);
	const ports = watchers.map((w) => w.wss.address().port);
	assert.equal(new Set(ports).size, 4);
	const clients = await Promise.all(watchers.map(connect));
	watchers[1].forceUpdate(null); // cn only
	await scheduler.idle();
	await sleep(20);
	assert.ok(clients[1].msgs.some((m) => m.type === "update_complete"));
	for (const i of [0, 2, 3]) {
		assert.ok(!clients[i].msgs.some((m) => m.type === "update_complete"), `${watchers[i].key} heard cn`);
	}
	await closeAll(watchers, clients);
});

test("a failed scheduled run arms the backoff; a failed manual run does not", async () => {
	const world = fakeWorld({ failDownload: new Set(["en"]) });
	const { watchers, scheduler } = await buildRegions(world);
	const en = watchers[0];
	const c = await connect(en);
	en.tick();
	await scheduler.idle();
	const err = await c.next((m) => m.type === "error");
	assert.equal(err.message, "Update failed: fake download failure");
	assert.equal(err.consecutiveFailures, 1);
	assert.equal(en.tick(), null, "backing off");
	assert.equal(en.state, "backing_off");
	en.forceUpdate(null);
	await scheduler.idle();
	assert.equal(en.statusMessage().backoff.consecutiveFailures, 1, "manual failure leaves the counter");
	await closeAll(watchers, [c]);
});

test("memory guard defers a job below the threshold, runs it once memory returns, skips it past the limit", async () => {
	let mb = 100;
	const guard = createResourceGuard({ minFreeMb: 500, readMb: () => mb, waitMs: 30, pollMs: 5 });
	const world = fakeWorld();
	const scheduler = createScheduler({ guard, deferRetryMs: 10, maxDeferMs: 10_000 });
	const { watchers } = await buildRegions(world, { scheduler, settings: { guard } });
	const c = await connect(watchers[0]);
	watchers[0].tick();
	const deferred = await c.next((m) => m.type === "status" && m.state === "queued" && m.message);
	assert.match(deferred.message, /Deferred: 100 MiB available, need 500 MiB/);
	assert.equal(world.events.length, 0, "nothing ran while memory was low");
	mb = 900;
	await scheduler.idle();
	assert.ok(world.events.includes("unpack:end:en"));

	// Skip: memory never returns within maxDeferMs.
	mb = 100;
	const s2 = createScheduler({ guard, deferRetryMs: 5, maxDeferMs: 20 });
	let skipped = null;
	s2.submit("kr", "scheduled", async () => assert.fail("must not run"), { onSkip: (x) => (skipped = x) });
	await s2.idle();
	assert.deepEqual(skipped, { ok: false, availableMb: 100, minFreeMb: 500 });
	await closeAll(watchers, [c]);
});

test("memory guard before the extract: the download is kept, the extract fails into the backoff", async () => {
	let mb = 900;
	const guard = createResourceGuard({ minFreeMb: 500, readMb: () => mb, waitMs: 20, pollMs: 5 });
	const world = fakeWorld();
	const realDownload = world.deps.runDownload;
	world.deps.runDownload = async (o) => {
		const r = await realDownload(o);
		mb = 100; // memory drops while downloading
		return r;
	};
	const { watchers, scheduler } = await buildRegions(world, { settings: { guard } });
	const c = await connect(watchers[0]);
	watchers[0].tick();
	await scheduler.idle();
	const err = await c.next((m) => m.type === "error");
	assert.match(err.message, /only 100 MiB available, need 500 MiB/);
	assert.ok(!world.events.some((e) => e.startsWith("unpack")), "no extract started");
	assert.equal(err.consecutiveFailures, 1);
	await closeAll(watchers, [c]);
});

test("unpack -j is clamped by WS_MAX_UNPACK_JOBS and the priority reaches both binaries", async () => {
	const world = fakeWorld();
	const priority = { nice: 10, ioniceClass: 2, ioniceLevel: 7 };
	const scheduler = createScheduler();
	const w = createRegionWatcher({
		config: {
			serverKey: "en",
			savedir: "/x/ArkAssets/en",
			outputDir: "/x/output/en",
			threads: 4,
			profile: "full",
			port: 0,
			intervalMin: 30,
			startDelayMin: 0,
		},
		deps: world.deps,
		scheduler,
		settings: { ...baseSettings, maxUnpackJobs: 2, priority },
		log: silentLogger,
	});
	w.forceUpdate(null);
	await scheduler.idle();
	assert.deepEqual(world.lastUnpack, { jobs: 2, priority });
	assert.deepEqual(world.lastDownloadPriority, priority);
});

test("lowPriorityCommand wraps in ionice + nice, and is inert when switched off", () => {
	const all = () => true;
	assert.deepEqual(lowPriorityCommand("/b/unpacker", ["extract"], { nice: 10, ioniceClass: 2, ioniceLevel: 7 }, all), [
		"ionice",
		["-c", "2", "-n", "7", "nice", "-n", "10", "/b/unpacker", "extract"],
	]);
	assert.deepEqual(
		lowPriorityCommand("/b/unpacker", ["extract"], { nice: 10, ioniceClass: 3 }, (c) => c === "nice"),
		["nice", ["-n", "10", "/b/unpacker", "extract"]],
		"no ionice binary (macOS): nice only",
	);
	assert.deepEqual(lowPriorityCommand("/b/unpacker", ["extract"], null, all), ["/b/unpacker", ["extract"]]);
	assert.deepEqual(lowPriorityCommand("/b/u", ["x"], { nice: 0, ioniceClass: 0 }, all), ["/b/u", ["x"]]);
});

test("kill switches: every resource setting turns off and restores the legacy values", () => {
	const off = resolveSettings(
		{ WS_NICE: "0", WS_IONICE_CLASS: "0", WS_MIN_FREE_MB: "0", WS_LOCK: "0", WS_MAX_UNPACK_JOBS: "0", WS_MAX_BACKOFF_MIN: "0" },
		{ savedirRoot: "/tmp", platform: "linux", log: () => {} },
	);
	assert.equal(off.priority, null);
	assert.equal(off.guard, null);
	assert.equal(off.fileLock, null);
	assert.equal(off.maxUnpackJobs, 0);
	assert.equal(off.maxBackoffMs, 0);

	const on = resolveSettings({ WS_MAX_BACKOFF_MIN: "" }, { savedirRoot: "/tmp/r", platform: "linux", log: () => {} });
	assert.deepEqual(on.priority, { nice: 10, ioniceClass: 2, ioniceLevel: 7 });
	assert.equal(on.guard.minFreeMb, 1536);
	assert.equal(on.fileLock.path, "/tmp/r/.watcher.lock");
	assert.equal(on.maxBackoffMs, 360 * 60_000, "a blank cap takes the default, not 0");
	assert.equal(
		resolveSettings({}, { savedirRoot: "/tmp", platform: "darwin", log: () => {} }).guard,
		null,
		"no default memory guard off Linux (no MemAvailable)",
	);
});

test("parseServers reads the ecosystem config and rejects collisions", () => {
	const eco = REGIONS.map((r, i) => ({ ...r, port: 9160 + i }));
	const parsed = parseServers(JSON.stringify(eco), { savedir: "./ArkAssets", outputDir: "./output", threads: 1, intervalMin: 30 }, [
		"en",
		"cn",
		"jp",
		"kr",
	]);
	assert.deepEqual(
		parsed.map((p) => [p.serverKey, p.port, p.profile, p.startDelayMin, p.savedir, p.outputDir]),
		[
			["en", 9160, "full", 0, "ArkAssets/en", "output/en"],
			["cn", 9161, "operators,release", 15, "ArkAssets/cn", "output/cn"],
			["jp", 9162, "gamedata", 7, "ArkAssets/jp", "output/jp"],
			["kr", 9163, "gamedata", 22, "ArkAssets/kr", "output/kr"],
		],
	);
	const d = { savedir: ".", outputDir: ".", threads: 1, intervalMin: 30 };
	assert.throws(() => parseServers('[{"server":"en","port":1},{"server":"cn","port":1}]', d, ["en", "cn"]), /port 1 listed twice/);
	assert.throws(() => parseServers('[{"server":"xx","port":1}]', d, ["en"]), /unknown server/);
	assert.throws(() => parseServers("[]", d, ["en"]), /non-empty/);
});

test("file lock: a live holder in another process blocks, a dead one is taken over", async () => {
	const dir = mkdtempSync(join(tmpdir(), "ws-lock-"));
	const path = join(dir, ".watcher.lock");
	const other = spawn(process.execPath, ["-e", "setTimeout(() => {}, 10000)"], { stdio: "ignore" });
	await new Promise((r) => other.once("spawn", r));
	writeFileSync(path, JSON.stringify({ pid: other.pid, owner: "cn:scheduled", since: Date.now() }));

	const lock = createFileLock(path, { pollMs: 10, heartbeatMs: 1000 });
	let waitedOn = null;
	let got = false;
	const p = lock.acquire("en:manual", { onWait: (h) => (waitedOn = h) }).then((release) => {
		got = true;
		return release;
	});
	await sleep(80);
	assert.equal(got, false, "blocked while the other process is alive");
	assert.equal(waitedOn.owner, "cn:scheduled");

	other.kill("SIGKILL");
	const release = await p;
	assert.equal(got, true, "taken over once the holder died");
	assert.equal(lock.holder().pid, process.pid);
	release();
	assert.equal(existsSync(path), false, "released");
});

test("legacy single-region mode speaks the same protocol through the same code", async () => {
	// One scheduler per region is what `run.mjs ws --server en` builds.
	const world = fakeWorld();
	const w = createRegionWatcher({
		config: {
			serverKey: "en",
			savedir: "/x/ArkAssets/en",
			outputDir: "/x/output/en",
			threads: 1,
			profile: "full",
			port: 0,
			intervalMin: 30,
			startDelayMin: 0,
		},
		deps: world.deps,
		scheduler: createScheduler(),
		settings: baseSettings,
		log: silentLogger,
	});
	await w.listen();
	const c = await connect(w);
	c.send({ type: "force_update" });
	await c.next((m) => m.type === "update_complete");
	await sleep(20);
	assert.deepEqual(
		c.msgs.map((m) => (m.type === "status" ? `status:${m.state}` : m.type)),
		[
			"status:idle",
			"status:downloading",
			"download_progress",
			"download_complete",
			"prune_complete",
			"status:unpacking",
			"unpack_progress",
			"update_complete",
			"status:idle",
		],
	);
	await closeAll([w], [c]);
});
