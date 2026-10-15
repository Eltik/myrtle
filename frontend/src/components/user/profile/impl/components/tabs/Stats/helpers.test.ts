import { describe, expect, it } from "vitest";
import type { ISkinIndexEntry } from "#/lib/api/skins";
import type { IRosterEntry, IRosterMastery, IRosterModule } from "#/lib/api/user";
import type { IOperatorIndexEntry } from "#/types/operators";
import { computeUserStats } from "./helpers";

interface IIndexFixture {
    id: string;
    name: string;
    rarity: number;
    profession: string;
    subProfessionId?: string;
    professionName?: string | null;
    isNotObtainable?: boolean;
    modules?: { uniEquipId: string; typeName1: string; typeName2: string | null }[];
}

function indexEntry(f: IIndexFixture): IOperatorIndexEntry {
    return { subProfessionId: "", isNotObtainable: false, modules: [], ...f } as unknown as IOperatorIndexEntry;
}

function rosterEntry(operator_id: string, elite: number, masteries: IRosterMastery[] = [], modules: IRosterModule[] = []): IRosterEntry {
    return {
        user_id: "u",
        operator_id,
        elite,
        level: 1,
        exp: 0,
        potential: 0,
        skill_level: 7,
        favor_point: 0,
        skin_id: null,
        default_skill: null,
        voice_lan: null,
        current_equip: null,
        current_tmpl: null,
        obtained_at: null,
        masteries,
        modules,
    };
}

const m = (index: number, mastery: number): IRosterMastery => ({ index, mastery });

const INDEX: IOperatorIndexEntry[] = [
    indexEntry({ id: "char_002_amiya", name: "Amiya", rarity: 5, profession: "CASTER", subProfessionId: "corecaster", modules: [{ uniEquipId: "uniequip_002_amiya", typeName1: "CCR", typeName2: "X" }] }),
    indexEntry({ id: "char_102_texas", name: "Texas", rarity: 5, profession: "PIONEER", subProfessionId: "charger", professionName: "先锋干员", modules: [{ uniEquipId: "uniequip_002_texas", typeName1: "CHG", typeName2: null }] }),
    indexEntry({ id: "char_103_angel", name: "Exusiai", rarity: 6, profession: "SNIPER", subProfessionId: "fastshot" }),
    indexEntry({ id: "char_291_aglina", name: "Angelina", rarity: 6, profession: "SUPPORT", subProfessionId: "slower" }),
    indexEntry({ id: "char_129_bluep", name: "Blue Poison", rarity: 5, profession: "SNIPER", subProfessionId: "fastshot" }),
    indexEntry({ id: "char_151_myrtle", name: "Myrtle", rarity: 4, profession: "PIONEER", subProfessionId: "bearer" }),
    indexEntry({ id: "char_130_doberm", name: "Lancet-2", rarity: 2, profession: "MEDIC", subProfessionId: "physician" }),
    // Never counted: tokens, traps, unobtainable operators and id-less rows.
    indexEntry({ id: "token_10000_silent_healrb", name: "Drone", rarity: 1, profession: "TOKEN" }),
    indexEntry({ id: "trap_001_crate", name: "Crate", rarity: 1, profession: "TRAP" }),
    indexEntry({ id: "char_npc", name: "NPC", rarity: 6, profession: "WARRIOR", isNotObtainable: true }),
    indexEntry({ id: "", name: "Blank", rarity: 6, profession: "WARRIOR" }),
];

const ROSTER: IRosterEntry[] = [
    rosterEntry(
        "char_002_amiya",
        2,
        [m(0, 3), m(1, 1), m(2, 0)],
        [
            { id: "uniequip_001_amiya", level: 1, locked: false },
            { id: "uniequip_002_amiya", level: 2, locked: false },
        ],
    ),
    rosterEntry("char_102_texas", 2, [m(1, 3), m(0, 3)], [{ id: "uniequip_002_texas", level: 0, locked: true }]),
    rosterEntry("char_103_angel", 2, [m(0, 0), m(1, 0), m(2, 0)], [{ id: "uniequip_003_angel", level: 3, locked: false }]),
    rosterEntry("char_291_aglina", 2, [m(0, 0)]),
    rosterEntry("char_129_bluep", 2, [m(1, 2), m(0, 0)]),
    rosterEntry("char_151_myrtle", 1),
    // Not in the index: ignored.
    rosterEntry("char_unknown", 2, [m(0, 3)]),
];

const SKINS = {
    a: { skinId: "char_002_amiya#1" },
    b: { skinId: "char_002_amiya@epoque#4" },
    c: { skinId: "char_102_texas@wild#1" },
} as unknown as Record<string, ISkinIndexEntry>;

describe("computeUserStats", () => {
    const stats = computeUserStats(ROSTER, INDEX, SKINS, 1);

    it("counts only obtainable, collectible operators", () => {
        expect(stats.totalAvailable).toBe(7);
        expect(stats.totalOwned).toBe(6);
        expect(stats.collectionPercentage).toBeCloseTo((6 / 7) * 100, 10);
    });

    it("breaks the roster down by promotion", () => {
        expect(stats.eliteBreakdown).toEqual({ e0: 0, e1: 1, e2: 5, total: 6 });
    });

    it("counts M3 skills per operator and the mastery levels left on E2s", () => {
        expect(stats.masteries).toMatchObject({ m3Count: 2, m6Count: 1, m9Count: 0, totalMasteryLevels: 12, maxPossibleMasteryLevels: 33, e2Count: 5 });
    });

    it("lists mastery gaps by rarity then name, with skills in index order", () => {
        const { pendingM3, pendingM6, pendingM9 } = stats.masteries.details;
        expect(pendingM3.map((g) => [g.name, g.rarity, g.sub])).toEqual([
            ["Angelina", 6, "M0"],
            ["Exusiai", 6, "M0 · M0 · M0"],
            ["Blue Poison", 5, "M0 · M2"],
        ]);
        expect(pendingM6.map((g) => [g.name, g.sub])).toEqual([["Amiya", "M3 · M1 · M0"]]);
        // Texas has two M3s but only two skills, so there is no third to chase.
        expect(pendingM9).toEqual([]);
        expect(pendingM6[0]).toMatchObject({ id: "char_002_amiya", operatorId: "char_002_amiya", charId: "char_002_amiya" });
    });

    it("skips the default modules and reports locked and below-max ones", () => {
        expect(stats.modules).toMatchObject({ unlocked: 2, atMax: 1, totalAvailable: 3 });
        expect(stats.modules.details.locked).toEqual([{ id: "char_102_texas:uniequip_002_texas", operatorId: "char_102_texas", name: "Texas", charId: "char_102_texas", rarity: 5, sub: "CHG • Locked" }]);
        expect(stats.modules.details.belowMax.map((g) => [g.id, g.sub])).toEqual([["char_002_amiya:uniequip_002_amiya", "CCR-X • Lv 2/3"]]);
    });

    it("orders classes by the list sort order and prefers the server's class name", () => {
        expect(stats.professions.map((p) => p.profession)).toEqual(["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "SUPPORT", "MEDIC", "SPECIAL"]);
        const vanguard = stats.professions[0];
        expect(vanguard).toMatchObject({ displayName: "先锋干员", owned: 2, total: 2, percentage: 100 });
        expect(vanguard.subProfessions.map((s) => [s.subProfessionId, s.owned, s.total])).toEqual([
            ["charger", 1, 1],
            ["bearer", 1, 1],
        ]);
        const guard = stats.professions[1];
        expect(guard).toMatchObject({ displayName: "Guard", owned: 0, total: 0, percentage: 0, subProfessions: [] });
        const medic = stats.professions.find((p) => p.profession === "MEDIC");
        expect(medic).toMatchObject({ owned: 0, total: 1, percentage: 0 });
        expect(medic?.subProfessions).toMatchObject([{ subProfessionId: "physician", owned: 0, total: 1, percentage: 0 }]);
        const sniper = stats.professions.find((p) => p.profession === "SNIPER");
        expect(sniper).toMatchObject({ displayName: "Sniper", owned: 2, total: 2 });
    });

    it("counts only non-default skins as available", () => {
        expect(stats.skins).toEqual({ totalOwned: 1, totalAvailable: 2, percentage: 50 });
        expect(computeUserStats(ROSTER, INDEX, undefined, null).skins).toEqual({ totalOwned: 0, totalAvailable: 0, percentage: 0 });
    });

    it("reads all zeros for an empty account", () => {
        const empty = computeUserStats([], [], undefined, null);
        expect(empty.totalAvailable).toBe(0);
        expect(empty.collectionPercentage).toBe(0);
        expect(empty.eliteBreakdown).toEqual({ e0: 0, e1: 0, e2: 0, total: 0 });
        expect(empty.professions).toHaveLength(8);
        expect(empty.professions.every((p) => p.total === 0 && p.percentage === 0)).toBe(true);
    });

    it("counts masteries on a non-E2 entry but no headroom for them", () => {
        // An E1 entry should never carry masteries; if one does, its levels land in
        // the total while the ceiling stays E2-only.
        const odd = computeUserStats([rosterEntry("char_151_myrtle", 1, [m(0, 3)])], INDEX, undefined, null);
        expect(odd.masteries).toMatchObject({ m3Count: 1, totalMasteryLevels: 3, maxPossibleMasteryLevels: 0 });
        expect(odd.masteries.details.pendingM3).toEqual([]);
    });
});
