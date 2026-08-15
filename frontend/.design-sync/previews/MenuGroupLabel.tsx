import { Menu, MenuCheckboxItem, MenuGroup, MenuGroupLabel, MenuItem, MenuPopup, MenuRadioGroup, MenuRadioItem, MenuSeparator, MenuTrigger } from "frontend";
import { ChevronDownIcon, SlidersHorizontalIcon, FilterIcon } from "lucide-react";

export const SectionHeaders = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Operator table</span>
            <Menu modal={false} open>
                <MenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <SlidersHorizontalIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">View options</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </MenuTrigger>
                <MenuPopup align="start" className="w-64" sideOffset={6}>
                    <MenuGroup>
                        <MenuGroupLabel>Columns</MenuGroupLabel>
                        <MenuCheckboxItem defaultChecked>Rarity</MenuCheckboxItem>
                        <MenuCheckboxItem defaultChecked>Class &amp; branch</MenuCheckboxItem>
                        <MenuCheckboxItem>Module stage</MenuCheckboxItem>
                        <MenuCheckboxItem disabled>Trust bonus</MenuCheckboxItem>
                    </MenuGroup>
                    <MenuSeparator />
                    <MenuGroup>
                        <MenuGroupLabel>Sort by</MenuGroupLabel>
                        <MenuRadioGroup defaultValue="rarity">
                            <MenuRadioItem value="rarity">Rarity</MenuRadioItem>
                            <MenuRadioItem value="name">Name</MenuRadioItem>
                            <MenuRadioItem value="release">Release date</MenuRadioItem>
                        </MenuRadioGroup>
                    </MenuGroup>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);

export const InsetHeaders = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Stage filter</span>
            <Menu modal={false} open>
                <MenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <FilterIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Filters · 2 active</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </MenuTrigger>
                <MenuPopup align="start" className="w-64" sideOffset={6}>
                    <MenuGroup>
                        <MenuGroupLabel inset>Chapter</MenuGroupLabel>
                        <MenuCheckboxItem defaultChecked>Chapter 8 — Roaring Flare</MenuCheckboxItem>
                        <MenuCheckboxItem>Chapter 9 — Stormwatch</MenuCheckboxItem>
                    </MenuGroup>
                    <MenuSeparator />
                    <MenuGroup>
                        <MenuGroupLabel inset>Sanity cost</MenuGroupLabel>
                        <MenuCheckboxItem defaultChecked>18 or less</MenuCheckboxItem>
                        <MenuCheckboxItem>21 and above</MenuCheckboxItem>
                    </MenuGroup>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);
