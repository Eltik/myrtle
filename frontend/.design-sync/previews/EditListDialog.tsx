import { EditListDialog } from "frontend";
import type { ReactNode } from "react";

const INITIAL = {
    slug: "endgame-dps-rankings",
    name: "Endgame DPS rankings",
    description: "Single-target and burst damage ranked for CC risk 18+ and IS#5 Ashring. Assumes E2 90, module stage 3, and no external buffs.",
};

const noop = () => {};

const Stage = ({ children }: { children: ReactNode }) => <div className="min-h-[520px]">{children}</div>;

export const Default = () => (
    <Stage>
        <EditListDialog initial={INITIAL} onOpenChange={noop} onSubmit={noop} isSubmitting={false} errorMessage={null} />
    </Stage>
);

export const Saving = () => (
    <Stage>
        <EditListDialog initial={{ slug: "newcomer-first-20-pulls", name: "Newcomer: your first 20 pulls", description: "What to keep from the beginner banner, and who to skip." }} onOpenChange={noop} onSubmit={noop} isSubmitting errorMessage={null} />
    </Stage>
);

export const SaveFailed = () => (
    <Stage>
        <EditListDialog initial={INITIAL} onOpenChange={noop} onSubmit={noop} isSubmitting={false} errorMessage="Another doctor already uses that name. Pick something else." />
    </Stage>
);
