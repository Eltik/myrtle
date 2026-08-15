import { Button, Tooltip, TooltipPopup, TooltipProvider, TooltipTrigger } from "frontend";
import { BoxesIcon, ImageIcon, SwordsIcon, UserIcon } from "lucide-react";

// One provider wraps a whole page section so that, once any tooltip has opened,
// the neighbours skip their own delay. OperatorTabs wraps its rail this way.

/** A rail of tooltipped nav buttons sharing one provider; the active one is open. */
export const SharedDelay = () => (
    <TooltipProvider delay={200}>
        <div className="flex h-96 w-full items-start justify-center pt-10">
            <div className="flex flex-col gap-1 rounded-xl border bg-card p-1">
                <Tooltip open>
                    <TooltipTrigger render={<Button aria-label="Overview" size="icon" variant="ghost" />}>
                        <UserIcon />
                    </TooltipTrigger>
                    <TooltipPopup side="right">Overview</TooltipPopup>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger render={<Button aria-label="Skills" size="icon" variant="ghost" />}>
                        <SwordsIcon />
                    </TooltipTrigger>
                    <TooltipPopup side="right">Skills</TooltipPopup>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger render={<Button aria-label="Modules" size="icon" variant="ghost" />}>
                        <BoxesIcon />
                    </TooltipTrigger>
                    <TooltipPopup side="right">Modules</TooltipPopup>
                </Tooltip>
                <Tooltip>
                    <TooltipTrigger render={<Button aria-label="Art and files" size="icon" variant="ghost" />}>
                        <ImageIcon />
                    </TooltipTrigger>
                    <TooltipPopup side="right">Art &amp; files</TooltipPopup>
                </Tooltip>
            </div>
        </div>
    </TooltipProvider>
);

/** The same provider over a horizontal action bar, at rest. */
export const ActionBar = () => (
    <TooltipProvider closeDelay={100} delay={200}>
        <div className="flex w-fit items-center gap-2 rounded-xl border bg-card p-1">
            <Tooltip>
                <TooltipTrigger render={<Button aria-label="Overview" size="icon-sm" variant="ghost" />}>
                    <UserIcon />
                </TooltipTrigger>
                <TooltipPopup side="bottom">Overview</TooltipPopup>
            </Tooltip>
            <Tooltip>
                <TooltipTrigger render={<Button aria-label="Skills" size="icon-sm" variant="ghost" />}>
                    <SwordsIcon />
                </TooltipTrigger>
                <TooltipPopup side="bottom">Skills</TooltipPopup>
            </Tooltip>
            <Tooltip>
                <TooltipTrigger render={<Button aria-label="Modules" size="icon-sm" variant="ghost" />}>
                    <BoxesIcon />
                </TooltipTrigger>
                <TooltipPopup side="bottom">Modules</TooltipPopup>
            </Tooltip>
        </div>
    </TooltipProvider>
);
