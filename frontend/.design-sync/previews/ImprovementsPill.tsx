import { ImprovementsPill, SectionHeader } from "frontend";
import type { ReactNode } from "react";

const STAGE = "oklch(0.62 0.20 255)";
const MEDAL = "oklch(0.62 0.22 295)";
const SANDBOX = "oklch(0.70 0.14 200)";
const URGENT = "oklch(0.65 0.22 30)";

const Panel = ({ children }: { children: ReactNode }) => <div className="flex w-full max-w-xl flex-col gap-5 rounded-xl border border-border bg-card px-4 pt-4 pb-4 sm:px-5 sm:pb-5">{children}</div>;

/** The stage chip from StagePanel: state dot, code, name, weight pill. */
const StageChip = ({ code, name, weight }: { code: string; name: string; weight?: number }) => (
    <span className="group flex items-center gap-1.5 rounded-md border border-border/40 bg-muted/15 px-2 py-1">
        <span aria-hidden className="size-1.5 shrink-0 rounded-full bg-amber-500/75" />
        <span className="max-w-[22ch] truncate font-mono font-semibold text-[10px] tabular-nums" style={{ color: `color-mix(in oklch, ${STAGE} 55%, var(--foreground))` }}>
            {code}
        </span>
        <span className="hidden max-w-[16ch] truncate text-[11px] text-muted-foreground sm:inline">{name}</span>
        {weight !== undefined && (
            <ImprovementsPill color={STAGE}>×{weight.toFixed(2)}</ImprovementsPill>
        )}
    </span>
);

export const StageWeights = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={STAGE} count="6 total" title="Cleared, not 3★" />
            <div className="flex flex-wrap gap-1.5">
                <StageChip code="H8-4" name="Roaring Flare" weight={1.35} />
                <StageChip code="S4-1" name="Frozen Ground" weight={1.2} />
                <StageChip code="IW-EX-8" name="Ashen Vigil" weight={0.65} />
                <StageChip code="12-17" name="Deepness" />
                <StageChip code="R8-11" name="Wine of Lungmen" weight={0.4} />
                <StageChip code="1-7" name="Sanity Farm" />
            </div>
        </div>
    </Panel>
);

export const EventDeadlines = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={MEDAL} count="4 missing" title="Event medals" />
            <div className="flex flex-wrap items-center gap-1.5">
                <ImprovementsPill className="whitespace-nowrap" color={URGENT}>
                    ending now
                </ImprovementsPill>
                <ImprovementsPill className="whitespace-nowrap" color={URGENT}>
                    5d left
                </ImprovementsPill>
                <ImprovementsPill className="whitespace-nowrap" color={MEDAL}>
                    12d left
                </ImprovementsPill>
                <ImprovementsPill className="whitespace-nowrap" color={MEDAL}>
                    26d left
                </ImprovementsPill>
                <ImprovementsPill className="whitespace-nowrap" color={MEDAL}>
                    Ended Feb 2021
                </ImprovementsPill>
            </div>
        </div>
    </Panel>
);

export const NeutralTags = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={SANDBOX} title="Reclamation Algorithm" />
            <div className="flex flex-wrap items-center gap-1.5">
                <ImprovementsPill>32% of RA</ImprovementsPill>
                <ImprovementsPill>18% of RA</ImprovementsPill>
                <ImprovementsPill>Requires Ines</ImprovementsPill>
                <ImprovementsPill color={SANDBOX}>Season 3</ImprovementsPill>
            </div>
        </div>
    </Panel>
);
