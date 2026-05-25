import type * as React from "react";

export interface IKpiCell {
    label: string;
    value: React.ReactNode;
    meta?: React.ReactNode;
    featured?: boolean;
}

interface IKpiStripProps {
    cells: IKpiCell[];
    columnsClass?: string;
}

export function KpiCell({ label, value, meta, featured }: IKpiCell) {
    return (
        <div
            className={`relative flex flex-col gap-2 not-last:border-border not-last:border-r px-4 py-3.5 max-[1180px]:not-last:border-border max-[1180px]:nth-[2n-1]:border-border max-[1180px]:nth-[2n-1]:border-r max-[1180px]:not-last:border-r-0 max-[520px]:nth-[2n-1]:border-r-0 max-[1180px]:not-last:border-b sm:gap-2.5 sm:px-5 sm:py-4.5 ${featured ? "bg-linear-[150deg] from-primary/8 to-transparent" : ""}`}
        >
            <div className="inline-flex items-center gap-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">
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

export function KpiStrip({ cells, columnsClass = "grid-cols-[1.4fr_1fr_1fr]" }: IKpiStripProps) {
    return (
        <div className={`grid ${columnsClass} overflow-hidden rounded-[14px] border border-border bg-card max-[1180px]:grid-cols-2! max-[520px]:grid-cols-1!`}>
            {cells.map((c) => (
                <KpiCell key={c.label} {...c} />
            ))}
        </div>
    );
}
