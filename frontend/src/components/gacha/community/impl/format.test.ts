import { afterEach, describe, expect, it, vi } from "vitest";
import { compareRate, fmtPct, fmtRelative, fmtUTCStamp } from "./format";

describe("fmtPct", () => {
    it("formats a fraction with two decimals by default", () => {
        expect(fmtPct(0.02)).toBe("2.00%");
        expect(fmtPct(0.123456, 1)).toBe("12.3%");
        expect(fmtPct(1)).toBe("100.00%");
    });
});

describe("compareRate", () => {
    it("reads the observed rate as a share of the baseline", () => {
        expect(compareRate(0.04, 0.02)).toEqual({ ratio: 2, ratioLabel: "200% of expected", deviationLabel: "above by 100%", deviation: 1, direction: "above" });
        const below = compareRate(0.0122, 0.02);
        expect(below?.direction).toBe("below");
        expect(below?.ratioLabel).toBe("61% of expected");
        expect(below?.deviationLabel).toBe("below by 39%");
    });

    it("calls anything within 1% of the baseline on target", () => {
        const on = compareRate(0.0201, 0.02);
        expect(on?.direction).toBe("on");
        expect(on?.ratioLabel).toBe("on target");
        expect(on?.deviationLabel).toBe("on target");
        expect(compareRate(0.0203, 0.02)?.direction).toBe("above");
    });

    it("keeps one decimal on small deviations", () => {
        expect(compareRate(0.0212, 0.02)?.deviationLabel).toBe("above by 6.0%");
    });

    it("switches to a multiplier from 10x up", () => {
        expect(compareRate(0.2, 0.02)?.ratioLabel).toBe("10.0× baseline");
        expect(compareRate(0.1998, 0.02)?.ratioLabel).toBe("999% of expected");
    });

    it("refuses a non-positive or non-finite baseline", () => {
        expect(compareRate(0.02, 0)).toBeNull();
        expect(compareRate(0.02, -1)).toBeNull();
        expect(compareRate(Number.NaN, 0.02)).toBeNull();
        expect(compareRate(0.02, Number.POSITIVE_INFINITY)).toBeNull();
    });
});

describe("fmtRelative", () => {
    afterEach(() => vi.useRealTimers());

    it("buckets the age into moments, minutes, hours and days", () => {
        vi.useFakeTimers();
        vi.setSystemTime(new Date("2026-10-08T12:00:00Z"));
        expect(fmtRelative("2026-10-08T11:59:30Z")).toBe("moments ago");
        expect(fmtRelative("2026-10-08T11:15:00Z")).toBe("45m ago");
        expect(fmtRelative("2026-10-08T07:00:00Z")).toBe("5h ago");
        expect(fmtRelative("2026-10-05T11:00:00Z")).toBe("3d ago");
        // A future stamp is clamped to now.
        expect(fmtRelative("2026-10-09T00:00:00Z")).toBe("moments ago");
    });

    it("dashes a missing or unreadable stamp", () => {
        expect(fmtRelative(null)).toBe("-");
        expect(fmtRelative("")).toBe("-");
        expect(fmtRelative("not a date")).toBe("-");
    });
});

describe("fmtUTCStamp", () => {
    it("prints minutes in UTC", () => {
        expect(fmtUTCStamp("2026-10-08T21:09:45+09:00")).toBe("2026-10-08 12:09 UTC");
        expect(fmtUTCStamp(undefined)).toBe("-");
        expect(fmtUTCStamp("nope")).toBe("-");
    });
});
