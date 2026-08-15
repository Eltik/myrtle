import { Avatar, AvatarFallback, AvatarImage, DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "frontend";
import { BookmarkIcon, ChevronDownIcon, Cog, ExternalLinkIcon, LayoutList, ListFilterIcon, LogOut, MoreHorizontalIcon, PencilIcon, ShieldIcon, TargetIcon, TrashIcon } from "lucide-react";

export const UserAccount = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-end">
            <DropdownMenu modal={false} open>
                <div className="flex h-8 min-w-0 shrink-0 items-center rounded-md border border-border bg-transparent text-foreground text-sm">
                    <span className="flex h-full min-w-0 items-center gap-2 rounded-l-md px-2">
                        <Avatar className="h-5 w-5">
                            <AvatarImage alt="User avatar" src="https://api.myrtle.moe/api/avatar/char_102_texas" />
                            <AvatarFallback className="text-[0.625rem]">K</AvatarFallback>
                        </Avatar>
                        <span className="max-w-24 truncate font-medium">Dr. Kal'tsit</span>
                    </span>
                    <DropdownMenuTrigger
                        render={(triggerProps: any) => (
                            <button {...triggerProps} aria-label="Open user menu" className="flex h-full cursor-default items-center rounded-r-md border-border border-l px-1.5" type="button">
                                <ChevronDownIcon className="h-3 w-3" />
                            </button>
                        )}
                    />
                </div>
                <DropdownMenuContent align="end" className="w-56" sideOffset={6}>
                    <div className="px-2 pb-1.5">
                        <span className="font-medium text-sm">Dr. Kal'tsit</span>
                        <p className="m-0 text-muted-foreground text-xs">Level 120 · Rhodes Island</p>
                    </div>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem>
                        <LayoutList className="text-muted-foreground" />
                        My tier lists
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <BookmarkIcon className="text-muted-foreground" />
                        Saved operators
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <Cog className="text-muted-foreground" />
                        Settings
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <ShieldIcon className="text-primary" />
                        Admin panel
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem className="text-primary">
                        <LogOut />
                        Log out
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
);

export const RosterRowActions = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <article className="flex items-center gap-3 rounded-lg border border-border bg-card px-3 py-2.5">
            <span className="inline-flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-muted">
                <img alt="Skadi" className="h-full w-full object-cover" src="https://api.myrtle.moe/api/avatar/char_263_skadi" />
            </span>
            <div className="min-w-0 flex-1">
                <h3 className="m-0 truncate font-sans font-semibold text-[14px] text-foreground leading-tight tracking-tight">Skadi</h3>
                <p className="m-0 mt-0.5 truncate font-sans text-[11.5px] text-muted-foreground leading-snug">E2 90 · S3 M3 · Trust 200</p>
            </div>
            <span className="shrink-0 font-mono text-[10.5px] text-muted-foreground tabular-nums">added 12 May</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger aria-label="List actions" className="inline-flex h-8 w-8 shrink-0 cursor-default items-center justify-center rounded-md border border-transparent text-muted-foreground">
                    <MoreHorizontalIcon className="h-4 w-4" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="min-w-52" sideOffset={6}>
                    <DropdownMenuItem>
                        <ExternalLinkIcon />
                        Open operator page
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <TargetIcon />
                        Add to E2 plan
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <PencilIcon />
                        Edit levels &amp; skills
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem variant="destructive">
                        <TrashIcon />
                        Remove from roster
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </article>
    </div>
);

export const FilterOptions = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Operator index</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <ListFilterIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Filters · 3 active</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-64" sideOffset={6}>
                    <DropdownMenuGroup>
                        <DropdownMenuLabel>Operator class</DropdownMenuLabel>
                        <DropdownMenuCheckboxItem defaultChecked>Guard</DropdownMenuCheckboxItem>
                        <DropdownMenuCheckboxItem defaultChecked>Sniper</DropdownMenuCheckboxItem>
                        <DropdownMenuCheckboxItem>Caster</DropdownMenuCheckboxItem>
                        <DropdownMenuCheckboxItem disabled>Vanguard — no results</DropdownMenuCheckboxItem>
                    </DropdownMenuGroup>
                    <DropdownMenuSeparator />
                    <DropdownMenuGroup>
                        <DropdownMenuLabel>Availability</DropdownMenuLabel>
                        <DropdownMenuCheckboxItem defaultChecked>Owned by me</DropdownMenuCheckboxItem>
                        <DropdownMenuCheckboxItem>CN-only</DropdownMenuCheckboxItem>
                    </DropdownMenuGroup>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
);
