import { Button, Tooltip, TooltipPopup, TooltipTrigger } from "frontend";
import { InfoIcon, LayoutGridIcon, RefreshCwIcon } from "lucide-react";

/** The popup is the visible surface: popover tokens, 12px text, a soft shadow. */
export const Open = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <Tooltip open>
            <TooltipTrigger render={<Button aria-label="Recalculate DPS" size="icon" variant="outline" />}>
                <RefreshCwIcon />
            </TooltipTrigger>
            <TooltipPopup side="bottom">Recalculate DPS</TooltipPopup>
        </Tooltip>
    </div>
);

/** `side` and `align` are forwarded to the positioner. */
export const Placement = () => (
    <div className="flex h-96 w-full items-start justify-center gap-16 pt-16">
        <Tooltip open>
            <TooltipTrigger render={<Button size="sm" variant="outline" />}>side="top"</TooltipTrigger>
            <TooltipPopup side="top">Estimated over 30 days</TooltipPopup>
        </Tooltip>
        <Tooltip open>
            <TooltipTrigger render={<Button size="sm" variant="outline" />}>side="right"</TooltipTrigger>
            <TooltipPopup align="center" side="right">
                Roster
            </TooltipPopup>
        </Tooltip>
    </div>
);

/** Structured content — the recruitment tag breakdown, wrapped and width-capped. */
export const RichContent = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <Tooltip open>
            <TooltipTrigger render={<Button size="sm" variant="outline" />}>
                <LayoutGridIcon />
                Guaranteed 6★
            </TooltipTrigger>
            <TooltipPopup className="max-w-72" side="bottom">
                <span className="flex flex-col gap-1 py-1">
                    <span className="font-medium">Top Operator</span>
                    <span className="text-muted-foreground">18 operators in the pool, all 6★. Average wait is 9 hours.</span>
                </span>
            </TooltipPopup>
        </Tooltip>
    </div>
);

/** A long hint balances across lines instead of running off the popup. */
export const LongHint = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <Tooltip open>
            <TooltipTrigger render={<Button aria-label="About sanity estimates" size="icon-sm" variant="ghost" />}>
                <InfoIcon />
            </TooltipTrigger>
            <TooltipPopup className="max-w-72" side="bottom">
                Sanity estimates come from 1,284 community drop reports and assume the EN server's current event multipliers.
            </TooltipPopup>
        </Tooltip>
    </div>
);
