import { describe, expect, it } from "vitest";
import { type IStageIndexEntry, stageVariantsOf } from "#/lib/api/stages";
import { buildStageTree } from "./derive";

function entry(stageId: string, code: string, extra: Partial<IStageIndexEntry> = {}): IStageIndexEntry {
    return { stageId, levelId: null, code, name: null, zoneId: "main_0", zoneName: "Evil Time Part 1", zoneOrder: 0, group: "story", category: "stages", apCost: 6, boss: false, originitePrime: 1, isHard: false, difficulty: "NORMAL", canView: true, preview: null, banner: null, ...extra };
}

// Chapter 0 in miniature: a stage with a CM twin, a training stage, and an
// Episode 10 code with a retired Story-mode twin and an Adverse twin.
const INDEX: IStageIndexEntry[] = [
    entry("main_00-01", "0-1"),
    entry("main_00-01#f#", "0-1", { difficulty: "FOUR_STAR", isHard: true }),
    entry("tr_01", "TR-1", { apCost: 0 }),
    entry("easy_10-01", "10-2", { originitePrime: 0, canView: false }),
    entry("main_10-01", "10-2"),
    entry("tough_10-01", "10-2", { isHard: true }),
];

describe("buildStageTree", () => {
    const event = buildStageTree(INDEX).groups[0]?.events[0];

    it("counts every first-clear Originite Prime in the zone, CM and Adverse included", () => {
        expect(event?.originitePrime).toBe(5);
        expect(event?.stageCount).toBe(3);
    });

    it("folds CM and Adverse twins into their Standard card as links", () => {
        const byCode = Object.fromEntries((event?.stages ?? []).map((c) => [c.badge, c]));
        expect(byCode["0-1"]?.stageId).toBe("main_00-01");
        expect(byCode["0-1"]?.variants).toEqual([{ stageId: "main_00-01#f#", kind: "cm", canView: true }]);
        expect(byCode["10-2"]?.stageId).toBe("main_10-01");
        expect(byCode["10-2"]?.variants).toEqual([{ stageId: "tough_10-01", kind: "adverse", canView: true }]);
        expect(byCode["TR-1"]?.variants).toEqual([]);
    });
});

describe("stageVariantsOf", () => {
    it("lists the viewable versions of a code, Standard first, from any of them", () => {
        const want = [
            { stageId: "main_10-01", kind: null },
            { stageId: "tough_10-01", kind: "adverse" },
        ];
        expect(stageVariantsOf(INDEX, "tough_10-01")).toEqual(want);
        expect(stageVariantsOf(INDEX, "main_10-01")).toEqual(want);
        expect(stageVariantsOf(INDEX, "main_00-01#f#").map((v) => v.kind)).toEqual([null, "cm"]);
    });

    it("offers no switcher for a stage with one version", () => {
        expect(stageVariantsOf(INDEX, "tr_01")).toEqual([]);
    });
});
