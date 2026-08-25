import { Crown, Hammer, Medal, Mountain, Sparkles, Swords } from "lucide-react";
import type { ElementType } from "react";
import type { IUserScore } from "#/lib/api/user";
import { SCORE_PALETTE } from "./palette";

export interface ISubscore {
    key: keyof Pick<IUserScore, "operator_score" | "base_score" | "stage_score" | "roguelike_score" | "sandbox_score" | "medal_score">;
    label: string;
    description: string;
    icon: ElementType;
    color: string;
    weight: number;
}

// `weight` mirrors the backend's SECTION_WEIGHT_* constants in
// `backend/src/core/grade/calculate.rs` - change both together.
export const SUBSCORES: ISubscore[] = [
    { key: "operator_score", label: "Operator", description: "Roster depth & investment", icon: Crown, color: SCORE_PALETTE.operator, weight: 0.85 },
    { key: "base_score", label: "Base", description: "Yield vs. your optimal setup", icon: Hammer, color: SCORE_PALETTE.base, weight: 0.35 },
    { key: "stage_score", label: "Stages", description: "Story & event clears", icon: Swords, color: SCORE_PALETTE.stage, weight: 0.6 },
    { key: "roguelike_score", label: "Roguelike", description: "IS endings & relics", icon: Sparkles, color: SCORE_PALETTE.roguelike, weight: 0.3 },
    { key: "sandbox_score", label: "Sandbox", description: "RA progress & nodes", icon: Mountain, color: SCORE_PALETTE.sandbox, weight: 0.2 },
    { key: "medal_score", label: "Medals", description: "Achievement collection", icon: Medal, color: SCORE_PALETTE.medal, weight: 0.2 },
];

const TOTAL_WEIGHT = SUBSCORES.reduce((s, x) => s + x.weight, 0);

export function weightShare(weight: number): number {
    return (weight / TOTAL_WEIGHT) * 100;
}

export function toPct(score01: number | null | undefined): number {
    if (score01 == null || Number.isNaN(score01)) return 0;
    return Math.max(0, Math.min(100, score01 * 100));
}

export function formatPct(score01: number | null | undefined, digits = 1): string {
    return `${toPct(score01).toFixed(digits)}%`;
}

export function formatCalculatedAt(iso: string | null | undefined): string | null {
    if (!iso) return null;
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return null;
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
}

// Mirrors the backend's `score_to_grade` thresholds in
// `backend/src/core/grade/calculate.rs` - change both together.
export const GRADE_LADDER = [
    { grade: "F", min: 0.0 },
    { grade: "D", min: 0.15 },
    { grade: "C", min: 0.3 },
    { grade: "B", min: 0.45 },
    { grade: "A", min: 0.6 },
    { grade: "S", min: 0.75 },
    { grade: "S+", min: 0.9 },
] as const;

/** The next rung above `total` (0-1 scale), or null at the top. */
export function nextGradeStep(total: number): { grade: string; min: number; pointsAway: number } | null {
    const next = GRADE_LADDER.find((g) => g.min > total + 1e-9);
    if (!next) return null;
    return { grade: next.grade, min: next.min, pointsAway: (next.min - total) * 100 };
}

/**
 * The section where a point of section-score buys the most composite score:
 * highest weight share among sections with real headroom left. Returns null
 * when everything is effectively maxed.
 */
export function fastestSection(score: IUserScore): ISubscore | null {
    const candidates = SUBSCORES.filter((sub) => toPct(score[sub.key]) < 99.5);
    if (candidates.length === 0) return null;
    return candidates.reduce((best, sub) => (sub.weight > best.weight ? sub : best));
}
