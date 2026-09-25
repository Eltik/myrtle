import { describe, expect, it } from "vitest";
import { anchorRow, type BacklogEntry, reachOf, rebuiltTo, withChoice, withDecision, withHalt } from "./backlog";

const line = (haltIndex: number, text = `line ${haltIndex}`): BacklogEntry => ({ haltIndex, kind: "line", text, isNarration: false });
const choice = (haltIndex: number, value: string): BacklogEntry => ({ haltIndex, kind: "choice", text: `option ${value}`, isNarration: false, value });

/** Read 0..last in order, one halt at a time, the way `advance` applies them. */
function readTo(last: number, from: BacklogEntry[] = []): BacklogEntry[] {
    let log = from;
    for (let i = log.length; i <= last; i++) log = withHalt(log, line(i));
    return log;
}

/** The probe walk of a straight path of lines, up to `limit`. */
const walk = (limit: number): BacklogEntry[] => Array.from({ length: limit + 1 }, (_, i) => line(i));

const halts = (log: BacklogEntry[]): number[] => log.map((e) => e.haltIndex);

describe("the log keeps everything up to the furthest halt", () => {
    it("85 -> 16 -> 85: a jump back keeps rows 17..85, and the jump forward lands on them", () => {
        const at85 = readTo(85);
        expect(at85).toHaveLength(86);

        // Jump back to 16: the probe walks to 16, the existing log walked the same path.
        const at16 = withHalt(rebuiltTo(walk(16), at85, 16), line(16));
        expect(halts(at16)).toEqual(halts(at85));
        expect(reachOf(at16, 16)).toBe(85);

        // And back out to 85 from the log: nothing is lost on the way.
        const back85 = withHalt(rebuiltTo(walk(85), at16, 85), line(85));
        expect(halts(back85)).toEqual(halts(at85));
    });

    it("reading forward from 16 over known ground keeps the tail", () => {
        const at16 = rebuiltTo(walk(16), readTo(85), 16);
        let log = at16;
        for (let i = 17; i <= 20; i++) log = withHalt(log, line(i));
        expect(reachOf(log, 20)).toBe(85);
        expect(log).toHaveLength(86);
    });

    it("Back one line keeps the tail too", () => {
        const log = rebuiltTo(walk(84), readTo(85), 84);
        expect(reachOf(log, 84)).toBe(85);
    });

    it("a jump to halt 0 keeps the tail (no rows sit before it to compare)", () => {
        const log = rebuiltTo(walk(0), readTo(85), 0);
        expect(log).toHaveLength(86);
    });

    it("a first visit past the reach appends", () => {
        const log = withHalt(readTo(10), line(11));
        expect(halts(log)).toEqual(halts(readTo(11)));
    });

    it("a different kind at a known halt is a different path and drops the tail", () => {
        const log = withHalt(readTo(10), { haltIndex: 5, kind: "cutscene", text: "Cutscene", isNarration: false });
        expect(halts(log)).toEqual([0, 1, 2, 3, 4, 5]);
        expect(log[5].kind).toBe("cutscene");
    });

    it("the re-applied row takes the new text in place of the old", () => {
        const log = withHalt(readTo(10), line(3, "renamed"));
        expect(log[3].text).toBe("renamed");
        expect(log).toHaveLength(11);
    });
});

describe("choices", () => {
    /** 0..4 lines, a decision at 5 answered `a`, then 6..20 on that branch. */
    const branchA = (): BacklogEntry[] => {
        let log = readTo(4);
        log = withChoice(withDecision(log, 5), choice(5, "a"));
        for (let i = 6; i <= 20; i++) log = withHalt(log, line(i));
        return log;
    };

    it("the SAME option at a known decision keeps the tail", () => {
        const at5 = withDecision(rebuiltTo([...walk(4), choice(5, "a")], branchA(), 5), 5);
        expect(reachOf(at5, 5)).toBe(20);
        const again = withChoice(at5, choice(5, "a"));
        expect(reachOf(again, 6)).toBe(20);
    });

    it("a DIFFERENT option drops every row after the decision", () => {
        const at5 = withDecision(rebuiltTo([...walk(4), choice(5, "a")], branchA(), 5), 5);
        const diverged = withChoice(at5, choice(5, "b"));
        expect(halts(diverged)).toEqual([0, 1, 2, 3, 4, 5]);
        expect(diverged[5].value).toBe("b");
        expect(reachOf(diverged, 6)).toBe(6);
    });

    it("a first visit to a decision truncates at it", () => {
        expect(halts(withDecision(readTo(10), 5))).toEqual([0, 1, 2, 3, 4]);
    });

    it("a probe that took another option before the target replaces the log", () => {
        const probe = [...walk(4), choice(5, "b"), line(6), line(7)];
        expect(rebuiltTo(probe, branchA(), 7)).toEqual(probe);
    });
});

describe("resume rebuilds the log out to the saved reach", () => {
    it("an empty log takes the probe's walk to the reach", () => {
        const log = rebuiltTo(walk(85), [], 16);
        expect(log).toHaveLength(86);
        expect(reachOf(withHalt(log, line(16)), 16)).toBe(85);
    });

    it("a scrub past the reach appends the probe's rows beyond it", () => {
        const log = rebuiltTo(walk(100), readTo(85), 100);
        expect(halts(log)).toEqual(halts(readTo(100)));
    });
});

describe("anchorRow", () => {
    it("is the current row, or the last row before a decision, or -1 on an empty log", () => {
        const log = [...readTo(4), choice(6, "a"), line(7)];
        expect(anchorRow(log, 3)).toBe(3);
        expect(anchorRow(log, 5)).toBe(4);
        expect(anchorRow(log, 7)).toBe(6);
        expect(anchorRow([], 3)).toBe(-1);
        expect(anchorRow([line(4)], 0)).toBe(0);
    });
});
