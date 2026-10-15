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

describe("E2 rule, 6★ only", () => {
    const pool = [
        { id: "op6", rarity: 6, profession: "CASTER" },
        { id: "op5", rarity: 5, profession: "CASTER" },
        { id: "op4", rarity: 4, profession: "CASTER" },
        { id: "op3", rarity: 3, profession: "CASTER" },
        { id: "unowned5", rarity: 5, profession: "CASTER" },
    ] as IRandomizerOperator[];
    // Only the 4★ is at E2; every other owned operator is at E1.
    const roster = { owned: new Set(["op6", "op5", "op4", "op3"]), e2: new Set<string>(["op4"]) };
    const on: IRandomizerSettings = { ...DEFAULT_SETTINGS, onlyOwnedOperators: true, onlyE2Operators: true };

    it("is off by default and then reproduces the original rule exactly: E2 binds every rarity", () => {
        expect(DEFAULT_SETTINGS.e2SixStarOnly).toBe(false);
        const original = pool.filter((op) => roster.owned.has(op.id) && roster.e2.has(op.id));
        expect(selectAvailableOperators(pool, on, roster)).toEqual(original);
        expect(selectAvailableOperators(pool, on, roster).map((o) => o.id)).toEqual(["op4"]);
    });

    it("on, lets owned lower rarities at E1 through but still holds 6★ to E2 and still requires owned", () => {
        expect(selectAvailableOperators(pool, { ...on, e2SixStarOnly: true }, roster).map((o) => o.id)).toEqual(["op5", "op4", "op3"]);
    });

    it("does nothing while E2 only is off", () => {
        const owned = { ...DEFAULT_SETTINGS, onlyOwnedOperators: true };
        expect(selectAvailableOperators(pool, { ...owned, e2SixStarOnly: true }, roster)).toEqual(selectAvailableOperators(pool, owned, roster));
    });
});
