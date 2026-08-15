import { Button, Input, Popover, PopoverDescription, PopoverPopup, PopoverTitle, PopoverTrigger } from "frontend";
import { MonitorIcon, MoonIcon, MoreHorizontalIcon, PaletteIcon, SunIcon } from "lucide-react";

export const JumpToPage = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger
                render={
                    <Button aria-label="Jump to a hidden page" size="icon" variant="outline">
                        <MoreHorizontalIcon className="h-4 w-4" />
                    </Button>
                }
            />
            <PopoverPopup align="center" className="w-64" sideOffset={8}>
                <div className="flex flex-col gap-3">
                    <PopoverTitle className="text-sm">Jump to page</PopoverTitle>
                    <PopoverDescription>Pages 6–24 are hidden. Enter any page between 1 and 27.</PopoverDescription>
                    <div className="flex items-center gap-2">
                        <Input aria-label="Page number" defaultValue="15" size="sm" />
                        <Button size="sm">Go</Button>
                    </div>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);

export const AppearanceSettings = () => (
    <div className="flex min-h-70 w-full items-start justify-center pt-4">
        <Popover open>
            <PopoverTrigger
                render={
                    <Button aria-label="Theme: Dark. Open appearance settings." size="icon" variant="ghost">
                        <MoonIcon className="h-4.5 w-4.5" />
                    </Button>
                }
            />
            <PopoverPopup align="end" className="w-64" initialFocus={false} sideOffset={8}>
                <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-1.5 font-medium text-muted-foreground text-xs">
                        <SunIcon className="h-3.5 w-3.5" />
                        <span>Appearance</span>
                    </div>
                    <div className="grid grid-cols-3 gap-1">
                        <Button className="h-8 gap-1.5 text-xs" size="sm" variant="ghost">
                            <SunIcon className="h-3.5 w-3.5" />
                            Light
                        </Button>
                        <Button className="h-8 gap-1.5 text-xs" size="sm" variant="outline">
                            <MoonIcon className="h-3.5 w-3.5" />
                            Dark
                        </Button>
                        <Button className="h-8 gap-1.5 text-xs" size="sm" variant="ghost">
                            <MonitorIcon className="h-3.5 w-3.5" />
                            Auto
                        </Button>
                    </div>
                    <div className="-mx-1 h-px bg-border" />
                    <span className="flex items-center gap-1.5 font-medium text-muted-foreground text-xs">
                        <PaletteIcon className="h-3.5 w-3.5" />
                        Accent
                    </span>
                    <div className="grid grid-cols-6 gap-1.5">
                        {["#e0455a", "#e08a3c", "#d9b13c", "#4fa06a", "#3f7fd0", "#8a5bd0"].map((hex) => (
                            <span className="aspect-square w-full rounded-full" key={hex} style={{ backgroundColor: hex }} />
                        ))}
                    </div>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);
