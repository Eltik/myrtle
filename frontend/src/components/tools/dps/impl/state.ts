import { reduceInstances } from "#/components/tools/shared/instance";
import { rankToRequest } from "#/components/tools/shared/skill";
import type { IDpsCalculateRequest } from "#/lib/api/dps";
import { DEFAULT_SWEEP } from "./constants";
import type { DpsAction, IDpsInstance, IDpsState, IEnemyConfig, IRankSettings, ITeamBuffs } from "./types";
import { buildRequestZones, EMPTY_ZONES, type ISupporterSource } from "./zones";

export const DEFAULT_ENEMY: IEnemyConfig = {
    defense: 0,
    res: 0,
    targets: 1,
    secondary: { enabled: false, def: 0, res: 0, count: 1 },
    cycleModel: "legacy",
};

/** The backend's `SupporterInvestment::default()`: E2, M3, P1, no module. */
export const DEFAULT_TEAM: ITeamBuffs = {
    zones: EMPTY_ZONES,
    supporters: [],
    investment: { promotion: 2, skillLevel: 10, potential: 1, moduleLevel: 0 },
};

export const DEFAULT_RANK: IRankSettings = {
    preset: "p1Max",
    sortBy: "skillDps",
    professions: [],
    rarities: [],
    damageTypes: [],
};

export const INITIAL_STATE: IDpsState = {
    instances: [],
    enemy: DEFAULT_ENEMY,
    team: DEFAULT_TEAM,
    xAxis: "defense",
    yMetric: "skill_dps",
    sweep: { ...DEFAULT_SWEEP },
    markers: true,
    view: "compare",
    rank: DEFAULT_RANK,
};

export function dpsReducer(state: IDpsState, action: DpsAction): IDpsState {
    switch (action.type) {
        case "SET_ENEMY":
            return { ...state, enemy: { ...state.enemy, ...action.patch } };
        case "SET_SECONDARY":
            return { ...state, enemy: { ...state.enemy, secondary: { ...state.enemy.secondary, ...action.patch } } };
        case "SET_TEAM_ZONES":
            return { ...state, team: { ...state.team, zones: { ...state.team.zones, ...action.patch } } };
        case "RESET_TEAM_ZONES":
            return { ...state, team: { ...state.team, zones: EMPTY_ZONES, supporters: [] } };
        case "TOGGLE_SUPPORTER": {
            const has = state.team.supporters.includes(action.id);
            if (has === action.on) return state;
            const supporters = action.on ? [...state.team.supporters, action.id] : state.team.supporters.filter((id) => id !== action.id);
            return { ...state, team: { ...state.team, supporters } };
        }
        case "SET_INVESTMENT":
            return { ...state, team: { ...state.team, investment: { ...state.team.investment, ...action.patch } } };
        case "SET_CARD_ZONES":
            return {
                ...state,
                instances: state.instances.map((i) => (i.uid === action.uid ? { ...i, config: { ...i.config, zones: { ...(i.config.zones ?? EMPTY_ZONES), ...action.patch } } } : i)),
            };
        case "SET_AXIS":
            return { ...state, xAxis: action.axis };
        case "SET_METRIC":
            return { ...state, yMetric: action.metric };
        case "SET_SWEEP":
            return { ...state, sweep: { ...state.sweep, [action.axis]: { ...state.sweep[action.axis], ...action.patch } } };
        case "SET_MARKERS":
            return { ...state, markers: action.markers };
        case "SET_VIEW":
            return { ...state, view: action.view };
        case "SET_RANK":
            return { ...state, rank: { ...state.rank, ...action.patch } };
        case "HYDRATE":
            return action.state;
        default: {
            const instances = reduceInstances(state.instances, action);
            return instances === state.instances ? state : { ...state, instances };
        }
    }
}

/** The enemy-side request fields every card and the ranking share. `targets` and `secondary` are exclusive on the backend. */
export function enemyToRequest(enemy: IEnemyConfig): Pick<IDpsCalculateRequest, "defense" | "res" | "targets" | "secondary" | "cycleModel"> {
    const secondary = enemy.secondary.enabled ? { def: enemy.secondary.def, res: enemy.secondary.res, count: Math.min(49, Math.max(1, Math.round(enemy.secondary.count))) } : undefined;
    return {
        defense: enemy.defense,
        res: enemy.res,
        // With a secondary target the primary is ONE enemy; `secondary.count` carries the rest.
        targets: secondary ? 1 : enemy.targets,
        ...(secondary ? { secondary } : {}),
        // Absent restores the HEAD average exactly; only v2 is sent.
        ...(enemy.cycleModel === "v2" ? { cycleModel: "v2" as const } : {}),
    };
}

/**
 * One card's request. The card no longer sends the legacy `buffs` / `shred` / `spBoost`:
 * they were migrated into zones (`zones.ts`), whose one-source values the backend holds
 * bit-identical to the legacy fields.
 */
export function instanceToRequest(inst: IDpsInstance, enemy: IEnemyConfig, team: ITeamBuffs, supporters: readonly ISupporterSource[], profession?: string): IDpsCalculateRequest {
    const cfg = inst.config;
    const zones = buildRequestZones(team.zones, cfg.zones, supporters, profession);
    return {
        operatorId: inst.op.id,
        promotion: cfg.promotion,
        level: cfg.level,
        potential: cfg.potential,
        trust: cfg.trust,
        skillIndex: cfg.skillIndex,
        ...rankToRequest(cfg.skillRank),
        moduleIndex: cfg.moduleIndex,
        moduleLevel: cfg.moduleIndex > 0 ? cfg.moduleLevel : undefined,
        ...enemyToRequest(enemy),
        ...(zones ? { zones } : {}),
        conditionals: cfg.conditionals,
        allCond: cfg.allCond,
    };
}
