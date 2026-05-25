import { reduceInstances, stripEmptyNumbers } from "#/components/tools/shared/instance";
import { rankToRequest } from "#/components/tools/shared/skill";
import type { IDpsCalculateRequest } from "#/lib/api/dps";
import { DEFAULT_SWEEP } from "./constants";
import type { DpsAction, IDpsInstance, IDpsState, IEnemyConfig } from "./types";

const DEFAULT_ENEMY: IEnemyConfig = {
    defense: 0,
    res: 0,
    targets: 1,
    spBoost: 0,
    shred: { def: 0, defFlat: 0, res: 0, resFlat: 0 },
};

export const INITIAL_STATE: IDpsState = {
    instances: [],
    enemy: DEFAULT_ENEMY,
    xAxis: "defense",
    yMetric: "skill_dps",
    sweep: { ...DEFAULT_SWEEP },
};

export function dpsReducer(state: IDpsState, action: DpsAction): IDpsState {
    switch (action.type) {
        case "SET_ENEMY":
            return { ...state, enemy: { ...state.enemy, ...action.patch } };
        case "SET_SHRED":
            return { ...state, enemy: { ...state.enemy, shred: { ...state.enemy.shred, ...action.patch } } };
        case "SET_AXIS":
            return { ...state, xAxis: action.axis };
        case "SET_METRIC":
            return { ...state, yMetric: action.metric };
        case "SET_SWEEP":
            return { ...state, sweep: { ...state.sweep, [action.axis]: { ...state.sweep[action.axis], ...action.patch } } };
        case "HYDRATE":
            return action.state;
        default: {
            const instances = reduceInstances(state.instances, action);
            return instances === state.instances ? state : { ...state, instances };
        }
    }
}

export function instanceToRequest(inst: IDpsInstance, enemy: IEnemyConfig, axisOverride?: { defense?: number; res?: number }): IDpsCalculateRequest {
    const cfg = inst.config;
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
        defense: axisOverride?.defense ?? enemy.defense,
        res: axisOverride?.res ?? enemy.res,
        targets: enemy.targets,
        spBoost: enemy.spBoost,
        buffs: stripEmptyNumbers(cfg.buffs as Record<string, number | undefined>) as IDpsCalculateRequest["buffs"],
        shred: stripEmptyNumbers(enemy.shred as Record<string, number | undefined>) as IDpsCalculateRequest["shred"],
        conditionals: cfg.conditionals,
        allCond: cfg.allCond,
    };
}
