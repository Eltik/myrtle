import type { ILevel } from "#/lib/api/level";
import type { IStage } from "#/types/stages";
import { FlagRow, Meta, SectionHead } from "./primitives";

export function PropertiesSection({ stage, level }: { stage: IStage; level: ILevel | null }) {
    const opts = level?.options;
    return (
        <>
            <section>
                <SectionHead>Properties</SectionHead>
                <div className="flex flex-col gap-2.5">
                    <div className="flex flex-col rounded-[10px] border border-border bg-card px-3.5 py-1.5">
                        <FlagRow label="Can Practice" on={stage.canPractice} />
                        <FlagRow label="Battle Replay" on={stage.canBattleReplay} />
                        <FlagRow label="Auto-Deploy (Multi)" on={stage.canMultipleBattle} />
                        <FlagRow label="Story Only" on={stage.isStoryOnly} />
                        <FlagRow label="Predefined Squad" on={stage.isPredefined} />
                        {opts?.isTrainingLevel != null && <FlagRow label="Training Level" on={opts.isTrainingLevel} />}
                        {opts?.steeringEnabled != null && <FlagRow label="Steering Enabled" on={opts.steeringEnabled} />}
                        {opts?.isPredefinedCardsSelectable != null && <FlagRow label="Cards Selectable" on={opts.isPredefinedCardsSelectable} />}
                    </div>
                </div>
            </section>

            <section>
                <SectionHead>Identifiers</SectionHead>
                <div className="grid grid-cols-2 gap-2">
                    <Meta label="Stage ID" value={stage.stageId} />
                    <Meta label="Level ID" value={stage.levelId ?? "-"} />
                    <Meta label="Zone ID" value={stage.zoneId} />
                    {stage.hardStagedId && <Meta label="Challenge ID" value={stage.hardStagedId} />}
                </div>
            </section>
        </>
    );
}
