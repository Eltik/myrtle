import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { coerceProgress, emptyProgress, isStoryRead, loadProgress, PROGRESS_KEY, type StoryPosition, type StoryProgress, saveProgress } from "./progress";

const getMock = vi.fn();
const putMock = vi.fn();

// The two server functions are the only thing in here that would cross a
// network. Mocking them is what lets the engine below be driven a tick at a
// time; everything else, including the localStorage document, is real.
const importMock = vi.fn();
vi.mock("#/lib/api/storyProgress", () => ({
    getStoryProgressFn: () => getMock(),
    importStoryProgressFn: () => importMock(),
    putStoryProgressFn: (args: unknown) => putMock(args),
}));

const {
    __enableStorySyncForTests,
    __pullStorySyncForTests,
    __resetStorySyncForTests,
    __resolveStorySyncForTests,
    __reverdictStorySyncForTests,
    REPULL_AFTER_MS,
    __storyGameImportForTests,
    __storyGameReadForTests,
    __storySyncStateForTests,
    DEFAULT_SYNC_POLICY,
    flushStorySync,
    merge,
    PUSH_DEBOUNCE_MS,
    sameProgress,
    setStorySyncPolicy,
    SYNC_POLICY_KEY,
} = await import("./sync");

const at = (halt: number, ts: number, choices: Record<number, string> = {}): StoryPosition => ({ halt, total: 40, ts, choices });

const doc = (p: Partial<StoryProgress>): StoryProgress => ({ ...emptyProgress(), ...p });
/** A document as the OLD client wrote it: the version that may carry baked game marks. */
const v1 = (p: Partial<StoryProgress>): StoryProgress => ({ ...emptyProgress(), ...p, v: 1 });

describe("merge", () => {
    it("unions the read marks", () => {
        const local = doc({ read: { a: 1, b: 1 } });
        const remote = doc({ read: { b: 1, c: 1 } });
        expect(merge(local, remote).read).toEqual({ a: 1, b: 1, c: 1 });
        expect(merge(remote, local).read).toEqual({ a: 1, b: 1, c: 1 });
    });

    it("keeps the position with the larger ts, from either side", () => {
        const local = doc({ pos: { s1: at(3, 200), s2: at(9, 100) } });
        const remote = doc({ pos: { s1: at(7, 100), s2: at(2, 300), s3: at(1, 50) } });
        const got = merge(local, remote).pos;
        // The HALT is the newer side's; the older side's further halt on the
        // same (empty) choices rides along as the reach.
        expect(got.s1).toEqual({ ...at(3, 200), reach: 7 });
        expect(got.s2).toEqual({ ...at(2, 300), reach: 9 });
        expect(got.s3).toEqual(at(1, 50));
    });

    it("carries the larger reach of two positions on the same choices, whichever wins the halt", () => {
        // This device jumped back to 16 after reading to 85; the other one is at 40 and older.
        const local = doc({ pos: { s1: { ...at(16, 300, { 0: "a" }), reach: 85 } } });
        const remote = doc({ pos: { s1: at(40, 100, { 0: "a" }) } });
        expect(merge(local, remote).pos.s1).toEqual({ ...at(16, 300, { 0: "a" }), reach: 85 });
        // The newer side is behind the older side's halt: the halt is still the newer one's.
        const older = doc({ pos: { s1: at(85, 100, { 0: "a", 1: "b" }) } });
        const newer = doc({ pos: { s1: at(16, 300, { 0: "a" }) } });
        const got = merge(newer, older).pos.s1;
        expect(got.halt).toBe(16);
        expect(got.reach).toBe(85);
        // The reach was read on the older side's later choice, so that choice travels with it.
        expect(got.choices).toEqual({ 0: "a", 1: "b" });
        expect(merge(older, newer).pos.s1).toEqual(got);
    });

    it("does not carry a reach across two positions that took different options", () => {
        const local = doc({ pos: { s1: at(16, 300, { 0: "a" }) } });
        const remote = doc({ pos: { s1: { ...at(85, 100, { 0: "b" }), reach: 90 } } });
        expect(merge(local, remote).pos.s1).toEqual(at(16, 300, { 0: "a" }));
    });

    it("a merge carrying a reach is idempotent", () => {
        const local = doc({ pos: { s1: at(16, 300, { 0: "a" }) } });
        const remote = doc({ pos: { s1: at(85, 100, { 0: "a", 1: "c" }) } });
        const once = merge(local, remote);
        expect(merge(once, remote)).toEqual(once);
        expect(merge(local, once)).toEqual(once);
    });

    it("keeps the local entry on a tie", () => {
        const local = doc({ pos: { s1: at(3, 500) } });
        const remote = doc({ pos: { s1: at(8, 500) } });
        expect(merge(local, remote).pos.s1.halt).toBe(3);
    });

    it("takes the last whose position is newer", () => {
        const local = doc({ pos: { s1: at(3, 100) }, last: "s1" });
        const remote = doc({ pos: { s2: at(4, 900) }, last: "s2" });
        expect(merge(local, remote).last).toBe("s2");
        expect(merge(remote, local).last).toBe("s2");
    });

    it("takes the only last there is, and none when there is none", () => {
        expect(merge(doc({ last: "s1" }), doc({})).last).toBe("s1");
        expect(merge(doc({}), doc({ last: "s2" })).last).toBe("s2");
        expect(merge(doc({}), doc({})).last).toBeUndefined();
    });

    it("prefers a last that names a position over one that names none", () => {
        const local = doc({ last: "ghost" });
        const remote = doc({ pos: { s2: at(4, 1) }, last: "s2" });
        expect(merge(local, remote).last).toBe("s2");
    });

    it("keeps the local last when neither names a position", () => {
        expect(merge(doc({ last: "a" }), doc({ last: "b" })).last).toBe("a");
    });

    it("is idempotent: merging a merged document changes nothing", () => {
        const local = doc({ read: { a: 1 }, pos: { s1: at(3, 200) }, last: "s1" });
        const remote = doc({ read: { b: 1 }, pos: { s2: at(4, 900) }, last: "s2" });
        const once = merge(local, remote);
        expect(merge(once, remote)).toEqual(once);
        expect(merge(local, once)).toEqual(once);
    });

    it("always stamps v: 2, whatever the two sides carry", () => {
        expect(merge(emptyProgress(), emptyProgress()).v).toBe(2);
        expect(merge(v1({}), v1({})).v).toBe(2);
    });
});

describe("sameProgress", () => {
    it("ignores key order", () => {
        const a: StoryProgress = { v: 2, read: { x: 1, y: 1 }, pos: { s1: at(1, 2, { 0: "a", 1: "b" }), s2: at(3, 4) }, last: "s1" };
        const b: StoryProgress = { v: 2, read: { y: 1, x: 1 }, pos: { s2: at(3, 4), s1: at(1, 2, { 1: "b", 0: "a" }) }, last: "s1" };
        expect(sameProgress(a, b)).toBe(true);
    });

    it("separates a different halt, ts, choice, read mark or last", () => {
        const base: StoryProgress = { v: 2, read: { x: 1 }, pos: { s1: at(1, 2, { 0: "a" }) }, last: "s1" };
        expect(sameProgress(base, { ...base, pos: { s1: at(2, 2, { 0: "a" }) } })).toBe(false);
        expect(sameProgress(base, { ...base, pos: { s1: at(1, 3, { 0: "a" }) } })).toBe(false);
        expect(sameProgress(base, { ...base, pos: { s1: at(1, 2, { 0: "b" }) } })).toBe(false);
        expect(sameProgress(base, { ...base, read: { x: 1, y: 1 } })).toBe(false);
        expect(sameProgress(base, { ...base, last: "s2" })).toBe(false);
    });

    it("separates a different reach, and encodes a position without one as it always did", () => {
        const base: StoryProgress = { v: 2, read: {}, pos: { s1: at(1, 2) } };
        expect(sameProgress(base, { ...base, pos: { s1: { ...at(1, 2), reach: 9 } } })).toBe(false);
        expect(sameProgress({ ...base, pos: { s1: { ...at(1, 2), reach: 9 } } }, { ...base, pos: { s1: { ...at(1, 2), reach: 9 } } })).toBe(true);
    });

    it("separates a v1 document from a v2 one carrying the same marks", () => {
        const marks = { read: { x: 1 as const }, pos: { s1: at(1, 2) }, last: "s1" };
        expect(sameProgress(v1(marks), doc(marks))).toBe(false);
    });

    it("a merge of two equal documents equals both", () => {
        const a: StoryProgress = { v: 2, read: { x: 1 }, pos: { s1: at(1, 2) }, last: "s1" };
        expect(sameProgress(merge(a, a), a)).toBe(true);
    });
});

describe("merge and hand-cleared marks", () => {
    it("keeps the NEWER of a read stamp and a clear on the same id, and a legacy 1 loses to any clear", () => {
        const local = doc({ read: { a: 10, b: 1, c: 30 } });
        const remote = doc({ read: {}, unread: { a: 20, b: 5, c: 25, d: 40 } });
        const got = merge(local, remote);
        expect(got.read).toEqual({ c: 30 });
        expect(got.unread).toEqual({ a: 20, b: 5, d: 40 });
        // Symmetric: the same answer whichever side holds which.
        const flipped = merge(remote, local);
        expect(flipped.read).toEqual(got.read);
        expect(flipped.unread).toEqual(got.unread);
        // A clear counts as a change worth pushing.
        expect(sameProgress(doc({ read: {} }), doc({ read: {}, unread: { z: 1 } }))).toBe(false);
    });
});

describe("merge and the game's mis-imported marks", () => {
    it("withdraws a refused LEGACY mark from both sides, whatever their version, and stamps the result v2", () => {
        const local = v1({ read: { a: 1, b: 1 } });
        const remote = doc({ read: { b: 1, c: 1 } });
        const got = merge(local, remote, ["b"]);
        expect(got.read).toEqual({ a: 1, c: 1 });
        expect(got.v).toBe(2);
    });

    it("never withdraws a TIMESTAMPED mark: the reader made it here, whatever the game says", () => {
        expect(merge(doc({ read: { b: 1_700_000_000_000 } }), emptyProgress(), ["b"]).read).toEqual({ b: 1_700_000_000_000 });
        expect(merge(emptyProgress(), v1({ read: { b: 1_700_000_000_000 } }), ["b"]).read).toEqual({ b: 1_700_000_000_000 });
        // A legacy 1 on one side and the reader's own mark on the other: the reader's mark wins the union.
        expect(merge(v1({ read: { b: 1 } }), doc({ read: { b: 5 } }), ["b"]).read).toEqual({ b: 5 });
    });

    it("keeps the withdrawal idempotent and leaves every mark the game does not refuse", () => {
        const once = merge(v1({ read: { a: 1, b: 1 }, pos: { s1: at(3, 200) }, last: "s1" }), emptyProgress(), ["b"]);
        expect(once.read).toEqual({ a: 1 });
        expect(merge(once, once, ["b"])).toEqual(once);
    });

    it("changes nothing but the version when the unread list is empty", () => {
        const local = v1({ read: { a: 1 } });
        const remote = v1({ read: { b: 1 } });
        expect(merge(local, remote, [])).toEqual(merge(local, remote));
        expect(merge(local, remote, []).read).toEqual({ a: 1, b: 1 });
    });
});

// ---------------------------------------------------------------------------
// The engine: what actually reaches the account, and how often.
// ---------------------------------------------------------------------------

const pullAnswer = (over: Partial<{ progress: StoryProgress | null; updatedAt: number | null; gameRead: string[]; gameUnread: string[]; gameArchived: number; gameSyncedAt: number | null }> = {}) => ({
    ok: true as const,
    progress: null,
    updatedAt: null,
    gameRead: [] as string[],
    gameUnread: [] as string[],
    gameArchived: 0,
    gameSyncedAt: null,
    ...over,
});

const pushedDocs = (): StoryProgress[] => putMock.mock.calls.map((c) => (c[0] as { data: StoryProgress }).data);

describe("the sync engine", () => {
    beforeEach(() => {
        window.localStorage.clear();
        getMock.mockReset();
        putMock.mockReset();
        putMock.mockResolvedValue({ ok: true, updatedAt: 1 });
        __resetStorySyncForTests();
        __enableStorySyncForTests();
    });

    afterEach(() => {
        __resetStorySyncForTests();
        vi.useRealTimers();
    });

    // The game's verdict is now read BESIDE the document. Nothing it says
    // enters `read`, which is what lets the backend correct it later; the
    // library still shows those stories read, through `isStoryRead`.
    it("keeps the game's read marks out of the document and writes nothing for them", async () => {
        const gameRead = ["main_0_level_main_00-01_beg", "main_0_level_main_00-01_end"];
        getMock.mockResolvedValue(pullAnswer({ gameRead, gameSyncedAt: 1_700_000_000 }));
        await __pullStorySyncForTests();

        expect(loadProgress().read).toEqual({});
        expect(putMock).toHaveBeenCalledTimes(0);

        const set = __storyGameReadForTests();
        expect([...set].sort()).toEqual([...gameRead].sort());
        expect(isStoryRead(loadProgress(), set, gameRead[0])).toBe(true);
        expect(isStoryRead(loadProgress(), set, "main_0_level_main_00-02_beg")).toBe(false);
    });

    // "Sync now" is a RE-DERIVATION on the backend, never a refresh: it calls
    // the import route and reads its answer exactly as a pull would, so a
    // corrected verdict withdraws and adds marks without the game being asked.
    it("re-derives through the import route on Sync now, and the automatic pull does not", async () => {
        importMock.mockResolvedValue(pullAnswer({ gameRead: ["main_8_level_st_08-01"], gameUnread: ["act15d0_level_act15d0_07_end"], gameSyncedAt: 1_700_000_000 }));
        getMock.mockResolvedValue(pullAnswer({ gameRead: [], gameSyncedAt: 1_700_000_000 }));
        saveProgress(v1({ read: { act15d0_level_act15d0_07_end: 1, own_level_1_beg: 1 } }));
        await __reverdictStorySyncForTests();

        expect(importMock).toHaveBeenCalledTimes(1);
        expect(getMock).toHaveBeenCalledTimes(0);
        expect(loadProgress().v).toBe(2);
        expect(loadProgress().read).toEqual({ own_level_1_beg: 1 });
        expect([...__storyGameReadForTests()]).toEqual(["main_8_level_st_08-01"]);

        await __pullStorySyncForTests();
        expect(importMock).toHaveBeenCalledTimes(1);
        expect(getMock).toHaveBeenCalledTimes(1);
    });

    // The game answers out of TWO records and the response names both counts:
    // `gameRead` is their union and `gameArchived` is the Archive half of it.
    // The mainline lives only in the other half, which is why the union is the
    // number that reaches `read` and the Archive count is only ever quoted.
    it("takes the union of both game sources and keeps the Archive count beside it", async () => {
        const archive = ["1stact_level_a001_01_beg", "1stact_level_a001_01_end"];
        const played = ["main_0_level_main_00-01_beg", "main_0_level_main_00-01_end", "main_0_level_main_00-02_beg"];
        getMock.mockResolvedValue(pullAnswer({ gameRead: [...archive, ...played], gameArchived: archive.length, gameSyncedAt: 1_700_000_000 }));
        await __pullStorySyncForTests();

        expect([...__storyGameReadForTests()].sort()).toEqual([...archive, ...played].sort());
        expect(loadProgress().read).toEqual({});
        expect(__storyGameImportForTests()).toEqual({ count: 5, archived: 2, at: 1_700_000_000 });
        expect(putMock).toHaveBeenCalledTimes(0);
    });

    it("upgrades the account's v1 document, withdrawing what the old import baked into it", async () => {
        const held = { v: 1 as const, read: { a: 1 as const, b: 1 as const }, pos: {} };
        getMock.mockResolvedValue(pullAnswer({ progress: held, updatedAt: 2, gameRead: ["a", "b"], gameUnread: ["b"], gameSyncedAt: 1_700_000_500 }));
        await __pullStorySyncForTests();

        expect(putMock).toHaveBeenCalledTimes(1);
        expect(pushedDocs()[0].read).toEqual({ a: 1 });
        expect(pushedDocs()[0].v).toBe(2);
        expect(loadProgress().read).toEqual({ a: 1 });
    });

    it("upgrades a v1 document sitting in localStorage and pushes it as v2", async () => {
        window.localStorage.setItem(PROGRESS_KEY, JSON.stringify({ v: 1, read: { a: 1, b: 1 }, pos: {} }));
        expect(loadProgress().v).toBe(1);

        getMock.mockResolvedValue(pullAnswer({ gameRead: ["a", "b"], gameUnread: ["b"], gameSyncedAt: 1_700_000_500 }));
        await __pullStorySyncForTests();

        const stored = loadProgress();
        expect(stored.v).toBe(2);
        expect(stored.read).toEqual({ a: 1 });
        expect(putMock).toHaveBeenCalledTimes(1);
        expect(pushedDocs()[0]).toMatchObject({ v: 2, read: { a: 1 } });
    });

    // Same marks, different version: the account is still holding the claim
    // that its marks came partly from the game, and that claim has to be
    // withdrawn even when no id moves.
    it("pushes a v1 document that loses nothing, because the version itself changed", async () => {
        const held = { v: 1 as const, read: { a: 1 as const }, pos: {} };
        getMock.mockResolvedValue(pullAnswer({ progress: held, updatedAt: 2, gameUnread: ["b"], gameSyncedAt: 1_700_000_500 }));
        await __pullStorySyncForTests();

        expect(putMock).toHaveBeenCalledTimes(1);
        expect(pushedDocs()[0]).toMatchObject({ v: 2, read: { a: 1 } });

        // The account holds the v2 document now; a repeat pull writes nothing.
        getMock.mockResolvedValue(pullAnswer({ progress: coerceProgress({ v: 2, read: { a: 1 }, pos: {} }), updatedAt: 3, gameUnread: ["b"], gameSyncedAt: 1_700_000_500 }));
        await __pullStorySyncForTests();
        expect(putMock).toHaveBeenCalledTimes(1);
    });

    it("shares one round trip between concurrent pulls", async () => {
        window.localStorage.setItem(PROGRESS_KEY, JSON.stringify({ v: 1, read: { a: 1, b: 1 }, pos: {} }));
        getMock.mockResolvedValue(pullAnswer({ gameRead: ["a"], gameUnread: ["b"], gameSyncedAt: 1 }));
        await Promise.all([__pullStorySyncForTests(), __pullStorySyncForTests(), __pullStorySyncForTests()]);
        expect(getMock).toHaveBeenCalledTimes(1);
        expect(putMock).toHaveBeenCalledTimes(1);
    });

    it("debounces trailing-only: three writes inside the window cost one PUT, of the last one", () => {
        vi.useFakeTimers();
        for (const halt of [1, 2, 3]) {
            saveProgress({ v: 2, read: {}, pos: { s1: { halt, total: 40, ts: 1000 + halt, choices: {} } } });
            vi.advanceTimersByTime(500);
        }
        expect(putMock).toHaveBeenCalledTimes(0);
        vi.advanceTimersByTime(PUSH_DEBOUNCE_MS);
        expect(putMock).toHaveBeenCalledTimes(1);
        expect(pushedDocs()[0].pos.s1.halt).toBe(3);
    });

    it("sends what the window is still holding when it is flushed", () => {
        vi.useFakeTimers();
        saveProgress({ v: 2, read: { a: 1 }, pos: {} });
        expect(putMock).toHaveBeenCalledTimes(0);
        flushStorySync();
        expect(putMock).toHaveBeenCalledTimes(1);
        expect(pushedDocs()[0].read).toEqual({ a: 1 });
        vi.advanceTimersByTime(PUSH_DEBOUNCE_MS * 2);
        expect(putMock).toHaveBeenCalledTimes(1);
    });

    it("leaves the local document alone when the pull fails", async () => {
        saveProgress({ v: 2, read: { a: 1 }, pos: {} });
        const before = window.localStorage.getItem(PROGRESS_KEY);
        getMock.mockResolvedValue({ ok: false, reason: "error" });
        await __pullStorySyncForTests();
        expect(window.localStorage.getItem(PROGRESS_KEY)).toBe(before);
    });
});

// ---------------------------------------------------------------------------
// Conflicts: the pull that refuses to decide for the reader.
// ---------------------------------------------------------------------------

describe("a conflict between the browser and the account", () => {
    /** A local document written STRAIGHT to storage, so the write listener never schedules a push behind the assertion. */
    const putLocal = (p: StoryProgress): void => window.localStorage.setItem(PROGRESS_KEY, JSON.stringify(p));

    const local = doc({ read: { a: 2, b: 3 }, pos: { s1: at(3, 200) }, last: "s1" });
    const remote = doc({ read: { c: 4 }, pos: { s2: at(5, 900) }, last: "s2" });

    beforeEach(() => {
        window.localStorage.clear();
        getMock.mockReset();
        putMock.mockReset();
        putMock.mockResolvedValue({ ok: true, updatedAt: 1 });
        __resetStorySyncForTests();
        __enableStorySyncForTests();
        // The shipped default is `merge`, so a prompt is something a reader
        // has to ASK for. Every test below but the first two sets it.
        setStorySyncPolicy("ask");
    });

    afterEach(() => {
        __resetStorySyncForTests();
        vi.useRealTimers();
    });

    // What a browser that has never opened the setting does, which is what the
    // sync did before any of this existed.
    it("merges two differing sides without asking, by default", async () => {
        window.localStorage.removeItem(SYNC_POLICY_KEY);
        __resetStorySyncForTests();
        __enableStorySyncForTests();
        putLocal(local);
        getMock.mockResolvedValue(pullAnswer({ progress: remote, updatedAt: 2 }));
        await __pullStorySyncForTests();

        const merged = merge(local, remote);
        expect(__storySyncStateForTests().kind).toBe("idle");
        expect(sameProgress(loadProgress(), merged)).toBe(true);
        expect(putMock).toHaveBeenCalledTimes(1);
        expect(sameProgress(pushedDocs()[0], merged)).toBe(true);
    });

    it("stops with both documents untouched and counts what each outcome costs", async () => {
        putLocal(local);
        const before = window.localStorage.getItem(PROGRESS_KEY);
        getMock.mockResolvedValue(pullAnswer({ progress: remote, updatedAt: 2 }));
        await __pullStorySyncForTests();

        const state = __storySyncStateForTests();
        expect(state.kind).toBe("conflict");
        if (state.kind !== "conflict") return;
        expect(state.summary.local).toEqual({ marks: 2, positions: 1, last: "s1", newest: 200 });
        expect(state.summary.remote).toEqual({ marks: 1, positions: 1, last: "s2", newest: 900 });
        expect(state.summary.merged).toEqual({ marks: 3, positions: 2 });
        expect(state.summary.dropsLocal).toEqual({ marks: 2, positions: 1 });
        expect(state.summary.dropsRemote).toEqual({ marks: 1, positions: 1 });
        expect(state.summary.newer).toBe("remote");
        // Nothing moved on either side: that is the whole point of the prompt.
        expect(window.localStorage.getItem(PROGRESS_KEY)).toBe(before);
        expect(putMock).toHaveBeenCalledTimes(0);
    });

    it("replaces this browser and pushes nothing when the account wins", async () => {
        putLocal(local);
        getMock.mockResolvedValue(pullAnswer({ progress: remote, updatedAt: 2 }));
        await __pullStorySyncForTests();
        await __resolveStorySyncForTests("account");

        expect(sameProgress(loadProgress(), remote)).toBe(true);
        expect(putMock).toHaveBeenCalledTimes(0);
        expect(__storySyncStateForTests().kind).toBe("idle");
    });

    it("pushes this browser's document and leaves it untouched when the browser wins", async () => {
        putLocal(local);
        const before = window.localStorage.getItem(PROGRESS_KEY);
        getMock.mockResolvedValue(pullAnswer({ progress: remote, updatedAt: 2 }));
        await __pullStorySyncForTests();
        await __resolveStorySyncForTests("browser");

        expect(putMock).toHaveBeenCalledTimes(1);
        expect(sameProgress(pushedDocs()[0], local)).toBe(true);
        expect(window.localStorage.getItem(PROGRESS_KEY)).toBe(before);
    });

    it("writes exactly what the old unconditional merge wrote when both are kept", async () => {
        putLocal(local);
        getMock.mockResolvedValue(pullAnswer({ progress: remote, updatedAt: 2 }));
        await __pullStorySyncForTests();
        await __resolveStorySyncForTests("merge");

        const merged = merge(local, remote);
        expect(sameProgress(loadProgress(), merged)).toBe(true);
        expect(putMock).toHaveBeenCalledTimes(1);
        expect(sameProgress(pushedDocs()[0], merged)).toBe(true);
    });

    it("never asks when one side is empty, and never asks when the two agree", async () => {
        getMock.mockResolvedValue(pullAnswer({ progress: remote, updatedAt: 2 }));
        await __pullStorySyncForTests();
        expect(__storySyncStateForTests().kind).toBe("idle");
        expect(sameProgress(loadProgress(), remote)).toBe(true);

        __resetStorySyncForTests();
        __enableStorySyncForTests();
        putMock.mockClear();
        putLocal(remote);
        await __pullStorySyncForTests();
        expect(__storySyncStateForTests().kind).toBe("idle");
        expect(putMock).toHaveBeenCalledTimes(0);
    });

    // A subset is not a disagreement: the merge IS the bigger side, so there is
    // no outcome a reader could prefer and nothing to show them.
    it("never asks when one side holds everything the other does", async () => {
        const bigger = doc({ read: { a: 2, b: 3, c: 4 }, pos: { s1: at(3, 200), s2: at(5, 900) }, last: "s1" });
        putLocal(bigger);
        getMock.mockResolvedValue(pullAnswer({ progress: doc({ read: { a: 2 }, pos: { s1: at(3, 200) }, last: "s1" }), updatedAt: 2 }));
        await __pullStorySyncForTests();

        expect(__storySyncStateForTests().kind).toBe("idle");
        expect(sameProgress(loadProgress(), bigger)).toBe(true);
        expect(putMock).toHaveBeenCalledTimes(1);
        expect(sameProgress(pushedDocs()[0], bigger)).toBe(true);
    });

    it("ships `merge` as the default, so nothing is asked until a reader asks for it", () => {
        expect(DEFAULT_SYNC_POLICY).toBe("merge");
    });

    it("applies the stored policy without a prompt", async () => {
        setStorySyncPolicy("account");
        putLocal(local);
        getMock.mockResolvedValue(pullAnswer({ progress: remote, updatedAt: 2 }));
        await __pullStorySyncForTests();

        expect(__storySyncStateForTests().kind).toBe("idle");
        expect(sameProgress(loadProgress(), remote)).toBe(true);
        expect(putMock).toHaveBeenCalledTimes(0);
    });

    // The two sides differ only by a mark the backend has since refused, which
    // is not a reading history at all and never reaches the reader.
    it("never asks when the difference is only a withdrawn game mark", async () => {
        putLocal(v1({ read: { a: 1, b: 1 } }));
        getMock.mockResolvedValue(pullAnswer({ progress: v1({ read: { a: 1 } }), updatedAt: 2, gameUnread: ["b"] }));
        await __pullStorySyncForTests();

        expect(__storySyncStateForTests().kind).toBe("idle");
        expect(loadProgress().read).toEqual({ a: 1 });
        expect(loadProgress().v).toBe(2);
        expect(putMock).toHaveBeenCalledTimes(1);
    });
});

describe("a tab that comes back", () => {
    beforeEach(() => {
        vi.useFakeTimers();
        localStorage.clear();
        __resetStorySyncForTests();
        getMock.mockReset();
        putMock.mockReset();
        __enableStorySyncForTests();
    });
    afterEach(() => {
        __resetStorySyncForTests();
        vi.useRealTimers();
    });

    it("re-pulls on focus only once its last pull is older than the threshold", async () => {
        getMock.mockResolvedValue(pullAnswer({ gameSyncedAt: 1 }));
        await __pullStorySyncForTests();
        expect(getMock).toHaveBeenCalledTimes(1);
        window.dispatchEvent(new Event("focus"));
        await vi.runOnlyPendingTimersAsync();
        expect(getMock).toHaveBeenCalledTimes(1);
        vi.advanceTimersByTime(REPULL_AFTER_MS + 1);
        window.dispatchEvent(new Event("focus"));
        await vi.runOnlyPendingTimersAsync();
        expect(getMock).toHaveBeenCalledTimes(2);
    });
});
