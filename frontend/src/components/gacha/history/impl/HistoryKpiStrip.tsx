import { Skeleton } from "#/components/ui/skeleton";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IClientGachaRecords, IGachaItem } from "#/lib/api/gacha";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Kpi, KpiGrid } from "../../KpiGrid";
import type { messages } from "./HistoryKpiStrip.messages";

interface IHistoryKpiStripProps {
    records: IClientGachaRecords | null;
    isLoading: boolean;
}

function allItems(records: IClientGachaRecords): IGachaItem[] {
    return [...records.limited.records, ...records.linkage.records, ...records.regular.records, ...records.special.records, ...records.boot.records];
}

// Headhunting cost per pull; free/discounted pulls make the total an estimate.
const ORUNDUM_PER_PULL = 600;
// Starter Headhunting's price per pull: `GachaPrice` of every `BOOT_` pool in gacha_table's `NewbeeGachaPoolClient`.
const ORUNDUM_PER_STARTER_PULL = 380;

const COLUMNS = "grid-cols-2 lg:grid-cols-4";

export function HistoryKpiStrip({ records, isLoading }: IHistoryKpiStripProps) {
    const t: TypedT<typeof messages> = useT("gacha");
    const f = useFormatters();

    if (isLoading) {
        return (
            <KpiGrid columnsClass={COLUMNS}>
                {Array.from({ length: 4 }).map((_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                    <div key={i} className="flex flex-col gap-2 bg-card px-5 py-4.5">
                        <Skeleton className="h-2.5 w-20 rounded-full" />
                        <Skeleton className="h-9 w-28 rounded-md" />
                    </div>
                ))}
            </KpiGrid>
        );
    }

    if (!records) return null;

    const items = allItems(records);
    const total = items.length;
    const bannerTypeCount = [records.limited, records.linkage, records.regular, records.special, records.boot].filter((g) => g.total > 0).length;
    const starterPulls = records.boot.total;
    const fullPricePulls = total - starterPulls;
    const orundum = fullPricePulls * ORUNDUM_PER_PULL + starterPulls * ORUNDUM_PER_STARTER_PULL;
    const orundumTitle = starterPulls > 0 ? t("history.kpi.orundum.titleStarter", { pulls: f.number(fullPricePulls), rate: ORUNDUM_PER_PULL, starterPulls: f.number(starterPulls), starterRate: ORUNDUM_PER_STARTER_PULL }) : t("history.kpi.orundum.title", { pulls: f.number(total), rate: ORUNDUM_PER_PULL });
    const sixStars = items.filter((i) => i.star === "6").length;
    const fiveStars = items.filter((i) => i.star === "5").length;
    const sixRate = total > 0 ? (sixStars / total) * 100 : 0;
    const fiveRate = total > 0 ? (fiveStars / total) * 100 : 0;

    return (
        <KpiGrid columnsClass={COLUMNS}>
            <Kpi
                featured
                label={t("history.kpi.totalPulls")}
                value={f.number(total)}
                meta={
                    <>
                        <span>{t("history.kpi.bannerTypes", { count: bannerTypeCount })}</span>
                        {total > 0 ? (
                            <Tooltip>
                                <TooltipTrigger render={<button type="button" className="cursor-help rounded-sm underline decoration-dotted underline-offset-2 outline-none focus-visible:ring-2 focus-visible:ring-ring" />}>{t("history.kpi.orundum", { count: f.number(orundum) })}</TooltipTrigger>
                                <TooltipPopup className="max-w-72">{orundumTitle}</TooltipPopup>
                            </Tooltip>
                        ) : null}
                    </>
                }
            />
            <Kpi
                label={t("history.kpi.sixStars")}
                value={
                    <>
                        {f.number(sixStars)}
                        <span className="ml-1 self-end pb-1 font-medium font-mono text-[13px] text-muted-foreground">{t("history.kpi.pullsUnit")}</span>
                    </>
                }
                meta={<span>{t("history.kpi.rate", { value: sixRate.toFixed(2) })}</span>}
            />
            <Kpi
                label={t("history.kpi.fiveStars")}
                value={
                    <>
                        {f.number(fiveStars)}
                        <span className="ml-1 self-end pb-1 font-medium font-mono text-[13px] text-muted-foreground">{t("history.kpi.pullsUnit")}</span>
                    </>
                }
                meta={<span>{t("history.kpi.rate", { value: fiveRate.toFixed(2) })}</span>}
            />
            <Kpi label={t("history.kpi.other")} value={f.number(total - sixStars - fiveStars)} meta={total > 0 ? <span>{t("history.kpi.shareOfPulls", { value: (((total - sixStars - fiveStars) / total) * 100).toFixed(1) })}</span> : undefined} />
        </KpiGrid>
    );
}
