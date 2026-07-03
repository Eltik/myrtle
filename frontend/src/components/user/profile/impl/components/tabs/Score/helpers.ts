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
    { key: "operator_score", label: "Operator", description: "Roster depth & investment", icon: Crown, color: SCORE_PALETTE.operator, weight: 1.0 },
    { key: "base_score", label: "Base", description: "Drone & facility upgrades", icon: Hammer, color: SCORE_PALETTE.base, weight: 0.5 },
    { key: "stage_score", label: "Stages", description: "Story & event clears", icon: Swords, color: SCORE_PALETTE.stage, weight: 0.4 },
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
