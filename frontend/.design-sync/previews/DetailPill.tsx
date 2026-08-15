import { Skull } from "lucide-react";
import { DetailKicker, DetailPill } from "frontend";

export const Tones = () => (
    <div className="flex flex-wrap items-center gap-2">
        <DetailPill>Main Theme</DetailPill>
        <DetailPill tone="primary">Predefined Squad</DetailPill>
        <DetailPill tone="warning">Challenge Mode</DetailPill>
        <DetailPill tone="danger">
            <Skull className="h-3 w-3" /> Boss
        </DetailPill>
    </div>
);

/** StageHeader stacks the boss mark and the difficulty above the stage name. */
export const InStageHeader = () => (
    <header className="max-w-xl">
        <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-md bg-primary px-2.5 py-1 font-bold font-mono text-[13px] text-primary-foreground leading-none tracking-wide">4-10</span>
            <DetailPill tone="danger">
                <Skull className="h-3 w-3" /> Boss
            </DetailPill>
            <DetailPill tone="warning">Challenge Mode</DetailPill>
        </div>
        <h1 className="mt-3 text-balance font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight">Extinguished Flames</h1>
        <div className="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
            <span>Main Theme</span>
            <span aria-hidden="true">·</span>
            <span className="text-foreground normal-case tracking-normal">Burning Run</span>
        </div>
    </header>
);

/** SpawnSchedule lists the level's hidden route groups as muted pills. */
export const HiddenRouteGroups = () => (
    <div className="flex max-w-xl flex-wrap items-center gap-2.5 rounded-[14px] border border-border bg-card px-4 py-3">
        <DetailKicker>Spawn Schedule</DetailKicker>
        <span className="h-px flex-1 bg-border" />
        <span className="font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.12em]">Hidden Routes</span>
        <DetailPill>frostnova_phase2</DetailPill>
        <DetailPill>reinforce_a</DetailPill>
    </div>
);
