import { describe, expect, it } from "vitest";
import type { IStage, IZone, StageClearsMap } from "#/types/stages";
import { buildActivityLookup } from "./activity-lookup";
import { DEFAULT_SETTINGS } from "./constants";
import type { IRandomizerOperator, IRandomizerSettings } from "./types";
import { applyProfileGate, buildRosterIndex, selectAvailableOperators, selectAvailableStages } from "./utils";

const ops = [
    { id: "char_002_amiya", rarity: 5, profession: "CASTER" },
    { id: "char_103_angel", rarity: 6, profession: "SNIPER" },
] as IRandomizerOperator[];

const zones = [{ zoneId: "main_0", type: "MAINLINE" }] as IZone[];
const stages = [
    { stageId: "main_00-01", zoneId: "main_0" },
    { stageId: "main_00-02", zoneId: "main_0" },
    { stageId: "main_00-03", zoneId: "main_0" },
] as IStage[];
const clears = { "main_00-01": { state: 3 } } as unknown as StageClearsMap;
const lookup = buildActivityLookup([], []);

const saved: IRandomizerSettings = { ...DEFAULT_SETTINGS, onlyAvailableStages: false, onlyOwnedOperators: true, onlyE2Operators: true, onlyCompletedStages: true };
const signedOut = { hasProfile: false, hasStageClears: false };
const signedIn = { hasProfile: true, hasStageClears: true };

describe("applyProfileGate", () => {
    it("drops the profile filters when signed out, so a saved E2-only choice cannot empty the pool", () => {
        const gated = applyProfileGate(saved, signedOut);
        expect(gated.onlyOwnedOperators).toBe(false);
        expect(gated.onlyE2Operators).toBe(false);
        expect(gated.onlyCompletedStages).toBe(false);
        expect(selectAvailableOperators(ops, gated, buildRosterIndex(null))).toHaveLength(2);
        expect(selectAvailableStages(stages, zones, gated, null, lookup)).toHaveLength(3);
    });

    it("keeps them signed in", () => {
        expect(applyProfileGate(saved, signedIn)).toBe(saved);
        // The unguarded case the gate exists for: E2-only over an empty roster draws nothing.
        expect(selectAvailableOperators(ops, saved, buildRosterIndex(null))).toHaveLength(0);
    });

    it("holds the cleared filter back until the clears load, instead of reading every stage as uncleared", () => {
        const loading = applyProfileGate(saved, { hasProfile: true, hasStageClears: false });
        expect(loading.onlyCompletedStages).toBe(false);
        expect(loading.onlyE2Operators).toBe(true);
        expect(selectAvailableStages(stages, zones, loading, null, lookup)).toHaveLength(3);
        expect(selectAvailableStages(stages, zones, saved, null, lookup)).toHaveLength(0);
    });

    it("leaves every other setting alone", () => {
        const { onlyOwnedOperators: _a, onlyE2Operators: _b, onlyCompletedStages: _c, ...rest } = applyProfileGate(saved, signedOut);
        const { onlyOwnedOperators: _d, onlyE2Operators: _e, onlyCompletedStages: _f, ...restSaved } = saved;
        expect(rest).toEqual(restSaved);
    });
});

describe("only cleared stages", () => {
    it("filters without touching the deselection, so turning it off restores the pool", () => {
        const on = { ...saved, onlyCompletedStages: true };
        expect(selectAvailableStages(stages, zones, on, clears, lookup).map((s) => s.stageId)).toEqual(["main_00-01"]);
        expect(on.deselectedStageIds).toEqual([]);
        expect(selectAvailableStages(stages, zones, { ...on, onlyCompletedStages: false }, clears, lookup)).toHaveLength(3);
    });

    it("still honours a hand-made deselection alongside it", () => {
        const on = { ...saved, onlyCompletedStages: true, deselectedStageIds: ["main_00-01"] };
        expect(selectAvailableStages(stages, zones, on, clears, lookup)).toHaveLength(0);
        expect(selectAvailableStages(stages, zones, { ...on, onlyCompletedStages: false }, clears, lookup).map((s) => s.stageId)).toEqual(["main_00-02", "main_00-03"]);
    });
});
