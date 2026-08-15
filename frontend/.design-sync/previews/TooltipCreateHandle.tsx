import { Button, Tooltip, TooltipCreateHandle, TooltipPopup, TooltipTrigger } from "frontend";
import { BookmarkIcon, RefreshCwIcon, Trash2Icon } from "lucide-react";

// One handle per tooltip, created once at module scope: triggers anywhere in the
// tree can drive it without being nested inside the Tooltip root, and every
// trigger can hand it a different payload.
const rowTip = TooltipCreateHandle<string>();
const recalcTip = TooltipCreateHandle();

/** Opened through the handle — the popup anchors to whichever trigger owns it. */
export const OpenedByHandle = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <TooltipTrigger handle={recalcTip} render={<Button aria-label="Recalculate DPS" size="icon" variant="outline" />}>
            <RefreshCwIcon />
        </TooltipTrigger>
        <Tooltip handle={recalcTip} open>
            <TooltipPopup side="bottom">Recalculate DPS for all 6 operators</TooltipPopup>
        </Tooltip>
    </div>
);

/** One handle, several triggers, a per-trigger payload — the roster row's action column. */
export const SharedByRowActions = () => (
    <div className="flex w-full max-w-md flex-col gap-2">
        {[
            { name: "Młynar", hint: "Pin Młynar to the top of your roster" },
            { name: "Skadi", hint: "Pin Skadi to the top of your roster" },
            { name: "Eyjafjalla", hint: "Pin Eyjafjalla to the top of your roster" },
        ].map((op) => (
            <div className="flex items-center justify-between gap-3 rounded-lg border bg-card px-3 py-2" key={op.name}>
                <span className="font-medium text-sm">{op.name}</span>
                <div className="flex items-center gap-1">
                    <TooltipTrigger handle={rowTip} payload={op.hint} render={<Button aria-label={`Pin ${op.name}`} size="icon-sm" variant="ghost" />}>
                        <BookmarkIcon />
                    </TooltipTrigger>
                    <Button aria-label={`Remove ${op.name}`} size="icon-sm" variant="ghost">
                        <Trash2Icon />
                    </Button>
                </div>
            </div>
        ))}
        <Tooltip handle={rowTip}>
            <TooltipPopup side="bottom">Pin to the top of your roster</TooltipPopup>
        </Tooltip>
    </div>
);
