import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { Card } from "#/components/ui/card";
import { releaseEventsQueryOptions, releaseLagQueryOptions } from "#/lib/api/release";
import { cn } from "#/lib/utils";
import type { ReleaseEvent } from "#/types/generated/ReleaseEvent";
import { useAutoTranslate } from "../autoTranslate";
import { formatDateRange, humanizeTag, isPast, sortKey } from "../helpers";
import { ModelSummary } from "./ModelSummary";
import { ResolutionBadge } from "./ResolutionBadge";
import { CnName, ListRow, ReleaseEmpty, ReleaseError, ReleaseLoading, RowImage, resolveName, Tag, ToggleField, useArt } from "./shared";

interface IEventsTabProps {
    today: Date;
}

export function EventsTab({ today }: IEventsTabProps): React.ReactElement {
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
                <ToggleField id="events-stage-only" label="Stage events only" checked={stageOnly} onChange={setStageOnly} />
                <ToggleField id="events-show-past" label="Show past" checked={showPast} onChange={setShowPast} />
                <span className="font-medium font-mono text-[11px] text-muted-foreground">
                    {rows.length} of {all.length}
                </span>
            </div>
            {rows.length === 0 ? (
                <ReleaseEmpty title="No events" description={all.length === 0 ? "The backend returned no CN activities." : "Every event is filtered out. Turn on Show past or turn off Stage events only."} />
            ) : (
                <Card className="px-4 sm:px-5">
                    {rows.map((e) => (
                        <EventRow key={e.cnId} event={e} today={today} />
                    ))}
                </Card>
            )}
        </div>
    );
}

function EventRow({ event, today }: { event: ReleaseEvent; today: Date }): React.ReactElement {
    const autoOn = useAutoTranslate();
    const art = useArt(event.imagePath);
    const alt = resolveName(event.nameCn, event.nameEn, event.nameEnAuto, autoOn).text;
    return (
        <ListRow visual={art.src && <RowImage src={art.src} alt={alt} onError={art.onError} />} badge={<ResolutionBadge resolution={event.resolution} today={today} />}>
            <CnName cn={event.nameCn} en={event.nameEn} auto={event.nameEnAuto} primaryClassName={cn("font-sans text-[13.5px]", event.hasStage ? "font-semibold text-foreground" : "font-medium text-muted-foreground")}>
                <Tag>{humanizeTag(event.activityType)}</Tag>
            </CnName>
            <div className="font-mono text-[11.5px] text-muted-foreground tabular-nums">
                <span className="mr-1 uppercase tracking-[0.06em]">CN</span>
                {formatDateRange(event.cnStart, event.cnEnd)}
            </div>
        </ListRow>
    );
}
