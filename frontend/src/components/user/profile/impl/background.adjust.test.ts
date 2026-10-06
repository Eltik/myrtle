import { describe, expect, it } from "vitest";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import { backgroundSources, DEAD_DRAG_PX, deadDragAxis, draftDirty, keyAdjust, PAN_STEP_LARGE, resetBackground, shownElite, wheelScale, withElite, ZOOM_STEP } from "./background";

const BOTH = { x: true, y: true };
const Y_ONLY = { x: false, y: true };
const cg: ProfileBackground = { kind: "story_cg", id: "cg_1", focus_x: 40, focus_y: 60, scale: 150 };

describe("keyAdjust", () => {
    it("moves the picture the way the arrow points, as a drag does: right lowers focus_x", () => {
        expect(keyAdjust(cg, "ArrowRight", false, BOTH)).toEqual({ next: { ...cg, focus_x: 39 }, dead: null });
        expect(keyAdjust(cg, "ArrowLeft", false, BOTH)).toEqual({ next: { ...cg, focus_x: 41 }, dead: null });
        expect(keyAdjust(cg, "ArrowDown", false, BOTH)).toEqual({ next: { ...cg, focus_y: 59 }, dead: null });
        expect(keyAdjust(cg, "ArrowUp", false, BOTH)).toEqual({ next: { ...cg, focus_y: 61 }, dead: null });
    });

    it("steps 10 with Shift and clamps at the edges", () => {
        expect(keyAdjust(cg, "ArrowRight", true, BOTH)?.next?.focus_x).toBe(40 - PAN_STEP_LARGE);
        expect(keyAdjust({ ...cg, focus_x: 4 }, "ArrowRight", true, BOTH)?.next?.focus_x).toBe(0);
        expect(keyAdjust({ ...cg, focus_y: 97 }, "ArrowUp", true, BOTH)?.next?.focus_y).toBe(100);
    });

    it("reads a missing focus as the centre", () => {
        expect(keyAdjust({ kind: "archive_pic", id: "a" }, "ArrowLeft", false, BOTH)?.next).toEqual({ kind: "archive_pic", id: "a", focus_x: 51 });
    });

    it("moves nothing along a dead axis and names it", () => {
        expect(keyAdjust(cg, "ArrowLeft", false, Y_ONLY)).toEqual({ next: null, dead: "x" });
        expect(keyAdjust(cg, "ArrowUp", false, { x: true, y: false })).toEqual({ next: null, dead: "y" });
    });

    it("zooms with + (or =) and -, inside 100..300, dropping the key at 100", () => {
        expect(keyAdjust(cg, "+", false, BOTH)?.next?.scale).toBe(150 + ZOOM_STEP);
        expect(keyAdjust(cg, "=", false, BOTH)?.next?.scale).toBe(160);
        expect(keyAdjust(cg, "-", false, BOTH)?.next?.scale).toBe(140);
        expect(keyAdjust({ ...cg, scale: 295 }, "+", false, BOTH)?.next?.scale).toBe(300);
        expect(keyAdjust({ ...cg, scale: 105 }, "-", false, BOTH)?.next).not.toHaveProperty("scale");
    });

    it("leaves every other key to the page", () => {
        expect(keyAdjust(cg, "Enter", false, BOTH)).toBeNull();
        expect(keyAdjust(cg, "a", false, BOTH)).toBeNull();
    });
});

describe("deadDragAxis", () => {
    it("says nothing for a short drag or one along a live axis", () => {
        expect(deadDragAxis(DEAD_DRAG_PX - 1, 0, Y_ONLY)).toBeNull();
        expect(deadDragAxis(0, 40, Y_ONLY)).toBeNull();
        expect(deadDragAxis(40, 0, BOTH)).toBeNull();
    });

    it("names the dead axis a drag is mostly along", () => {
        expect(deadDragAxis(DEAD_DRAG_PX, 0, Y_ONLY)).toBe("x");
        expect(deadDragAxis(-30, 10, Y_ONLY)).toBe("x");
        expect(deadDragAxis(10, -30, { x: true, y: false })).toBe("y");
        expect(deadDragAxis(10, 30, Y_ONLY)).toBeNull();
    });
});

describe("wheelScale", () => {
    it("zooms in on a wheel up, about 15 points a 100 px notch from 100", () => {
        expect(wheelScale(100, -100, 0, false)).toBeCloseTo(100 * Math.exp(0.15), 10);
        expect(wheelScale(100, 100, 0, false)).toBeLessThan(100);
    });

    it("counts a line-mode delta as 16 px and scales a pinch's ctrl-wheel up", () => {
        expect(wheelScale(100, -3, 1, false)).toBeCloseTo(wheelScale(100, -48, 0, false), 12);
        expect(wheelScale(100, -10, 0, true)).toBeCloseTo(100 * Math.exp(0.1), 10);
    });
});

describe("resetBackground", () => {
    it("puts an art back where a fresh pick starts", () => {
        expect(resetBackground(cg)).toEqual({ kind: "story_cg", id: "cg_1" });
        expect(resetBackground({ kind: "skin", id: "s", focus_x: 10, focus_y: 80, scale: 200 })).toEqual({ kind: "skin", id: "s", focus_y: 25 });
    });
});

describe("draftDirty", () => {
    it("is clean while the draft draws what is saved", () => {
        expect(draftDirty(null, null)).toBe(false);
        expect(draftDirty(cg, { ...cg })).toBe(false);
        expect(draftDirty({ kind: "archive_pic", id: "a" }, { kind: "archive_pic", id: "a", focus_x: 50, focus_y: 50, scale: 100 })).toBe(false);
    });

    it("is dirty on a new picture, crop, zoom or a removal", () => {
        expect(draftDirty(null, cg)).toBe(true);
        expect(draftDirty(cg, null)).toBe(true);
        expect(draftDirty(cg, { ...cg, id: "cg_2" })).toBe(true);
        expect(draftDirty(cg, { ...cg, focus_x: 41 })).toBe(true);
        expect(draftDirty(cg, { ...cg, scale: 151 })).toBe(true);
    });
});

describe("elite art", () => {
    const op: ProfileBackground = { kind: "operator", id: "char_103_angel", focus_y: 10, scale: 120 };

    it("draws elite 2 with the elite 1 fallback by default, and elite 1 alone when chosen", () => {
        const files = (bg: ProfileBackground) => backgroundSources(bg).map((url) => url.slice(url.lastIndexOf("/") + 1));
        expect(files(op)).toEqual(["char_103_angel_2b.png", "char_103_angel_2.png", "char_103_angel_1b.png", "char_103_angel_1.png", "char_103_angel_2b.png", "char_103_angel_2.png", "char_103_angel_1b.png", "char_103_angel_1.png"]);
        expect(files({ ...op, elite: 2 })).toEqual(files(op));
        expect(files({ ...op, elite: 1 })).toEqual(["char_103_angel_1b.png", "char_103_angel_1.png", "char_103_angel_1b.png", "char_103_angel_1.png"]);
    });

    it("shows the stored choice, else elite 2 where the operator has it", () => {
        expect(shownElite(op, true)).toBe(2);
        expect(shownElite(op, false)).toBe(1);
        expect(shownElite({ ...op, elite: 1 }, true)).toBe(1);
        expect(shownElite({ kind: "skin" }, true)).toBeNull();
    });

    it("stores the key only for elite 1 of an operator that has elite 2, keeping the crop", () => {
        expect(withElite(op, 1, true)).toEqual({ ...op, elite: 1 });
        expect(withElite({ ...op, elite: 1 }, 2, true)).toEqual(op);
        expect(withElite(op, 1, false)).toEqual(op);
        expect(withElite({ kind: "skin", id: "a@b", elite: 1 }, 1, true)).toEqual({ kind: "skin", id: "a@b" });
    });

    it("is part of what makes a draft dirty, and survives Reset", () => {
        expect(draftDirty(op, { ...op, elite: 1 })).toBe(true);
        expect(resetBackground({ ...op, elite: 1 })).toEqual({ kind: "operator", id: "char_103_angel", focus_y: 25, elite: 1 });
        expect(resetBackground(op)).toEqual({ kind: "operator", id: "char_103_angel", focus_y: 25 });
    });
});
