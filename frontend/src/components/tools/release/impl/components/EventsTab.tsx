import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { Card } from "#/components/ui/card";
import { releaseEventsQueryOptions, releaseLagQueryOptions } from "#/lib/api/release";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ReleaseEvent } from "#/types/generated/ReleaseEvent";
import { useAutoTranslate } from "../autoTranslate";
import { formatDateRange, isPast, sortKey } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { useReleaseTagLabel } from "../labels";
import type { messages } from "./EventsTab.messages";
import { ModelSummary } from "./ModelSummary";
import { ResolutionBadge } from "./ResolutionBadge";
import { FarmStages } from "./ScheduleShared";
import { CnName, ListRow, ReleaseEmpty, ReleaseError, ReleaseLoading, RowImage, resolveName, Tag, ToggleField, useArt } from "./shared";

/** This tab renders its own chrome plus the date wording `helpers.ts` derives. */
type EventsT = TypedT<typeof messages & typeof helperMessages>;

interface IEventsTabProps {
    today: Date;
}

export function EventsTab({ today }: IEventsTabProps): React.ReactElement {
    const t: EventsT = useT("tools");
    const events = useQuery(releaseEventsQueryOptions());
    const lag = useQuery(releaseLagQueryOptions());
    const [showPast, setShowPast] = React.useState(false);
    const [stageOnly, setStageOnly] = React.useState(true);

    const model = events.data?.model ?? null;
    const all: ReleaseEvent[] = events.data?.events ?? [];

    const rows = React.useMemo(() => {
        return all.filter((e) => {
            if (stageOnly && !e.hasStage) return false;
            if (!showPast && isPast(sortKey(e.resolution, e.cnStart, model), today)) return false;
            return true;
        });
    }, [all, stageOnly, showPast, model, today]);

    if (events.isPending) return <ReleaseLoading />;
    if (events.isError) return <ReleaseError error={events.error} onRetry={() => events.refetch()} />;

    return (
        <div className="flex flex-col gap-3">
            <ModelSummary model={model} backtest={lag.data?.backtest} yearly={events.data?.yearly} />
            <div className="flex flex-wrap items-center gap-x-5 gap-y-2">
                <ToggleField id="events-stage-only" label={t("release.events.stageOnly")} checked={stageOnly} onChange={setStageOnly} />
                <ToggleField id="events-show-past" label={t("release.events.showPast")} checked={showPast} onChange={setShowPast} />
                <span className="font-medium font-mono text-[11px] text-muted-foreground">{t("release.events.count", { shown: rows.length, total: all.length })}</span>
            </div>
            {rows.length === 0 ? (
                <ReleaseEmpty title={t("release.events.empty.title")} description={all.length === 0 ? t("release.events.empty.none") : t("release.events.empty.filtered")} />
            ) : (
                <Card className="px-4 sm:px-5">
                    {rows.map((e) => (
                        <EventRow key={e.cnId} event={e} today={today} t={t} />
                    ))}
                </Card>
            )}
        </div>
    );
}

function EventRow({ event, today, t }: { event: ReleaseEvent; today: Date; t: EventsT }): React.ReactElement {
    const locale = useLocale();
    const tagLabel = useReleaseTagLabel();
    const autoOn = useAutoTranslate();
    const art = useArt(event.imagePath);
    const alt = resolveName(event.nameCn, event.nameEn, event.nameEnAuto, autoOn).text;
    return (
        <ListRow visual={art.src && <RowImage src={art.src} alt={alt} onError={art.onError} />} badge={<ResolutionBadge resolution={event.resolution} today={today} />}>
            <CnName cn={event.nameCn} en={event.nameEn} auto={event.nameEnAuto} primaryClassName={cn("font-sans text-[13.5px]", event.hasStage ? "font-semibold text-foreground" : "font-medium text-muted-foreground")}>
                <Tag>{tagLabel(event.activityType)}</Tag>
            </CnName>
            <div className="font-mono text-[11.5px] text-muted-foreground tabular-nums">
                <span className="mr-1 uppercase tracking-[0.06em]">{t("release.events.cn")}</span>
                {formatDateRange(event.cnStart, event.cnEnd, locale, t)}
            </div>
            <FarmStages stages={event.farmStages} compact />
        </ListRow>
    );
}
