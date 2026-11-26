import { describe, expect, it } from "vitest";
import type { IBanner, IGachaItem, IGachaRecords } from "#/lib/api/gacha";
import { bannerOptions, filterRecordsByPools, formatBannerParam, MAX_BANNER_FILTER, parseBannerParam } from "./banner-filter";

function pull(poolId: string, at: number, poolName = ""): IGachaItem {
    return { charId: "char_x", charName: "", star: "3", color: "", poolId, poolName, typeName: "", at, atStr: "" } as IGachaItem;
}

function records(limited: IGachaItem[], regular: IGachaItem[] = [], special: IGachaItem[] = []): IGachaRecords {
    const group = (gacha_type: string, list: IGachaItem[]) => ({ gacha_type, records: list, total: list.length });
    return { limited: group("limited", limited), regular: group("regular", regular), special: group("special", special) } as IGachaRecords;
}

describe("parseBannerParam", () => {
    it("reads distinct pool ids in the order given", () => {
        expect(parseBannerParam("LIMITED_EN_40_0_4, SINGLE_EN_40_0_1,LIMITED_EN_40_0_4")).toEqual(["LIMITED_EN_40_0_4", "SINGLE_EN_40_0_1"]);
    });

    it("drops what the backend would refuse instead of failing the page", () => {
        expect(parseBannerParam("LIMITED'--,NORM_0_0_1,a b,")).toEqual(["NORM_0_0_1"]);
        expect(parseBannerParam(undefined)).toEqual([]);
        expect(parseBannerParam(42)).toEqual([]);
        expect(parseBannerParam("")).toEqual([]);
    });

    it("stops at the backend's cap", () => {
        const many = Array.from({ length: MAX_BANNER_FILTER + 5 }, (_, i) => `P_${i}`).join(",");
        expect(parseBannerParam(many)).toHaveLength(MAX_BANNER_FILTER);
    });

    it("round-trips through formatBannerParam", () => {
        expect(parseBannerParam(formatBannerParam(["A_1", "B_2"]))).toEqual(["A_1", "B_2"]);
        expect(formatBannerParam([])).toBeUndefined();
    });
});

describe("filterRecordsByPools", () => {
    const all = records([pull("LIMITED_A", 3), pull("LIMITED_B", 2)], [pull("NORM_A", 1)]);

    it("keeps only the chosen banners and recounts each group", () => {
        const kept = filterRecordsByPools(all, ["LIMITED_B", "NORM_A"]);
        expect(kept.limited.records.map((r) => r.poolId)).toEqual(["LIMITED_B"]);
        expect(kept.limited.total).toBe(1);
        expect(kept.regular.total).toBe(1);
        expect(kept.special.total).toBe(0);
    });

    it("is no filter when nothing is chosen", () => {
        expect(filterRecordsByPools(all, [])).toBe(all);
    });
});

describe("bannerOptions", () => {
    it("lists each banner once, newest last pull first, named from game data first", () => {
        const banners = new Map([["LIMITED_A", { gachaPoolName: "Golden Fleece" } as IBanner]]);
        const options = bannerOptions(records([pull("LIMITED_A", 10), pull("LIMITED_A", 30), pull("LIMITED_B", 20, "Stored name")], [pull("NORM_A", 5)]), banners);
        expect(options).toEqual([
            { poolId: "LIMITED_A", label: "Golden Fleece", pulls: 2, lastAt: 30 },
            { poolId: "LIMITED_B", label: "Stored name", pulls: 1, lastAt: 20 },
            { poolId: "NORM_A", label: "NORM_A", pulls: 1, lastAt: 5 },
        ]);
    });

    it("is empty before the records load", () => {
        expect(bannerOptions(null, new Map())).toEqual([]);
    });
});
