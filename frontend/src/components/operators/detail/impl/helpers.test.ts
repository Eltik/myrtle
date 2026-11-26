import { describe, expect, it } from "vitest";
import type { IOperatorModule } from "#/types/operators";
import { moduleStats } from "./helpers";

/** A module whose stages carry the given `attributeBlackboard` entries, keyed as the game spells them. */
function moduleWith(...stages: Array<Array<[string, number]>>): IOperatorModule {
    return { data: { phases: stages.map((entries) => ({ attributeBlackboard: entries.map(([key, value]) => ({ key, value })) })) } } as unknown as IOperatorModule;
}

describe("moduleStats", () => {
    it("adds every stat a stage grants, under the game's snake_case keys", () => {
        // Shu's GUA-X at stage 3: the HP was dropped before the keys were mapped.
        const guaX = moduleWith(
            [
                ["max_hp", 160],
                ["atk", 30],
                ["def", 30],
            ],
            [
                ["max_hp", 210],
                ["atk", 40],
                ["def", 40],
            ],
            [
                ["max_hp", 270],
                ["atk", 50],
                ["def", 50],
            ],
        );
        expect(moduleStats(guaX, 3)).toMatchObject({ maxHp: 270, atk: 50, def: 50 });
        expect(moduleStats(guaX, 1)).toMatchObject({ maxHp: 160, atk: 30, def: 30 });
    });

    it("maps attack speed, RES, cost, redeploy time and block", () => {
        const mod = moduleWith([
            ["attack_speed", 8],
            ["magic_resistance", 10],
            ["cost", -1],
            ["respawn_time", -10],
            ["block_cnt", 1],
        ]);
        expect(moduleStats(mod, 1)).toEqual({ atk: 0, maxHp: 0, def: 0, attackSpeed: 8, magicResistance: 10, cost: -1, respawnTime: -10, blockCnt: 1 });
    });

    it("ignores keys that are not stats, and stage 0", () => {
        const mod = moduleWith([
            ["max_hp", 100],
            ["base_attack_time", -0.1],
        ]);
        expect(moduleStats(mod, 1).maxHp).toBe(100);
        expect(moduleStats(mod, 0).maxHp).toBe(0);
    });
});
