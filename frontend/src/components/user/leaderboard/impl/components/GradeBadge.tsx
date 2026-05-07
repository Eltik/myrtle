import { cn } from "#/lib/utils";

const GRADE_STYLES: Record<string, string> = {
    "SS+": "bg-[color-mix(in_srgb,oklch(0.7_0.22_25)_25%,var(--card))] text-[oklch(0.7_0.22_25)] border-[color-mix(in_srgb,oklch(0.7_0.22_25)_40%,transparent)]",
    SS: "bg-[color-mix(in_srgb,oklch(0.72_0.18_50)_22%,var(--card))] text-[oklch(0.72_0.18_50)] border-[color-mix(in_srgb,oklch(0.72_0.18_50)_40%,transparent)]",
    S: "bg-[color-mix(in_srgb,oklch(0.75_0.16_80)_22%,var(--card))] text-[oklch(0.7_0.16_80)] border-[color-mix(in_srgb,oklch(0.75_0.16_80)_40%,transparent)]",
    A: "bg-[color-mix(in_srgb,oklch(0.7_0.16_145)_20%,var(--card))] text-[oklch(0.55_0.16_145)] border-[color-mix(in_srgb,oklch(0.7_0.16_145)_35%,transparent)]",
    B: "bg-[color-mix(in_srgb,oklch(0.65_0.14_200)_20%,var(--card))] text-[oklch(0.55_0.14_200)] border-[color-mix(in_srgb,oklch(0.65_0.14_200)_35%,transparent)]",
    C: "bg-[color-mix(in_srgb,oklch(0.6_0.04_280)_20%,var(--card))] text-muted-foreground border-border",
};

export function GradeBadge({ grade, className }: { grade: string | null; className?: string }) {
    const label = grade ?? "—";
    const style = GRADE_STYLES[label] ?? "bg-muted text-muted-foreground border-border";
    return <span className={cn("inline-flex size-7 shrink-0 items-center justify-center rounded-md border font-sans text-[13px] font-bold leading-none tracking-tight tabular-nums", style, className)}>{label}</span>;
}
