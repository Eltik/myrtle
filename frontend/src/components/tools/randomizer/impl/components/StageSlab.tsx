import { MapPinned, RotateCcw, Skull } from "lucide-react";
import type React from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import type { IStage, IZone } from "#/types/stages";
import { type ActivityLookup, getActivityIdFromZoneId, getEventNameFromZoneId, getPermanentZoneName } from "../activity-lookup";
import { getZoneDisplayName } from "../utils";
import { SlabFrame } from "./SlabFrame";
import { StagePreview } from "./StagePreview";

interface IStageSlabProps {
    stage: IStage;
    zone: IZone | undefined;
    lookup: ActivityLookup;
    onReroll: () => void;
}

function resolveZoneLabel(stage: IStage, zone: IZone | undefined, lookup: ActivityLookup): { display: string; family: string | null } {
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
            <div className="flex items-start justify-between gap-3">
                <div className="flex min-w-0 flex-1 items-center gap-2 text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                    <MapPinned aria-hidden="true" className="h-3.5 w-3.5" />
                    <span>Target</span>
                </div>
                <Button onClick={onReroll} size="xs" variant="outline" aria-label="Reroll stage">
                    <RotateCcw aria-hidden="true" />
                    Reroll
                </Button>
            </div>

            <div className="mt-3 grid gap-4 md:grid-cols-[1fr_minmax(220px,320px)] md:items-end md:gap-6">
                <div className="min-w-0">
                    <div className="flex flex-wrap items-baseline gap-2">
                        <h2 className="m-0 font-bold text-[44px] leading-none tracking-tight text-foreground sm:text-[56px]">{stage.code}</h2>
                        {codeIsCM && (
                            <Badge variant="outline" className="font-mono uppercase tracking-[0.18em]">
                                CM
                            </Badge>
                        )}
                    </div>
                    {stage.name && <p className="mt-2 text-sm text-muted-foreground sm:text-base">{stage.name}</p>}
                    <p className="mt-1.5 font-mono text-[12px] uppercase tracking-[0.14em] text-muted-foreground/80">
                        {display}
                        {family && <span className="text-muted-foreground/50"> · {family}</span>}
                    </p>

                    <div className="mt-3 flex flex-wrap items-center gap-2">
                        <Badge variant="secondary" className="h-7 px-2.5 font-mono uppercase tracking-[0.12em]">
                            AP {stage.apCost}
                        </Badge>
                        {stage.dangerLevel && (
                            <Badge variant="secondary" className="h-7 px-2.5 font-mono uppercase tracking-[0.12em]">
                                LV {stage.dangerLevel}
                            </Badge>
                        )}
                        {stage.bossMark && (
                            <Badge variant="error" className="h-7 px-2.5 font-mono uppercase tracking-[0.12em]">
                                <Skull aria-hidden="true" className="h-3 w-3" />
                                Boss
                            </Badge>
                        )}
                    </div>
                </div>

                <StagePreview stage={stage} className="md:max-w-[320px]" />
            </div>
        </SlabFrame>
    );
}
