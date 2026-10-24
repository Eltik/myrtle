import { migrateInstance } from "#/components/tools/shared/instance";
import { normaliseRankSettings } from "./rank";
import { migrateCardBuffs } from "./share";
import { DEFAULT_ENEMY, DEFAULT_RANK, DEFAULT_TEAM, INITIAL_STATE } from "./state";
import type { DpsView, IDpsState, IEnemyConfig, ITeamBuffs, XAxisKind, YMetric } from "./types";
import { legacyEnemyToZones, normaliseZones } from "./zones";

const VALID_X_AXES: ReadonlySet<XAxisKind> = new Set<XAxisKind>(["defense", "res"]);
const VALID_Y_METRICS: ReadonlySet<YMetric> = new Set<YMetric>(["skill_dps", "average_dps", "total_damage"]);
const VALID_VIEWS: ReadonlySet<DpsView> = new Set<DpsView>(["compare", "rank"]);

/**
 * Validate stored state and fill missing fields from `INITIAL_STATE`. Tolerant
 * of forward changes; per-instance configs are normalised to the current shape.
 * A state saved before the zone panel (no `team`) carries SP boost and shred on
 * `enemy` and External buffs on each card: those move into zones.
 */
export function mergeWithDefaults(value: unknown): IDpsState | null {
    if (!value || typeof value !== "object") return null;
    const v = value as Partial<IDpsState> & { enemy?: Partial<IEnemyConfig> & { spBoost?: number; shred?: Record<string, number> } };
    if (!Array.isArray(v.instances)) return null;
    const instances = v.instances
        .map(migrateInstance)
        .filter((i): i is NonNullable<typeof i> => i !== null)
        .map((i) => ({ ...i, config: migrateCardBuffs({ ...i.config, ...(i.config.zones ? { zones: normaliseZones(i.config.zones) ?? undefined } : {}) }) }));
    const { spBoost: _spBoost, shred: _shred, ...enemy }: Partial<IEnemyConfig> & { spBoost?: number; shred?: Record<string, number> } = v.enemy ?? {};
    return {
        instances,
        enemy: { ...DEFAULT_ENEMY, ...enemy, secondary: { ...DEFAULT_ENEMY.secondary, ...(enemy.secondary ?? {}) }, cycleModel: enemy.cycleModel === "v2" ? "v2" : "legacy" },
        team: mergeTeam(v.team, v.enemy),
        view: VALID_VIEWS.has(v.view as DpsView) ? (v.view as DpsView) : INITIAL_STATE.view,
        rank: normaliseRankSettings(v.rank, DEFAULT_RANK),
        xAxis: VALID_X_AXES.has(v.xAxis as XAxisKind) ? (v.xAxis as XAxisKind) : INITIAL_STATE.xAxis,
        yMetric: VALID_Y_METRICS.has(v.yMetric as YMetric) ? (v.yMetric as YMetric) : INITIAL_STATE.yMetric,
        sweep: {
            defense: { ...INITIAL_STATE.sweep.defense, ...(v.sweep?.defense ?? {}) },
            res: { ...INITIAL_STATE.sweep.res, ...(v.sweep?.res ?? {}) },
        },
        markers: typeof v.markers === "boolean" ? v.markers : INITIAL_STATE.markers,
    };
}

function mergeTeam(team: Partial<ITeamBuffs> | undefined, legacyEnemy: { spBoost?: number; shred?: Record<string, number> } | undefined): ITeamBuffs {
    if (!team) return { ...DEFAULT_TEAM, zones: legacyEnemyToZones(legacyEnemy ?? {}) };
    return {
        zones: normaliseZones(team.zones) ?? DEFAULT_TEAM.zones,
        supporters: Array.isArray(team.supporters) ? team.supporters.filter((id): id is string => typeof id === "string") : [],
        investment: { ...DEFAULT_TEAM.investment, ...(team.investment ?? {}) },
    };
}
