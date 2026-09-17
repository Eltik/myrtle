import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./GradeBadge.messages";

const GRADE_STYLES: Record<string, string> = {
    "S+": "bg-[color-mix(in_srgb,oklch(0.7_0.22_25)_25%,var(--card))] text-[oklch(0.7_0.22_25)] border-[color-mix(in_srgb,oklch(0.7_0.22_25)_40%,transparent)]",
    S: "bg-[color-mix(in_srgb,oklch(0.75_0.16_80)_22%,var(--card))] text-[oklch(0.7_0.16_80)] border-[color-mix(in_srgb,oklch(0.75_0.16_80)_40%,transparent)]",
    A: "bg-[color-mix(in_srgb,oklch(0.7_0.16_145)_20%,var(--card))] text-[oklch(0.55_0.16_145)] border-[color-mix(in_srgb,oklch(0.7_0.16_145)_35%,transparent)]",
    B: "bg-[color-mix(in_srgb,oklch(0.65_0.14_200)_20%,var(--card))] text-[oklch(0.55_0.14_200)] border-[color-mix(in_srgb,oklch(0.65_0.14_200)_35%,transparent)]",
    C: "bg-[color-mix(in_srgb,oklch(0.6_0.04_280)_20%,var(--card))] text-muted-foreground border-border",
    D: "bg-[color-mix(in_srgb,oklch(0.6_0.04_280)_14%,var(--card))] text-muted-foreground border-border",
    F: "bg-muted text-muted-foreground border-border",
};

/**
 * The grade bands, as percentages, mirroring `score_to_grade` in
 * `backend/src/core/grade/calculate.rs`. Kept in the same order the backend
 * matches them, so a change there is easy to mirror here.
 *
 * "SS" used to be styled here and the backend never emits it; D and F did NOT
 * have a style and do get emitted, so both fell through to the generic grey.
 * That is now the other way round.
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
    const style = GRADE_STYLES[label] ?? "bg-muted text-muted-foreground border-border";
    const band = GRADE_BANDS.find((b) => b.grade === label);
    // A plain `title` rather than the tooltip primitive: every badge sits inside
    // a row-wide `<Link>`, and a hover-triggered popup nested in a navigable
    // anchor swallows the row's own hover and click targets.
    const explanation = band ? t("leaderboard.grade.tooltip", { grade: band.grade, from: band.from, to: band.to }) : t("leaderboard.grade.unknown");
    return (
        <span className={cn("inline-flex size-7 shrink-0 cursor-help items-center justify-center rounded-md border font-bold font-sans text-[13px] tabular-nums leading-none tracking-tight", style, className)} title={explanation}>
            {label}
        </span>
    );
}
