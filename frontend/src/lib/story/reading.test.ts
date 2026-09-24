import { describe, expect, it } from "vitest";
import { clampMinutesPerDay, clampWpm, clockTime, DEFAULT_MINUTES_PER_DAY, DEFAULT_WPM, humanTime, minutesFor, paceSpan } from "./reading";

describe("humanTime", () => {
    it("prints minutes under an hour", () => {
        expect(humanTime(31)).toBe("31m");
        expect(humanTime(59.4)).toBe("59m");
    });

    it("prints hours and minutes up to a day, dropping a zero remainder", () => {
        expect(humanTime(448)).toBe("7h 28m");
        expect(humanTime(120)).toBe("2h");
    });

    it("prints days and hours from a day to a month", () => {
        expect(humanTime(1800)).toBe("1d 6h");
        expect(humanTime(1440)).toBe("1d");
    });

    it("prints months beyond one average month, to one decimal", () => {
        expect(humanTime(6 * 30.436875 * 1440)).toBe("6.0 months");
        expect(humanTime(30.436875 * 1440 + 1)).toBe("1.0 months");
        // A minute SHORT of the boundary is still a day span, which is the
        // rounding rule and not an accident: the shape steps at the unit.
        expect(humanTime(30.436875 * 1440 - 60)).toBe("30d 9h");
    });

    it("is 0m on nothing and on nonsense, never NaN", () => {
        expect(humanTime(0)).toBe("0m");
        expect(humanTime(-5)).toBe("0m");
        expect(humanTime(Number.NaN)).toBe("0m");
    });

    it("says under a minute rather than rounding a real span to zero", () => {
        expect(humanTime(0.4)).toBe("<1m");
    });
});

describe("minutesFor", () => {
    it("divides words by the speed", () => {
        expect(minutesFor(2250, 225)).toBe(10);
    });

    it("is 0 on an absent or negative count", () => {
        expect(minutesFor(0, 225)).toBe(0);
        expect(minutesFor(Number.NaN, 225)).toBe(0);
    });

    it("clamps an out-of-range speed rather than dividing by it", () => {
        expect(minutesFor(4000, 0)).toBe(100);
    });
});

describe("clampWpm", () => {
    it("keeps a value inside the band", () => {
        expect(clampWpm(400)).toBe(400);
    });

    it("clamps both ends", () => {
        expect(clampWpm(1)).toBe(40);
        expect(clampWpm(99999)).toBe(2000);
    });

    it("falls back to the default on a MISSING value rather than clamping zero", () => {
        expect(clampWpm(null)).toBe(DEFAULT_WPM);
        expect(clampWpm(undefined)).toBe(DEFAULT_WPM);
        expect(clampWpm("")).toBe(DEFAULT_WPM);
    });

    it("reads a typed string", () => {
        expect(clampWpm("300")).toBe(300);
    });
});

describe("clampMinutesPerDay", () => {
    it("clamps both ends and defaults on a missing value", () => {
        expect(clampMinutesPerDay(1)).toBe(5);
        expect(clampMinutesPerDay(5000)).toBe(960);
        expect(clampMinutesPerDay(null)).toBe(DEFAULT_MINUTES_PER_DAY);
    });
});

describe("paceSpan", () => {
    it("speaks in days under two months", () => {
        expect(paceSpan(120 * 30, 120)).toEqual({ unit: "day", value: 30 });
    });

    it("speaks in months beyond that, to one decimal", () => {
        expect(paceSpan(120 * 30.436875 * 6, 120)).toEqual({ unit: "month", value: 6 });
    });

    it("is zero days on nothing left", () => {
        expect(paceSpan(0, 120)).toEqual({ unit: "day", value: 0 });
    });
});

describe("clockTime", () => {
    it("prints m:ss with a padded second", () => {
        expect(clockTime(0)).toBe("0:00");
        expect(clockTime(9)).toBe("0:09");
        expect(clockTime(184)).toBe("3:04");
    });

    it("has no hour field: a cue past an hour reads 71:30 rather than lie", () => {
        expect(clockTime(71 * 60 + 30)).toBe("71:30");
    });

    it("floors the seconds and refuses a negative or non-finite one", () => {
        expect(clockTime(59.9)).toBe("0:59");
        expect(clockTime(-5)).toBe("0:00");
        expect(clockTime(Number.NaN)).toBe("0:00");
    });
});
