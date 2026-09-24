import { describe, expect, it } from "vitest";
import { clamp, clamp01 } from "./num";

describe("clamp", () => {
    it("holds a value between the bounds", () => {
        expect(clamp(5, 0, 10)).toBe(5);
        expect(clamp(-1, 0, 10)).toBe(0);
        expect(clamp(11, 0, 10)).toBe(10);
    });

    it("keeps the bounds themselves", () => {
        expect(clamp(0, 0, 10)).toBe(0);
        expect(clamp(10, 0, 10)).toBe(10);
    });

    it("passes NaN through, which is what every hand-written copy did", () => {
        expect(clamp(Number.NaN, 0, 1)).toBeNaN();
        expect(clamp01(Number.NaN)).toBeNaN();
    });
});

describe("clamp01", () => {
    it("is the 0..1 case", () => {
        expect(clamp01(0.5)).toBe(0.5);
        expect(clamp01(-0.2)).toBe(0);
        expect(clamp01(1.7)).toBe(1);
    });

    it("agrees with the engine's old `n < 0 ? 0 : n > max ? max : n`", () => {
        for (const n of [-3, -1, -0.0001, 0, 0.25, 1, 1.0001, 255]) {
            expect(clamp01(n)).toBe(n < 0 ? 0 : n > 1 ? 1 : n);
        }
    });
});
