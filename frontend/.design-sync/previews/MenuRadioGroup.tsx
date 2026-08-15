import { Menu, MenuGroup, MenuGroupLabel, MenuPopup, MenuRadioGroup, MenuRadioItem, MenuTrigger } from "frontend";
import { ArrowUpDownIcon, ChevronDownIcon, GlobeIcon } from "lucide-react";

export const SortOrder = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Roster</span>
            <Menu modal={false} open>
                <MenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <ArrowUpDownIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Rarity, high to low</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </MenuTrigger>
                <MenuPopup align="start" className="w-64" sideOffset={6}>
                    <MenuGroup>
                        <MenuGroupLabel>Sort operators by</MenuGroupLabel>
                        <MenuRadioGroup defaultValue="rarity">
                            <MenuRadioItem value="rarity">Rarity, high to low</MenuRadioItem>
                            <MenuRadioItem value="name">Name (A–Z)</MenuRadioItem>
                            <MenuRadioItem value="release">Release date</MenuRadioItem>
                            <MenuRadioItem value="trust">Trust level</MenuRadioItem>
                        </MenuRadioGroup>
                    </MenuGroup>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);

export const ServerRegion = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Data source</span>
            <Menu modal={false} open>
                <MenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <GlobeIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Global (EN)</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </MenuTrigger>
                <MenuPopup align="start" className="w-56" sideOffset={6}>
                    <MenuGroup>
                        <MenuGroupLabel>Game server</MenuGroupLabel>
                        <MenuRadioGroup defaultValue="en">
                            <MenuRadioItem value="en">Global (EN)</MenuRadioItem>
                            <MenuRadioItem value="cn">Mainland (CN)</MenuRadioItem>
                            <MenuRadioItem value="jp">Japan (JP)</MenuRadioItem>
                            <MenuRadioItem value="kr">Korea (KR)</MenuRadioItem>
                        </MenuRadioGroup>
                    </MenuGroup>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);
