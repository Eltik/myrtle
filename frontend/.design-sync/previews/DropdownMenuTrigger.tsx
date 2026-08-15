import { Avatar, AvatarFallback, AvatarImage, DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "frontend";
import { BookmarkIcon, ChevronDownIcon, Cog, ListFilterIcon, LayoutList, LogOut, ShieldIcon } from "lucide-react";

export const AvatarSplitButton = () => (
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

export const ToolbarButton = () => (
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
