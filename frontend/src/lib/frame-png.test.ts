import * as PIXI from "pixi.js";
import { describe, expect, it } from "vitest";
import { FRAME_PNG_PADDING, framePngFilename, framePngSize, frameProjection, unionRegion } from "./frame-png";

describe("framePngSize", () => {
    it("is the region at scale plus the padding on both sides", () => {
        expect(framePngSize({ x: -50, y: -200, width: 100, height: 200 }, 1)).toEqual({ width: 100 + FRAME_PNG_PADDING * 2, height: 200 + FRAME_PNG_PADDING * 2, scale: 1 });
        expect(framePngSize({ x: -50, y: -200, width: 100, height: 200 }, 2)).toEqual({ width: 200 + FRAME_PNG_PADDING * 2, height: 400 + FRAME_PNG_PADDING * 2, scale: 2 });
    });

    it("shrinks the scale so the longest side fits the cap", () => {
        const size = framePngSize({ x: 0, y: 0, width: 3000, height: 1000 }, 2, 4096);
        expect(size.width).toBeLessThanOrEqual(4096);
        expect(size.scale).toBeLessThan(2);
        expect(size.height).toBe(Math.ceil(1000 * size.scale) + FRAME_PNG_PADDING * 2);
    });
});

describe("frameProjection", () => {
    it("maps the region's corners onto the PNG through the target's own transform", () => {
        // A target placed and scaled the way the chibi viewer places its skeleton.
        const local = new PIXI.Matrix(0.34, 0, 0, 0.34, 210, 153);
        const region = { x: -60, y: -180, width: 120, height: 190 };
        const m = frameProjection(local, region, 2);
        const corner = (x: number, y: number) => m.apply(local.apply(new PIXI.Point(x, y)));

        const topLeft = corner(region.x, region.y);
        expect(topLeft.x).toBeCloseTo(FRAME_PNG_PADDING);
        expect(topLeft.y).toBeCloseTo(FRAME_PNG_PADDING);

        const bottomRight = corner(region.x + region.width, region.y + region.height);
        expect(bottomRight.x).toBeCloseTo(FRAME_PNG_PADDING + 240);
        expect(bottomRight.y).toBeCloseTo(FRAME_PNG_PADDING + 380);
    });
});

describe("frameProjection flipped", () => {
    it("mirrors the region inside the same box", () => {
        const local = new PIXI.Matrix(0.5, 0, 0, 0.5, 100, 80);
        const region = { x: -60, y: -180, width: 120, height: 190 };
        const m = frameProjection(local, region, 1, true);
        const corner = (x: number, y: number) => m.apply(local.apply(new PIXI.Point(x, y)));
        const left = corner(region.x, region.y);
        expect(left.x).toBeCloseTo(FRAME_PNG_PADDING + 120);
        expect(left.y).toBeCloseTo(FRAME_PNG_PADDING);
        expect(corner(region.x + region.width, region.y).x).toBeCloseTo(FRAME_PNG_PADDING);
    });
});

describe("unionRegion", () => {
    it("covers both regions and ignores an empty side", () => {
        expect(unionRegion({ x: 0, y: 0, width: 10, height: 10 }, { x: -5, y: 5, width: 10, height: 10 })).toEqual({ x: -5, y: 0, width: 15, height: 15 });
        expect(unionRegion(null, { x: 1, y: 2, width: 3, height: 4 })).toEqual({ x: 1, y: 2, width: 3, height: 4 });
        expect(unionRegion({ x: 1, y: 2, width: 3, height: 4 }, { x: 0, y: 0, width: 0, height: 0 })).toEqual({ x: 1, y: 2, width: 3, height: 4 });
        expect(unionRegion(null, null)).toBeNull();
    });
});

describe("framePngFilename", () => {
    it("names subject, clip, 1-based padded frame and time", () => {
        expect(framePngFilename({ parts: ["amiya", "default", "front", "Idle"], frame: 6, total: 40, seconds: 0.2, scale: 1 })).toBe("amiya_default_front_Idle_f07-of-40_0.20s.png");
    });

    it("marks a scaled copy and makes skin names file-safe", () => {
        expect(framePngFilename({ parts: ["Amiya", "epoque#4", "dorm", "Sit/Idle"], frame: 0, total: 120, seconds: 0, scale: 2 })).toBe("Amiya_epoque-4_dorm_Sit-Idle_f001-of-120_0.00s@2x.png");
    });

    it("drops missing parts", () => {
        expect(framePngFilename({ parts: [undefined, "", "Attack"], frame: 2, total: 9, seconds: 0.07, scale: 1 })).toBe("Attack_f3-of-9_0.07s.png");
    });
});
