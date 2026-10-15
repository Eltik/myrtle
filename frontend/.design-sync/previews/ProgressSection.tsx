import { ProgressSection } from "frontend";
import type { ReactNode } from "react";

// The last block of the reader's Settings sheet: Back up, Restore and the one
// destructive control (Reset), with an outcome line under them. The actions
// object comes from `useProgressActions`, which also drives the reset confirm
// rendered beside the sheet; here only its `notice` matters.

const noop = () => {};

function Sheet({ children }: { children: ReactNode }) {
    return <div className="flex w-full max-w-xl flex-col gap-4 rounded-2xl border bg-popover p-6 text-popover-foreground shadow-lg">{children}</div>;
}

const actions = (notice: string | null) => ({ notice, confirmOpen: false, setConfirmOpen: noop, backup: noop, restore: async () => {}, reset: noop });

// At rest: the three buttons, no outcome yet.
export const Idle = () => (
    <Sheet>
        <ProgressSection actions={actions(null)} />
    </Sheet>
);

// After a successful restore: the outcome line counts what came back.
export const Restored = () => (
    <Sheet>
        <ProgressSection actions={actions("Progress restored: 214 stories read, 9 positions.")} />
    </Sheet>
);

// A file that was not a backup: the restore refuses it and says so.
export const RestoreFailed = () => (
    <Sheet>
        <ProgressSection actions={actions("That file is not a progress backup.")} />
    </Sheet>
);
