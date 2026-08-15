import { CreateListDialog } from "frontend";
import type { ReactNode } from "react";

const noop = () => {};

const Stage = ({ children }: { children: ReactNode }) => <div className="min-h-[520px]">{children}</div>;

export const Default = () => (
    <Stage>
        <CreateListDialog open onOpenChange={noop} onSubmit={noop} isSubmitting={false} errorMessage={null} />
    </Stage>
);

export const Submitting = () => (
    <Stage>
        <CreateListDialog open onOpenChange={noop} onSubmit={noop} isSubmitting errorMessage={null} />
    </Stage>
);

export const QuotaReached = () => (
    <Stage>
        <CreateListDialog open onOpenChange={noop} onSubmit={noop} isSubmitting={false} errorMessage="You already have 10 community tier lists. Delete one before creating another." />
    </Stage>
);
