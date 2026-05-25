import type { IAxisInput, ISweepRange } from "#/components/tools/shared/types";
import type { HpsXAxis, HpsYMetric } from "./types";

export const DEFAULT_SWEEP: Record<HpsXAxis, ISweepRange> = {
    targets: { min: 1, max: 8, steps: 8 },
    atk: { min: 0, max: 200, steps: 21 },
    aspd: { min: 0, max: 120, steps: 13 },
};

export const Y_METRIC_LABELS: Record<HpsYMetric, string> = {
    skill_hps: "Skill HPS",
    base_hps: "Base HPS",
    avg_hps: "Average HPS",
};

export const Y_METRIC_HINTS: Record<HpsYMetric, string> = {
    skill_hps: "Healing per second while the skill is active.",
    base_hps: "Always-on healing per second during SP recharge (0 for burst healers).",
    avg_hps: "Sustained HPS averaged over a full skill + recharge cycle.",
};

export const X_AXIS_LABELS: Record<HpsXAxis, string> = {
    targets: "Targets healed",
    atk: "ATK buff (%)",
    aspd: "ASPD buff",
};

export const X_AXIS_INPUT: Record<HpsXAxis, IAxisInput> = {
    targets: { step: 1, maxBound: 12, integer: true, unit: "" },
    atk: { step: 10, maxBound: 400, integer: false, unit: "%" },
    aspd: { step: 10, maxBound: 200, integer: false, unit: "" },
};

export interface IBuffPreset {
    id: string;
    label: string;
    summary: string;
    /** ATK buff as a decimal. */
    atk: number;
    aspd: number;
    targets: number;
}

/** Quick presets covering common healing scenarios. */
export const BUFF_PRESETS: readonly IBuffPreset[] = [
    { id: "solo", label: "Solo", summary: "No buffs · 1 target", atk: 0, aspd: 0, targets: 1 },
    { id: "atk-aura", label: "ATK aura", summary: "ATK +40% · 1 target", atk: 0.4, aspd: 0, targets: 1 },
    { id: "aspd", label: "ASPD push", summary: "ASPD +60 · 1 target", atk: 0, aspd: 60, targets: 1 },
    { id: "aoe", label: "AoE", summary: "No buffs · 3 targets", atk: 0, aspd: 0, targets: 3 },
    { id: "buffed-aoe", label: "Buffed AoE", summary: "ATK +40% · 3 targets", atk: 0.4, aspd: 0, targets: 3 },
];
