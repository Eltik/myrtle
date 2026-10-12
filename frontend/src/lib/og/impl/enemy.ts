/**
 * An enemy's social card, as data. Built from the one `/enemies/{id}` record
 * by both the enemy route's `head()` (to hash the og:image URL) and the
 * `enemy` OG handler (to draw it), through {@link buildEnemyOgData}.
 *
 * Every enemy's `attackType` is `null` on production (1,609 of 1,609 on EN,
 * 2026-10-12), so the card does not draw one; its chips are the movement mode
 * and the damage types the record does carry.
 */

import type { GamedataServer } from "#/lib/api/gamedata";
import { plainText } from "#/lib/gamedata/richtext";
import type { DamageType } from "#/types/generated/DamageType";
import type { Enemy } from "#/types/generated/Enemy";
import type { EnemyLevel } from "#/types/generated/EnemyLevel";

export interface IEnemyOgStats {
    hp: number;
    atk: number;
    def: number;
    res: number;
}

export interface IEnemyOgData {
    id: string;
    server: GamedataServer;
    name: string;
    /** The handbook code (`B1`, `PT`); empty on many enemies. */
    index: string;
    level: EnemyLevel;
    /** Rich text stripped. */
    description: string;
    /** `WALK` or `FLY`, from the first stat level. */
    motion: string | null;
    damageTypes: DamageType[];
    /** The first stat level's HP/ATK/DEF/RES; `null` for a record without stats. */
    stats: IEnemyOgStats | null;
    /** The icon inlined as a data URI; filled by the OG handler only (it follows from `id` and `server`). */
    iconURL?: string;
}

export function buildEnemyOgData(enemy: Enemy, server: GamedataServer): IEnemyOgData {
    const first = enemy.stats?.levels[0];
    const a = first?.attributes;
    return {
        id: enemy.enemyId,
        server,
        name: enemy.name,
        index: enemy.enemyIndex ?? "",
        level: enemy.enemyLevel,
        description: plainText(enemy.description ?? "")
            .replace(/\s+/g, " ")
            .trim(),
        motion: first?.motion ?? null,
        damageTypes: enemy.damageType ?? [],
        stats: a ? { hp: a.maxHp, atk: a.atk, def: a.def, res: a.magicResistance } : null,
    };
}

/** Everything drawn but the inlined icon. */
export function enemyOgHashParts(data: IEnemyOgData): unknown[] {
    const s = data.stats;
    return [data.id, data.server, data.name, data.index, data.level, data.description, data.motion ?? "", data.damageTypes.join(","), s ? `${s.hp}/${s.atk}/${s.def}/${s.res}` : ""];
}
