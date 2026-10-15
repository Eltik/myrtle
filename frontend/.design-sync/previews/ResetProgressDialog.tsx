import { ProgressSection, ResetProgressDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// The confirm behind Settings' "Reset progress": the reader's one destructive
// control. It is a SIBLING of the settings sheet, driven by the same actions
// object (`confirmOpen`), so the story renders the sheet's progress block with
// the alert open over it. The auto-focus ring is cleared once the open
// transition lands.

const noop = () => {};

function Stage({ children }: { children: ReactNode }) {
    useEffect(() => {
        const timers = [60, 180, 400].map((ms) => window.setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => {
            for (const t of timers) window.clearTimeout(t);
        };
    }, []);
    return <div className="relative flex min-h-[520px] w-full items-start justify-center p-6">{children}</div>;
}

const actions = { notice: null, confirmOpen: true, setConfirmOpen: noop, backup: noop, restore: async () => {}, reset: noop };

// The alert open over the progress block that raised it.
export const Open = () => (
    <Stage>
        <div className="w-full max-w-xl rounded-2xl border bg-popover p-6 text-popover-foreground shadow-lg">
            <ProgressSection actions={actions} />
        </div>
        <ResetProgressDialog actions={actions} />
    </Stage>
);
