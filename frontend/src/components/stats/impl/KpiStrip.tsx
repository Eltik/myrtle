import type * as React from "react";

export interface IKpiCell {
    label: string;
    value: React.ReactNode;
    meta?: React.ReactNode;
}

export function KpiCell({ label, value, meta }: IKpiCell) {
    return (
        <div className="relative flex flex-col gap-2 not-last:border-border not-last:border-r px-4 py-3.5 max-[1180px]:not-last:border-border max-[1180px]:nth-[2n-1]:border-border max-[1180px]:nth-[2n-1]:border-r max-[1180px]:not-last:border-r-0 max-[520px]:nth-[2n-1]:border-r-0 max-[1180px]:not-last:border-b sm:gap-2.5 sm:px-5 sm:py-4.5 min-[520px]:max-[1180px]:last:odd:col-span-2 min-[520px]:max-[1180px]:last:odd:border-r-0">
            <div className="kicker inline-flex items-center gap-2 leading-none">{label}</div>
            <div className="flex items-baseline gap-1 font-bold font-sans text-[24px] text-foreground tabular-nums leading-none tracking-tight">{value}</div>
            {meta ? <div className="flex flex-wrap items-center gap-2 font-mono text-[11px] text-muted-foreground">{meta}</div> : null}
        </div>
    );
}
