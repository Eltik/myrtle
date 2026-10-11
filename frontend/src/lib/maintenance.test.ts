import { describe, expect, it } from "vitest";
import type { MaintenanceWindow } from "#/types/generated/MaintenanceWindow";
import { formatSpan, isGameDown, maintenanceAppliesTo, maintenanceView } from "./maintenance";

const START = 1_800_000_000;
const window = (over: Partial<MaintenanceWindow> = {}): MaintenanceWindow => ({
    server: "en",
    phase: "upcoming",
    starts_at: START,
    ends_at: START + 4 * 3600,
    notice_url: "https://example.invalid/1.html",
    title: "MAINTENANCE",
    ...over,
});

describe("maintenanceView", () => {
    it("counts down to the start, then to the end", () => {
        expect(maintenanceView(window(), START - 90)).toEqual({ stage: "upcoming", secondsLeft: 90 });
        expect(maintenanceView(window(), START)).toEqual({ stage: "ongoing", secondsLeft: 4 * 3600 });
    });

    it("moves to ongoing on the client even when the poll still says upcoming", () => {
        expect(maintenanceView(window({ phase: "upcoming" }), START + 10).stage).toBe("ongoing");
    });

    it("reads an extension and an overrun apart", () => {
        expect(maintenanceView(window({ phase: "extended" }), START + 60).stage).toBe("extended");
        expect(maintenanceView(window({ phase: "extended" }), START + 4 * 3600)).toEqual({ stage: "overrun", secondsLeft: null });
    });

    it("has nothing to count down to without an end", () => {
        expect(maintenanceView(window({ ends_at: null }), START + 1)).toEqual({ stage: "openEnded", secondsLeft: null });
    });
});

describe("formatSpan", () => {
    it.each([
        [0, "0s"],
        [45, "45s"],
        [60, "1m"],
        [61, "2m"],
        [3600, "1h"],
        [2 * 3600 + 14 * 60, "2h 14m"],
        [86400, "1d"],
        [3 * 86400 + 4 * 3600, "3d 4h"],
    ])("%i s -> %s", (sec, out) => {
        expect(formatSpan(sec)).toBe(out);
    });
});

describe("maintenanceAppliesTo", () => {
    it("shows signed-out visitors and same-server players, hides other servers", () => {
        expect(maintenanceAppliesTo("en", null)).toBe(true);
        expect(maintenanceAppliesTo("en", "en")).toBe(true);
        expect(maintenanceAppliesTo("en", "cn")).toBe(false);
    });
});

describe("isGameDown", () => {
    it("is false only before the window starts", () => {
        expect((["upcoming", "ongoing", "extended", "overrun", "openEnded"] as const).map(isGameDown)).toEqual([false, true, true, true, true]);
    });
});
