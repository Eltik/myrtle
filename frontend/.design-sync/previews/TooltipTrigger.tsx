import { Badge, Button, Tooltip, TooltipPopup, TooltipTrigger } from "frontend";
import { CircleQuestionMarkIcon, SwordsIcon } from "lucide-react";

/** The common case: `render` hands the trigger an existing Button. */
export const IconButton = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <Tooltip open>
            <TooltipTrigger render={<Button aria-label="Skill details" size="icon" variant="outline" />}>
                <SwordsIcon />
            </TooltipTrigger>
            <TooltipPopup side="bottom">Sworn Enemy of Evil · M3</TooltipPopup>
        </Tooltip>
    </div>
);

/** The trigger can wrap any element — here a badge in the potential row. */
export const OnBadge = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <Tooltip open>
            <TooltipTrigger render={<Badge className="cursor-default" variant="secondary" />}>P3 · ATK +30</TooltipTrigger>
            <TooltipPopup side="bottom">Potential 3 raises attack power by 30 at max level.</TooltipPopup>
        </Tooltip>
    </div>
);

/** Trigger with no `render` prop — it emits its own button. */
export const DefaultElement = () => (
    <div className="flex h-96 w-full items-start justify-center pt-10">
        <Tooltip open>
            <TooltipTrigger className="inline-flex size-7 cursor-default items-center justify-center rounded-md text-muted-foreground">
                <CircleQuestionMarkIcon className="size-4" />
            </TooltipTrigger>
            <TooltipPopup side="bottom">Sanity per item, averaged over the last 30 days.</TooltipPopup>
        </Tooltip>
    </div>
);

/** Several triggers in a row, at rest. */
export const Resting = () => (
    <div className="flex w-full items-center gap-2">
        {["Overview", "Skills", "Modules"].map((label) => (
            <Tooltip key={label}>
                <TooltipTrigger render={<Button size="sm" variant="outline" />}>{label}</TooltipTrigger>
                <TooltipPopup side="bottom">{label}</TooltipPopup>
            </Tooltip>
        ))}
    </div>
);
