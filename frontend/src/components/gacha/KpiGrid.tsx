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
}

export function Kpi({ label, value, meta }: IKpiProps) {
    return (
        <div className="relative flex flex-col gap-2 bg-card px-4 py-3.5 sm:gap-2.5 sm:px-5 sm:py-4.5">
            <div className="kicker inline-flex items-center gap-2 leading-none">{label}</div>
            <div className="flex items-baseline gap-1 font-bold font-sans text-[30px] text-foreground tabular-nums leading-[0.95] tracking-[-0.04em] sm:text-[36px] sm:tracking-[-0.045em] lg:text-[42px]">{value}</div>
            {meta ? <div className="flex flex-wrap items-center gap-2 font-mono text-[11px] text-muted-foreground">{meta}</div> : null}
        </div>
    );
}
