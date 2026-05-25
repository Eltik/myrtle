import type { InstanceAction } from "#/components/tools/shared/instance";
import type { ICalcBuffs, IInstance, ISweepRange } from "#/components/tools/shared/types";

/**
 * Healing ignores enemy DEF/RES, so the chart sweeps a team-buff or target
 * dimension instead. `targets` is the default - single-target vs. AoE healers
 * diverge and cross there, the healing analog of the DPS DEF/RES crossover.
 */
export type HpsXAxis = "targets" | "atk" | "aspd";
export type HpsYMetric = "skill_hps" | "base_hps" | "avg_hps";

export type IHpsInstance = IInstance;

/** Shared team buffs + target count applied to every healer. */
export interface IHpsBuffConfig {
    buffs: ICalcBuffs;
    targets: number;
    spBoost: number;
}

export interface IHpsState {
    instances: IHpsInstance[];
    buffs: IHpsBuffConfig;
    xAxis: HpsXAxis;
    yMetric: HpsYMetric;
    sweep: Record<HpsXAxis, ISweepRange>;
}

export type HpsAction =
    | InstanceAction
    | { type: "SET_BUFFS"; patch: Partial<IHpsBuffConfig> }
    | { type: "SET_BUFF_VALUES"; patch: Partial<ICalcBuffs> }
    | { type: "SET_AXIS"; axis: HpsXAxis }
    | { type: "SET_METRIC"; metric: HpsYMetric }
    | { type: "SET_SWEEP"; axis: HpsXAxis; patch: Partial<ISweepRange> }
    | { type: "HYDRATE"; state: IHpsState };
