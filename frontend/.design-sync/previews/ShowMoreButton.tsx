import { SectionHeader, ShowMoreButton } from "frontend";
import type { ReactNode } from "react";

const STAGE = "oklch(0.62 0.20 255)";
const MEDAL = "oklch(0.62 0.22 295)";

const noop = () => undefined;

const Panel = ({ children }: { children: ReactNode }) => <div className="flex w-full max-w-xl flex-col gap-5 rounded-xl border border-border bg-card px-4 pt-4 pb-4 sm:px-5 sm:pb-5">{children}</div>;

const Kicker = ({ children }: { children: ReactNode }) => <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{children}</span>;

const StageChip = ({ code, name }: { code: string; name: string }) => (
    <span className="flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/15 px-2 py-1">
        <span aria-hidden className="size-1.5 shrink-0 rounded-full bg-rose-500/65" />
        <span className="max-w-[22ch] truncate font-mono font-semibold text-[10px] tabular-nums" style={{ color: `color-mix(in oklch, ${STAGE} 55%, var(--foreground))` }}>
            {code}
        </span>
        <span className="hidden max-w-[16ch] truncate text-[11px] text-muted-foreground sm:inline">{name}</span>
    </span>
);

export const TruncatedStageList = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={STAGE} count="128 / 214 3★" title="Event" />
            <div className="flex flex-col gap-2">
                <div className="flex items-center justify-between">
                    <Kicker>Missing</Kicker>
                    <span className="font-mono text-[10px] text-muted-foreground tabular-nums">42 total</span>
                </div>
                <div className="flex flex-wrap gap-1.5">
                    <StageChip code="IW-8" name="Ashen Vigil" />
                    <StageChip code="IW-EX-7" name="Last Rite" />
                    <StageChip code="LT-5" name="Lone Trail" />
                    <StageChip code="SN-9" name="Stultifera Navis" />
                    <StageChip code="DV-EX-8" name="Dorothy's Vision" />
                    <StageChip code="WD-8" name="Where Vernal Winds" />
                </div>
                <ShowMoreButton label="Show 36 more" onClick={noop} />
            </div>
        </div>
    </Panel>
);

const MedalRow = ({ name, requirement }: { name: string; requirement: string }) => (
    <div className="flex items-center justify-between gap-3 rounded-md border border-border/40 bg-muted/15 px-2 py-1.5">
        <span className="truncate font-semibold text-[11.5px] leading-tight">{name}</span>
        <span className="shrink-0 truncate text-[10.5px] text-muted-foreground">{requirement}</span>
    </div>
);

export const ExpandedList = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={MEDAL} count="9 missing" title="Medals · Operational Records" />
            <div className="flex flex-col gap-2">
                <div className="flex items-center justify-between gap-3">
                    <Kicker>Showing all</Kicker>
                    <ShowMoreButton label="Show 118 unobtainable" onClick={noop} />
                </div>
                <div className="flex flex-col gap-1.5">
                    <MedalRow name="Deepness" requirement="Clear 12-17 with a squad of 6 or fewer." />
                    <MedalRow name="Chronicles" requirement="Complete every Operational Record." />
                    <MedalRow name="Ashen Vigil" requirement="Clear IW-EX-8 on Challenge Mode." />
                    <MedalRow name="Wine of Lungmen" requirement="3★ every stage in Chapter 8." />
                    <MedalRow name="Silent Night" requirement="Clear WD-EX-7 without deploying a Caster." />
                </div>
                <ShowMoreButton label="Show less" onClick={noop} />
            </div>
        </div>
    </Panel>
);

const OperatorRow = ({ name, gap }: { name: string; gap: string }) => (
    <div className="flex items-center justify-between gap-3 rounded-md border border-border/40 bg-muted/15 px-2 py-1.5">
        <span className="truncate font-semibold text-[11.5px] leading-tight">{name}</span>
        <span className="shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums">{gap}</span>
    </div>
);

export const ClearFilter = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent="oklch(0.74 0.17 75)" count="212 total · +4.7 to overall grade" title="Operators below milestone" />
            <ShowMoreButton label="Clear filter (38 shown)" onClick={noop} />
            <div className="flex flex-col gap-1.5">
                <OperatorRow gap="2 skills to M3" name="Mlynar" />
                <OperatorRow gap="1 skill to M3" name="Texas the Omertosa" />
                <OperatorRow gap="3 skills to M3" name="Ch'en the Holungday" />
                <OperatorRow gap="1 skill to M3" name="Surtr" />
            </div>
            <ShowMoreButton label="Show 34 more" onClick={noop} />
        </div>
    </Panel>
);
