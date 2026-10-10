import { describe, expect, it } from "vitest";
import type { ActivityBasicInfo } from "#/types/generated/ActivityBasicInfo";
import type { GachaPoolClient } from "#/types/generated/GachaPoolClient";
import { countdownParts, elapsedPercent, LIVE_LIMIT, liveCandidates, liveItems, namedFeatured, selectLive, showsBannerPlaceholder } from "./live";

function event(id: string, startTime: number, endTime: number, hasStage = true, extra: Partial<ActivityBasicInfo> = {}): ActivityBasicInfo {
    return { id, name: id, startTime, endTime, rewardEndTime: 0, medalGroupId: "", hasStage, isReplicate: false, type: "TYPE_ACT1SIDE", displayType: "SIDESTORY", templateShopId: null, ...extra };
}

function pool(gachaPoolId: string, openTime: number, endTime: number, extra: Partial<GachaPoolClient> = {}): GachaPoolClient {
    return { gachaPoolId, openTime, endTime, gachaPoolName: gachaPoolId, gachaRuleType: "LIMITED", featured6: [], featured5: [], ...extra } as unknown as GachaPoolClient;
}

describe("liveItems", () => {
    const now = 1000;

    it("merges running events and banners, soonest end first, capped at three", () => {
        const items = liveItems([event("a", 0, 5000), event("b", 0, 2000)], [pool("x", 0, 3000), pool("y", 0, 1500)], "en", now);
        expect(LIVE_LIMIT).toBe(3);
        expect(items.map((i) => i.key)).toEqual(["banner:y", "event:b", "banner:x"]);
    });

    it("drops stageless events, returning pools, and anything not running at now", () => {
        const items = liveItems([event("stageless", 0, 2000, false), event("future", 1001, 2000), event("over", 0, 999)], [pool("returning", 0, 2000, { returning: true }), pool("future", 2000, 3000)], "en", now);
        expect(items).toEqual([]);
    });

    it("counts a window that starts or ends exactly at now as running", () => {
        const items = liveItems([event("starts", 1000, 2000)], [pool("ends", 0, 1000)], "en", now);
        expect(items.map((i) => i.key)).toEqual(["banner:ends", "event:starts"]);
    });

    it("breaks an end-time tie by key", () => {
        const items = liveItems([event("b", 0, 2000)], [pool("a", 0, 2000)], "en", now);
        expect(items.map((i) => i.key)).toEqual(["banner:a", "event:b"]);
    });
});

describe("returning pools", () => {
    it("drops the pool the backend flags, whatever its window", () => {
        // RETURN_EN_41_0_1 (BACKFLOW) carries a 1,513 to 1,523 day placeholder window.
        const backflow = pool("RETURN_EN_41_0_1", 0, 1523 * 86_400, { gachaRuleType: "BACKFLOW", returning: true });
        const short = pool("RETURN_SHORT", 0, 2000, { returning: true });
        expect(liveItems([], [backflow, short], "en", 1000)).toEqual([]);
    });

    it("keeps a long pool the backend does not flag: the window length is not the signal", () => {
        const long = pool("LIMITED_LONG", 0, 1523 * 86_400, { returning: false });
        expect(liveItems([], [long], "en", 1000).map((i) => i.key)).toEqual(["banner:LIMITED_LONG"]);
    });
});

describe("normalised items", () => {
    it("carries the server's own name, type, window and featured lists", () => {
        const [banner] = liveItems([], [pool("DOUBLE_JP_41_0_9", 0, 2000, { gachaPoolName: "スカウト", gachaRuleType: "DOUBLE", featured6: ["char_1"], featured5: ["char_2", "char_3"] })], "jp", 1000);
        expect(banner).toEqual({ kind: "banner", key: "banner:DOUBLE_JP_41_0_9", name: "スカウト", type: "DOUBLE", start: 0, end: 2000, featured6: ["char_1"], featured5: ["char_2", "char_3"], artPaths: ["/jp/banner-image/DOUBLE_JP_41_0_9"] });

        const [act] = liveItems([event("act3break", 0, 2000, true, { name: "矢量突破", type: "VEC_BREAK_V2" })], [], "cn", 1000);
        expect(act).toEqual({ kind: "event", key: "event:act3break", name: "矢量突破", type: "VEC_BREAK_V2", start: 0, end: 2000, featured6: [], featured5: [], artPaths: ["/cn/event-image/act3break"] });
    });

    it("uses the bare art path on the default server", () => {
        const [act] = liveItems([event("act54side", 0, 2000)], [], "en", 1000);
        expect(act?.artPaths).toEqual(["/event-image/act54side"]);
    });

    it("retries a rerun's art as the original side story's", () => {
        const [act] = liveItems([event("act20sre", 0, 2000)], [], "kr", 1000);
        expect(act?.artPaths).toEqual(["/kr/event-image/act20sre", "/kr/event-image/act20side"]);
    });
});

describe("liveCandidates + selectLive", () => {
    it("ships every window not yet ended, upcoming ones included, and nothing ended", () => {
        const feed = liveCandidates([event("over", 0, 999), event("running", 0, 2000), event("stageless", 0, 2000, false)], [pool("upcoming", 1500, 3000), pool("returning", 0, 5000, { returning: true })], "en", 1000);
        expect(feed.map((i) => i.key)).toEqual(["event:running", "banner:upcoming"]);
    });

    it("drops a window as it closes and takes in the next, as the page ticks", () => {
        const feed = liveCandidates([event("a", 0, 1100), event("b", 0, 1200), event("c", 0, 1300), event("d", 0, 1400)], [pool("later", 1150, 5000)], "en", 1000);
        expect(selectLive(feed, 1000).map((i) => i.key)).toEqual(["event:a", "event:b", "event:c"]);
        expect(selectLive(feed, 1101).map((i) => i.key)).toEqual(["event:b", "event:c", "event:d"]);
        expect(selectLive(feed, 1250).map((i) => i.key)).toEqual(["event:c", "event:d", "banner:later"]);
    });
});

describe("namedFeatured", () => {
    it("names the 6★ rate-ups, or the 5★ ones when there are none", () => {
        expect(namedFeatured({ featured6: ["six"], featured5: ["five"] })).toEqual(["six"]);
        expect(namedFeatured({ featured6: [], featured5: ["five"] })).toEqual(["five"]);
    });
});

describe("elapsedPercent", () => {
    it("clamps to 0..100", () => {
        expect(elapsedPercent(100, 200, 50)).toBe(0);
        expect(elapsedPercent(100, 200, 150)).toBe(50);
        expect(elapsedPercent(100, 200, 250)).toBe(100);
    });

    it("reads a zero-length window as done", () => {
        expect(elapsedPercent(100, 100, 100)).toBe(100);
    });
});

describe("countdownParts", () => {
    it("splits days from a padded clock", () => {
        expect(countdownParts(((2 * 24 + 3) * 3600 + 4 * 60 + 5) * 1000 + 999)).toEqual({ days: 2, clock: "03:04:05" });
    });

    it("returns null once the window has ended", () => {
        expect(countdownParts(0)).toBeNull();
        expect(countdownParts(-1)).toBeNull();
    });
});

describe("showsBannerPlaceholder", () => {
    const banner = (featured6: string[], featured5: string[]) => ({ kind: "banner" as const, featured6, featured5 });

    it("is true only for a banner with neither art nor rate-ups (DOUBLE_JP_41_0_9)", () => {
        expect(showsBannerPlaceholder(banner([], []), false)).toBe(true);
    });

    it("keeps a banner's art even when it names no rate-up", () => {
        expect(showsBannerPlaceholder(banner([], []), true)).toBe(false);
    });

    it("shows faces, not the placeholder, when only 6★ or only 5★ rate-ups exist", () => {
        expect(showsBannerPlaceholder(banner(["char_1"], []), false)).toBe(false);
        expect(showsBannerPlaceholder(banner([], ["char_2"]), false)).toBe(false);
    });

    it("never applies to an event", () => {
        expect(showsBannerPlaceholder({ kind: "event", featured6: [], featured5: [] }, false)).toBe(false);
    });
});
