import type { IEnemy, IEnemyHandbook, IEnemyLevel } from "#/lib/api/enemies";
import { ENEMY_LEVELS } from "./constants";
import type { ApplyWay, IEnemyFlatStats, IEnemyImmunities, IEnemyStatMax, IEnemyStatMaxByLevel, IEnemyView } from "./types";

const EMPTY_STATS: IEnemyFlatStats = {
    maxHp: 0,
    atk: 0,
    def: 0,
    res: 0,
    aspd: 0,
    ms: 0,
    weight: 0,
    baseAttackTime: 0,
    hpRecoveryPerSec: 0,
};

const EMPTY_IMMUNITIES: IEnemyImmunities = {
    stun: false,
    silence: false,
    sleep: false,
    frozen: false,
    levitate: false,
};

function normalizeApplyWay(raw: string | null | undefined): ApplyWay | null {
    if (!raw) return null;
    const upper = raw.toUpperCase();
    if (upper === "MELEE" || upper === "RANGED" || upper === "NONE") return upper;
    return null;
}

function deriveRace(enemy: IEnemy, raceData: IEnemyHandbook["raceData"] | undefined): string | null {
    const tags = enemy.enemyTags;
    if (!tags || tags.length === 0) return null;
    if (raceData) {
        for (const tag of tags) {
            const race = raceData[tag];
            if (race) return race.raceName;
        }
    }
    return tags[0] ?? null;
}

export function enrichEnemy(enemy: IEnemy, raceData?: IEnemyHandbook["raceData"]): IEnemyView {
    const level0 = enemy.stats?.levels?.[0];
    const attrs = level0?.attributes;
    const flatStats: IEnemyFlatStats = attrs
        ? {
              maxHp: attrs.maxHp ?? 0,
              atk: attrs.atk ?? 0,
              def: attrs.def ?? 0,
              res: attrs.magicResistance ?? 0,
              aspd: attrs.attackSpeed ?? 0,
              ms: attrs.moveSpeed ?? 0,
              weight: attrs.massLevel ?? 0,
              baseAttackTime: attrs.baseAttackTime ?? 0,
              hpRecoveryPerSec: attrs.hpRecoveryPerSec ?? 0,
          }
        : EMPTY_STATS;

    const immunities: IEnemyImmunities = attrs
        ? {
              stun: !!attrs.stunImmune,
              silence: !!attrs.silenceImmune,
              sleep: !!attrs.sleepImmune,
              frozen: !!attrs.frozenImmune,
              levitate: !!attrs.levitateImmune,
          }
        : EMPTY_IMMUNITIES;

    return {
        ...enemy,
        flatStats,
        immunities,
        applyWay: normalizeApplyWay(level0?.applyWay ?? null),
        race: deriveRace(enemy, raceData),
    };
}

export function enrichEnemies(enemies: IEnemy[], raceData?: IEnemyHandbook["raceData"]): IEnemyView[] {
    return enemies.map((e) => enrichEnemy(e, raceData));
}

function emptyMax(): IEnemyStatMax {
    return { hp: 0, atk: 0, def: 0 };
}

export function computeStatMaxByLevel(enemies: IEnemyView[]): IEnemyStatMaxByLevel {
    const out: IEnemyStatMaxByLevel = {
        NORMAL: emptyMax(),
        ELITE: emptyMax(),
        BOSS: emptyMax(),
    };
    for (const e of enemies) {
        if (e.hideInHandbook) continue;
        const bucket = out[e.enemyLevel];
        if (e.flatStats.maxHp > bucket.hp) bucket.hp = e.flatStats.maxHp;
        if (e.flatStats.atk > bucket.atk) bucket.atk = e.flatStats.atk;
        if (e.flatStats.def > bucket.def) bucket.def = e.flatStats.def;
    }
    // Floor every tier with at least 1 so an unused tier doesn't divide by zero.
    for (const level of ENEMY_LEVELS as readonly IEnemyLevel[]) {
        if (out[level].hp === 0) out[level].hp = 1;
        if (out[level].atk === 0) out[level].atk = 1;
        if (out[level].def === 0) out[level].def = 1;
    }
    return out;
}
