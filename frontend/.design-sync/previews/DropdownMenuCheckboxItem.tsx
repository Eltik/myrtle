import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "frontend";
import { ChevronDownIcon, ListFilterIcon, BellIcon } from "lucide-react";

export const ClassFilters = () => (
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

export const SwitchVariant = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Notifications</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <BellIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Alerts</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-72" sideOffset={6}>
                    <DropdownMenuGroup>
                        <DropdownMenuLabel>Notify me about</DropdownMenuLabel>
                        <DropdownMenuCheckboxItem defaultChecked variant="switch">
                            New banner announcements
                        </DropdownMenuCheckboxItem>
                        <DropdownMenuCheckboxItem defaultChecked variant="switch">
                            Sanity is nearly full
                        </DropdownMenuCheckboxItem>
                        <DropdownMenuCheckboxItem variant="switch">Replies to my tier lists</DropdownMenuCheckboxItem>
                        <DropdownMenuCheckboxItem disabled variant="switch">
                            Weekly roster digest
                        </DropdownMenuCheckboxItem>
                    </DropdownMenuGroup>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
);
