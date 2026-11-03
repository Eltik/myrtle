import { describe, expect, it } from "vitest";
import { FALLBACK_TIER_COLORS } from "#/lib/utils";
import { colorLuminance, contrastRatio, readableTextColor } from "./contrast";

describe("readableTextColor", () => {
    it("puts black on a light tier colour", () => {
        // oklch(0.78 0.15 92), the default yellow: black 10.47, white 2.01.
        expect(readableTextColor(FALLBACK_TIER_COLORS[2])).toBe("black");
        expect(readableTextColor("#f7d54c")).toBe("black");
    });

    it("puts white on a dark tier colour", () => {
        // oklch(0.50 0.04 285), the default slate: white 6.06, black 3.47.
        expect(readableTextColor(FALLBACK_TIER_COLORS[6])).toBe("white");
        expect(readableTextColor("#1e1e2e")).toBe("white");
    });

    it("picks the higher ratio on a mid tier colour", () => {
        // oklch(0.62 0.21 24), the default red: black 5.20, white 4.04. The old luminance-0.55 rule gave white.
        expect(readableTextColor(FALLBACK_TIER_COLORS[0])).toBe("black");
        // oklch(0.55 0.18 290), the default violet: white 5.22, black 4.03.
        expect(readableTextColor(FALLBACK_TIER_COLORS[5])).toBe("white");
    });

    it("never picks the lower ratio for any default tier colour", () => {
        for (const color of FALLBACK_TIER_COLORS) {
            const y = colorLuminance(color);
            expect(y).not.toBeNull();
            const black = contrastRatio(y as number, 0);
            const white = contrastRatio(y as number, 1);
            expect(readableTextColor(color)).toBe(black > white ? "black" : "white");
            expect(Math.max(black, white)).toBeGreaterThanOrEqual(4.5);
        }
    });

    it("falls back to white for a colour it cannot read", () => {
        expect(readableTextColor(null)).toBe("white");
        expect(readableTextColor("var(--primary)")).toBe("white");
        expect(readableTextColor("#zz")).toBe("white");
    });
});
