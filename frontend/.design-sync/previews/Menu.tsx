import { Avatar, AvatarFallback, AvatarImage, Menu, MenuGroup, MenuGroupLabel, MenuCheckboxItem, MenuItem, MenuPopup, MenuRadioGroup, MenuRadioItem, MenuSeparator, MenuTrigger } from "frontend";
import { BookmarkIcon, ChevronDownIcon, Cog, CopyIcon, ExternalLinkIcon, LayoutGridIcon, LayoutList, LogOut, MoreHorizontalIcon, PencilIcon, ShieldIcon, SlidersHorizontalIcon, TrashIcon } from "lucide-react";

export const RowActions = () => (
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

export const AccountMenu = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-end">
            <Menu modal={false} open>
                <div className="flex h-8 min-w-0 shrink-0 items-center rounded-md border border-border bg-transparent text-foreground text-sm">
                    <span className="flex h-full min-w-0 items-center gap-2 rounded-l-md px-2">
                        <Avatar className="h-5 w-5">
                            <AvatarImage alt="User avatar" src="https://api.myrtle.moe/api/avatar/char_102_texas" />
                            <AvatarFallback className="text-[0.625rem]">K</AvatarFallback>
                        </Avatar>
                        <span className="max-w-24 truncate font-medium">Dr. Kal'tsit</span>
                    </span>
                    <MenuTrigger
                        render={(triggerProps: any) => (
                            <button {...triggerProps} aria-label="Open user menu" className="flex h-full cursor-default items-center rounded-r-md border-border border-l px-1.5" type="button">
                                <ChevronDownIcon className="h-3 w-3" />
                            </button>
                        )}
                    />
                </div>
                <MenuPopup align="end" className="w-56" sideOffset={6}>
                    <div className="px-2 pb-1.5">
                        <span className="font-medium text-sm">Dr. Kal'tsit</span>
                        <p className="m-0 text-muted-foreground text-xs">Level 120 · Rhodes Island</p>
                    </div>
                    <MenuSeparator />
                    <MenuItem>
                        <LayoutList className="text-muted-foreground" />
                        My tier lists
                    </MenuItem>
                    <MenuItem>
                        <BookmarkIcon className="text-muted-foreground" />
                        Saved operators
                    </MenuItem>
                    <MenuItem>
                        <Cog className="text-muted-foreground" />
                        Settings
                    </MenuItem>
                    <MenuItem>
                        <ShieldIcon className="text-primary" />
                        Admin panel
                    </MenuItem>
                    <MenuSeparator />
                    <MenuItem className="text-primary">
                        <LogOut />
                        Log out
                    </MenuItem>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);

export const ViewOptions = () => (
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
