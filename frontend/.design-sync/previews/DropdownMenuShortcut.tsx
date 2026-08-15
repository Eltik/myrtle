import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuTrigger } from "frontend";
import { ChevronDownIcon, CopyIcon, DownloadIcon, PlusIcon, SearchIcon, SquarePenIcon, TargetIcon, TrashIcon } from "lucide-react";

export const PlannerShortcuts = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Material planner</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <TargetIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Plan</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-64" sideOffset={6}>
                    <DropdownMenuItem>
                        <PlusIcon />
                        Add operator goal
                        <DropdownMenuShortcut>⌘K</DropdownMenuShortcut>
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <CopyIcon />
                        Duplicate plan
                        <DropdownMenuShortcut>⌘D</DropdownMenuShortcut>
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <DownloadIcon />
                        Export material list
                        <DropdownMenuShortcut>⇧⌘E</DropdownMenuShortcut>
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem variant="destructive">
                        <TrashIcon />
                        Clear all goals
                        <DropdownMenuShortcut>⌫</DropdownMenuShortcut>
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
);

export const SearchShortcuts = () => (
    <div className="min-h-[60vh] w-full max-w-2xl">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-card/60 px-3 py-2">
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Quick jump</span>
            <DropdownMenu modal={false} open>
                <DropdownMenuTrigger className="inline-flex h-8 cursor-default items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none">
                    <span className="flex min-w-0 items-center gap-2">
                        <SearchIcon className="h-3.5 w-3.5 opacity-70" />
                        <span className="truncate">Go to</span>
                    </span>
                    <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" className="w-64" sideOffset={6}>
                    <DropdownMenuItem>
                        Operator index
                        <DropdownMenuShortcut>⌘1</DropdownMenuShortcut>
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        Stage browser
                        <DropdownMenuShortcut>⌘2</DropdownMenuShortcut>
                    </DropdownMenuItem>
                    <DropdownMenuItem>
                        <SquarePenIcon />
                        New tier list
                        <DropdownMenuShortcut>⇧⌘N</DropdownMenuShortcut>
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    </div>
);
