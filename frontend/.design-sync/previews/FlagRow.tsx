import { FlagRow, StagesDetailSectionHead } from "frontend";

/** PropertiesSection's panel for 4-10 — Extinguished Flames. */
export const StagePropertiesPanel = () => (
    <section className="max-w-md">
        <StagesDetailSectionHead>Properties</StagesDetailSectionHead>
        <div className="flex flex-col rounded-[10px] border border-border bg-card px-3.5 py-1.5">
            <FlagRow label="Can Practice" on={true} />
            <FlagRow label="Battle Replay" on={true} />
            <FlagRow label="Auto-Deploy (Multi)" on={true} />
            <FlagRow label="Story Only" on={false} />
            <FlagRow label="Predefined Squad" on={false} hint="The stage hands you a fixed roster of operators to clear it with, instead of letting you bring your own." />
            <FlagRow label="Training Level" on={false} hint="A tutorial / practice stage that teaches a mechanic and does not count toward normal progression." />
            <FlagRow label="Steering Enabled" on={true} hint="A pathfinding flag: when on, enemies steer smoothly around each other and corners rather than snapping tile-to-tile." />
            <FlagRow label="Cards Selectable" on={false} hint="In a predefined-squad stage, whether you may pick which of the provided operators to deploy. When off, the loadout is locked." />
        </div>
    </section>
);

/** A story-only tutorial stage flips almost every flag the other way. */
export const TrainingStage = () => (
    <section className="max-w-md">
        <StagesDetailSectionHead>Properties · TR-1</StagesDetailSectionHead>
        <div className="flex flex-col rounded-[10px] border border-border bg-card px-3.5 py-1.5">
            <FlagRow label="Can Practice" on={false} />
            <FlagRow label="Battle Replay" on={false} />
            <FlagRow label="Auto-Deploy (Multi)" on={false} />
            <FlagRow label="Story Only" on={true} />
            <FlagRow label="Predefined Squad" on={true} hint="The stage hands you a fixed roster of operators to clear it with, instead of letting you bring your own." />
            <FlagRow label="Training Level" on={true} hint="A tutorial / practice stage that teaches a mechanic and does not count toward normal progression." />
        </div>
    </section>
);

export const SingleRow = () => (
    <div className="flex max-w-sm flex-col rounded-[10px] border border-border bg-card px-3.5 py-1.5">
        <FlagRow label="Predefined Squad" on={true} hint="The stage hands you a fixed roster of operators to clear it with, instead of letting you bring your own." />
    </div>
);
