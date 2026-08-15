import { OperatorPlannerDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// The planner's create/edit modal. It pulls the operator list, the upcoming
// list, the user's roster and their existing plans from server functions, all of
// which are stubbed in previews — so what a card can honestly show is the
// opened shell: header, operator combobox, the "pick an operator" hint, and the
// footer with Create disabled until a selection exists.

const noop = () => {};

// An open Base UI dialog auto-focuses its first tabbable control, which lands the
// brand-red focus ring on the empty operator combobox and reads as a validation
// error. Two frames after mount, blur it — the dialog stays open.
function Stage({ children }: { children: ReactNode }) {
    useEffect(() => {
        let f2 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur());
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
        };
    }, []);
    return <div className="min-h-[520px] w-full">{children}</div>;
}

export const OpenOperatorListUnavailable = () => (
    <Stage>
        <OperatorPlannerDialog onOpenChange={noop} open />
    </Stage>
);
