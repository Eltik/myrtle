import type { IAxisInput, ISweepRange } from "#/components/tools/shared/types";
import type { XAxisKind, YMetric } from "./types";

export const DEFAULT_SWEEP: Record<XAxisKind, ISweepRange> = {
    defense: { min: 0, max: 2000, steps: 21 },
    res: { min: 0, max: 80, steps: 17 },
};

export const Y_METRIC_LABELS: Record<YMetric, string> = {
    skill_dps: "Skill DPS",
    average_dps: "Average DPS",
    total_damage: "Total Damage",
};

export const X_AXIS_LABELS: Record<XAxisKind, string> = {
    defense: "Enemy DEF",
    res: "Enemy RES (%)",
};

export const X_AXIS_INPUT: Record<XAxisKind, IAxisInput> = {
    defense: { step: 100, maxBound: 5000, integer: false, unit: "" },
    res: { step: 5, maxBound: 100, integer: false, unit: "%" },
};

export interface IEnemyPreset {
    id: string;
    label: string;
    summary: string;
    defense: number;
    res: number;
    targets: number;
}

/** Quick presets matching common Arknights enemy archetypes. */
export const ENEMY_PRESETS: readonly IEnemyPreset[] = [
    { id: "trash", label: "Trash", summary: "DEF 0 · RES 0", defense: 0, res: 0, targets: 1 },
    { id: "elite", label: "Elite", summary: "DEF 1000 · RES 0", defense: 1000, res: 0, targets: 1 },
    { id: "armored", label: "Armored", summary: "DEF 1800 · RES 20", defense: 1800, res: 20, targets: 1 },
    { id: "boss", label: "Boss", summary: "DEF 2500 · RES 30", defense: 2500, res: 30, targets: 1 },
    { id: "caster-trash", label: "Caster trash", summary: "DEF 0 · RES 60", defense: 0, res: 60, targets: 1 },
    { id: "drone", label: "Drone wave", summary: "DEF 200 · 3 targets", defense: 200, res: 0, targets: 3 },
];
