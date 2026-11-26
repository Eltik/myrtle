import type * as React from "react";

interface IKpiGridProps {
    /** The column template per breakpoint; each screen keeps its own. */
    columnsClass: string;
    children: React.ReactNode;
}

/**
 * The headline-number strip both gacha screens open with. The 1px gaps over a border-coloured
 * grid draw the dividers, so they stay right at every column count without per-breakpoint
 * nth-child border rules.
 */
export function KpiGrid({ columnsClass, children }: IKpiGridProps) {
    return <div className={`grid gap-px overflow-hidden rounded-2xl border border-border bg-border ${columnsClass}`}>{children}</div>;
}

interface IKpiProps {
    label: string;
    value: React.ReactNode;
    meta?: React.ReactNode;
    /** The lead figure: an accent wash, a live dot and an accent value. */
    featured?: boolean;
}

export function Kpi({ label, value, meta, featured }: IKpiProps) {
    return (
        <div className={`relative flex flex-col gap-2 bg-card px-4 py-3.5 sm:gap-2.5 sm:px-5 sm:py-4.5 ${featured ? "bg-linear-[150deg] from-primary/8 to-transparent" : ""}`}>
            <div className="kicker inline-flex items-center gap-2 leading-none">
                {featured ? (
                    <span className="relative inline-flex h-1.5 w-1.5 shrink-0" aria-hidden>
                        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-primary opacity-70" />
                        <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-primary" />
                    </span>
                ) : null}
                {label}
            </div>
            <div className={`flex items-baseline gap-1 font-bold font-sans text-[30px] tabular-nums leading-[0.95] tracking-[-0.04em] sm:text-[36px] sm:tracking-[-0.045em] lg:text-[42px] ${featured ? "text-primary dark:text-[color-mix(in_oklab,var(--primary)_70%,white)]" : "text-foreground"}`}>{value}</div>
            {meta ? <div className="flex flex-wrap items-center gap-2 font-mono text-[11px] text-muted-foreground">{meta}</div> : null}
        </div>
    );
}
