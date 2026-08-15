import { Command, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { CalculatorIcon, ChartColumnIcon, HistoryIcon, MapIcon, TrophyIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
];

const Shell = ({ children }: { children?: React.ReactNode }) => <div className="mx-auto flex w-full max-w-xl flex-col rounded-2xl border bg-popover text-popover-foreground shadow-lg">{children}</div>;

const OperatorRow = ({ op }: { op: (typeof OPERATORS)[number] }) => (
    <CommandItem className="flex flex-row gap-2" value={`operator:${op.id}`}>
        <span aria-hidden="true" className="op-chip">
            <OperatorAvatar charId={op.id} name={op.name} />
        </span>
        <span className="flex-1 font-medium">{op.name}</span>
        <span className="text-muted-foreground text-xs">{op.meta}</span>
    </CommandItem>
);

const Hints = () => (
    <CommandFooter>
        <span className="flex items-center gap-1">
            <Kbd>↑</Kbd>
            <Kbd>↓</Kbd> to navigate
        </span>
        <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
    </CommandFooter>
);

/** One rule between the operator hits and the page hits. */
export const BetweenGroups = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Operators</CommandGroupLabel>
                        {OPERATORS.map((op) => (
                            <OperatorRow key={op.id} op={op} />
                        ))}
                    </CommandGroup>
                    <CommandSeparator />
                    <CommandGroup>
                        <CommandGroupLabel>Pages</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="page:leaderboard">
                            <TrophyIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Leaderboard</span>
                            <span className="text-muted-foreground text-xs">Top Doctors by score</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

/** All three sections of the search palette, rules between each. */
export const ThreeSections = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Operators</CommandGroupLabel>
                        {OPERATORS.map((op) => (
                            <OperatorRow key={op.id} op={op} />
                        ))}
                    </CommandGroup>
                    <CommandSeparator />
                    <CommandGroup>
                        <CommandGroupLabel>Pages</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="page:stages">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Stages</span>
                            <span className="text-muted-foreground text-xs">Enemy-pathing simulator</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="page:gacha-history">
                            <HistoryIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Gacha History</span>
                            <span className="text-muted-foreground text-xs">Pulls, rarity splits, pity</span>
                        </CommandItem>
                    </CommandGroup>
                    <CommandSeparator />
                    <CommandGroup>
                        <CommandGroupLabel>Tools</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="tool:dps">
                            <ChartColumnIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">DPS charts</span>
                            <span className="text-muted-foreground text-xs">Damage curves per skill</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="tool:recruitment">
                            <CalculatorIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Recruitment calculator</span>
                            <span className="text-muted-foreground text-xs">Guaranteed tag combos</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);
