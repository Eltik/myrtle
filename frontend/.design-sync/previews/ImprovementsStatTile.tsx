import { ImprovementsStatTile, SectionHeader } from "frontend";
import type { ReactNode } from "react";

const OPERATOR = "oklch(0.74 0.17 75)";

const noop = () => undefined;

const Panel = ({ children }: { children: ReactNode }) => <div className="flex w-full max-w-2xl flex-col gap-4 rounded-xl border border-border bg-card px-4 pt-4 pb-4 sm:px-5 sm:pb-5">{children}</div>;

const Kicker = ({ children }: { children: ReactNode }) => <span className="font-mono font-semibold text-[10.5px] text-muted-foreground/70 uppercase tracking-[0.12em]">{children}</span>;

export const ByUpgradeType = () => (
    <Panel>
        <SectionHeader accent={OPERATOR} count="212 total · +4.7 to overall grade" title="Operators below milestone" />
        <div className="flex flex-col gap-2">
            <Kicker>By upgrade type · % gained in this section</Kicker>
            <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-6">
                <ImprovementsStatTile accent={OPERATOR} label="E↑" onClick={noop} sub="+8.4%" value={64} />
                <ImprovementsStatTile accent={OPERATOR} label="Lvl" onClick={noop} sub="+6.1%" value={131} />
                <ImprovementsStatTile accent={OPERATOR} label="M3" onClick={noop} sub="+11.9%" value={97} />
                <ImprovementsStatTile accent={OPERATOR} label="Mod" onClick={noop} sub="+5.2%" value={83} />
                <ImprovementsStatTile accent={OPERATOR} label="Pot" onClick={noop} sub="+2.0%" value={176} />
                <ImprovementsStatTile accent={OPERATOR} label="Trust" onClick={noop} sub="+1.3%" value={58} />
            </div>
        </div>
    </Panel>
);

export const FilterActive = () => (
    <Panel>
        <SectionHeader accent={OPERATOR} count="97 shown · M3" title="Operators below milestone" />
        <div className="flex flex-col gap-2">
            <Kicker>By upgrade type · % gained in this section</Kicker>
            <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-6">
                <ImprovementsStatTile accent={OPERATOR} label="E↑" onClick={noop} sub="+8.4%" value={64} />
                <ImprovementsStatTile accent={OPERATOR} label="Lvl" onClick={noop} sub="+6.1%" value={131} />
                <ImprovementsStatTile accent={OPERATOR} active label="M3" onClick={noop} sub="+11.9%" value={97} />
                <ImprovementsStatTile accent={OPERATOR} label="SL7" sub="Skill 7" value={0} />
                <ImprovementsStatTile accent={OPERATOR} label="Mod" onClick={noop} sub="+5.2%" value={83} />
                <ImprovementsStatTile accent={OPERATOR} label="Pot" onClick={noop} sub="+2.0%" value={176} />
            </div>
        </div>
    </Panel>
);

export const NothingRemaining = () => (
    <Panel>
        <SectionHeader accent={OPERATOR} count="0 total" title="Operators below milestone" />
        <div className="flex flex-col gap-2">
            <Kicker>By upgrade type · % gained in this section</Kicker>
            <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-6">
                <ImprovementsStatTile accent={OPERATOR} label="E↑" sub="Elite promotion" value={0} />
                <ImprovementsStatTile accent={OPERATOR} label="Lvl" sub="Level cap" value={0} />
                <ImprovementsStatTile accent={OPERATOR} label="M3" sub="Mastery 3" value={0} />
                <ImprovementsStatTile accent={OPERATOR} label="SL7" sub="Skill 7" value={0} />
                <ImprovementsStatTile accent={OPERATOR} label="Mod" sub="Module L3" value={0} />
                <ImprovementsStatTile accent={OPERATOR} label="Pot" sub="Potential 6" value={0} />
            </div>
        </div>
    </Panel>
);
