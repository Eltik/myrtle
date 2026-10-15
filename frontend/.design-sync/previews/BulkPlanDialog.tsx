import { BulkPlanDialog } from "frontend";
import { useEffect } from "react";

// "Bulk add plans": one target for many operators. The left column is the
// operator search with the picks under it and a live preview of what each
// pick will be saved as; the right holds the target form, presets, groups
// and the two switches. The operator index, roster and plans come from
// server queries the design-system bundle cannot reach, so the honest render
// is the dialog freshly opened: no picks, the default target, an empty preview.

const noop = () => undefined;

/** Freshly opened: nothing picked, "Add plans" disabled. */
export const Opened = () => {
    // The popup focuses the preset select once its transition lands; the
    // brand-red ring there reads as an error, so drop it on a short timer.
    useEffect(() => {
        const timers = [60, 180, 400].map((ms) => window.setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => {
            for (const t of timers) window.clearTimeout(t);
        };
    }, []);
    return (
        <div className="min-h-[520px] w-full">
            <BulkPlanDialog open onOpenChange={noop} />
        </div>
    );
};
