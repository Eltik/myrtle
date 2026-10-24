/**
 * Client side of the ranking view: the shared-context request, filters, fuzzy search and
 * sort over the rows `POST /dps/rank` returns. The backend caches the full row set per
 * context and can filter and sort too, but doing it here means a filter or sort change
 * costs no request at all. Pure, tested in `rank.test.ts`.
 */

import { clampRankToElite, MAX_SKILL_RANK } from "#/components/tools/shared/skill";
import type { IBuildConfig } from "#/components/tools/shared/types";
import type { IDpsRankRow } from "#/lib/api/dps";
import { searchAndRank } from "#/lib/search/fuzzy";
import type { DamageKind } from "#/types/generated/DamageKind";
import type { RankPreset } from "#/types/generated/RankPreset";
import type { RankRequest } from "#/types/generated/RankRequest";
import { enemyToRequest } from "./state";
import type { IEnemyConfig, IRankSettings, ITeamBuffs, RankSortKey } from "./types";
import { buildRequestZones, type ISupporterSource } from "./zones";

/**
 * The ranking's shared context: the page's enemy, team zones and cycle model, at the
 * preset's investment. Class-limited supporters apply to every row (no per-row profession
 * is known before the rows exist); the view says so.
 */
export function rankRequest(enemy: IEnemyConfig, team: ITeamBuffs, sources: readonly ISupporterSource[], preset: IRankSettings["preset"]): RankRequest {
    const { defense, res, targets, secondary, cycleModel } = enemyToRequest(enemy);
    const zones = buildRequestZones(team.zones, undefined, sources);
    return {
        defense,
        res,
        targets,
        ...(secondary ? { secondary } : {}),
        ...(cycleModel ? { cycleModel } : {}),
        ...(zones ? { zones } : {}),
        preset,
    };
}

/** Stable identity of a row: one (operator, skill, module). */
export function rankRowKey(row: Pick<IDpsRankRow, "operatorId" | "skillIndex" | "moduleIndex">): string {
    return `${row.operatorId}|${row.skillIndex}|${row.moduleIndex}`;
}

/** The sort value of a row; a missing retention sorts last. */
export function rankMetric(row: IDpsRankRow, key: RankSortKey): number {
    switch (key) {
        case "skillDps":
            return row.skillDps;
        case "averageDps":
            return row.averageDps;
        case "totalDamage":
            return row.totalDamage;
        case "retention":
            return typeof row.retention === "number" ? row.retention : Number.NEGATIVE_INFINITY;
    }
}

/** True once the backend serves the roster score on at least one row. */
export function hasRetention(rows: readonly IDpsRankRow[]): boolean {
    return rows.some((r) => typeof r.retention === "number");
}

/**
 * Filter by class, rarity and damage type (each empty = all), keep the rows the fuzzy
 * query matches (operator name strongest, then skill and module names), and sort
 * descending by `sortBy`. Ties break by name, skill, module, so the order is stable
 * across refetches. Search narrows; it never reorders past the metric.
 */
export function selectRankRows(rows: readonly IDpsRankRow[], settings: Pick<IRankSettings, "professions" | "rarities" | "damageTypes" | "sortBy">, query: string): IDpsRankRow[] {
    const professions = new Set(settings.professions);
    const rarities = new Set(settings.rarities);
    const damageTypes = new Set(settings.damageTypes);
    let out = rows.filter((r) => (professions.size === 0 || professions.has(r.profession)) && (rarities.size === 0 || rarities.has(r.rarity)) && (damageTypes.size === 0 || damageTypes.has(r.damageType)));
    if (query.trim()) {
        const matched = new Set(searchAndRank(query, out, (r) => ({ name: r.name, extra: [r.skillName, r.moduleName].filter(Boolean).join(" ") })).map((m) => m.item));
        out = out.filter((r) => matched.has(r));
    }
    return out.sort((a, b) => rankMetric(b, settings.sortBy) - rankMetric(a, settings.sortBy) || a.name.localeCompare(b.name) || a.skillIndex - b.skillIndex || a.moduleIndex - b.moduleIndex);
}

/** `value / top`, the "vs #1" column; null when either is not a positive finite number. */
export function ratioToTop(value: number, top: number | undefined): number | null {
    if (top === undefined || !Number.isFinite(top) || top <= 0 || !Number.isFinite(value)) return null;
    return value / top;
}

/** Highest promotion an operator of this rarity has: E2 from 4 stars, E1 at 3, E0 below. */
export function maxPromotionForRarity(rarity: number): number {
    if (rarity >= 4) return 2;
    if (rarity === 3) return 1;
    return 0;
}

/**
 * A card build that reproduces a ranked row: the row's skill and module at the preset's
 * investment. `p1Max` / `p6Max` mirror the backend's presets (highest promotion at max
 * level, trust 100, highest rank, module level 3, potential 1 or 6); `default` leaves the
 * investment to the card's own defaults, as the backend leaves it to the engine.
 *
 * Conditionals: `/dps/rank` sends no flags and no `allCond`, and the engine reads both as on.
 * A fresh card would take each conditional's `default` (some are off), so the card is given
 * an EMPTY flag set and `allCond: true`, which the engine resolves to the same all-on set.
 */
export function rowToBuild(row: Pick<IDpsRankRow, "skillIndex" | "moduleIndex" | "rarity">, preset: IRankSettings["preset"]): Partial<IBuildConfig> {
    const base: Partial<IBuildConfig> = { skillIndex: row.skillIndex, moduleIndex: row.moduleIndex, conditionals: {}, allCond: true };
    if (preset === "default") return base;
    const promotion = maxPromotionForRarity(row.rarity);
    return { ...base, potential: preset === "p6Max" ? 6 : 1, trust: 100, skillRank: clampRankToElite(MAX_SKILL_RANK, promotion), moduleLevel: 3 };
}

export const RANK_CLASSES = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "SUPPORT", "MEDIC", "SPECIAL"] as const;
const PRESETS: readonly RankPreset[] = ["p1Max", "p6Max", "default"];
const SORTS: readonly RankSortKey[] = ["skillDps", "averageDps", "totalDamage", "retention"];
const DAMAGE_KINDS: readonly DamageKind[] = ["physical", "arts"];

/**
 * A stored or decoded rank block, every field checked against the values the view offers;
 * anything unknown falls back to `fallback`'s value (lists drop their unknown entries).
 */
export function normaliseRankSettings(raw: unknown, fallback: IRankSettings): IRankSettings {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return fallback;
    const r = raw as Record<string, unknown>;
    const pick = <T>(options: readonly T[], v: unknown, d: T): T => (options.includes(v as T) ? (v as T) : d);
    const list = <T>(v: unknown, ok: (x: unknown) => x is T): T[] => (Array.isArray(v) ? [...new Set(v.filter(ok))] : []);
    return {
        preset: pick(PRESETS, r.preset, fallback.preset),
        sortBy: pick(SORTS, r.sortBy, fallback.sortBy),
        professions: list(r.professions, (x): x is string => (RANK_CLASSES as readonly unknown[]).includes(x)),
        rarities: list(r.rarities, (x): x is number => Number.isInteger(x) && (x as number) >= 1 && (x as number) <= 6),
        damageTypes: list(r.damageTypes, (x): x is DamageKind => DAMAGE_KINDS.includes(x as DamageKind)),
    };
}
