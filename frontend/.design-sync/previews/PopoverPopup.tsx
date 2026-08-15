import { Button, Popover, PopoverDescription, PopoverPopup, PopoverTitle, PopoverTrigger } from "frontend";
import { InfoIcon } from "lucide-react";

export const StageRewards = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger render={<Button size="sm" variant="outline" />}>S4-1 rewards</PopoverTrigger>
            <PopoverPopup align="start" className="w-72" sideOffset={8}>
                <div className="flex flex-col gap-3">
                    <PopoverTitle className="text-sm">S4-1 — Roaring Flare</PopoverTitle>
                    <PopoverDescription>18 sanity · first clear grants 2 Chip Catalysts.</PopoverDescription>
                    <ul className="m-0 flex list-none flex-col gap-1.5 p-0 font-sans text-sm">
                        <li className="flex items-center justify-between gap-4">
                            <span className="text-foreground">Polyester Pack</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">28.6%</span>
                        </li>
                        <li className="flex items-center justify-between gap-4">
                            <span className="text-foreground">Manganese Ore</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">17.4%</span>
                        </li>
                        <li className="flex items-center justify-between gap-4">
                            <span className="text-foreground">Orirock Cluster</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">9.1%</span>
                        </li>
                    </ul>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);

export const TooltipStyle = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger
                render={
                    <Button aria-label="What is DPS scoring?" size="icon" variant="ghost">
                        <InfoIcon className="h-4 w-4" />
                    </Button>
                }
            />
            <PopoverPopup align="center" className="max-w-64" side="top" sideOffset={6} tooltipStyle>
                Damage is averaged over a full skill cycle against 1,200 DEF and 40 RES.
            </PopoverPopup>
        </Popover>
    </div>
);

export const AlignedToTheEnd = () => (
    <div className="flex min-h-70 w-full items-start justify-end pt-4 pr-6">
        <Popover open>
            <PopoverTrigger render={<Button size="sm" variant="outline" />}>Sanity budget</PopoverTrigger>
            <PopoverPopup align="end" className="w-64" sideOffset={8}>
                <div className="flex flex-col gap-2 font-sans text-sm">
                    <div className="flex items-center justify-between gap-4">
                        <span className="text-muted-foreground">Spent this week</span>
                        <span className="font-mono text-foreground tabular-nums">1,140</span>
                    </div>
                    <div className="flex items-center justify-between gap-4">
                        <span className="text-muted-foreground">Regenerated</span>
                        <span className="font-mono text-foreground tabular-nums">1,008</span>
                    </div>
                    <div className="-mx-1 h-px bg-border" />
                    <div className="flex items-center justify-between gap-4">
                        <span className="text-muted-foreground">Net</span>
                        <span className="font-mono text-primary tabular-nums">−132</span>
                    </div>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);
