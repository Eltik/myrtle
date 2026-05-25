import type { InstanceAction } from "#/components/tools/shared/instance";
import type { IInstance, ISweepRange } from "#/components/tools/shared/types";
import type { IDpsCalculateShred } from "#/lib/api/dps";

export type XAxisKind = "defense" | "res";
export type YMetric = "skill_dps" | "average_dps" | "total_damage";

export type IDpsInstance = IInstance;

/** Enemy + target context shared across all DPS instances. */
export interface IEnemyConfig {
    defense: number;
    res: number;
    targets: number;
    spBoost: number;
    shred: IDpsCalculateShred;
}

export interface IDpsState {
    instances: IDpsInstance[];
    enemy: IEnemyConfig;
    xAxis: XAxisKind;
    yMetric: YMetric;
    sweep: Record<XAxisKind, ISweepRange>;
}

/** DPS-specific chart-control actions; instance actions come from the shared union. */
export type DpsAction =
    | InstanceAction
    | { type: "SET_ENEMY"; patch: Partial<IEnemyConfig> }
    | { type: "SET_SHRED"; patch: Partial<IDpsCalculateShred> }
    | { type: "SET_AXIS"; axis: XAxisKind }
    | { type: "SET_METRIC"; metric: YMetric }
    | { type: "SET_SWEEP"; axis: XAxisKind; patch: Partial<ISweepRange> }
    | { type: "HYDRATE"; state: IDpsState };
