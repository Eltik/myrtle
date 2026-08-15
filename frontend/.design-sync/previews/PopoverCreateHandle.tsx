import { Button, Popover, PopoverCreateHandle, PopoverDescription, PopoverPopup, PopoverTitle, PopoverTrigger } from "frontend";

// One handle shared by several detached triggers — the popover lives outside the
// toolbar it belongs to, so the row can scroll without re-mounting the popup.
const rosterHandle = PopoverCreateHandle();

export const DetachedTriggers = () => (
    <div className="flex min-h-70 w-full flex-col items-center gap-4 pt-4">
        <div className="flex items-center gap-2">
            <PopoverTrigger handle={rosterHandle} id="detached-rarity" render={<Button size="sm" variant="outline" />}>
                Rarity
            </PopoverTrigger>
            <PopoverTrigger handle={rosterHandle} id="detached-class" render={<Button size="sm" variant="outline" />}>
                Class
            </PopoverTrigger>
            <PopoverTrigger handle={rosterHandle} id="detached-faction" render={<Button size="sm" variant="outline" />}>
                Faction
            </PopoverTrigger>
        </div>

        <Popover handle={rosterHandle} open triggerId="detached-class">
            <PopoverPopup align="center" className="w-64" sideOffset={8}>
                <div className="flex flex-col gap-2">
                    <PopoverTitle className="text-sm">Filter by class</PopoverTitle>
                    <PopoverDescription>One popover instance is reused by every filter chip in the toolbar.</PopoverDescription>
                    <div className="flex flex-wrap gap-1.5 pt-1">
                        <Button size="sm" variant="secondary">
                            Guard
                        </Button>
                        <Button size="sm" variant="ghost">
                            Sniper
                        </Button>
                        <Button size="sm" variant="ghost">
                            Defender
                        </Button>
                        <Button size="sm" variant="ghost">
                            Medic
                        </Button>
                    </div>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);

export const SingleDetachedTrigger = () => (
    <div className="flex min-h-70 w-full flex-col items-center gap-4 pt-4">
        <PopoverTrigger handle={rosterHandle} id="detached-sync" render={<Button size="sm" variant="outline" />}>
            Roster sync status
        </PopoverTrigger>
        <Popover handle={rosterHandle} open triggerId="detached-sync">
            <PopoverPopup align="center" className="w-64" sideOffset={8}>
                <div className="flex flex-col gap-2">
                    <PopoverTitle className="text-sm">Last sync</PopoverTitle>
                    <PopoverDescription>4 hours ago · 231 operators · 12 modules pending import.</PopoverDescription>
                </div>
            </PopoverPopup>
        </Popover>
    </div>
);
