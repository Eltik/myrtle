import { CollapsibleSection, ImprovementsPill, ProgressLine, SectionHeader } from "frontend";
import type { ReactNode } from "react";

const MEDAL = "oklch(0.62 0.22 295)";
const ROGUE = "oklch(0.65 0.22 340)";
const URGENT = "oklch(0.65 0.22 30)";

const Panel = ({ children }: { children: ReactNode }) => <div className="flex w-full max-w-xl flex-col gap-5 rounded-xl border border-border bg-card px-4 pt-4 pb-4 sm:px-5 sm:pb-5">{children}</div>;

const MedalRow = ({ name, method, tag, urgent }: { name: string; method: string; tag: string; urgent?: boolean }) => (
    <div className="flex items-start justify-between gap-3 rounded-md border border-border/40 bg-muted/15 px-2 py-1.5">
        <div className="min-w-0">
            <p className="truncate font-semibold text-[11.5px] leading-tight">{name}</p>
            <p className="text-[10.5px] text-muted-foreground/85 leading-snug">{method}</p>
        </div>
        <ImprovementsPill className="shrink-0 whitespace-nowrap" color={urgent ? URGENT : MEDAL}>
            {tag}
        </ImprovementsPill>
    </div>
);

export const Expanded = () => (
    <Panel>
        <CollapsibleSection accent={MEDAL} count="4 missing" defaultOpen title="Event medals ending soon">
            <MedalRow method="Clear IW-EX-8 on Challenge Mode." name="Ashes of Ashes" tag="5d left" urgent />
            <MedalRow method="Recruit Ines during the event window." name="Wind of the Old Days" tag="12d left" />
            <MedalRow method="Deploy only Sarkaz operators in IW-7." name="Kin of the Sand" tag="12d left" />
            <MedalRow method="Finish every side story stage." name="A Death in Chernobog" tag="26d left" />
        </CollapsibleSection>
    </Panel>
);

export const Collapsed = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={MEDAL} count="9 missing" title="Still obtainable" />
            <MedalRow method="Clear 12-17 with a squad of 6 or fewer." name="Deepness" tag="T3" />
            <MedalRow method="Complete every Operational Record." name="Chronicles" tag="T2" />
        </div>
        <CollapsibleSection accent={MEDAL} count="118 medals" title="No longer obtainable">
            <MedalRow method="Event ended 3 Feb 2021." name="Heart of Ice" tag="Ended 2021" />
        </CollapsibleSection>
    </Panel>
);

export const TwoSections = () => (
    <Panel>
        <CollapsibleSection accent={ROGUE} count="ROGUE_3" defaultOpen title="Mizuki & Caerula Arbor">
            <ProgressLine accent={ROGUE} current={5} label="Endings" max={6} />
            <ProgressLine accent={ROGUE} current={98} label="BP levels" max={140} />
            <ProgressLine accent={ROGUE} current={214} label="Collectibles" max={311} />
        </CollapsibleSection>
        <CollapsibleSection accent={ROGUE} count="ROGUE_2" title="Phantom & Crimson Solitaire">
            <ProgressLine accent={ROGUE} current={4} label="Endings" max={5} />
            <ProgressLine accent={ROGUE} current={140} label="BP levels" max={140} />
        </CollapsibleSection>
    </Panel>
);
