import { describe, expect, it } from "vitest";
import { type BacklogLine, backlogJumpTarget, backlogPlainText, firstWords, fractionAt, fractionForHalt, type HaltSummary, haltAtFraction } from "./scrub";

const list: HaltSummary[] = Array.from({ length: 10 }, (_, i) => ({ haltIndex: i, kind: "line", preview: `line ${i}` }));

describe("fractionAt", () => {
    it("maps a pointer to 0..1 across the bar", () => {
        const rect = { left: 100, width: 400 };
        expect(fractionAt(100, rect)).toBe(0);
        expect(fractionAt(300, rect)).toBe(0.5);
        expect(fractionAt(500, rect)).toBe(1);
    });

    it("clamps outside the bar and never divides by a zero width", () => {
        const rect = { left: 100, width: 400 };
        expect(fractionAt(0, rect)).toBe(0);
        expect(fractionAt(9999, rect)).toBe(1);
        expect(fractionAt(50, { left: 0, width: 0 })).toBe(0);
    });
});

describe("haltAtFraction", () => {
    it("puts the far left on the first halt and the far right on the last", () => {
        expect(haltAtFraction(list, 0)?.haltIndex).toBe(0);
        expect(haltAtFraction(list, 1)?.haltIndex).toBe(9);
    });

    it("agrees with the bar's own filled width", () => {
        // The bar draws (haltIndex + 1) / total, so halt 4 fills 0.5 and 0.5 must read back as halt 4.
        expect(haltAtFraction(list, 0.5)?.haltIndex).toBe(4);
        expect(fractionForHalt(list, 4)).toBeCloseTo(0.5, 10);
    });

    it("clamps out-of-range fractions instead of returning undefined", () => {
        expect(haltAtFraction(list, -3)?.haltIndex).toBe(0);
        expect(haltAtFraction(list, 4)?.haltIndex).toBe(9);
    });

    it("has no answer for an empty list, and says so with null rather than halt 0", () => {
        expect(haltAtFraction([], 0.5)).toBeNull();
        expect(fractionForHalt([], 0)).toBe(0);
        expect(fractionForHalt(list, 999)).toBe(0);
    });

    it("finds a halt by its own index, not by its position, when the walk skips ordinals", () => {
        const sparse: HaltSummary[] = [
            { haltIndex: 0, kind: "line", preview: "a" },
            { haltIndex: 7, kind: "decision", preview: "b" },
        ];
        expect(fractionForHalt(sparse, 7)).toBe(1);
        expect(haltAtFraction(sparse, 0.9)?.haltIndex).toBe(7);
    });
});

describe("firstWords", () => {
    it("keeps a short line whole and collapses its whitespace", () => {
        expect(firstWords("Is it just  me?")).toBe("Is it just me?");
    });

    it("cuts a long line on a word boundary and marks the cut", () => {
        const out = firstWords("The sky over Chernobog is getting darker by the minute and nobody is saying why", 30);
        expect(out.endsWith("…")).toBe(true);
        expect(out.length).toBeLessThanOrEqual(31);
        expect(out).not.toContain("  ");
    });

    it("cuts mid-word when there is no late space, which is the CJK case", () => {
        const out = firstWords("塔露拉塔露拉塔露拉塔露拉塔露拉塔露拉", 8);
        expect(out).toBe("塔露拉塔露拉塔露…");
    });
});

describe("backlogJumpTarget", () => {
    it("returns the row's own halt, which for a choice row is the decision", () => {
        expect(backlogJumpTarget({ haltIndex: 12 }, 40)).toBe(12);
        expect(backlogJumpTarget({ haltIndex: 0 }, 5)).toBe(0);
    });

    it("refuses to replay to the line already on screen", () => {
        expect(backlogJumpTarget({ haltIndex: 12 }, 12)).toBeNull();
    });

    it("refuses a negative halt, which is the reader before its first step", () => {
        expect(backlogJumpTarget({ haltIndex: -1 }, 3)).toBeNull();
    });
});

/**
 * What the Copy button puts on the clipboard. The flattener is the reader's
 * own, passed in, so this pins the SHAPE of a line and nothing about markup:
 * a speaker carries a colon, narration carries neither name nor colon, and a
 * decision carries the chevron the dialog draws.
 */
describe("backlogPlainText", () => {
    const plain = (t: string) => t.replace(/<[^>]*>/g, "");
    const log: BacklogLine[] = [
        { kind: "line", speaker: "Amiya", text: "Doctor, are you awake?" },
        { kind: "line", text: "The lights flicker <i>once</i>." },
        { kind: "choice", text: "Say nothing" },
        { kind: "line", speaker: "Kal'tsit", text: "Then we move." },
    ];

    it("writes one line per beat, a speaker with a colon and narration without one", () => {
        expect(backlogPlainText(log, plain)).toBe(["Amiya: Doctor, are you awake?", "The lights flicker once.", "> Say nothing", "Kal'tsit: Then we move."].join("\n"));
    });

    it("marks a decision with the chevron and never with a speaker", () => {
        expect(backlogPlainText([{ kind: "choice", speaker: "Amiya", text: "Go left" }], plain)).toBe("> Go left");
    });

    it("puts the story title and a blank line above the log when there is one", () => {
        expect(backlogPlainText([log[0]], plain, "Chapter 0")).toBe("Chapter 0\n\nAmiya: Doctor, are you awake?");
    });

    it("copies an empty log as an empty string rather than as a lone blank line", () => {
        expect(backlogPlainText([], plain)).toBe("");
    });

    it("keeps an empty speaker out of the prefix, so a blank name is narration", () => {
        expect(backlogPlainText([{ kind: "line", speaker: "", text: "Silence." }], plain)).toBe("Silence.");
    });
});
