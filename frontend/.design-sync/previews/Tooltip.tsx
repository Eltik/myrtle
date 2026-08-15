import { Button, Tooltip, TooltipPopup, TooltipTrigger } from "frontend";
import { BookmarkIcon, InfoIcon, RefreshCwIcon, Trash2Icon } from "lucide-react";

/** Open — the collapsed sidebar's label for an icon-only nav button. */
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

/** Two sides at once, so the positioner's placement reads. */
export const Sides = () => (
    <div className="flex h-96 w-full items-start justify-center gap-16 pt-16">
        <Tooltip open>
            <TooltipTrigger render={<Button aria-label="Save to depot" size="icon" variant="outline" />}>
                <BookmarkIcon />
            </TooltipTrigger>
            <TooltipPopup side="top">Save to depot</TooltipPopup>
        </Tooltip>
        <Tooltip open>
            <TooltipTrigger render={<Button aria-label="Remove operator" size="icon" variant="destructive-outline" />}>
                <Trash2Icon />
            </TooltipTrigger>
            <TooltipPopup side="bottom">Remove from plan</TooltipPopup>
        </Tooltip>
    </div>
);

/** A longer hint — the tooltip balances and wraps inside the available width. */
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

/** Closed — the resting state of a row of icon actions. */
export const Closed = () => (
    <div className="flex w-full items-center gap-2">
        <Tooltip>
            <TooltipTrigger render={<Button aria-label="Recalculate DPS" size="icon" variant="outline" />}>
                <RefreshCwIcon />
            </TooltipTrigger>
            <TooltipPopup side="bottom">Recalculate DPS</TooltipPopup>
        </Tooltip>
        <Tooltip>
            <TooltipTrigger render={<Button aria-label="Save to depot" size="icon" variant="outline" />}>
                <BookmarkIcon />
            </TooltipTrigger>
            <TooltipPopup side="bottom">Save to depot</TooltipPopup>
        </Tooltip>
        <Tooltip>
            <TooltipTrigger render={<Button aria-label="Remove operator" size="icon" variant="destructive-outline" />}>
                <Trash2Icon />
            </TooltipTrigger>
            <TooltipPopup side="bottom">Remove from plan</TooltipPopup>
        </Tooltip>
    </div>
);
