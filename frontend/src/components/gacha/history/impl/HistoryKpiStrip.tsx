import type { IClientGachaRecords, IGachaItem } from "#/lib/api/gacha";
import { formatNumber } from "#/lib/utils";

interface IHistoryKpiStripProps {
    records: IClientGachaRecords | null;
    isLoading: boolean;
}

interface IKpiProps {
    label: string;
    value: React.ReactNode;
    meta?: React.ReactNode;
    featured?: boolean;
}

function Kpi({ label, value, meta, featured }: IKpiProps) {
    return (
        <div
            className={`relative flex flex-col gap-2 px-4 py-3.5 not-last:border-r not-last:border-border sm:gap-2.5 sm:px-5 sm:py-4.5 max-[1100px]:not-last:border-r-0 max-[1100px]:not-last:border-b max-[1100px]:nth-[2n-1]:border-r max-[520px]:nth-[2n-1]:border-r-0 ${featured ? "bg-linear-[150deg] from-primary/8 to-transparent" : ""}`}
        >
            <div className="inline-flex items-center gap-2 font-mono text-[10.5px] font-medium uppercase leading-none tracking-[0.14em] text-muted-foreground">
                {featured ? (
                    <span className="relative inline-flex h-1.5 w-1.5 shrink-0" aria-hidden>
                        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-primary opacity-70" />
                        <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-primary" />
                    </span>
                ) : null}
                {label}
            </div>
            <div className={`flex items-baseline gap-1 font-sans text-[30px] font-bold leading-[0.95] tracking-[-0.04em] tabular-nums sm:text-[36px] sm:tracking-[-0.045em] lg:text-[42px] ${featured ? "text-[oklch(0.92_0.12_25)]" : "text-foreground"}`}>{value}</div>
            {meta ? <div className="flex flex-wrap items-center gap-2 font-mono text-[11px] text-muted-foreground">{meta}</div> : null}
        </div>
    );
}

function allItems(records: IClientGachaRecords): IGachaItem[] {
    return [...records.limited.records, ...records.linkage.records, ...records.regular.records, ...records.special.records];
}

export function HistoryKpiStrip({ records, isLoading }: IHistoryKpiStripProps) {
    if (isLoading) {
        return (
            <div className="grid overflow-hidden rounded-[14px] border border-border bg-card grid-cols-2 lg:grid-cols-4">
                {Array.from({ length: 4 }).map((_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                    <div key={i} className="flex flex-col gap-2 px-5 py-4.5 not-last:border-r not-last:border-border">
                        <div className="h-2.5 w-20 rounded-full bg-muted animate-pulse" />
                        <div className="h-9 w-28 rounded-md bg-muted animate-pulse" />
                    </div>
                ))}
            </div>
        );
    }

    if (!records) return null;

    const items = allItems(records);
    const total = items.length;
    const sixStars = items.filter((i) => i.star === "6").length;
    const fiveStars = items.filter((i) => i.star === "5").length;
    const sixRate = total > 0 ? (sixStars / total) * 100 : 0;
    const fiveRate = total > 0 ? (fiveStars / total) * 100 : 0;

    return (
        <div className="grid overflow-hidden rounded-[14px] border border-border bg-card grid-cols-2 lg:grid-cols-4">
            <Kpi
                featured
                label="Total pulls"
                value={formatNumber(total)}
                meta={
                    <span>
                        across {[records.limited, records.linkage, records.regular, records.special].filter((g) => g.total > 0).length} banner type
                        {[records.limited, records.linkage, records.regular, records.special].filter((g) => g.total > 0).length !== 1 ? "s" : ""}
                    </span>
                }
            />
            <Kpi
                label="6★ operators"
                value={
                    <>
                        {formatNumber(sixStars)}
                        <span className="self-end pb-1 font-mono text-[13px] font-medium text-muted-foreground ml-1">pulls</span>
                    </>
                }
                meta={<span>{sixRate.toFixed(2)}% rate</span>}
            />
            <Kpi
                label="5★ operators"
                value={
                    <>
                        {formatNumber(fiveStars)}
                        <span className="self-end pb-1 font-mono text-[13px] font-medium text-muted-foreground ml-1">pulls</span>
                    </>
                }
                meta={<span>{fiveRate.toFixed(2)}% rate</span>}
            />
            <Kpi label="Other (4★ / 3★)" value={formatNumber(total - sixStars - fiveStars)} meta={total > 0 ? <span>{(((total - sixStars - fiveStars) / total) * 100).toFixed(1)}% of pulls</span> : undefined} />
        </div>
    );
}
