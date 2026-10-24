import { describe, expect, it } from "vitest";
import type { ReleaseOverride } from "#/types/generated/ReleaseOverride";
import { overrideForm, putOverrideInput, scheduleOverrideTarget, unixToUtcInput, utcInputToUnix, validateOverrideForm } from "./overrides";

const existing: ReleaseOverride = {
    kind: "pool",
    cnId: "SINGLE_42_0_1",
    enId: null,
    enName: "Example banner",
    enStart: Date.UTC(2026, 10, 3, 8) / 1000,
    enEnd: Date.UTC(2026, 10, 17, 3, 59) / 1000,
    featuredChars: ["char_002_amiya"],
    source: "manual",
    note: "",
    updatedAt: 0,
};

describe("unixToUtcInput / utcInputToUnix", () => {
    it("reads and writes in UTC whatever the local zone", () => {
        const s = Date.UTC(2026, 10, 3, 8, 0) / 1000;
        expect(unixToUtcInput(s)).toBe("2026-11-03T08:00");
        expect(utcInputToUnix("2026-11-03T08:00")).toBe(s);
    });

    it("round-trips minute-aligned times and drops seconds on the way out", () => {
        const s = Date.UTC(2027, 0, 1, 23, 59) / 1000;
        expect(utcInputToUnix(unixToUtcInput(s))).toBe(s);
        expect(unixToUtcInput(s + 42)).toBe("2027-01-01T23:59");
    });

    it("accepts the seconds a stepped input adds", () => {
        expect(utcInputToUnix("2026-11-03T08:00:30")).toBe(Date.UTC(2026, 10, 3, 8, 0, 30) / 1000);
    });

    it("refuses empty, malformed and impossible values", () => {
        expect(utcInputToUnix("")).toBeNull();
        expect(utcInputToUnix("2026-11-03")).toBeNull();
        expect(utcInputToUnix("not a date")).toBeNull();
        expect(utcInputToUnix("2026-02-30T00:00")).toBeNull();
        expect(utcInputToUnix("2026-11-03T24:00")).toBeNull();
    });
});

describe("scheduleOverrideTarget", () => {
    it("maps event, banner and review keys to their override kind", () => {
        expect(scheduleOverrideTarget("event:act42sre")).toEqual({ kind: "activity", cnId: "act42sre" });
        expect(scheduleOverrideTarget("banner:SINGLE_42_0_1")).toEqual({ kind: "pool", cnId: "SINGLE_42_0_1" });
        expect(scheduleOverrideTarget("skin:review:1761868800")).toEqual({ kind: "review", cnId: "1761868800" });
    });

    it("gives skin groups and reruns none", () => {
        expect(scheduleOverrideTarget("skin:act42sre")).toBeNull();
        expect(scheduleOverrideTarget("rerun:whatever")).toBeNull();
    });
});

describe("validateOverrideForm", () => {
    const blank = overrideForm("activity", "act42sre", null);

    it("requires a start and a CN id", () => {
        expect(validateOverrideForm({ ...blank, cnId: " " })).toEqual({ cnId: "cnIdRequired", enStart: "startRequired" });
    });

    it("wants the end after the start", () => {
        expect(validateOverrideForm({ ...blank, enStart: "2026-11-03T08:00", enEnd: "2026-11-03T08:00" })).toEqual({ enEnd: "endBeforeStart" });
        expect(validateOverrideForm({ ...blank, enStart: "2026-11-03T08:00", enEnd: "2026-11-17T03:59" })).toEqual({});
    });

    it("leaves an empty end alone", () => {
        expect(validateOverrideForm({ ...blank, enStart: "2026-11-03T08:00" })).toEqual({});
    });
});

describe("overrideForm / putOverrideInput", () => {
    it("prefills a new activity override's EN id with the CN id only", () => {
        expect(overrideForm("activity", "act42sre", null).enId).toBe("act42sre");
        expect(overrideForm("pool", "SINGLE_42_0_1", null).enId).toBe("");
    });

    it("carries the fields the form does not show across an edit", () => {
        const form = { ...overrideForm("pool", existing.cnId, existing), note: "  stream  " };
        expect(putOverrideInput(form, existing)).toEqual({
            kind: "pool",
            cnId: "SINGLE_42_0_1",
            enId: null,
            enName: "Example banner",
            featuredChars: ["char_002_amiya"],
            enStart: existing.enStart,
            enEnd: existing.enEnd,
            source: "manual",
            note: "stream",
        });
    });

    it("sends blanks as null so the backend applies its defaults", () => {
        const form = { ...overrideForm("review", "1761868800", null), enStart: "2026-11-03T08:00" };
        expect(putOverrideInput(form, null)).toMatchObject({ enId: null, enName: null, featuredChars: null, enEnd: null, source: null, note: null });
    });
});
