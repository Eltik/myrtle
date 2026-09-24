import { describe, expect, it } from "vitest";
import { CHROME_IDLE_MS, CHROME_REVEAL_BAND_PX, type ChromeInputs, chromeIdleMs, chromeShown, clampTooltipLeft, nextPeek, nextTheater, PEEK_GRACE_MS, revealHandleShown, revealsChrome, theaterConsumes } from "./chrome";

const reading: ChromeInputs = { theater: false, collapsed: false, alwaysShow: false, dialogOpen: false, pointerOverChrome: false, pointerOverHandle: false, phase: "reading", idle: false };

describe("chromeShown", () => {
    it("shows the chrome while the reader is being used and hides it once idle", () => {
        expect(chromeShown(reading)).toBe(true);
        expect(chromeShown({ ...reading, idle: true })).toBe(false);
    });

    it("never hides while a dialog is open, while the pointer is on the bar, or on a card", () => {
        expect(chromeShown({ ...reading, idle: true, dialogOpen: true })).toBe(true);
        expect(chromeShown({ ...reading, idle: true, pointerOverChrome: true })).toBe(true);
        for (const phase of ["title", "resume", "end"]) {
            expect(chromeShown({ ...reading, idle: true, phase })).toBe(true);
        }
    });

    it("the kill switch outranks the idle timer", () => {
        expect(chromeShown({ ...reading, alwaysShow: true, idle: true })).toBe(true);
    });

    it("theater mode outranks everything", () => {
        expect(chromeShown({ ...reading, theater: true })).toBe(false);
        expect(chromeShown({ ...reading, theater: true, alwaysShow: true })).toBe(false);
        expect(chromeShown({ ...reading, theater: true, alwaysShow: true, dialogOpen: true, pointerOverChrome: true })).toBe(false);
        expect(chromeShown({ ...reading, theater: true, phase: "title" })).toBe(false);
    });

    it("THE TWO HIDES ARE DIFFERENT: a collapsed toolbar outranks the kill switch and a dialog, and the eye outranks it", () => {
        expect(chromeShown({ ...reading, collapsed: true, alwaysShow: true })).toBe(false);
        expect(chromeShown({ ...reading, collapsed: true, dialogOpen: true })).toBe(false);
        // The pointer on a PILL does not undo it either: the chevron would
        // otherwise do nothing visible until the pointer left it.
        expect(chromeShown({ ...reading, collapsed: true, pointerOverChrome: true })).toBe(false);
        // Only the reveal handle peeks, and it un-collapses nothing.
        expect(chromeShown({ ...reading, collapsed: true, pointerOverHandle: true })).toBe(true);
        expect(chromeShown({ ...reading, theater: true, collapsed: true, pointerOverHandle: true })).toBe(false);
    });

    it("the auto-hide is OPT-IN, so the default settings object keeps the pills on an idle stage", () => {
        // `alwaysShow` is `!autoHideToolbar`, and `autoHideToolbar` defaults off.
        expect(chromeShown({ ...reading, alwaysShow: true, idle: true, phase: "reading" })).toBe(true);
    });
});

describe("revealHandleShown", () => {
    it("is the collapsed toolbar's only affordance, and theater mode has none", () => {
        expect(revealHandleShown({ collapsed: true, theater: false })).toBe(true);
        expect(revealHandleShown({ collapsed: false, theater: false })).toBe(false);
        expect(revealHandleShown({ collapsed: true, theater: true })).toBe(false);
    });
});

describe("revealsChrome", () => {
    it("READING NEVER REVEALS: Space, Enter, the arrows, a tap on the stage and auto-play all leave it hidden", () => {
        expect(revealsChrome("advance")).toBe(false);
        expect(revealsChrome("key")).toBe(false);
    });

    it("pointer movement reveals, wherever it is", () => {
        expect(revealsChrome("pointermove")).toBe(true);
        expect(revealsChrome("pointermove", 800)).toBe(true);
    });

    it("a touch reveals only from the top band, so a page turn does not raise the bar", () => {
        expect(revealsChrome("pointerdown", 0)).toBe(true);
        expect(revealsChrome("pointerdown", CHROME_REVEAL_BAND_PX)).toBe(true);
        expect(revealsChrome("pointerdown", CHROME_REVEAL_BAND_PX + 1)).toBe(false);
        expect(revealsChrome("pointerdown", 500)).toBe(false);
        // A pointer with no y at all is not in the band: the default is "far
        // down the stage", never "reveal".
        expect(revealsChrome("pointerdown")).toBe(false);
    });

    it("`T` and an opening dialog reveal", () => {
        expect(revealsChrome("pin")).toBe(true);
        expect(revealsChrome("dialog")).toBe(true);
    });
});

describe("chromeIdleMs", () => {
    it("is the reader's own seconds, clamped to 1..15", () => {
        expect(chromeIdleMs(2.5)).toBe(2500);
        expect(chromeIdleMs(15)).toBe(15000);
        expect(chromeIdleMs(0.1)).toBe(1000);
        expect(chromeIdleMs(99)).toBe(15000);
    });

    it("falls back to the shipped default rather than to zero, which would hide the bar instantly", () => {
        expect(chromeIdleMs(Number.NaN)).toBe(CHROME_IDLE_MS);
        expect(chromeIdleMs(Number.POSITIVE_INFINITY)).toBe(CHROME_IDLE_MS);
    });
});

describe("clampTooltipLeft", () => {
    it("centres the tip on the pointer where there is room", () => {
        expect(clampTooltipLeft(700, 200, 1440)).toBe(600);
    });

    it("slides the tip against the stage edges rather than off them", () => {
        expect(clampTooltipLeft(0, 200, 1440)).toBe(8);
        expect(clampTooltipLeft(1440, 200, 1440)).toBe(1232);
        expect(clampTooltipLeft(1440, 200, 1440) + 200).toBeLessThanOrEqual(1440 - 8);
    });

    it("pins a tip wider than the stage to the left pad instead of a negative left", () => {
        expect(clampTooltipLeft(100, 400, 375)).toBe(8);
        expect(clampTooltipLeft(100, 375, 375)).toBe(8);
    });

    it("has an answer before the tip has been measured, which is the first render", () => {
        expect(clampTooltipLeft(700, 0, 1440)).toBe(700);
        expect(clampTooltipLeft(0, 0, 1440)).toBe(8);
    });
});

describe("nextTheater", () => {
    it("the eye and Esc toggle it, and any tap or key gives the UI back", () => {
        expect(nextTheater(false, "toggle")).toBe(true);
        expect(nextTheater(true, "toggle")).toBe(false);
        expect(nextTheater(true, "activity")).toBe(false);
    });

    it("activity never HIDES the UI, which is what separates it from the toggle", () => {
        expect(nextTheater(false, "activity")).toBe(false);
    });
});

describe("theaterConsumes", () => {
    it("a tap that restores the UI does not also advance the line", () => {
        expect(theaterConsumes(true, "activity")).toBe(true);
    });

    it("the toggle itself is never swallowed, and nothing is swallowed with the UI up", () => {
        expect(theaterConsumes(true, "toggle")).toBe(false);
        expect(theaterConsumes(false, "activity")).toBe(false);
    });
});

describe("the collapsed toolbar's peek", () => {
    it("is held by entering anything in the union of the handle and the pills", () => {
        expect(nextPeek("enter")).toBe(true);
    });

    it("drops only when the grace runs out, never on the leave itself", () => {
        expect(nextPeek("expire")).toBe(false);
        expect(nextPeek("open")).toBe(false);
    });

    it("gives the pointer 300 ms to cross the stage between the handle and a pill", () => {
        expect(PEEK_GRACE_MS).toBe(300);
    });

    it("shows the chrome while the peek is held and hides it the moment it is not", () => {
        const base = { theater: false, collapsed: true, alwaysShow: true, dialogOpen: false, pointerOverChrome: true, phase: "reading", idle: false };
        expect(chromeShown({ ...base, pointerOverHandle: true })).toBe(true);
        expect(chromeShown({ ...base, pointerOverHandle: false })).toBe(false);
    });
});
