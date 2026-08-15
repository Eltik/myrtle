import { Command, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, CommandShortcut, Kbd, OperatorAvatar } from "frontend";
import { CalculatorIcon, ChartColumnIcon, HistoryIcon, ListTodoIcon, MapIcon, PackageIcon, TrophyIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
];

const Shell = ({ children }: { children?: React.ReactNode }) => <div className="mx-auto flex w-full max-w-xl flex-col rounded-2xl border bg-popover text-popover-foreground shadow-lg">{children}</div>;

const Hints = () => (
    <CommandFooter>
        <span className="flex items-center gap-1">
            <Kbd>↵</Kbd> to select
        </span>
        <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
    </CommandFooter>
);

/** Operator rows — avatar chip, name, rarity and class on the trailing edge. */
export const OperatorRows = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Operators</CommandGroupLabel>
                        {OPERATORS.map((op) => (
                            <CommandItem className="flex flex-row gap-2" key={op.id} value={`operator:${op.id}`}>
                                <span aria-hidden="true" className="op-chip">
                                    <OperatorAvatar charId={op.id} name={op.name} />
                                </span>
                                <span className="flex-1 font-medium">{op.name}</span>
                                <span className="text-muted-foreground text-xs">{op.meta}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

/** Navigation rows — icon, label, description. */
export const NavigationRows = () => (
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
                            <span className="text-muted-foreground text-xs">Stats, skills, modules</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="page:stages">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Stages</span>
                            <span className="text-muted-foreground text-xs">Enemy-pathing simulator</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="page:leaderboard">
                            <TrophyIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Leaderboard</span>
                            <span className="text-muted-foreground text-xs">Top Doctors by score</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="page:gacha-history">
                            <HistoryIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Gacha History</span>
                            <span className="text-muted-foreground text-xs">Pulls, rarity splits, pity</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

/** Rows carrying their own keyboard shortcut. */
export const WithShortcuts = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
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
                        <CommandItem className="flex flex-row gap-2" value="tool:recruitment">
                            <CalculatorIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Recruitment calculator</span>
                            <CommandShortcut>⌘R</CommandShortcut>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

/** A disabled row — the operator is already in the plan. */
export const Disabled = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Add an operator to the planner…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Owned</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="operator:char_4064_mlynar">
                            <span aria-hidden="true" className="op-chip">
                                <OperatorAvatar charId="char_4064_mlynar" name="Młynar" />
                            </span>
                            <span className="flex-1 font-medium">Młynar</span>
                            <span className="text-muted-foreground text-xs">6★ · Guard</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" disabled value="operator:char_1012_skadi2">
                            <span aria-hidden="true" className="op-chip">
                                <OperatorAvatar charId="char_1012_skadi2" name="Skadi the Corrupting Heart" />
                            </span>
                            <span className="flex-1 font-medium">Skadi the Corrupting Heart</span>
                            <span className="text-muted-foreground text-xs">Already planned</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" disabled value="operator:char_249_mlyss">
                            <span aria-hidden="true" className="op-chip">
                                <OperatorAvatar charId="char_249_mlyss" name="Muelsyse" />
                            </span>
                            <span className="flex-1 font-medium">Muelsyse</span>
                            <span className="text-muted-foreground text-xs">Already planned</span>
                        </CommandItem>
                    </CommandGroup>
                    <CommandSeparator />
                    <CommandGroup>
                        <CommandGroupLabel>Not owned</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" disabled value="operator:char_1035_wisdel">
                            <span aria-hidden="true" className="op-chip">
                                <OperatorAvatar charId="char_1035_wisdel" name="Wiš'adel" />
                            </span>
                            <span className="flex-1 font-medium">Wiš'adel</span>
                            <span className="text-muted-foreground text-xs">Not recruited</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);
