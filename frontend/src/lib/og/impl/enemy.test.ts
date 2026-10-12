import { describe, expect, it } from "vitest";
import type { Enemy } from "#/types/generated/Enemy";
import { buildEnemyOgData } from "./enemy";
import { ogHashers } from "./hashers";
import { parseServerScopedOgId, serverScopedOgId } from "./ids";

function enemy(over: Partial<Enemy> = {}): Enemy {
    return {
        enemyId: "enemy_1506_patrt",
        enemyIndex: "PT",
        name: "Patriot",
        enemyLevel: "BOSS",
        description: "His <@ba.kw>armor</> is in disrepair.",
        damageType: ["PHYSIC"],
        stats: { levels: [{ level: 0, motion: "WALK", attributes: { maxHp: 45000, atk: 1600, def: 500, magicResistance: 45 } }] },
        ...over,
    } as unknown as Enemy;
}

describe("enemy card", () => {
    it("strips rich text and reads the first stat level", () => {
        const data = buildEnemyOgData(enemy(), "en");
        expect(data.description).toBe("His armor is in disrepair.");
        expect(data.stats).toEqual({ hp: 45000, atk: 1600, def: 500, res: 45 });
        expect(data.motion).toBe("WALK");
    });

    it("hashes what it draws", () => {
        const hash = (e: Enemy, server: "en" | "cn" = "en") => ogHashers.enemy.hash(buildEnemyOgData(e, server));
        const base = hash(enemy());
        expect(hash(enemy())).toBe(base);
        expect(hash(enemy({ name: "Patriot (Revenant)" }))).not.toBe(base);
        expect(hash(enemy({ enemyLevel: "ELITE" }))).not.toBe(base);
        expect(hash(enemy({ damageType: ["MAGIC"] }))).not.toBe(base);
        expect(hash(enemy(), "cn")).not.toBe(base);
        expect(hash(enemy({ stats: null }))).not.toBe(base);
    });

    it("scopes ids by server like the grid image", () => {
        expect(serverScopedOgId("enemy_1007_slime")).toBe("enemy_1007_slime");
        expect(serverScopedOgId("enemy_1007_slime", "cn")).toBe("cn:enemy_1007_slime");
        expect(parseServerScopedOgId("cn:enemy_1007_slime")).toEqual({ server: "cn", id: "enemy_1007_slime" });
        expect(parseServerScopedOgId("zz:enemy_x")).toEqual({ server: "en", id: "zz:enemy_x" });
    });
});
