import type { IGachaEnhancedStats } from "#/lib/api/gacha";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./KpiStrip.messages";

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
            <div className="flex flex-wrap items-center gap-2 font-mono text-[11px] text-muted-foreground">{meta}</div>
        </div>
    );
}

const SKELETON = <span className="text-muted-foreground/60">-</span>;

export function KpiStrip({ data }: IKpiStripProps) {
    const t: TypedT<typeof messages> = useT("gacha");
    const f = useFormatters();
    const cs = data?.collectiveStats;
    const pr = data?.pullRates;

    return (
        <div className="grid grid-cols-[1.4fr_1fr_1fr_1fr] overflow-hidden rounded-[14px] border border-border bg-card max-[1180px]:grid-cols-2 max-[520px]:grid-cols-1">
            <Kpi
                featured
                label={t("community.kpi.sixStarRate")}
                value={
                    pr ? (
                        <>
                            {(pr.sixStarRate * 100).toFixed(2)}
                            <span className="self-end pb-1 font-medium font-mono text-[13px] text-muted-foreground">%</span>
                        </>
                    ) : (
                        SKELETON
                    )
                }
                meta={cs ? <span>{t("community.kpi.sixStarRate.meta", { six: f.number(cs.totalSixStars), total: f.number(cs.totalPulls) })}</span> : <span>-</span>}
            />
            <Kpi
                label={t("community.kpi.fiveStarRate")}
                value={
                    pr ? (
                        <>
                            {(pr.fiveStarRate * 100).toFixed(2)}
                            <span className="self-end pb-1 font-medium font-mono text-[13px] text-muted-foreground">%</span>
                        </>
                    ) : (
                        SKELETON
                    )
                }
                meta={cs ? <span>{t("community.kpi.fiveStarRate.meta", { count: f.number(cs.totalFiveStars) })}</span> : <span>-</span>}
            />
            <Kpi label={t("community.kpi.avgPullsSixStar")} value={data ? data.averagePullsToSixStar.toFixed(1) : SKELETON} meta={data ? <span>{t("community.kpi.avgPullsFiveStar.meta", { value: data.averagePullsToFiveStar.toFixed(1) })}</span> : <span>-</span>} />
            <Kpi label={t("community.kpi.totalPulls")} value={cs ? f.compact(cs.totalPulls) : SKELETON} meta={cs ? <span>{t("community.kpi.totalPulls.meta", { count: f.compact(cs.totalUsers) })}</span> : <span>-</span>} />
        </div>
    );
}
