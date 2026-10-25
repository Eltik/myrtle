import { describe, expect, it } from "vitest";
import type { IEnemy } from "#/lib/api/enemies";
import type { IStageDetail, IStageIndexEntry } from "#/lib/api/stages";
import type { ThreatThresholds } from "#/types/generated/ThreatThresholds";
import { deriveStageContext, normalStageId, pickerStages } from "./stage";

/** `GET /squads/rules` on EN: `LevelInfoList` grade A RES and DEF, grade S move speed. */
const GRADES: ThreatThresholds = { high_res: 50, high_def: 1000, fast_move_speed: 1.5, res_grade: "A", def_grade: "A", move_speed_grade: "S" };

function enemy(id: string, name: string, attrs: { def?: number; res?: number; speed?: number }, extra: { motion?: string; lifePointReduce?: number; levels?: number } = {}): IEnemy {
    const level = (i: number) => ({
        level: i,
        attributes: {
            maxHp: 1000,
            atk: 100,
            def: (attrs.def ?? 100) + i * 500,
            magicResistance: attrs.res ?? 0,
            moveSpeed: attrs.speed ?? 1,
            attackSpeed: 100,
            baseAttackTime: 1,
            massLevel: 1,
            hpRecoveryPerSec: 0,
            stunImmune: false,
            silenceImmune: false,
            sleepImmune: false,
            frozenImmune: false,
            levitateImmune: false,
            fearImmune: false,
            paralysisImmune: false,
            trembleImmune: false,
            lureImmune: false,
        },
        applyWay: null,
        motion: extra.motion ?? "WALK",
        rangeRadius: null,
        lifePointReduce: extra.lifePointReduce ?? 1,
        skills: [],
    });
    return { enemyId: id, name, enemyLevel: "NORMAL", stats: { levels: Array.from({ length: extra.levels ?? 1 }, (_, i) => level(i)) } } as unknown as IEnemy;
}

function spawn(key: string, count: number) {
    return { actionType: "SPAWN", key, count, preDelay: 0 };
}

function detail(): IStageDetail {
    return {
        stage: { stageId: "main_01-07", code: "1-7", name: "Escape", apCost: 6, hardStagedId: "main_01-07#f#" },
        zone: { zoneId: "main_1", zoneNameFirst: "Episode 01", zoneNameSecond: "Separated Hearts" },
        levelData: {
            options: { characterLimit: 8, initialCost: 10, costIncreaseTime: 1, maxLifePoint: 3 },
            enemyDbRefs: [
                { id: "e_caster", level: 0 },
                { id: "e_tank", level: 1 },
                { id: "e_drone", level: 0 },
                { id: "e_dog", level: 0, overwrittenData: { attributes: { moveSpeed: { m_defined: true, m_value: 2 } } } },
                { id: "e_boss", level: 0 },
            ],
            waves: [
                { preDelay: 0, postDelay: 0, fragments: [{ preDelay: 0, actions: [spawn("e_caster", 3), spawn("e_tank", 2), spawn("e_drone", 4)] }] },
                { preDelay: 0, postDelay: 0, fragments: [{ preDelay: 0, actions: [spawn("e_dog", 5), spawn("e_boss", 1), { actionType: "STORY", key: "x", preDelay: 0 }] }] },
            ],
        },
        enemies: {
            e_caster: enemy("e_caster", "Caster", { res: 50 }),
            // DEF 500 at level 0, 1000 at the level the stage picks.
            e_tank: enemy("e_tank", "Tank", { def: 500 }, { levels: 2 }),
            e_drone: enemy("e_drone", "Drone", {}, { motion: "FLY" }),
            e_dog: enemy("e_dog", "Hound", { speed: 1 }),
            e_boss: enemy("e_boss", "Boss", { res: 49 }, { lifePointReduce: 10 }),
        },
        materials: {},
    } as unknown as IStageDetail;
}

describe("deriveStageContext", () => {
    it("reads the level options", () => {
        const ctx = deriveStageContext("main_01-07", detail(), GRADES);
        expect(ctx).toMatchObject({ stageId: "main_01-07", code: "1-7", name: "Escape", zoneName: "Separated Hearts", apCost: 6, deployLimit: 8, startDp: 10, dpPerSecond: 1, lifePoints: 3, enemyCount: 15, hasLevel: true });
    });

    it("turns enemy stats into threats with the rows that answer them", () => {
        const ctx = deriveStageContext("main_01-07", detail(), GRADES);
        const byKey = Object.fromEntries(ctx.threats.map((th) => [th.key, th]));
        expect(ctx.threats.map((th) => th.key)).toEqual(["high_res", "high_def", "aerial", "fast", "multi_life"]);
        expect(byKey.high_res).toMatchObject({ rows: ["physical", "true_elemental"], count: 3 });
        expect(byKey.high_def).toMatchObject({ rows: ["arts", "true_elemental"], count: 2, enemies: [{ enemyId: "e_tank", name: "Tank", count: 2 }] });
        expect(byKey.aerial).toMatchObject({ rows: ["anti_air"], count: 4 });
        // The stage's own moveSpeed override makes the hound fast.
        expect(byKey.fast).toMatchObject({ rows: ["cc", "shift"], count: 5 });
        expect(byKey.multi_life).toMatchObject({ rows: [], count: 1 });
    });

    it("keeps the grade bounds inclusive", () => {
        const d = detail();
        const caster = d.enemies.e_caster as IEnemy;
        if (caster.stats) caster.stats.levels[0].attributes.magicResistance = 49;
        const ctx = deriveStageContext("main_01-07", d, GRADES);
        expect(ctx.threats.find((th) => th.key === "high_res")).toBeUndefined();
    });

    it("tolerates a stage without a level file", () => {
        const d = { ...detail(), levelData: null } as IStageDetail;
        const ctx = deriveStageContext("main_01-07", d, GRADES);
        expect(ctx).toMatchObject({ hasLevel: false, deployLimit: null, startDp: null, dpPerSecond: null, lifePoints: null, enemyCount: 0, threats: [] });
    });

    it("flags nothing for a stat whose grade the table lacks", () => {
        const ctx = deriveStageContext("main_01-07", detail(), { ...GRADES, high_res: null, high_def: null, fast_move_speed: null });
        expect(ctx.threats.map((th) => th.key)).toEqual(["aerial", "multi_life"]);
    });

    it("maps a challenge id back to its normal stage through the table's link", () => {
        const normalOf = { "main_01-07#f#": "main_01-07" };
        expect(normalStageId("main_01-07#f#", normalOf)).toBe("main_01-07");
        expect(normalStageId("main_01-07", normalOf)).toBe("main_01-07");
    });
});

function entry(stageId: string, code: string, zoneId: string, zoneOrder: number, extra: Partial<IStageIndexEntry> = {}): IStageIndexEntry {
    return { stageId, levelId: null, code, name: null, zoneId, zoneName: null, zoneOrder, group: "story", category: "stages", apCost: 6, boss: false, originitePrime: 0, isHard: false, difficulty: "NORMAL", canView: true, preview: null, banner: null, ...extra };
}

describe("pickerStages", () => {
    it("folds a Story-mode twin into its viewable Standard stage and hides stages with no level file", () => {
        const out = pickerStages([entry("easy_11-01", "11-1", "main_11", 11, { canView: false }), entry("main_11-01", "11-1", "main_11", 11), entry("act4d0_01", "SW-EV-1", "main_1", 1, { canView: false })]);
        expect(out.map((s) => s.stageId)).toEqual(["main_11-01"]);
    });

    it("keeps Adverse and six-star variants as their own cards and drops challenge-mode ids", () => {
        const out = pickerStages([entry("tough_11-01", "11-1", "main_11", 11, { isHard: true }), entry("main_11-01", "11-1", "main_11", 11), entry("main_11-01#f#", "11-1", "main_11", 11, { difficulty: "FOUR_STAR", isHard: true })]);
        expect(out.map((s) => s.stageId)).toEqual(["main_11-01", "tough_11-01"]);
    });

    it("orders by zone, then code with numbers compared as numbers", () => {
        const out = pickerStages([entry("main_02-10", "2-10", "main_2", 2), entry("main_01-07", "1-7", "main_1", 1), entry("main_02-09", "2-9", "main_2", 2)]);
        expect(out.map((s) => s.code)).toEqual(["1-7", "2-9", "2-10"]);
    });
});
