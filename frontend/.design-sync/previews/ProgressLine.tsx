import { ProgressLine, SectionHeader } from "frontend";
import type { ReactNode } from "react";

const ROGUE = "oklch(0.65 0.22 340)";
const STAGE = "oklch(0.62 0.20 255)";

const Panel = ({ children }: { children: ReactNode }) => <div className="flex w-full max-w-xl flex-col gap-5 rounded-xl border border-border bg-card px-4 pt-4 pb-4 sm:px-5 sm:pb-5">{children}</div>;

export const RoguelikeTheme = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={ROGUE} count="ROGUE_3" title="Mizuki & Caerula Arbor" />
            <div className="grid gap-2.5 sm:grid-cols-2">
                <ProgressLine accent={ROGUE} current={5} label="Endings" max={6} />
                <ProgressLine accent={ROGUE} current={98} label="BP levels" max={140} />
                <ProgressLine accent={ROGUE} current={12} label="Difficulty" max={15} />
                <ProgressLine accent={ROGUE} current={41} label="Challenges" max={58} />
                <ProgressLine accent={ROGUE} current={214} label="Collectibles" max={311} />
                <ProgressLine accent={ROGUE} current={7} label="Squads" max={11} />
            </div>
        </div>
    </Panel>
);

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

export const EmptyAndFull = () => (
    <Panel>
        <div className="flex flex-col gap-3">
            <SectionHeader accent={ROGUE} count="ROGUE_1" title="Ceobe's Fungimist" />
            <div className="grid gap-2.5 sm:grid-cols-2">
                <ProgressLine accent={ROGUE} current={4} label="Endings" max={4} />
                <ProgressLine accent={ROGUE} current={140} label="BP levels" max={140} />
                <ProgressLine accent={ROGUE} current={0} label="Difficulty" max={12} />
                <ProgressLine accent={ROGUE} current={3} label="Challenges" max={44} />
            </div>
        </div>
    </Panel>
);
