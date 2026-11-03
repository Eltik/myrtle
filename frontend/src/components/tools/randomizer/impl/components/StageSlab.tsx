import { MapPinned, RotateCcw, Skull } from "lucide-react";
import type React from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Kicker } from "#/components/ui/kicker";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IStage, IZone } from "#/types/stages";
import { getActivityIdFromZoneId, getEventNameFromZoneId, getPermanentZoneName, type IActivityLookup } from "../activity-lookup";
import { getZoneDisplayName } from "../utils";
import { SlabFrame } from "./SlabFrame";
import { StagePreview } from "./StagePreview";
import type { messages } from "./StageSlab.messages";

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
    const t: TypedT<typeof messages> = useT("tools");
    const { display, family } = resolveZoneLabel(stage, zone, lookup);
    const codeIsCM = stage.difficulty === "FOUR_STAR";

    return (
        <SlabFrame index="01" kicker={t("randomizer.stage.kicker")} accent="primary">
            <div className="flex items-start justify-between gap-2 sm:gap-3">
                <Kicker className="flex min-w-0 flex-1 items-center gap-1.5 sm:gap-2">
                    <MapPinned aria-hidden="true" className="size-3.5 shrink-0" />
                    <span className="truncate">{t("randomizer.stage.target")}</span>
                </Kicker>
                <Button onClick={onReroll} size="xs" variant="outline" aria-label={t("randomizer.stage.rerollAria")} className="shrink-0">
                    <RotateCcw aria-hidden="true" />
                    {t("randomizer.stage.reroll")}
                </Button>
            </div>

            <div className="mt-3 grid gap-3 sm:gap-4 md:grid-cols-[1fr_minmax(220px,320px)] md:items-end md:gap-6">
                <div className="min-w-0">
                    <div className="flex flex-wrap items-baseline gap-1.5 sm:gap-2">
                        <h2 className="wrap-break-word m-0 font-bold text-[32px] text-foreground leading-none tracking-tight sm:text-[44px] md:text-[56px]">{stage.code}</h2>
                        {codeIsCM && (
                            <Badge variant="outline" className="font-mono">
                                CM
                            </Badge>
                        )}
                    </div>
                    {stage.name && <p className="wrap-break-word mt-1.5 text-[13px] text-muted-foreground sm:mt-2 sm:text-sm md:text-base">{stage.name}</p>}
                    <p className="wrap-break-word mt-1 font-mono text-[11px] text-subtle-foreground sm:mt-1.5 sm:text-[12px]">
                        {display}
                        {family && <span className="text-subtle-foreground">{t("randomizer.stage.family", { family })}</span>}
                    </p>

                    <div className="mt-2.5 flex flex-wrap items-center gap-1.5 sm:mt-3 sm:gap-2">
                        <Badge variant="secondary" className="h-6 px-2 font-mono sm:h-7 sm:px-2.5">
                            {t("randomizer.stage.ap", { cost: stage.apCost })}
                        </Badge>
                        {stage.dangerLevel && (
                            <Badge variant="secondary" className="h-6 px-2 font-mono sm:h-7 sm:px-2.5">
                                {t("randomizer.stage.dangerLevel", { level: stage.dangerLevel })}
                            </Badge>
                        )}
                        {stage.bossMark && (
                            <Badge variant="error" className="h-6 px-2 font-mono sm:h-7 sm:px-2.5">
                                <Skull aria-hidden="true" className="size-3" />
                                {t("randomizer.stage.boss")}
                            </Badge>
                        )}
                    </div>
                </div>

                <StagePreview stage={stage} className="mx-auto w-full max-w-md md:max-w-80" />
            </div>
        </SlabFrame>
    );
}
