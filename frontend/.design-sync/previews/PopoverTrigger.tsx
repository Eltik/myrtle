import { Button, Checkbox, Label, Popover, PopoverPopup, PopoverTrigger } from "frontend";
import { ChevronDown, MapPin, SlidersHorizontalIcon } from "lucide-react";

export const FilterChipTrigger = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger aria-label="Filter by where enemies appear" className="inline-flex h-9.5 w-fit cursor-pointer items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] px-3.5 font-medium font-sans text-[12.5px] text-foreground transition-colors hover:bg-card data-popup-open:border-primary">
                <MapPin aria-hidden="true" className="h-3.5 w-3.5 text-muted-foreground" />
                <span>Appears In</span>
                <span className="inline-flex h-4.5 min-w-4.5 items-center justify-center rounded-full bg-primary px-1 font-mono text-[10px] text-primary-foreground tabular-nums leading-none">2</span>
                <ChevronDown aria-hidden="true" className="h-3.5 w-3.5 text-muted-foreground" />
            </PopoverTrigger>
            <PopoverPopup align="start" className="w-72" sideOffset={6}>
                <div className="flex flex-col gap-2">
                    <span className="font-medium font-sans text-muted-foreground text-xs">Main theme</span>
                    <Label>
                        <Checkbox defaultChecked />
                        Chapter 8 — Roaring Flare
                    </Label>
                    <Label>
                        <Checkbox defaultChecked />
                        Chapter 9 — Stormwatch
                    </Label>
                    <Label>
                        <Checkbox />
                        Chapter 10 — Shen Lu
                    </Label>
                    <span className="mt-1 font-medium font-sans text-muted-foreground text-xs">Side story</span>
                    <Label>
                        <Checkbox />
                        Dorothy&apos;s Vision
                    </Label>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);

export const IconButtonTrigger = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger
                render={
                    <Button aria-label="Roster display options" size="icon" variant="ghost">
                        <SlidersHorizontalIcon className="h-4 w-4" />
                    </Button>
                }
            />
            <PopoverPopup align="end" className="w-64" sideOffset={8}>
                <div className="flex flex-col gap-2.5">
                    <span className="font-medium font-sans text-muted-foreground text-xs">Roster display</span>
                    <Label>
                        <Checkbox defaultChecked />
                        Show promotion badges
                    </Label>
                    <Label>
                        <Checkbox defaultChecked />
                        Show module levels
                    </Label>
                    <Label>
                        <Checkbox />
                        Show trust percentage
                    </Label>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);
