import { describe, expect, it } from "vitest";
import type { ReleaseBanner } from "#/types/generated/ReleaseBanner";
import type { ReleaseEvent } from "#/types/generated/ReleaseEvent";
import type { Resolution } from "#/types/generated/Resolution";
import { bannerItem, cnDay, DAY_SECS, dayOf, dayStart, eventArtIndex, eventItem, type IScheduleItem, overlaps, packLanes } from "./schedule";

const CN_START = 1_700_000_000;
const CN_END = CN_START + 14 * DAY_SECS;
const EN_START = 1_720_000_000;

const confirmed: Resolution = { status: "confirmed", enId: "act_en", enStart: EN_START, enEnd: EN_START + 10 * DAY_SECS };
const estimated: Resolution = { status: "estimated", enStart: EN_START, lo: EN_START - DAY_SECS, hi: EN_START + DAY_SECS };

function event(partial: Partial<ReleaseEvent>): ReleaseEvent {
    return {
        cnId: "act1",
        nameCn: "活动",
        nameEn: null,
        activityType: "SIDESTORY",
        hasStage: true,
        cnStart: CN_START,
        cnEnd: CN_END,
        opStages: [],
        farmStages: [],
        missionTokens: 0,
        shop: null,
        nameEnAuto: null,
        imagePath: null,
        resolution: estimated,
        ...partial,
    };
}

function banner(partial: Partial<ReleaseBanner>): ReleaseBanner {
    return {
        cnPoolId: "LIMITED_1",
        ruleType: "LIMITED",
        nameCn: "寻访",
        cnOpen: CN_START,
        cnEnd: CN_END,
        standing: false,
        featured6: [],
        featured5: [],
        debutChars: [],
        anchorActivity: null,
        alignment: {} as ReleaseBanner["alignment"],
        enFeatured6: [],
        overrideFeatured: [],
        nameEnAuto: null,
        imagePath: null,
        resolution: estimated,
        ...partial,
    };
}

function item(key: string, start: number, end: number | null): IScheduleItem {
    return { key, start, end } as IScheduleItem;
}

describe("cnDay", () => {
    it("formats the date as seen in China (UTC+8)", () => {
        expect(cnDay(Date.UTC(2026, 0, 1, 15, 59) / 1000)).toBe("2026-01-01");
        expect(cnDay(Date.UTC(2026, 0, 1, 16, 0) / 1000)).toBe("2026-01-02");
    });
});

describe("eventItem", () => {
    it("projects an estimated event with the CN run length", () => {
        const it = eventItem(event({}));
        expect(it).toMatchObject({ key: "event:act1", kind: "event", tag: "SIDESTORY", start: EN_START, end: EN_START + 14 * DAY_SECS, cnStart: CN_START, cnEnd: CN_END, estimated: true, charIds: [] });
    });

    it("uses the confirmed EN end", () => {
        const it = eventItem(event({ resolution: confirmed }));
        expect(it?.end).toBe(EN_START + 10 * DAY_SECS);
        expect(it?.estimated).toBe(false);
    });

    it("keeps an override's own end only when it has one", () => {
        const withEnd: Resolution = { status: "override", enId: null, enStart: EN_START, enEnd: EN_START + 3 * DAY_SECS, source: "manual", note: "" };
        const noEnd: Resolution = { status: "override", enId: null, enStart: EN_START, enEnd: null, source: "manual", note: "" };
        expect(eventItem(event({ resolution: withEnd }))?.end).toBe(EN_START + 3 * DAY_SECS);
        expect(eventItem(event({ resolution: noEnd }))?.end).toBe(EN_START + 14 * DAY_SECS);
    });

    it("has no end when the CN window is open or inverted", () => {
        expect(eventItem(event({ cnEnd: 0 }))?.end).toBeNull();
        expect(eventItem(event({ cnEnd: CN_START - 1 }))?.end).toBeNull();
    });

    it("drops events EN has no date for", () => {
        for (const status of ["unlisted", "unmodelled", "independent"] as const) {
            expect(eventItem(event({ resolution: { status } }))).toBeNull();
        }
    });
});

describe("bannerItem", () => {
    it("skips standing pools", () => {
        expect(bannerItem(banner({ standing: true }))).toBeNull();
    });

    it("dedupes featured characters in EN, CN, override, debut order", () => {
        const it = bannerItem(banner({ enFeatured6: ["char_b"], featured6: ["char_a", "char_b"], overrideFeatured: ["char_c"], debutChars: ["char_a", "char_d"] }));
        expect(it?.charIds).toEqual(["char_b", "char_a", "char_c", "char_d"]);
        expect(it).toMatchObject({ key: "banner:LIMITED_1", kind: "banner", tag: "LIMITED", nameEn: null });
    });

    it("borrows event art by anchor, then by CN day, unless the rule is independent", () => {
        const art = eventArtIndex([event({ cnId: "act_anchor", imagePath: "anchor.png", cnStart: CN_START - 5 * DAY_SECS }), event({ cnId: "act_same_day", imagePath: "day.png" })]);
        expect(bannerItem(banner({ anchorActivity: "act_anchor" }), art)?.imagePath).toBe("anchor.png");
        expect(bannerItem(banner({ anchorActivity: "act_missing" }), art)?.imagePath).toBe("day.png");
        expect(bannerItem(banner({}), art, false)?.imagePath).toBeNull();
        expect(bannerItem(banner({ imagePath: "own.png" }), art, false)?.imagePath).toBe("own.png");
    });
});

describe("eventArtIndex", () => {
    it("keys art by id and keeps the first staged event per CN day", () => {
        const art = eventArtIndex([event({ cnId: "a", imagePath: "a.png" }), event({ cnId: "b", imagePath: "b.png" }), event({ cnId: "c", imagePath: "c.png", hasStage: false, cnStart: CN_START + DAY_SECS }), event({ cnId: "d", imagePath: null })]);
        expect([...art.byId]).toEqual([
            ["a", "a.png"],
            ["b", "b.png"],
            ["c", "c.png"],
        ]);
        expect([...art.byDay]).toEqual([[cnDay(CN_START), "a.png"]]);
    });
});

describe("overlaps", () => {
    it("is inclusive at both ends and treats an open item as a point", () => {
        const it = item("x", 100, 200);
        expect(overlaps(it, 200, 300)).toBe(true);
        expect(overlaps(it, 0, 100)).toBe(true);
        expect(overlaps(it, 201, 300)).toBe(false);
        expect(overlaps(item("p", 100, null), 100, 100)).toBe(true);
        expect(overlaps(item("p", 100, null), 101, 150)).toBe(false);
    });
});

describe("packLanes", () => {
    it("reuses the first lane that ended strictly before the next start", () => {
        const lanes = packLanes([item("c", 250, 300), item("a", 0, 100), item("b", 50, 200), item("d", 100, 120)]);
        expect(Object.fromEntries(lanes)).toEqual({ a: 0, b: 1, d: 2, c: 0 });
    });

    it("pads each item by the gap", () => {
        const items = [item("a", 0, 100), item("b", 105, 200)];
        expect(Object.fromEntries(packLanes(items))).toEqual({ a: 0, b: 0 });
        expect(Object.fromEntries(packLanes(items, 10))).toEqual({ a: 0, b: 1 });
    });

    it("handles open-ended items and an empty list", () => {
        expect(Object.fromEntries(packLanes([item("a", 0, null), item("b", 1, null)]))).toEqual({ a: 0, b: 0 });
        expect(packLanes([]).size).toBe(0);
    });
});

describe("dayStart and dayOf", () => {
    it("snap to local midnight", () => {
        const noon = new Date(2026, 4, 17, 12, 34, 56);
        const start = dayStart(noon);
        expect(start).toBe(new Date(2026, 4, 17).getTime() / 1000);
        expect(dayOf(Math.floor(noon.getTime() / 1000))).toBe(start);
        expect(dayOf(start)).toBe(start);
    });
});
