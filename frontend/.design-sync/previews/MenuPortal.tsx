import { Menu, MenuCheckboxItem, MenuGroup, MenuGroupLabel, MenuItem, MenuPopup, MenuRadioGroup, MenuRadioItem, MenuSeparator, MenuTrigger } from "frontend";
import { ChevronDownIcon, CopyIcon, ExternalLinkIcon, LayoutGridIcon, MoreHorizontalIcon, PencilIcon, SlidersHorizontalIcon, TrashIcon } from "lucide-react";

export const EscapesTheRowClip = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <article className="flex items-center gap-3 rounded-lg border border-border bg-card px-3 py-2.5">
            <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-muted">
                <img alt="Mlynar" className="h-full w-full object-cover" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
            </span>
            <div className="min-w-0 flex-1">
                <h3 className="m-0 truncate font-sans font-semibold text-[14px] text-foreground leading-tight tracking-tight">Global Guard Rankings</h3>
                <p className="m-0 mt-0.5 truncate font-sans text-[11.5px] text-muted-foreground leading-snug">42 operators ranked · updated after Babel</p>
            </div>
            <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">12.4k views</span>
            <Menu modal={false} open>
                <MenuTrigger aria-label="List actions" className="inline-flex h-8 w-8 shrink-0 cursor-default items-center justify-center rounded-md border border-transparent text-muted-foreground">
                    <MoreHorizontalIcon className="h-4 w-4" />
                </MenuTrigger>
                <MenuPopup align="end" className="min-w-44" sideOffset={6}>
                    <MenuItem>
                        <ExternalLinkIcon />
                        Open
                    </MenuItem>
                    <MenuItem>
                        <LayoutGridIcon />
                        Edit tier list
                    </MenuItem>
                    <MenuItem>
                        <PencilIcon />
                        Edit details
                    </MenuItem>
                    <MenuItem>
                        <CopyIcon />
                        Copy share link
                    </MenuItem>
                    <MenuSeparator />
                    <MenuItem variant="destructive">
                        <TrashIcon />
                        Delete
                    </MenuItem>
                </MenuPopup>
            </Menu>
        </article>
    </div>
);

export const OverToolbarContent = () => (
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
