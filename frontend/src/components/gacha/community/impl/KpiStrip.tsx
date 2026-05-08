import type { IGachaEnhancedStats } from "#/lib/api/gacha";
import { formatNumber, formatNumberCompact } from "#/lib/utils";

interface IKpiStripProps {
    data: IGachaEnhancedStats | null;
}

interface IKpiProps {
    label: string;
    value: React.ReactNode;
    meta: React.ReactNode;
    featured?: boolean;
}

function Kpi({ label, value, meta, featured }: IKpiProps) {
    return (
        <div
            className={`relative flex flex-col gap-2 px-4 py-3.5 not-last:border-r not-last:border-border sm:gap-2.5 sm:px-5 sm:py-4.5 max-[1180px]:not-last:border-r-0 max-[1180px]:not-last:border-b max-[1180px]:not-last:border-border max-[1180px]:nth-[2n-1]:border-r max-[1180px]:nth-[2n-1]:border-border max-[520px]:nth-[2n-1]:border-r-0 ${featured ? "bg-linear-[150deg] from-primary/8 to-transparent" : ""}`}
        >
            <div className="inline-flex items-center gap-2 font-mono text-[10.5px] font-medium uppercase leading-none tracking-[0.14em] text-muted-foreground">
                {featured ? <span className="block h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_0_3px_oklch(0.58_0.22_25/0.18)]" aria-hidden /> : null}
                {label}
            </div>
            <div className={`flex items-baseline gap-1 font-sans text-[30px] font-bold leading-[0.95] tracking-[-0.04em] tabular-nums sm:text-[36px] sm:tracking-[-0.045em] lg:text-[42px] ${featured ? "text-[oklch(0.92_0.12_25)]" : "text-foreground"}`}>{value}</div>
            <div className="flex flex-wrap items-center gap-2 font-mono text-[11px] text-muted-foreground">{meta}</div>
        </div>
    );
}

const SKELETON = <span className="text-muted-foreground/60">-</span>;

export function KpiStrip({ data }: IKpiStripProps) {
    const cs = data?.collectiveStats;
    const pr = data?.pullRates;

    return (
        <div className="grid overflow-hidden rounded-[14px] border border-border bg-card grid-cols-[1.4fr_1fr_1fr_1fr] max-[1180px]:grid-cols-2 max-[520px]:grid-cols-1">
            <Kpi
                featured
                label="6★ rate · all-time"
                value={
                    pr ? (
                        <>
                            {(pr.sixStarRate * 100).toFixed(2)}
                            <span className="self-end pb-1 font-mono text-[13px] font-medium text-muted-foreground">%</span>
                        </>
                    ) : (
                        SKELETON
                    )
                }
                meta={
                    cs ? (
                        <span>
                            {formatNumber(cs.totalSixStars)} of {formatNumber(cs.totalPulls)} pulls
                        </span>
                    ) : (
                        <span>—</span>
                    )
                }
            />
            <Kpi
                label="5★ rate · all-time"
                value={
                    pr ? (
                        <>
                            {(pr.fiveStarRate * 100).toFixed(2)}
                            <span className="self-end pb-1 font-mono text-[13px] font-medium text-muted-foreground">%</span>
                        </>
                    ) : (
                        SKELETON
                    )
                }
                meta={cs ? <span>{formatNumber(cs.totalFiveStars)} pulls</span> : <span>—</span>}
            />
            <Kpi label="Avg pulls / 6★" value={data ? data.averagePullsToSixStar.toFixed(1) : SKELETON} meta={data ? <span>avg pulls / 5★ · {data.averagePullsToFiveStar.toFixed(1)}</span> : <span>—</span>} />
            <Kpi label="Total pulls" value={cs ? formatNumberCompact(cs.totalPulls) : SKELETON} meta={cs ? <span>{formatNumberCompact(cs.totalUsers)} contributing doctors</span> : <span>—</span>} />
        </div>
    );
}
