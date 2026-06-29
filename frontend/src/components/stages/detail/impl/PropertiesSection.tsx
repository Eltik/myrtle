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
                        <FlagRow label="Predefined Squad" on={stage.isPredefined} hint="The stage hands you a fixed roster of operators to clear it with, instead of letting you bring your own." />
                        {opts?.isTrainingLevel != null && <FlagRow label="Training Level" on={opts.isTrainingLevel} hint="A tutorial / practice stage that teaches a mechanic and does not count toward normal progression." />}
                        {opts?.steeringEnabled != null && <FlagRow label="Steering Enabled" on={opts.steeringEnabled} hint="A pathfinding flag: when on, enemies steer smoothly around each other and corners rather than snapping tile-to-tile. It rarely affects strategy." />}
                        {opts?.isPredefinedCardsSelectable != null && <FlagRow label="Cards Selectable" on={opts.isPredefinedCardsSelectable} hint="In a predefined-squad stage, whether you may pick which of the provided operators to deploy. When off, the loadout is locked." />}
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
