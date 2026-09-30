import { describe, expect, it } from "vitest";
import type { Resolution } from "#/types/generated/Resolution";
import type { SkinsResponse } from "#/types/generated/SkinsResponse";
import { isPast, resolvedEnStart, sortKey } from "./helpers";
import { dueNow, isOverdue, skinsDueNow } from "./resolution";

const DAY = 86_400;
/** 2026-09-30, local noon, so the day boundary is never in play. */
const TODAY = new Date(2026, 8, 30, 12);
const TODAY_START = Math.floor(new Date(2026, 8, 30).getTime() / 1000);
/** The Fashion Review estimate production served: 2026-09-28. */
const SEP_28 = Math.floor(new Date(2026, 8, 28, 19, 30).getTime() / 1000);
const MODEL = { window: 10, n: 10, medianDays: 158, p25Days: 150, p75Days: 165, samples: [] };
const CN_START = Math.floor(new Date(2026, 3, 23, 20).getTime() / 1000);

const estimate = (enStart: number, lo = enStart - 5 * DAY, hi = enStart + 5 * DAY): Resolution => ({ status: "estimated", enStart, lo, hi });

describe("dueNow", () => {
    it("moves a passed estimate to today and clamps its window", () => {
        const r = dueNow(estimate(SEP_28), TODAY);
        expect(r).toMatchObject({ status: "estimated", enStart: TODAY_START, lo: TODAY_START, hi: SEP_28 + 5 * DAY, overdue: true, estimatedStart: SEP_28 });
        expect(isOverdue(r)).toBe(true);
    });

    it("clamps a window that ended before today to today", () => {
        const r = dueNow(estimate(SEP_28, SEP_28 - 9 * DAY, SEP_28 + DAY), TODAY);
        expect(r).toMatchObject({ lo: TODAY_START, hi: TODAY_START });
    });

    it("leaves today, the future and every other status alone", () => {
        const today = estimate(TODAY_START + 3600);
        const later = estimate(TODAY_START + 10 * DAY);
        const confirmed: Resolution = { status: "confirmed", enId: "x", enStart: SEP_28, enEnd: SEP_28 + 14 * DAY };
        const override: Resolution = { status: "override", enId: null, enStart: SEP_28, enEnd: null, source: "stream", note: "" };
        const unlisted: Resolution = { status: "unlisted" };
        for (const r of [today, later, confirmed, override, unlisted]) {
            expect(dueNow(r, TODAY)).toBe(r);
            expect(isOverdue(r)).toBe(false);
        }
    });

    it("is idempotent", () => {
        const once = dueNow(estimate(SEP_28), TODAY);
        expect(dueNow(once, new Date(2026, 9, 5))).toBe(once);
    });
});

describe("skinsDueNow", () => {
    it("normalises every resolution in the response, and only those", () => {
        const passed = estimate(SEP_28);
        const res = {
            model: MODEL,
            yearly: { types: [], model: MODEL },
            anniversaries: [],
            batches: [{ name: "多主题时装/II", nameEnAuto: null, cnStart: CN_START, cnEnd: CN_START + 28 * DAY, resolution: passed }],
            newSkins: [],
            rerunForecasts: [{ skinGroupId: "g", skinGroupName: "EPOQUE", skinIds: [], skins: [], windows: [], lastSeen: 0, next: passed, basis: { kind: "none" } }],
            reviews: [
                { cnStart: CN_START, cnEnd: CN_START + 28 * DAY, poolCutoff: 0, resolution: { status: "unlisted" } },
                { cnStart: CN_START + 90 * DAY, cnEnd: 0, poolCutoff: 0, resolution: estimate(TODAY_START + 60 * DAY) },
            ],
            reviewPool: [],
            groupArt: {},
        } satisfies SkinsResponse;
        const out = skinsDueNow(res, TODAY);
        expect(isOverdue(out.batches[0].resolution)).toBe(true);
        expect(isOverdue(out.rerunForecasts[0].next)).toBe(true);
        expect(out.reviews[0]).toBe(res.reviews[0]);
        expect(out.reviews[1]).toBe(res.reviews[1]);
        expect(res.batches[0].resolution).toBe(passed);
    });
});

describe("sortKey and isPast", () => {
    it("put an overdue row at today, upcoming", () => {
        const r = dueNow(estimate(SEP_28), TODAY);
        expect(sortKey(r, CN_START, MODEL)).toBe(TODAY_START);
        expect(isPast(sortKey(r, CN_START, MODEL), TODAY)).toBe(false);
    });

    it("read a stale estimate as history until the normaliser moves it to today", () => {
        const stale = estimate(TODAY_START - 20 * DAY);
        expect(isPast(sortKey(stale, CN_START, MODEL), TODAY)).toBe(true);
        expect(isPast(sortKey(dueNow(stale, TODAY), CN_START, MODEL), TODAY)).toBe(false);
    });

    it("sort an unlisted row at its CN start, never CN start plus the median lag", () => {
        const r: Resolution = { status: "unlisted" };
        expect(resolvedEnStart(r)).toBeNull();
        expect(sortKey(r, CN_START, MODEL)).toBe(CN_START);
        expect(sortKey(r, CN_START, MODEL)).not.toBe(CN_START + MODEL.medianDays * DAY);
        expect(isPast(sortKey(r, CN_START, MODEL), TODAY)).toBe(true);
    });
});
