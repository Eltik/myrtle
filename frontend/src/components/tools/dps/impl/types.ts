import type { IDpsCalculateBuffs, IDpsCalculateConditionals, IDpsCalculateShred, IDpsOperatorListEntry } from "#/lib/api/dps";

export type XAxisKind = "defense" | "res";

export type YMetric = "skill_dps" | "average_dps" | "total_damage";

export interface IDpsInstanceConfig {
    promotion?: number;
    level?: number;
    potential: number;
    trust: number;
    skillIndex: number;
    masteryLevel: number;
    moduleIndex: number;
    moduleLevel: number;
    buffs: IDpsCalculateBuffs;
    conditionals: IDpsCalculateConditionals;
    allCond: boolean;
}

export interface IDpsInstance {
    uid: string;
    op: IDpsOperatorListEntry;
    color: string;
    visible: boolean;
    collapsed: boolean;
    config: IDpsInstanceConfig;
}

export interface IEnemyConfig {
    defense: number;
    res: number;
    targets: number;
    spBoost: number;
    shred: IDpsCalculateShred;
}

export interface ISweepRange {
    min: number;
    max: number;
    steps: number;
}

export interface IDpsState {
    instances: IDpsInstance[];
    enemy: IEnemyConfig;
    xAxis: XAxisKind;
    yMetric: YMetric;
    sweep: Record<XAxisKind, ISweepRange>;
}

export type DpsAction =
    | { type: "ADD_INSTANCE"; op: IDpsOperatorListEntry }
    | { type: "DUPLICATE_INSTANCE"; uid: string }
    | { type: "REMOVE_INSTANCE"; uid: string }
    | { type: "UPDATE_CONFIG"; uid: string; patch: Partial<IDpsInstanceConfig> }
    | { type: "UPDATE_BUFFS"; uid: string; patch: Partial<IDpsCalculateBuffs> }
    | { type: "TOGGLE_CONDITIONAL"; uid: string; key: keyof IDpsCalculateConditionals; value: boolean }
    | { type: "TOGGLE_VISIBILITY"; uid: string }
    | { type: "TOGGLE_COLLAPSED"; uid: string }
    | { type: "SET_ALL_COLLAPSED"; collapsed: boolean }
    | { type: "REORDER_INSTANCE"; uid: string; direction: "up" | "down" }
    | { type: "SET_ENEMY"; patch: Partial<IEnemyConfig> }
    | { type: "SET_SHRED"; patch: Partial<IDpsCalculateShred> }
    | { type: "SET_AXIS"; axis: XAxisKind }
    | { type: "SET_METRIC"; metric: YMetric }
    | { type: "SET_SWEEP"; axis: XAxisKind; patch: Partial<ISweepRange> }
    | { type: "RESET_INSTANCES" }
    | { type: "REFRESH_OPS"; freshOps: Map<string, IDpsOperatorListEntry> }
    | { type: "HYDRATE"; state: IDpsState };

export interface ICurvePoint {
    x: number;
    /** Map of instance uid → metric value at this X. */
    [uid: string]: number | null;
}
