/**
 * A picked stage, read for what it asks of the squad: the level options
 * (deploy limit, starting DP, DP rate, life points) and a short list of
 * threats from the enemies it spawns. Pure, so it is tested without React.
 */

import { tallyEnemies } from "#/components/stages/detail/impl/helpers";
import { naturalCompare, preferEntry } from "#/components/stages/list/impl/derive";
import type { IEnemy } from "#/lib/api/enemies";
import type { IEnemyDbRef } from "#/lib/api/level";
import type { IStageDetail, IStageIndexEntry } from "#/lib/api/stages";
import type { CoverageRowKey } from "#/types/generated/CoverageRowKey";
import type { ThreatThresholds } from "#/types/generated/ThreatThresholds";
import { MULTI_LIFE_ABOVE } from "./constants";
import type { IStageContext, IStageThreat, StageThreatKey } from "./model";

/*
 * Where a stat starts to count as a threat comes from the game's own enemy
 * grade bands (`enemy_handbook_table.LevelInfoList`), served by
 * `GET /squads/rules`: RES and DEF from grade A, move speed from grade S
 * (50, 1000 and 1.5 on every server on 2026-10-11). Which grade counts is
 * the backend's `policy` choice; a threshold the table lacks flags nothing.
 */

/** Display order of the threats, and the coverage rows that answer each. */
const THREAT_ROWS: ReadonlyArray<{ key: StageThreatKey; rows: CoverageRowKey[] }> = [
    { key: "high_res", rows: ["physical", "true_elemental"] },
    { key: "high_def", rows: ["arts", "true_elemental"] },
    { key: "aerial", rows: ["anti_air"] },
    { key: "fast", rows: ["cc", "shift"] },
    { key: "multi_life", rows: [] },
];

interface IEnemyFacts {
    def: number;
    res: number;
    moveSpeed: number;
    motion: string | null;
    lifePointReduce: number;
}

/** The enemy's stats at the level the stage picks, with the stage's own overrides applied. */
function enemyFacts(enemy: IEnemy, ref: IEnemyDbRef | undefined): IEnemyFacts | null {
    const levels = enemy.stats?.levels ?? [];
    const lv = levels[ref?.level ?? 0] ?? levels[0];
    if (!lv) return null;
    const attrs = ref?.overwrittenData?.attributes ?? null;
    const pick = (key: string, base: number): number => {
        const o = attrs?.[key];
        return o?.m_defined && typeof o.m_value === "number" ? o.m_value : base;
    };
    return {
        def: pick("def", lv.attributes.def),
        res: pick("magicResistance", lv.attributes.magicResistance),
        moveSpeed: pick("moveSpeed", lv.attributes.moveSpeed),
        motion: lv.motion,
        lifePointReduce: lv.lifePointReduce,
    };
}

function atLeast(value: number, threshold: number | null | undefined): boolean {
    return typeof threshold === "number" && value >= threshold;
}

function threatsOf(f: IEnemyFacts, t: ThreatThresholds | null): Set<StageThreatKey> {
    const out = new Set<StageThreatKey>();
    if (atLeast(f.res, t?.high_res)) out.add("high_res");
    if (atLeast(f.def, t?.high_def)) out.add("high_def");
    if (f.motion === "FLY") out.add("aerial");
    if (atLeast(f.moveSpeed, t?.fast_move_speed)) out.add("fast");
    if (f.lifePointReduce > MULTI_LIFE_ABOVE) out.add("multi_life");
    return out;
}

function positive(n: number | undefined): number | null {
    return typeof n === "number" && Number.isFinite(n) && n > 0 ? n : null;
}

export function deriveStageContext(stageId: string, detail: IStageDetail, thresholds: ThreatThresholds | null): IStageContext {
    const level = detail.levelData;
    const opts = level?.options;
    const tally = tallyEnemies(level, detail.enemies);
    const refs = new Map((level?.enemyDbRefs ?? []).map((r) => [r.id, r]));

    const buckets = new Map<StageThreatKey, IStageThreat>();
    for (const row of tally) {
        if (!row.enemy || row.count <= 0) continue;
        const facts = enemyFacts(row.enemy, refs.get(row.id));
        if (!facts) continue;
        for (const key of threatsOf(facts, thresholds)) {
            const def = THREAT_ROWS.find((r) => r.key === key);
            if (!def) continue;
            const bucket = buckets.get(key) ?? { key, rows: def.rows, count: 0, enemies: [] };
            bucket.count += row.count;
            bucket.enemies.push({ enemyId: row.id, name: row.enemy.name, count: row.count });
            buckets.set(key, bucket);
        }
    }

    const costTime = positive(opts?.costIncreaseTime);
    const zone = detail.zone;
    const zoneName = zone ? zone.zoneNameSecond || zone.zoneNameFirst || zone.zoneNameTitleCurrent || null : null;

    return {
        stageId,
        code: detail.stage.code,
        name: detail.stage.name ?? "",
        zoneName,
        apCost: typeof detail.stage.apCost === "number" ? detail.stage.apCost : null,
        deployLimit: positive(opts?.characterLimit),
        startDp: typeof opts?.initialCost === "number" && opts.initialCost >= 0 ? opts.initialCost : null,
        dpPerSecond: costTime ? 1 / costTime : null,
        lifePoints: positive(opts?.maxLifePoint),
        enemyCount: tally.reduce((sum, r) => sum + r.count, 0),
        hasLevel: level != null,
        threats: THREAT_ROWS.map((r) => buckets.get(r.key)).filter((b): b is IStageThreat => b !== undefined),
    };
}

/** The normal stage of a challenge stage, from `stage_table` `HardStagedId` inverted (`SquadRules.normal_of`); a normal stage is its own. */
export function normalStageId(stageId: string, normalOf: Readonly<Record<string, string | undefined>> | undefined): string {
    return normalOf?.[stageId] ?? stageId;
}

/**
 * The stages the picker offers, one card per stage a squad can be planned
 * against, ordered by chapter or event and then by code.
 *
 * - Challenge-mode `#f#` variants are left out: the stage card's Normal /
 *   Challenge toggle reaches them.
 * - Variants that share a zone, a code and a hard flag collapse to one card
 *   through the Stage List's own `preferEntry` (viewable, then NORMAL). That
 *   folds a retired "Story mode" `easy_*` stage into its Standard twin, while an
 *   Adverse `tough_*` or a six-star `#s` stage keeps its own card, since the
 *   toggle does not reach those.
 * - A stage with no level file (`canView` false) is hidden. Its detail has no
 *   level data, so no deploy limit, DP or enemies, and the Stage List already
 *   renders such a stage as an inert card without a link.
 */
export function pickerStages(index: readonly IStageIndexEntry[]): IStageIndexEntry[] {
    const byKey = new Map<string, IStageIndexEntry>();
    for (const s of index) {
        if (s.stageId.endsWith("#f#")) continue;
        const key = `${s.zoneId}\u0000${s.code || s.stageId}\u0000${s.isHard ? 1 : 0}`;
        const seen = byKey.get(key);
        byKey.set(key, seen ? preferEntry(seen, s) : s);
    }
    return [...byKey.values()].filter((s) => s.canView).sort((a, b) => a.zoneOrder - b.zoneOrder || a.zoneId.localeCompare(b.zoneId) || naturalCompare(a.code, b.code) || Number(a.isHard) - Number(b.isHard) || a.stageId.localeCompare(b.stageId));
}
