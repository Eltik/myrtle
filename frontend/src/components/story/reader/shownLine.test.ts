import { describe, expect, it } from "vitest";
import { nextShownLine, type ShownLine } from "./shownLine";

const narration = { text: "They give way when police or officials show up.", isNarration: true };
const police = { speaker: "Ursine Police", text: "Halt.", isNarration: false };

describe("nextShownLine", () => {
    it("prints the first line at once, box visible or not", () => {
        expect(nextShownLine(null, narration, 1, false)).toEqual({ ...narration, speaker: undefined, revealKey: 1 });
    });

    it("holds the old line while the box is hidden for the new halt (the hide-and-cut flash)", () => {
        const shown: ShownLine = { ...narration, revealKey: 1 };
        // The player is on the police line; the scene's first frame hides the box.
        expect(nextShownLine(shown, police, 2, false)).toBe(shown);
        // The box comes back with the halt's own frame: now the new line prints.
        expect(nextShownLine(shown, police, 2, true)).toEqual({ ...police, revealKey: 2 });
    });

    it("switches on the first frame when the box never hides (ordinary dialogue)", () => {
        const shown: ShownLine = { ...narration, revealKey: 1 };
        expect(nextShownLine(shown, police, 2, true)).toEqual({ ...police, revealKey: 2 });
    });

    it("latches on the reveal key: a later hidden frame in the same step keeps the new line", () => {
        const shown: ShownLine = { ...police, revealKey: 2 };
        expect(nextShownLine(shown, police, 2, false)).toBe(shown);
    });

    it("is idempotent within a render, so a strict-mode double render prints the same line", () => {
        const shown: ShownLine = { ...narration, revealKey: 1 };
        const once = nextShownLine(shown, police, 2, true);
        expect(nextShownLine(once, police, 2, true)).toBe(once);
    });
});
