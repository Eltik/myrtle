import { Avatar, AvatarFallback, AvatarImage, Menu, MenuItem, MenuPopup, MenuSeparator, MenuTrigger } from "frontend";
import { BookmarkIcon, ChevronDownIcon, Cog, CopyIcon, ExternalLinkIcon, HistoryIcon, LayoutGridIcon, LayoutList, LogOut, MoreHorizontalIcon, PencilIcon, ShieldIcon, TrashIcon, CheckIcon } from "lucide-react";

export const IconButton = () => (
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

export const LabelledButton = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Version</span>
            <Menu modal={false} open>
                <MenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <HistoryIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Latest (live)</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </MenuTrigger>
                <MenuPopup align="start" className="w-80" sideOffset={6}>
                    <MenuItem className="flex-col items-start gap-0.5 py-2">
                        <span className="flex w-full items-center gap-2">
                            <span className="h-1.5 w-1.5 rounded-full bg-primary" />
                            <span className="font-medium">Latest (live)</span>
                            <CheckIcon className="ml-auto h-3.5 w-3.5" />
                        </span>
                        <span className="pl-3.5 font-mono text-[10.5px] text-muted-foreground">Always reflects the most recent edits</span>
                    </MenuItem>
                    <MenuItem className="flex-col items-start gap-0.5 py-2">
                        <span className="flex w-full items-center gap-2">
                            <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground" />
                            <span className="font-medium tabular-nums">v12</span>
                            <span className="font-mono text-[10.5px] text-muted-foreground">3 days ago</span>
                        </span>
                        <span className="wrap-anywhere line-clamp-2 pl-3.5 font-sans text-[11.5px] text-muted-foreground leading-snug">Moved Mlynar to S+ after the Babel banner; folded Guard and Lord into one tier.</span>
                    </MenuItem>
                    <MenuItem className="flex-col items-start gap-0.5 py-2">
                        <span className="flex w-full items-center gap-2">
                            <span className="h-1.5 w-1.5 rounded-full bg-muted-foreground" />
                            <span className="font-medium tabular-nums">v11</span>
                            <span className="font-mono text-[10.5px] text-muted-foreground">2 weeks ago</span>
                        </span>
                        <span className="wrap-anywhere line-clamp-2 pl-3.5 font-sans text-[11.5px] text-muted-foreground leading-snug">Added module X ratings for every 6★ Guard.</span>
                    </MenuItem>
                </MenuPopup>
            </Menu>
        </div>
    </div>
);

export const RenderedAsSplitButton = () => (
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
