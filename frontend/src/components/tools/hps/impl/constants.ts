import type { IAxisInput, ISweepRange } from "#/components/tools/shared/types";
import type { messages as hpsConstantsMessages } from "./constants.messages";
import type { HpsXAxis, HpsYMetric } from "./types";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type HpsMessageKey = keyof typeof hpsConstantsMessages & string;

export const DEFAULT_SWEEP: Record<HpsXAxis, ISweepRange> = {
    targets: { min: 1, max: 8, steps: 8 },
    atk: { min: 0, max: 200, steps: 21 },
    aspd: { min: 0, max: 120, steps: 13 },
};

export const Y_METRIC_LABEL_KEYS: Record<HpsYMetric, HpsMessageKey> = {
    skill_hps: "hps.metric.skillHps",
    base_hps: "hps.metric.baseHps",
    avg_hps: "hps.metric.avgHps",
};

export const Y_METRIC_HINT_KEYS: Record<HpsYMetric, HpsMessageKey> = {
    skill_hps: "hps.metric.skillHps.hint",
    base_hps: "hps.metric.baseHps.hint",
    avg_hps: "hps.metric.avgHps.hint",
};

/** Result-panel column order, with the abbreviated heading each one carries. */
export const METRIC_COLUMNS: readonly { key: HpsYMetric; labelKey: HpsMessageKey }[] = [
    { key: "skill_hps", labelKey: "hps.metric.skillHps.short" },
    { key: "base_hps", labelKey: "hps.metric.baseHps.short" },
    { key: "avg_hps", labelKey: "hps.metric.avgHps.short" },
];

export const X_AXIS_LABEL_KEYS: Record<HpsXAxis, HpsMessageKey> = {
    targets: "hps.axis.targets",
    atk: "hps.axis.atk",
    aspd: "hps.axis.aspd",
};

/** Compact tab labels so the three axes fit on narrow screens. */
export const X_AXIS_SHORT_KEYS: Record<HpsXAxis, HpsMessageKey> = {
    targets: "hps.axis.targets.short",
    atk: "hps.axis.atk.short",
    aspd: "hps.axis.aspd.short",
};

export const X_AXIS_INPUT: Record<HpsXAxis, IAxisInput> = {
    targets: { step: 1, maxBound: 12, integer: true, unit: "" },
    atk: { step: 10, maxBound: 400, integer: false, unit: "%" },
    aspd: { step: 10, maxBound: 200, integer: false, unit: "" },
};

export interface IBuffPreset {
    id: string;
    labelKey: HpsMessageKey;
    summaryKey: HpsMessageKey;
    /** ATK buff as a decimal. */
    atk: number;
    aspd: number;
    targets: number;
}

/** Quick presets covering common healing scenarios. */
export const BUFF_PRESETS: readonly IBuffPreset[] = [
    { id: "solo", labelKey: "hps.preset.solo", summaryKey: "hps.preset.solo.summary", atk: 0, aspd: 0, targets: 1 },
    { id: "atk-aura", labelKey: "hps.preset.atkAura", summaryKey: "hps.preset.atkAura.summary", atk: 0.4, aspd: 0, targets: 1 },
    { id: "aspd", labelKey: "hps.preset.aspd", summaryKey: "hps.preset.aspd.summary", atk: 0, aspd: 60, targets: 1 },
    { id: "aoe", labelKey: "hps.preset.aoe", summaryKey: "hps.preset.aoe.summary", atk: 0, aspd: 0, targets: 3 },
    { id: "buffed-aoe", labelKey: "hps.preset.buffedAoe", summaryKey: "hps.preset.buffedAoe.summary", atk: 0.4, aspd: 0, targets: 3 },
];
