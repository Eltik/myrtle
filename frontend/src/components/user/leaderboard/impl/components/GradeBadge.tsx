import type React from "react";
import { Badge } from "#/components/ui/badge";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./GradeBadge.messages";

/**
 * Grade hue per band, as a `--tone` for the kit tint badge: the tone fills (12%) and draws the
 * border, the letter stays foreground. The old tone-coloured letter on its own tint measured
 * S+ 2.36, S 2.28, B 3.45, A 3.82 (light) and A 3.07, B 3.41 (dark) in audit-003.
 */
const GRADE_TONES: Record<string, string> = {
    "S+": "oklch(0.7 0.22 25)",
    S: "oklch(0.75 0.16 80)",
    A: "oklch(0.7 0.16 145)",
    B: "oklch(0.65 0.14 200)",
    C: "oklch(0.6 0.04 280)",
    D: "oklch(0.6 0.04 280)",
};

/**
 * The grade bands, as percentages, mirroring `score_to_grade` in
 * `backend/src/core/grade/calculate.rs`. Same order the backend matches them, so a change
 * there is easy to mirror here.
 *
 * The backend never emits "SS" but does emit D and F, so those two carry styles and SS does not.
 */
const GRADE_BANDS: ReadonlyArray<{ grade: string; from: number; to: number }> = [
    { grade: "S+", from: 90, to: 100 },
    { grade: "S", from: 75, to: 90 },
    { grade: "A", from: 60, to: 75 },
    { grade: "B", from: 45, to: 60 },
    { grade: "C", from: 30, to: 45 },
    { grade: "D", from: 15, to: 30 },
    { grade: "F", from: 0, to: 15 },
];

export function GradeBadge({ grade, className }: { grade: string | null; className?: string }) {
    const t: TypedT<typeof messages> = useT("user");
    const label = grade ?? "-";
    const tone = GRADE_TONES[label];
    const band = GRADE_BANDS.find((b) => b.grade === label);
    // A plain `title` rather than the tooltip primitive: every badge sits inside
    // a row-wide `<Link>`, and a hover-triggered popup nested in a navigable
    // anchor swallows the row's own hover and click targets. The band itself is
    // sr-only text, so it is not a hover-only hint.
    const explanation = band ? t("leaderboard.grade.tooltip", { grade: band.grade, from: band.from, to: band.to }) : t("leaderboard.grade.unknown");
    const spoken = band ? t("leaderboard.grade.band", { grade: band.grade, from: band.from, to: band.to }) : t("leaderboard.grade.none");
    return (
        <Badge
            variant={tone ? "tint" : "secondary"}
            className={cn("size-7 rounded-md font-bold font-sans text-[13px] tabular-nums leading-none tracking-tight sm:size-7 sm:text-[13px]", !tone && "border-border bg-muted text-muted-foreground", className)}
            style={tone ? ({ "--tone": tone } as React.CSSProperties) : undefined}
            title={explanation}
        >
            <span aria-hidden>{label}</span>
            <span className="sr-only">{spoken}</span>
        </Badge>
    );
}
