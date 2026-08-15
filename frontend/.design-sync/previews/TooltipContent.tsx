import { Button, Tooltip, TooltipContent, TooltipTrigger } from "frontend";
import { BoxesIcon, SwordsIcon, UserIcon } from "lucide-react";

// TooltipContent is the shadcn-compatible alias for TooltipPopup.

/** Open on an icon-only action. */
export const Open = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <Tooltip open>
            <TooltipTrigger render={<Button aria-label="Modules" size="icon" variant="outline" />}>
                <BoxesIcon />
            </TooltipTrigger>
            <TooltipContent side="bottom">Modules</TooltipContent>
        </Tooltip>
    </div>
);

/** `side="right"` — the collapsed sidebar's label, exactly as SidebarMenuButton composes it. */
export const CollapsedSidebar = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <div className="flex flex-col gap-1 rounded-xl border bg-card p-1">
            <Tooltip open>
                <TooltipTrigger render={<Button aria-label="Roster" size="icon" variant="ghost" />}>
                    <UserIcon />
                </TooltipTrigger>
                <TooltipContent align="center" side="right">
                    Roster
                </TooltipContent>
            </Tooltip>
            <Button aria-label="Skills" size="icon" variant="ghost">
                <SwordsIcon />
            </Button>
            <Button aria-label="Modules" size="icon" variant="ghost">
                <BoxesIcon />
            </Button>
        </div>
    </div>
);

/** At rest, so the trigger row reads on its own. */
export const Resting = () => (
    <div className="flex w-full items-center gap-2">
        <Tooltip>
            <TooltipTrigger render={<Button aria-label="Roster" size="icon" variant="outline" />}>
                <UserIcon />
            </TooltipTrigger>
            <TooltipContent side="bottom">Roster</TooltipContent>
        </Tooltip>
        <Tooltip>
            <TooltipTrigger render={<Button aria-label="Skills" size="icon" variant="outline" />}>
                <SwordsIcon />
            </TooltipTrigger>
            <TooltipContent side="bottom">Skills</TooltipContent>
        </Tooltip>
    </div>
);
