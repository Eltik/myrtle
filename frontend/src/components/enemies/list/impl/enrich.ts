import type { IEnemy, IEnemyHandbook, IEnemyLevel } from "#/lib/api/enemies";
import { ENEMY_LEVELS } from "./constants";
import type { ApplyWay, IEnemyFlatStats, IEnemyImmunities, IEnemyStatMaxByLevel, IEnemyView } from "./types";

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
    fear: false,
    paralysis: false,
    tremble: false,
    lure: false,
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
              fear: !!attrs.fearImmune,
              paralysis: !!attrs.paralysisImmune,
              tremble: !!attrs.trembleImmune,
              lure: !!attrs.lureImmune,
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

/** The value at fraction `p` of an ascending list (nearest rank); 0 for an empty one. */
function percentile(sorted: number[], p: number): number {
    if (sorted.length === 0) return 0;
    return sorted[Math.min(sorted.length - 1, Math.round(p * (sorted.length - 1)))] ?? 0;
}

/**
 * The value a card's stat bar fills to, per level tier: the 95th percentile, not the maximum.
 * A handful of outliers set the maximum (EN 2026-10-20: Elite HP max 2,500,000 against a
 * median of 15,000), which left 99% of Elite HP bars and 88 to 100% of Normal bars under
 * a tenth full. Against the 95th percentile the median sits at 12 to 44% and the top 5%
 * read full. The printed value beside each bar is unchanged.
 */
export function computeStatMaxByLevel(enemies: IEnemyView[]): IEnemyStatMaxByLevel {
    const cols: Record<IEnemyLevel, { hp: number[]; atk: number[]; def: number[] }> = {
        NORMAL: { hp: [], atk: [], def: [] },
        ELITE: { hp: [], atk: [], def: [] },
        BOSS: { hp: [], atk: [], def: [] },
    };
    for (const e of enemies) {
        if (e.hideInHandbook) continue;
        const bucket = cols[e.enemyLevel];
        bucket.hp.push(e.flatStats.maxHp);
        bucket.atk.push(e.flatStats.atk);
        bucket.def.push(e.flatStats.def);
    }
    const scale = (values: number[]) =>
        Math.max(
            1,
            percentile(
                values.sort((a, b) => a - b),
                0.95,
            ),
        );
    const out = {} as IEnemyStatMaxByLevel;
    // Floored at 1 so an unused tier doesn't divide by zero.
    for (const level of ENEMY_LEVELS as readonly IEnemyLevel[]) {
        out[level] = { hp: scale(cols[level].hp), atk: scale(cols[level].atk), def: scale(cols[level].def) };
    }
    return out;
}
