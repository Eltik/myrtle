import type { IAxisInput, ISweepRange } from "#/components/tools/shared/types";
import type { messages as dpsConstantsMessages } from "./constants.messages";
import type { XAxisKind, YMetric } from "./types";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type DpsMessageKey = keyof typeof dpsConstantsMessages & string;

export const DEFAULT_SWEEP: Record<XAxisKind, ISweepRange> = {
    defense: { min: 0, max: 2000, steps: 21 },
    res: { min: 0, max: 80, steps: 17 },
};

export const Y_METRIC_LABEL_KEYS: Record<YMetric, DpsMessageKey> = {
    skill_dps: "dps.metric.skillDps",
    average_dps: "dps.metric.averageDps",
    total_damage: "dps.metric.totalDamage",
};

/** Result-panel column order, with the abbreviated heading each one carries. */
export const METRIC_COLUMNS: readonly { key: YMetric; labelKey: DpsMessageKey }[] = [
    { key: "skill_dps", labelKey: "dps.metric.skillDps.short" },
    { key: "average_dps", labelKey: "dps.metric.averageDps.short" },
    { key: "total_damage", labelKey: "dps.metric.totalDamage.short" },
];

export const X_AXIS_LABEL_KEYS: Record<XAxisKind, DpsMessageKey> = {
    defense: "dps.axis.defense",
    res: "dps.axis.res",
};

export const X_AXIS_SHORT_KEYS: Record<XAxisKind, DpsMessageKey> = {
    defense: "dps.axis.defense.short",
    res: "dps.axis.res.short",
};

export const X_AXIS_INPUT: Record<XAxisKind, IAxisInput> = {
    defense: { step: 100, maxBound: 5000, integer: false, unit: "" },
    res: { step: 5, maxBound: 100, integer: false, unit: "%" },
};

export interface IEnemyPreset {
    id: string;
    labelKey: DpsMessageKey;
    summaryKey: DpsMessageKey;
    defense: number;
    res: number;
    targets: number;
}

/** Quick presets matching common Arknights enemy archetypes. */
export const ENEMY_PRESETS: readonly IEnemyPreset[] = [
    { id: "trash", labelKey: "dps.preset.trash", summaryKey: "dps.preset.trash.summary", defense: 0, res: 0, targets: 1 },
    { id: "elite", labelKey: "dps.preset.elite", summaryKey: "dps.preset.elite.summary", defense: 1000, res: 0, targets: 1 },
    { id: "armored", labelKey: "dps.preset.armored", summaryKey: "dps.preset.armored.summary", defense: 1800, res: 20, targets: 1 },
    { id: "boss", labelKey: "dps.preset.boss", summaryKey: "dps.preset.boss.summary", defense: 2500, res: 30, targets: 1 },
    { id: "caster-trash", labelKey: "dps.preset.casterTrash", summaryKey: "dps.preset.casterTrash.summary", defense: 0, res: 60, targets: 1 },
    { id: "drone", labelKey: "dps.preset.droneWave", summaryKey: "dps.preset.droneWave.summary", defense: 200, res: 0, targets: 3 },
];
