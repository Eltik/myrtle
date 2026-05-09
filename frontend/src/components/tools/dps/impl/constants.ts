import type { ISweepRange, XAxisKind, YMetric } from "./types";

export const CHART_PALETTE: readonly string[] = ["oklch(0.65 0.20 30)", "oklch(0.62 0.18 220)", "oklch(0.62 0.18 145)", "oklch(0.62 0.18 305)", "oklch(0.70 0.16 65)", "oklch(0.60 0.20 0)", "oklch(0.62 0.16 190)", "oklch(0.55 0.20 270)"];

export function getInstanceColor(index: number): string {
    return CHART_PALETTE[index % CHART_PALETTE.length] ?? CHART_PALETTE[0];
}

export const DEFAULT_SWEEP: Record<XAxisKind, ISweepRange> = {
    defense: { min: 0, max: 2000, steps: 21 },
    res: { min: 0, max: 80, steps: 17 },
};

export const STEP_OPTIONS: readonly number[] = [11, 21, 41];

export const Y_METRIC_LABELS: Record<YMetric, string> = {
    skill_dps: "Skill DPS",
    average_dps: "Average DPS",
    total_damage: "Total Damage",
};

export const X_AXIS_LABELS: Record<XAxisKind, string> = {
    defense: "Enemy DEF",
    res: "Enemy RES (%)",
};

export function buildSweepPoints(range: ISweepRange): number[] {
    const steps = Math.max(2, Math.floor(range.steps));
    const min = Math.min(range.min, range.max);
    const max = Math.max(range.min, range.max);
    if (steps === 1 || min === max) return [min];
    const stride = (max - min) / (steps - 1);
    const out: number[] = [];
    for (let i = 0; i < steps; i++) {
        const value = min + stride * i;
        out.push(Math.round(value * 100) / 100);
    }
    return out;
}

export function formatLargeNumber(n: number | null | undefined): string {
    if (n == null || !Number.isFinite(n)) return "—";
    const v = Math.abs(n);
    if (v >= 1_000_000) return `${(n / 1_000_000).toFixed(2).replace(/\.?0+$/, "")}M`;
    if (v >= 10_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
    if (v >= 1_000) return Math.round(n).toLocaleString("en-US");
    return (Math.round(n * 10) / 10).toString();
}

export interface IEnemyPreset {
    id: string;
    label: string;
    /** Short caption shown under the chip (e.g. "DEF 0 · RES 0%"). */
    summary: string;
    defense: number;
    res: number;
    targets: number;
}

/**
 * Quick presets that match common Arknights enemy archetypes. Tuned to land
 * roughly on the breakpoints used in tier-list math: trash → naked, elite →
 * mid-game, boss → late-game gauntlet, ranged → mid-range mages, drone → AoE.
 */
export const ENEMY_PRESETS: readonly IEnemyPreset[] = [
    { id: "trash", label: "Trash", summary: "DEF 0 · RES 0", defense: 0, res: 0, targets: 1 },
    { id: "elite", label: "Elite", summary: "DEF 1000 · RES 0", defense: 1000, res: 0, targets: 1 },
    { id: "armored", label: "Armored", summary: "DEF 1800 · RES 20", defense: 1800, res: 20, targets: 1 },
    { id: "boss", label: "Boss", summary: "DEF 2500 · RES 30", defense: 2500, res: 30, targets: 1 },
    { id: "caster-trash", label: "Caster trash", summary: "DEF 0 · RES 60", defense: 0, res: 60, targets: 1 },
    { id: "drone", label: "Drone wave", summary: "DEF 200 · 3 targets", defense: 200, res: 0, targets: 3 },
];
