import { describe, expect, it } from "vitest";
import { deriveClientGachaRecords, type IGachaItem, type IGachaRecords } from "#/lib/api/gacha";
import { DEFAULT_HISTORY_TAB, HISTORY_TABS, isTabShown, mergeAllGroups, recordsForTab, totalForTab } from "./historyTabs";

function pull(poolId: string, at: number, charId = `char_${poolId}_${at}`): IGachaItem {
    return { charId, charName: "", star: "3", color: "", poolId, poolName: "", typeName: "", at, atStr: "" } as IGachaItem;
}

/** The API's three raw buckets, split into the five UI groups the way the page does it. */
function records(items: IGachaItem[]) {
    const bucket = (list: IGachaItem[]) => ({ records: list, total: list.length });
    return deriveClientGachaRecords({ limited: bucket(items), regular: bucket([]), special: bucket([]) } as unknown as IGachaRecords);
}

const HISTORY = [pull("LIMITED_EN_1", 300), pull("NORM_EN_1", 100), pull("LINKAGE_EN_1", 250), pull("CLASSIC_EN_1", 200), pull("SINGLE_EN_1", 400), pull("BOOT_EN_1", 50)];

describe("mergeAllGroups", () => {
    it("lists every group's pulls once, newest first", () => {
        const merged = mergeAllGroups(records(HISTORY));
        expect(merged.map((p) => p.at)).toEqual([400, 300, 250, 200, 100, 50]);
        expect(merged).toHaveLength(HISTORY.length);
    });

    it("keeps a 10-pull's rows in their own order, since they share one timestamp", () => {
        const tenPull = Array.from({ length: 10 }, (_, i) => pull("NORM_EN_1", 500, `char_${i}`));
        const merged = mergeAllGroups(records([pull("LIMITED_EN_1", 100), ...tenPull, pull("LIMITED_EN_1", 600)]));
        expect(merged.slice(1, 11).map((p) => p.charId)).toEqual(tenPull.map((p) => p.charId));
        expect(merged[0].at).toBe(600);
        expect(merged[11].at).toBe(100);
    });

    it("is empty for an empty history", () => {
        expect(mergeAllGroups(records([]))).toEqual([]);
    });
});

describe("tab helpers", () => {
    const r = records(HISTORY);

    it("gives a group tab only its own pulls and All every pull", () => {
        expect(recordsForTab(r, "limited").map((p) => p.poolId)).toEqual(["LIMITED_EN_1"]);
        expect(new Set(recordsForTab(r, "regular").map((p) => p.poolId))).toEqual(new Set(["NORM_EN_1", "SINGLE_EN_1"]));
        expect(recordsForTab(r, "all")).toHaveLength(6);
    });

    it("totals All as the sum of the disjoint groups", () => {
        expect(totalForTab(r, "all")).toBe(6);
        expect(totalForTab(r, "linkage")).toBe(1);
    });

    it("offers All always and Starter Headhunting only when it holds pulls", () => {
        expect(isTabShown(records([]), "all")).toBe(true);
        expect(isTabShown(records([]), "boot")).toBe(false);
        expect(isTabShown(r, "boot")).toBe(true);
    });

    it("leads with All and still opens on Limited", () => {
        expect(HISTORY_TABS[0]).toBe("all");
        expect(DEFAULT_HISTORY_TAB).toBe("limited");
    });
});
