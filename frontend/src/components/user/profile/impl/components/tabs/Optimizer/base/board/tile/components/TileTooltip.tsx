import type { ReactElement, ReactNode } from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";

/**
 * `focusable` puts a plain text trigger (a badge, a ledger chip) in the tab
 * order, so its explanation opens on keyboard focus as well as on hover. A
 * trigger that already is a control, or sits inside one, leaves it off.
 */
export function TileTooltip({ label, children, focusable = false }: { label: ReactNode; children: ReactElement; focusable?: boolean }) {
    return (
        <Tooltip>
            <TooltipTrigger
                render={focusable ? <span className="pointer-coarse:after:-translate-1/2 relative inline-flex shrink-0 rounded-sm pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:size-11">{children}</span> : children}
                tabIndex={focusable ? 0 : undefined}
            />
            <TooltipPopup className="px-2 py-1">{label}</TooltipPopup>
        </Tooltip>
    );
}
