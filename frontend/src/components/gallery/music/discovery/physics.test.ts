import { describe, expect, it } from "vitest";
import { coverPose, repeatDelay, rubberBand, velocityOf, visibleRadius } from "./physics";

describe("coverPose", () => {
    it("matches the reference's measured poses at rest", () => {
        const c = coverPose(0, 0);
        expect([c.x, c.z, c.rotateY, c.scale, c.brightness]).toEqual([0, 60, -0, 1.15, 1]);
        const n1 = coverPose(1, 0);
        expect([n1.x, n1.z, n1.rotateY]).toEqual([150, -150, -58]);
        expect(n1.scale).toBeCloseTo(0.8, 10);
        expect(n1.brightness).toBeCloseTo(0.55, 10);
        expect(coverPose(2, 0).x).toBe(242);
        expect(coverPose(2, 0).brightness).toBeCloseTo(0.51, 10);
        expect(coverPose(3, 0).x).toBe(334);
        expect(coverPose(-1, 0).rotateY).toBe(58);
        expect(coverPose(30, 0).brightness).toBe(0.35);
    });

    it("scales pixel offsets with the cover size, not angles", () => {
        const p = coverPose(2, 0, 0.5);
        expect(p.x).toBe(121);
        expect(p.rotateY).toBe(-58);
        expect(p.zIndex).toBe(980);
    });
});

describe("visibleRadius", () => {
    it("stays between 3 and 20", () => {
        expect(visibleRadius(200)).toBe(3);
        // (1350 * 550 / 1200 - 150) / 92 = 5.1 -> 6, plus 2.
        expect(visibleRadius(1100)).toBe(8);
        expect(visibleRadius(10000)).toBe(20);
    });
});

describe("motion", () => {
    it("rubber-bands past either end at .35", () => {
        expect(rubberBand(-2, 10)).toBeCloseTo(-0.7);
        expect(rubberBand(12, 10)).toBeCloseTo(10.7);
        expect(rubberBand(4, 10)).toBe(4);
    });

    it("reads velocity from the last 150 ms only", () => {
        const samples = [
            { t: 0, pos: 0 },
            { t: 900, pos: 5 },
            { t: 1000, pos: 6 },
        ];
        expect(velocityOf(samples, 1000)).toBeCloseTo(10);
        expect(velocityOf([{ t: 0, pos: 0 }], 0)).toBe(0);
    });

    it("accelerates held keys down to 60 ms", () => {
        expect(repeatDelay(0)).toBe(280);
        expect(repeatDelay(200)).toBe(180);
        expect(repeatDelay(5000)).toBe(60);
    });
});
