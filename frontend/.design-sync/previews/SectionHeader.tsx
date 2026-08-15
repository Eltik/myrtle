import { EmptyHint, ProgressLine, SectionHeader } from "frontend";
import type { ReactNode } from "react";

// Subscore accents, ported from Score/palette.ts (SCORE_PALETTE).
const STAGE = "oklch(0.62 0.20 255)";
const ROGUE = "oklch(0.65 0.22 340)";
const OPERATOR = "oklch(0.74 0.17 75)";
const SANDBOX = "oklch(0.70 0.14 200)";

/** The expanded subscore panel these headers live inside. */
const Panel = ({ children }: { children: ReactNode }) => <div className="flex w-full max-w-xl flex-col gap-5 rounded-xl border border-border bg-card px-4 pt-4 pb-4 sm:px-5 sm:pb-5">{children}</div>;

export const StagePool = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={STAGE} count="1,204 / 1,431 3★" title="Permanent" />
            <div className="grid gap-2.5 sm:grid-cols-3">
                <ProgressLine accent={STAGE} current={1362} label="Cleared" max={1431} />
                <ProgressLine accent={STAGE} current={1204} label="3-starred" max={1431} />
                <ProgressLine accent={STAGE} current={158} label="Not 3-starred" max={1431} />
            </div>
        </div>
    </Panel>
);

export const RoguelikeTheme = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={ROGUE} count="ROGUE_4" title="Zwillingstürme im Herbst" />
            <div className="grid gap-2.5 sm:grid-cols-3">
                <ProgressLine accent={ROGUE} current={6} label="Endings" max={8} />
                <ProgressLine accent={ROGUE} current={112} label="BP levels" max={140} />
                <ProgressLine accent={ROGUE} current={11} label="Difficulty" max={15} />
            </div>
        </div>
    </Panel>
);

export const WithoutCount = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={SANDBOX} title="Reclamation Algorithm" />
            <EmptyHint>No RA progress detected. Start RA in Operation Originium Dust to begin tracking.</EmptyHint>
        </div>
    </Panel>
);

export const StackedSections = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={OPERATOR} count="61.4% earned" title="Where your score comes from" />
            <div className="grid gap-2.5 sm:grid-cols-3">
                <ProgressLine accent={OPERATOR} current={148} label="Elites" max={268} />
                <ProgressLine accent={OPERATOR} current={388} label="Masteries" max={444} />
                <ProgressLine accent={OPERATOR} current={94} label="Modules" max={211} />
            </div>
        </div>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={OPERATOR} count="212 total · +4.7 to overall grade" title="Operators below milestone" />
            <EmptyHint>Pick an upgrade type above to filter the rarity buckets.</EmptyHint>
        </div>
    </Panel>
);
