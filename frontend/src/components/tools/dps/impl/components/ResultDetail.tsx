import type * as React from "react";
import { formatLargeNumber } from "#/components/tools/shared/constants";
import { Badge } from "#/components/ui/badge";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IDpsCalculateResponse } from "#/lib/api/dps";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { SkillCategory } from "#/types/generated/SkillCategory";
import type { YMetric } from "../types";
import type { messages } from "./ResultDetail.messages";

type DetailT = TypedT<typeof messages>;

/** Localised skill-shape label. */
export function useCategoryLabel(): (category: SkillCategory) => string {
    const t: DetailT = useT("tools");
    return (category) => {
        switch (category) {
            case "sustained":
                return t("dps.category.sustained");
            case "burst":
                return t("dps.category.burst");
            case "instant":
                return t("dps.category.instant");
        }
    };
}

/** Seconds at the precision the engine's timings carry meaning to a reader. */
export function formatSeconds(s: number): string {
    return (Math.round(s * 10) / 10).toString();
}

/**
 * The new outputs of one card's result: skill shape and timings (cycle model v2), the main /
 * secondary split (secondary targets on), and caveat badges for the approximations the
 * backend flags. Renders nothing for a plain legacy result.
 */
export function ResultDetail({ data, metric }: { data: IDpsCalculateResponse | undefined; metric: YMetric }): React.ReactElement | null {
    const t: DetailT = useT("tools");
    const locale = useLocale();
    const categoryLabel = useCategoryLabel();
    if (!data) return null;
    const notModelled = data.spRecovery === "defensive";
    const items: React.ReactNode[] = [];
    if (data.category)
        items.push(
            <Badge key="cat" variant="outline" size="sm">
                {categoryLabel(data.category)}
            </Badge>,
        );
    if (data.startupTime !== undefined) items.push(<span key="start">{t("dps.detail.startup", { seconds: formatSeconds(data.startupTime) })}</span>);
    if (data.cycleTime !== undefined) items.push(<span key="cycle">{t("dps.detail.cycle", { seconds: formatSeconds(data.cycleTime) })}</span>);
    if (data.primary && data.secondary) items.push(<span key="split">{t("dps.detail.split", { primary: formatLargeNumber(data.primary[metric], locale), secondary: formatLargeNumber(data.secondary[metric], locale) })}</span>);
    if (notModelled) items.push(<Caveat key="sp" label={t("dps.detail.spNotModelled")} hint={t("dps.detail.spNotModelled.hint")} />);
    if (data.offSkillSource === "normalAttack") items.push(<Caveat key="na" label={t("dps.detail.normalAttack")} hint={t("dps.detail.normalAttack.hint")} />);
    if (data.approximateTypeSplit) items.push(<Caveat key="split-approx" label={t("dps.detail.approxSplit")} hint={t("dps.detail.approxSplit.hint")} />);
    if (data.secondaryClamped) items.push(<Caveat key="clamp" label={t("dps.detail.clamped")} hint={t("dps.detail.clamped.hint")} />);
    if (items.length === 0) return null;
    return <div className="mt-2 flex flex-wrap items-center gap-x-2.5 gap-y-1 border-border/60 border-t pt-1.5 text-[11px] text-muted-foreground tabular-nums">{items}</div>;
}

export function Caveat({ label, hint }: { label: string; hint: string }): React.ReactElement {
    return (
        <Tooltip>
            <TooltipTrigger render={<Badge variant="warning" size="sm" className="cursor-help" />}>{label}</TooltipTrigger>
            <TooltipPopup className="max-w-64">{hint}</TooltipPopup>
        </Tooltip>
    );
}
