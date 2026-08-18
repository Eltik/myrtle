import type { ReactElement, ReactNode } from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";

export function TileTooltip({ label, children }: { label: ReactNode; children: ReactElement }) {
    return (
        <Tooltip>
            <TooltipTrigger render={children} />
            <TooltipPopup className="px-2 py-1">{label}</TooltipPopup>
        </Tooltip>
    );
}
