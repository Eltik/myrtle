import { reduceInstances, stripEmptyNumbers } from "#/components/tools/shared/instance";
import { rankToRequest } from "#/components/tools/shared/skill";
import type { IHpsCalculateRequest } from "#/lib/api/hps";
import { DEFAULT_SWEEP } from "./constants";
import type { HpsAction, IHpsBuffConfig, IHpsInstance, IHpsState } from "./types";

const DEFAULT_BUFFS: IHpsBuffConfig = {
    buffs: { atk: 0, flatAtk: 0, aspd: 0, fragile: 0 },
    targets: 1,
    spBoost: 0,
};

export const INITIAL_STATE: IHpsState = {
    instances: [],
    buffs: DEFAULT_BUFFS,
    xAxis: "targets",
    yMetric: "skill_hps",
    sweep: { ...DEFAULT_SWEEP },
};

export function hpsReducer(state: IHpsState, action: HpsAction): IHpsState {
    switch (action.type) {
        case "SET_BUFFS":
            return { ...state, buffs: { ...state.buffs, ...action.patch } };
        case "SET_BUFF_VALUES":
            return { ...state, buffs: { ...state.buffs, buffs: { ...state.buffs.buffs, ...action.patch } } };
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

/** Override applied for one sweep point (display units). */
export interface IAxisOverride {
    targets?: number;
    /** ATK buff in PERCENT (40 = +40%); converted to the decimal the API wants. */
    atkPercent?: number;
    aspd?: number;
}

export function instanceToRequest(inst: IHpsInstance, buffs: IHpsBuffConfig, override?: IAxisOverride): IHpsCalculateRequest {
    const cfg = inst.config;
    const atk = override?.atkPercent !== undefined ? override.atkPercent / 100 : (buffs.buffs.atk ?? 0);
    const aspd = override?.aspd ?? buffs.buffs.aspd ?? 0;
    const targets = override?.targets ?? buffs.targets;
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
        targets: Math.max(1, Math.round(targets)),
        spBoost: buffs.spBoost,
        buffs: stripEmptyNumbers({ atk, flatAtk: buffs.buffs.flatAtk, aspd, fragile: buffs.buffs.fragile }),
        conditionals: cfg.conditionals,
        allCond: cfg.allCond,
    };
}
