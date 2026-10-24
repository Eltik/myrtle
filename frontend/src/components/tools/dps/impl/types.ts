import type { InstanceAction } from "#/components/tools/shared/instance";
import type { IInstance, ISweepRange } from "#/components/tools/shared/types";
import type { DamageKind } from "#/types/generated/DamageKind";
import type { RankPreset } from "#/types/generated/RankPreset";
import type { RequestCycleModel } from "#/types/generated/RequestCycleModel";
import type { SupporterInvestment } from "#/types/generated/SupporterInvestment";
import type { IZoneConfig } from "./zones";

export type XAxisKind = "defense" | "res";
export type YMetric = "skill_dps" | "average_dps" | "total_damage";

export type IDpsInstance = IInstance;

/** Optional enemies beside the primary one; `enabled` keeps the numbers while it is off. */
export interface ISecondaryConfig {
    enabled: boolean;
    def: number;
    res: number;
    /** Extra enemies, 1-49. */
    count: number;
}

/** Enemy + target context shared across all DPS instances. */
export interface IEnemyConfig {
    defense: number;
    res: number;
    targets: number;
    secondary: ISecondaryConfig;
    cycleModel: RequestCycleModel;
}

/** Buffs every card receives: the zone inputs and the supporter presets switched on. */
export interface ITeamBuffs {
    zones: IZoneConfig;
    /** Supporter entry ids (`GET /dps/supporters`), in the order they were switched on. */
    supporters: string[];
    /** The investment the supporters' values are read at. */
    investment: SupporterInvestment;
}

export type DpsView = "compare" | "rank";
export type RankSortKey = "skillDps" | "averageDps" | "totalDamage" | "retention";

export interface IRankSettings {
    preset: RankPreset;
    sortBy: RankSortKey;
    professions: string[];
    rarities: number[];
    damageTypes: DamageKind[];
}

export interface IDpsState {
    instances: IDpsInstance[];
    enemy: IEnemyConfig;
    team: ITeamBuffs;
    xAxis: XAxisKind;
    yMetric: YMetric;
    sweep: Record<XAxisKind, ISweepRange>;
    /** Break-point and half-damage markers on the chart's curves. */
    markers: boolean;
    view: DpsView;
    rank: IRankSettings;
}

/** DPS-specific chart-control actions; instance actions come from the shared union. */
export type DpsAction =
    | InstanceAction
    | { type: "SET_ENEMY"; patch: Partial<Omit<IEnemyConfig, "secondary">> }
    | { type: "SET_SECONDARY"; patch: Partial<ISecondaryConfig> }
    | { type: "SET_TEAM_ZONES"; patch: Partial<IZoneConfig> }
    | { type: "RESET_TEAM_ZONES" }
    | { type: "TOGGLE_SUPPORTER"; id: string; on: boolean }
    | { type: "SET_INVESTMENT"; patch: Partial<SupporterInvestment> }
    | { type: "SET_CARD_ZONES"; uid: string; patch: Partial<IZoneConfig> }
    | { type: "SET_AXIS"; axis: XAxisKind }
    | { type: "SET_METRIC"; metric: YMetric }
    | { type: "SET_SWEEP"; axis: XAxisKind; patch: Partial<ISweepRange> }
    | { type: "SET_MARKERS"; markers: boolean }
    | { type: "SET_VIEW"; view: DpsView }
    | { type: "SET_RANK"; patch: Partial<IRankSettings> }
    | { type: "HYDRATE"; state: IDpsState };
