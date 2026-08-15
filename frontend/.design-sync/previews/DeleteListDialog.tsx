import { DeleteListDialog } from "frontend";
import type { ReactNode } from "react";

const TARGET = { slug: "endgame-dps-rankings", name: "Endgame DPS rankings" };

const noop = () => {};

const Stage = ({ children }: { children: ReactNode }) => <div className="min-h-[520px]">{children}</div>;

export const Default = () => (
    <Stage>
        <DeleteListDialog target={TARGET} onOpenChange={noop} onConfirm={noop} isSubmitting={false} errorMessage={null} />
    </Stage>
);

export const Deleting = () => (
    <Stage>
        <DeleteListDialog target={{ slug: "cc12-daybreak-risk-18", name: "CC#12 Daybreak — risk 18 picks" }} onOpenChange={noop} onConfirm={noop} isSubmitting errorMessage={null} />
    </Stage>
);

export const DeleteFailed = () => (
    <Stage>
        <DeleteListDialog target={TARGET} onOpenChange={noop} onConfirm={noop} isSubmitting={false} errorMessage="This list is featured on the browse page and can't be deleted while it is pinned." />
    </Stage>
);
