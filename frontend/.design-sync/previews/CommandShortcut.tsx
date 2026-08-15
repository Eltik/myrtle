import { Command, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, CommandShortcut, Kbd } from "frontend";
import { CalculatorIcon, ChartColumnIcon, HistoryIcon, ListTodoIcon, MapIcon, PackageIcon, TrophyIcon } from "lucide-react";

const Shell = ({ children }: { children?: React.ReactNode }) => <div className="mx-auto flex w-full max-w-xl flex-col rounded-2xl border bg-popover text-popover-foreground shadow-lg">{children}</div>;

/** Shortcut hints pinned to the trailing edge of each row. */
export const QuickActions = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Pages</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="page:operators">
                            <PackageIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Operators</span>
                            <CommandShortcut>⌘1</CommandShortcut>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="page:stages">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Stages</span>
                            <CommandShortcut>⌘2</CommandShortcut>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="page:leaderboard">
                            <TrophyIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Leaderboard</span>
                            <CommandShortcut>⌘3</CommandShortcut>
                        </CommandItem>
                    </CommandGroup>
                    <CommandSeparator />
                    <CommandGroup>
                        <CommandGroupLabel>Tools</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="tool:planner">
                            <ListTodoIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Operator planner</span>
                            <CommandShortcut>⌘P</CommandShortcut>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="tool:dps">
                            <ChartColumnIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">DPS charts</span>
                            <CommandShortcut>⌘D</CommandShortcut>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <CommandFooter>
                <span className="flex items-center gap-1">
                    <Kbd>↵</Kbd> to select
                </span>
                <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
            </CommandFooter>
        </Command>
    </Shell>
);

/** Word shortcuts rather than chords — recent stages replayed by code. */
export const RecentStages = () => (
    <Shell>
        <Command defaultValue="s4" mode="none">
            <CommandInput placeholder="Jump to a stage…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Recent</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="stage:s4-1">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">S4-1 — Roaring Flare</span>
                            <CommandShortcut>18 sanity</CommandShortcut>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="stage:1-7">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">1-7 — Amid the Chaos</span>
                            <CommandShortcut>6 sanity</CommandShortcut>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="tool:sanity">
                            <CalculatorIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Sanity calculator</span>
                            <CommandShortcut>⌘S</CommandShortcut>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="page:gacha-history">
                            <HistoryIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Gacha History</span>
                            <CommandShortcut>⌘H</CommandShortcut>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
        </Command>
    </Shell>
);
