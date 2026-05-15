import { MapPinned, RotateCcw, Skull } from "lucide-react";
import type React from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import type { IStage, IZone } from "#/types/stages";
import { getActivityIdFromZoneId, getEventNameFromZoneId, getPermanentZoneName, type IActivityLookup } from "../activity-lookup";
import { getZoneDisplayName } from "../utils";
import { SlabFrame } from "./SlabFrame";
import { StagePreview } from "./StagePreview";

interface IStageSlabProps {
    stage: IStage;
    zone: IZone | undefined;
    lookup: IActivityLookup;
    onReroll: () => void;
}

function resolveZoneLabel(stage: IStage, zone: IZone | undefined, lookup: IActivityLookup): { display: string; family: string | null } {
    const display = getZoneDisplayName(zone, stage.zoneId);
    const permanent = getPermanentZoneName(stage.zoneId, lookup);
    if (permanent) return { display: permanent, family: display };
    const activityId = getActivityIdFromZoneId(stage.zoneId);
    if (activityId) {
        const event = getEventNameFromZoneId(stage.zoneId, lookup);
        if (event !== stage.zoneId) return { display: event, family: display };
    }
    return { display, family: null };
}

export function StageSlab({ stage, zone, lookup, onReroll }: IStageSlabProps): React.ReactElement {
    const { display, family } = resolveZoneLabel(stage, zone, lookup);
    const codeIsCM = stage.difficulty === "FOUR_STAR";

    return (
        <SlabFrame index="01" kicker="STAGE" accent="primary">
            <div className="flex items-start justify-between gap-2 sm:gap-3">
                <div className="flex min-w-0 flex-1 items-center gap-1.5 text-[10.5px] text-muted-foreground uppercase tracking-[0.18em] sm:gap-2 sm:text-[11px]">
                    <MapPinned aria-hidden="true" className="size-3.5 shrink-0" />
                    <span className="truncate">Target</span>
                </div>
                <Button onClick={onReroll} size="xs" variant="outline" aria-label="Reroll stage" className="shrink-0">
                    <RotateCcw aria-hidden="true" />
                    Reroll
                </Button>
            </div>

            <div className="mt-3 grid gap-3 sm:gap-4 md:grid-cols-[1fr_minmax(220px,320px)] md:items-end md:gap-6">
                <div className="min-w-0">
                    <div className="flex flex-wrap items-baseline gap-1.5 sm:gap-2">
                        <h2 className="wrap-break-word m-0 font-bold text-[32px] text-foreground leading-none tracking-tight sm:text-[44px] md:text-[56px]">{stage.code}</h2>
                        {codeIsCM && (
                            <Badge variant="outline" className="font-mono uppercase tracking-[0.18em]">
                                CM
                            </Badge>
                        )}
                    </div>
                    {stage.name && <p className="wrap-break-word mt-1.5 text-[13px] text-muted-foreground sm:mt-2 sm:text-sm md:text-base">{stage.name}</p>}
                    <p className="wrap-break-word mt-1 font-mono text-[11px] text-muted-foreground/80 uppercase tracking-[0.12em] sm:mt-1.5 sm:text-[12px] sm:tracking-[0.14em]">
                        {display}
                        {family && <span className="text-muted-foreground/50"> · {family}</span>}
                    </p>

                    <div className="mt-2.5 flex flex-wrap items-center gap-1.5 sm:mt-3 sm:gap-2">
                        <Badge variant="secondary" className="h-6 px-2 font-mono uppercase tracking-[0.12em] sm:h-7 sm:px-2.5">
                            AP {stage.apCost}
                        </Badge>
                        {stage.dangerLevel && (
                            <Badge variant="secondary" className="h-6 px-2 font-mono uppercase tracking-[0.12em] sm:h-7 sm:px-2.5">
                                LV {stage.dangerLevel}
                            </Badge>
                        )}
                        {stage.bossMark && (
                            <Badge variant="error" className="h-6 px-2 font-mono uppercase tracking-[0.12em] sm:h-7 sm:px-2.5">
                                <Skull aria-hidden="true" className="size-3" />
                                Boss
                            </Badge>
                        )}
                    </div>
                </div>

                <StagePreview stage={stage} className="mx-auto w-full max-w-md md:max-w-[320px]" />
            </div>
        </SlabFrame>
    );
}
