import { describe, expect, it } from "vitest";
import { coerceProgress, emptyProgress, isStoryRead, onProgressStorage, overriddenByHand, PROGRESS_KEY, parseProgress, readSourceOf, type StoryProgress, trustingGame, withoutPosition, withPosition, withRead, withUnread } from "./progress";

describe("coerceProgress", () => {
    it("returns an empty progress for anything that is not an object", () => {
        for (const raw of [null, undefined, 3, "x", [], true]) expect(coerceProgress(raw)).toEqual(emptyProgress());
    });

    it("keeps only well-formed read marks and positions", () => {
        const got = coerceProgress({
            v: 7,
            read: { a: 1, b: true, c: "1", d: 0, e: "yes", "": 1 },
            pos: {
                ok: { halt: "12", total: 40.9, ts: 5, choices: { "0": "1", x: "2", "1": 3 } },
                bad: { halt: -1 },
                worse: "nope",
            },
            last: "ok",
        });
        expect(got).toEqual({
            v: 2,
            read: { a: 1, b: 1, c: 1 },
            pos: { ok: { halt: 12, total: 40, ts: 5, choices: { 0: "1" } } },
            last: "ok",
        });
    });

    it("keeps a reach only when it is past the halt, and drops a malformed one", () => {
        const pos = (reach: unknown) => coerceProgress({ pos: { s: { halt: 16, total: 90, ts: 1, choices: {}, reach } } }).pos.s;
        expect(pos(85)).toEqual({ halt: 16, total: 90, ts: 1, choices: {}, reach: 85 });
        expect(pos("85").reach).toBe(85);
        expect(pos(16).reach).toBeUndefined();
        expect(pos(3).reach).toBeUndefined();
        expect(pos("x").reach).toBeUndefined();
        expect(pos(undefined)).toEqual({ halt: 16, total: 90, ts: 1, choices: {} });
    });

    it("drops a non-string last", () => {
        expect(coerceProgress({ last: 4 }).last).toBeUndefined();
    });

    it("keeps a v1 document at v1 and reads everything else as v2", () => {
        expect(coerceProgress({ v: 1, read: { a: 1 } }).v).toBe(1);
        expect(coerceProgress({ v: 2, read: { a: 1 } }).v).toBe(2);
        expect(coerceProgress({ read: { a: 1 } }).v).toBe(2);
        expect(coerceProgress({ v: "1", read: { a: 1 } }).v).toBe(2);
        expect(emptyProgress().v).toBe(2);
    });

    it("parseProgress survives invalid JSON", () => {
        expect(parseProgress("{not json")).toEqual(emptyProgress());
    });
});

describe("isStoryRead", () => {
    const empty: ReadonlySet<string> = new Set<string>();

    it("answers for a mark in the document and for one only the game reports", () => {
        const p = withRead(emptyProgress(), "mine");
        expect(isStoryRead(p, empty, "mine")).toBe(true);
        expect(isStoryRead(p, empty, "theirs")).toBe(false);
        expect(isStoryRead(p, new Set(["theirs"]), "theirs")).toBe(true);
        expect(isStoryRead(emptyProgress(), empty, "mine")).toBe(false);
    });
});

describe("marking unread by hand", () => {
    const game: ReadonlySet<string> = new Set(["g1"]);

    it("outranks the game's verdict until the story is marked read again", () => {
        let p = emptyProgress();
        expect(isStoryRead(p, game, "g1")).toBe(true);
        expect(readSourceOf(p, game, "g1")).toBe("game");
        p = withUnread(p, "g1", 500);
        expect(isStoryRead(p, game, "g1")).toBe(false);
        expect(readSourceOf(p, game, "g1")).toBe("cleared");
        expect(p.unread).toEqual({ g1: 500 });
        p = withRead(p, "g1", 900);
        expect(isStoryRead(p, game, "g1")).toBe(true);
        expect(readSourceOf(p, game, "g1")).toBe("own");
        expect(p.unread).toBeUndefined();
    });

    it("clears the reader's own mark too, and survives a round trip through coerce", () => {
        const p = withUnread(withRead(emptyProgress(), "s1", 5), "s1", 7);
        expect(p.read).toEqual({});
        expect(readSourceOf(p, new Set(), "s1")).toBe("none");
        const back = coerceProgress(JSON.parse(JSON.stringify(p)));
        expect(back.unread).toEqual({ s1: 7 });
        expect(coerceProgress({ v: 2, read: {}, pos: {}, unread: { "": 3, x: "no", y: 0 } }).unread).toBeUndefined();
    });
});

describe("updates", () => {
    it("withPosition stamps the time and sets last", () => {
        const p = withPosition(emptyProgress(), "s1", { halt: 3, total: 10, choices: { 0: "2" } }, 1234);
        expect(p.pos.s1).toEqual({ halt: 3, total: 10, ts: 1234, choices: { 0: "2" } });
        expect(p.last).toBe("s1");
    });

    it("withRead carries a v1 document's version through, so the withdrawal can still run", () => {
        expect(withRead(coerceProgress({ v: 1, read: {} }), "s1").v).toBe(1);
        expect(withRead(emptyProgress(), "s1").v).toBe(2);
    });

    it("withRead is idempotent and withoutPosition removes only that story", () => {
        let p = withRead(withRead(emptyProgress(), "s1", 500), "s1", 900);
        // The FIRST finish is the mark, and it is a time, never the legacy 1.
        expect(p.read).toEqual({ s1: 500 });
        expect(withRead(emptyProgress(), "s1", 0).read).toEqual({ s1: 2 });
        p = withPosition(p, "s1", { halt: 1, total: 2, choices: {} }, 1);
        p = withPosition(p, "s2", { halt: 1, total: 2, choices: {} }, 1);
        const q = withoutPosition(p, "s1");
        expect(Object.keys(q.pos)).toEqual(["s2"]);
        expect(withoutPosition(q, "missing")).toBe(q);
    });
});

describe("another tab's write", () => {
    it("reaches an onProgressStorage listener, coerced, and other keys do not", () => {
        const seen: StoryProgress[] = [];
        const off = onProgressStorage((p) => seen.push(p));
        window.dispatchEvent(new StorageEvent("storage", { key: "myrtle.story.settings", newValue: "{}" }));
        window.dispatchEvent(new StorageEvent("storage", { key: PROGRESS_KEY, newValue: JSON.stringify({ v: 2, read: { a: 5, "": 1 }, pos: {} }) }));
        window.dispatchEvent(new StorageEvent("storage", { key: PROGRESS_KEY, newValue: null }));
        off();
        window.dispatchEvent(new StorageEvent("storage", { key: PROGRESS_KEY, newValue: JSON.stringify({ v: 2, read: { b: 5 }, pos: {} }) }));
        expect(seen).toEqual([{ v: 2, read: { a: 5 }, pos: {} }]);
    });
});

describe("trusting the game again", () => {
    const game: ReadonlySet<string> = new Set(["g1", "g2"]);

    it("lists the hand clears the game disagrees with and drops only those", () => {
        let p = withUnread(withUnread(withUnread(emptyProgress(), "g1", 5), "g2", 6), "own", 7);
        expect(overriddenByHand(p, game).sort()).toEqual(["g1", "g2"]);
        expect(isStoryRead(p, game, "g1")).toBe(false);
        p = trustingGame(p, game);
        expect(p.unread).toEqual({ own: 7 });
        expect(isStoryRead(p, game, "g1")).toBe(true);
        expect(isStoryRead(p, game, "g2")).toBe(true);
        expect(overriddenByHand(p, game)).toEqual([]);
    });

    it("returns the same document when there is nothing to drop, and removes an emptied map", () => {
        const untouched = withRead(emptyProgress(), "s1", 9);
        expect(trustingGame(untouched, game)).toBe(untouched);
        const cleared = withUnread(emptyProgress(), "g1", 5);
        expect(trustingGame(cleared, game).unread).toBeUndefined();
    });
});
