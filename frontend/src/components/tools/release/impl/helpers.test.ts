import { describe, expect, it } from "vitest";
import { assetUrl, daysFromToday, formatDateRange, formatDays, formatPercent, fromUnix, humanizeTag, relativeDays, weekdayNames } from "./helpers";

/** Unix seconds at local noon, so a local-date rendering never straddles midnight. */
const localNoon = (y: number, m: number, d: number) => new Date(y, m, d, 12).getTime() / 1000;

describe("formatDays", () => {
    it("keeps integers bare and rounds the rest to one decimal", () => {
        expect(formatDays(12)).toBe("12");
        expect(formatDays(0)).toBe("0");
        expect(formatDays(12.345)).toBe("12.3");
        expect(formatDays(-1.25)).toBe("-1.3");
    });
});

describe("formatPercent", () => {
    it("rounds a fraction to a whole percent", () => {
        expect(formatPercent(0.5)).toBe("50%");
        expect(formatPercent(0.666)).toBe("67%");
        expect(formatPercent(0)).toBe("0%");
        expect(formatPercent(1.005)).toBe("100%");
    });
});

describe("humanizeTag", () => {
    it("turns an upper snake code into a sentence-case label", () => {
        expect(humanizeTag("SIDE_STORY")).toBe("Side story");
        expect(humanizeTag("LIMITED")).toBe("Limited");
        expect(humanizeTag("")).toBe("");
    });
});

describe("relativeDays", () => {
    it("names the nearby days and counts the rest", () => {
        expect(relativeDays(0)).toBe("today");
        expect(relativeDays(1)).toBe("tomorrow");
        expect(relativeDays(-1)).toBe("yesterday");
        expect(relativeDays(5)).toBe("in 5 days");
        expect(relativeDays(-3)).toBe("3 days ago");
    });
});

describe("daysFromToday", () => {
    it("counts calendar days, ignoring the time of day", () => {
        const today = new Date(2026, 9, 8, 23, 30);
        expect(daysFromToday(localNoon(2026, 9, 8), today)).toBe(0);
        expect(daysFromToday(new Date(2026, 9, 9, 0, 5).getTime() / 1000, today)).toBe(1);
        expect(daysFromToday(localNoon(2026, 9, 1), today)).toBe(-7);
        expect(daysFromToday(localNoon(2027, 0, 1), today)).toBe(85);
    });
});

describe("formatDateRange", () => {
    it("shows a single date when there is no real end", () => {
        const start = localNoon(2026, 0, 15);
        expect(formatDateRange(start, null, "en-US")).toBe("Jan 15, 2026");
        expect(formatDateRange(start, start, "en-US")).toBe("Jan 15, 2026");
        expect(formatDateRange(start, start - 60, "en-US")).toBe("Jan 15, 2026");
    });

    it("drops the start year inside one year and keeps both across years", () => {
        expect(formatDateRange(localNoon(2026, 0, 15), localNoon(2026, 0, 29), "en-US")).toBe("Jan 15 to Jan 29, 2026");
        expect(formatDateRange(localNoon(2026, 11, 28), localNoon(2027, 0, 11), "en-US")).toBe("Dec 28, 2026 to Jan 11, 2027");
    });
});

describe("weekdayNames", () => {
    it("starts on Sunday", () => {
        expect(weekdayNames("en-US")).toEqual(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);
    });
});

describe("fromUnix and assetUrl", () => {
    it("converts seconds to a Date", () => {
        expect(fromUnix(1).getTime()).toBe(1000);
    });

    it("encodes each path segment and prefixes the api root", () => {
        expect(assetUrl(null)).toBeNull();
        expect(assetUrl("")).toBeNull();
        expect(assetUrl("/upk/arts/a b#1.png")?.endsWith("/api/upk/arts/a%20b%231.png")).toBe(true);
        expect(assetUrl("upk/x.png")?.endsWith("/api/upk/x.png")).toBe(true);
    });
});
