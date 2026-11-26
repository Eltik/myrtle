import type { IGachaEnhancedStats } from "#/lib/api/gacha";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Kpi, KpiGrid } from "../../KpiGrid";
import type { messages } from "./KpiStrip.messages";

interface IKpiStripProps {
    data: IGachaEnhancedStats | null;
}

const SKELETON = <span className="text-subtle-foreground">-</span>;

export function KpiStrip({ data }: IKpiStripProps) {
    const t: TypedT<typeof messages> = useT("gacha");
    const f = useFormatters();
    const cs = data?.collectiveStats;
    const pr = data?.pullRates;

    return (
        <KpiGrid columnsClass="grid-cols-[1.4fr_1fr_1fr_1fr] max-[1180px]:grid-cols-2 max-[520px]:grid-cols-1">
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
        </KpiGrid>
    );
}
