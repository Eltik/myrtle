import { describe, expect, it } from "vitest";
import type { Slot, SlotState } from "#/lib/story/scene";
import { contrastRatio, DARK_BOX_WORST, LIGHT_BOX_WORST } from "./colors";
import { lineColorFor, litSprite, readableLineTint, type SpeakerInks, speakerInk, speakerTintTargets } from "./speaker";

/** A hand-built slot state: only the four fields the rule reads carry meaning. */
function slot(bodyUrl: string, lit: boolean, swap: number): SlotState {
    return { sprite: { bodyUrl } as SlotState["sprite"], name: bodyUrl, lit, x: 0, y: 0, alpha: 1, scale: 1, pivotX: 0.5, pivotY: 0.5, swap };
}

const inks: Record<string, { c1: string }> = { "a.png": { c1: "#2f4f8f" }, "b.png": { c1: "#a03040" }, "c.png": { c1: "#3f8f4f" } };
const paletteFor = (url: string): { c1: string } | undefined => inks[url];

describe("litSprite", () => {
    it("is null with nothing on stage, and null when two unlit sprites make it ambiguous", () => {
        expect(litSprite({})).toBeNull();
        expect(litSprite({ l: slot("a.png", false, 1), r: slot("b.png", false, 1) })).toBeNull();
    });

    it('is the SOLE sprite on stage even with nothing lit, which is what `focus="n"` leaves behind', () => {
        expect(litSprite({ m: slot("a.png", false, 1) })?.name).toBe("a.png");
    });

    it("is the one lit slot, whichever slot it is", () => {
        expect(litSprite({ l: slot("a.png", false, 1), r: slot("b.png", true, 1) })?.name).toBe("b.png");
        expect(litSprite({ m: slot("c.png", true, 4) })?.name).toBe("c.png");
    });

    it("takes the most recently swapped slot when two or more are lit", () => {
        expect(litSprite({ l: slot("a.png", true, 3), r: slot("b.png", true, 7) })?.name).toBe("b.png");
        expect(litSprite({ l: slot("a.png", true, 9), m: slot("c.png", true, 2), r: slot("b.png", true, 7) })?.name).toBe("a.png");
    });

    it("falls back to the first lit slot in frame-state order on a tie", () => {
        const slots: Partial<Record<Slot, SlotState>> = { r: slot("b.png", true, 5), l: slot("a.png", true, 5) };
        // Insertion order is the engine's own frame-state order, so `r` wins here.
        expect(litSprite(slots)?.name).toBe("b.png");
    });
});

describe("speakerInk", () => {
    it("is null for narration, which has no speaker at all", () => {
        expect(speakerInk({ speaker: undefined, slots: { l: slot("a.png", true, 1) }, paletteFor, remembered: new Map() })).toBeNull();
    });

    it("is the lit sprite's dominant ink", () => {
        expect(speakerInk({ speaker: "Amiya", slots: { l: slot("a.png", true, 1) }, paletteFor, remembered: new Map() })).toBe("#2f4f8f");
    });

    it("is null when the speaker is off stage and the name has not been seen yet", () => {
        expect(speakerInk({ speaker: "Amiya", slots: {}, paletteFor, remembered: new Map() })).toBeNull();
    });

    it("keeps a name's colour while she speaks from off stage", () => {
        const remembered: SpeakerInks = new Map();
        speakerInk({ speaker: "Amiya", slots: { l: slot("a.png", true, 1) }, paletteFor, remembered });
        expect(speakerInk({ speaker: "Amiya", slots: {}, paletteFor, remembered })).toBe("#2f4f8f");
    });

    it("is FIRST SEEN WINS, so a later scene does not repaint a name mid-story", () => {
        const remembered: SpeakerInks = new Map();
        speakerInk({ speaker: "Amiya", slots: { l: slot("a.png", true, 1) }, paletteFor, remembered });
        expect(speakerInk({ speaker: "Amiya", slots: { l: slot("c.png", true, 2) }, paletteFor, remembered })).toBe("#2f4f8f");
    });

    it("is null while the lit sprite has not been sampled yet, so the plate keeps the hashed hue", () => {
        expect(speakerInk({ speaker: "Dobermann", slots: { l: slot("unsampled.png", true, 1) }, paletteFor, remembered: new Map() })).toBeNull();
    });
});

describe("speakerTintTargets", () => {
    it("reaches nothing when the colour is off, which is the kill switch", () => {
        expect(speakerTintTargets("off")).toEqual({ plate: false, line: false });
    });

    it("reaches the line AND the plate on text and name, never the line alone", () => {
        expect(speakerTintTargets("text")).toEqual({ plate: true, line: true });
    });

    it("reaches the plate alone on name only, so the line keeps the reader's text colour", () => {
        expect(speakerTintTargets("name")).toEqual({ plate: true, line: false });
    });
});

describe("readableLineTint", () => {
    // A sampled ink arrives conditioned by `fitToBox` for this surface, and a
    // line printed in it must match the plate to the byte.
    it("hands back an ink that already clears 4.5:1 UNTOUCHED", () => {
        expect(readableLineTint("#a4c3d0", false)).toBe("#a4c3d0");
        expect(readableLineTint("#bcbcdd", false)).toBe("#bcbcdd");
        expect(readableLineTint("#26264b", true)).toBe("#26264b");
    });

    // MEASURED on #4d4d4d: the seven dark plate hues run 2.951 to 4.447 and
    // every one of them fails, because they were curated against the plate's
    // own near-black chip and not against the box.
    it("walks every hashed hue up until it clears 4.5:1 on the dark box's worst composite", () => {
        for (const hue of ["#e0a458", "#6fb3e0", "#e07a8c", "#6fcf97", "#b48ee0", "#f08a5d", "#5cc8c0"]) {
            expect(contrastRatio(hue, DARK_BOX_WORST)).toBeLessThan(4.5);
            expect(contrastRatio(readableLineTint(hue, false), DARK_BOX_WORST)).toBeGreaterThanOrEqual(4.5);
        }
    });

    it("leaves the light surface's hues alone, because all seven already pass there", () => {
        for (const hue of ["#7a4c05", "#0d4f78", "#9c1f37", "#0a5c34", "#5a2e8a", "#8e3305", "#0a5a55"]) {
            expect(contrastRatio(hue, LIGHT_BOX_WORST)).toBeGreaterThanOrEqual(4.5);
            expect(readableLineTint(hue, true)).toBe(hue);
        }
    });
});

describe("lineColorFor", () => {
    const amiya = { plateColor: "#a4c3d0", textColor: "#a8d8f5", light: false };

    it("prints a spoken line in the speaker's ink on text and name", () => {
        expect(lineColorFor({ mode: "text", isNarration: false, ...amiya })).toBe("#a4c3d0");
    });

    it("keeps the reader's text colour on name only and off", () => {
        expect(lineColorFor({ mode: "name", isNarration: false, ...amiya })).toBe("#a8d8f5");
        expect(lineColorFor({ mode: "off", isNarration: false, ...amiya })).toBe("#a8d8f5");
    });

    // NARRATION has no speaker, so a voice printed on it is a voice nobody is
    // using. This is the rule the three-way select must not break.
    it("keeps the text colour on NARRATION in every one of the three modes", () => {
        for (const mode of ["off", "text", "name"] as const) {
            expect(lineColorFor({ mode, isNarration: true, ...amiya })).toBe("#a8d8f5");
            expect(lineColorFor({ mode, isNarration: true, plateColor: "", textColor: undefined, light: false })).toBeUndefined();
        }
    });

    it("leaves the box's own colour alone when nothing is set, which is the kill switch", () => {
        expect(lineColorFor({ mode: "off", isNarration: false, plateColor: "#6fcf97", textColor: undefined, light: false })).toBeUndefined();
    });

    it("walks a hashed hue up for the line when no sprite answered for the speaker", () => {
        expect(lineColorFor({ mode: "text", isNarration: false, plateColor: "#e07a8c", textColor: undefined, light: false })).toBe(readableLineTint("#e07a8c", false));
    });
});
