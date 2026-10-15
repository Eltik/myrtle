import { describe, expect, it } from "vitest";
import { expForNextLevel, getLevelProgress, MAX_PLAYER_LEVEL } from "./player-level";

describe("expForNextLevel", () => {
    it("reads the Doctor level table", () => {
        expect(expForNextLevel(1)).toBe(500);
        expect(expForNextLevel(3)).toBe(1240);
        expect(expForNextLevel(60)).toBe(13500);
        expect(expForNextLevel(101)).toBe(52000);
        expect(expForNextLevel(119)).toBe(97000);
    });

    it("never decreases from one level to the next", () => {
        for (let level = 1; level < MAX_PLAYER_LEVEL - 1; level++) {
            expect(expForNextLevel(level + 1) ?? 0).toBeGreaterThanOrEqual(expForNextLevel(level) ?? 0);
        }
    });

    it("has nothing past the cap or below the table", () => {
        expect(expForNextLevel(120)).toBeNull();
        expect(expForNextLevel(150)).toBeNull();
        expect(expForNextLevel(0)).toBeNull();
        expect(expForNextLevel(1.5)).toBeNull();
    });
});

describe("getLevelProgress", () => {
    it("is null without a real level", () => {
        expect(getLevelProgress(null, 100)).toBeNull();
        expect(getLevelProgress(0, 100)).toBeNull();
        expect(getLevelProgress(-1, 100)).toBeNull();
    });

    it("measures progress toward the next level", () => {
        expect(getLevelProgress(60, 6750)).toEqual({ level: 60, isMax: false, currentExp: 6750, requiredExp: 13500, ratio: 0.5 });
    });

    it("clamps the ratio and the exp floor", () => {
        expect(getLevelProgress(1, 900)?.ratio).toBe(1);
        expect(getLevelProgress(1, -50)).toEqual({ level: 1, isMax: false, currentExp: 0, requiredExp: 500, ratio: 0 });
        expect(getLevelProgress(1, null)?.currentExp).toBe(0);
    });

    it("reads a capped level as full", () => {
        expect(getLevelProgress(120, 0)).toEqual({ level: 120, isMax: true, currentExp: 0, requiredExp: null, ratio: 1 });
        expect(getLevelProgress(135, 10)).toEqual({ level: 120, isMax: true, currentExp: 10, requiredExp: null, ratio: 1 });
    });

    it("reads a level missing from the table as full", () => {
        expect(getLevelProgress(2.5, 10)).toEqual({ level: 2.5, isMax: false, currentExp: 10, requiredExp: null, ratio: 1 });
    });
});
