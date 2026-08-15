import { Command, CommandCollection, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { CalculatorIcon, ChartColumnIcon, HistoryIcon, MapIcon, PackageIcon, TrophyIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
];

const GROUPED = [
    { value: "Pages", items: [{ id: "operators", label: "Operators", desc: "Stats, skills, modules" }, { id: "stages", label: "Stages", desc: "Enemy-pathing simulator" }] },
    { value: "Tools", items: [{ id: "dps", label: "DPS charts", desc: "Damage curves per skill" }, { id: "recruitment", label: "Recruitment calculator", desc: "Guaranteed tag combos" }] },
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

/** Three sections — operators, pages, tools — separated the way search does it. */
export const Sections = () => (
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
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

/** A single group — the scoped picker case. */
export const SingleGroup = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Add an operator to the planner…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Owned, not yet planned</CommandGroupLabel>
                        {OPERATORS.map((op) => (
                            <OperatorRow key={op.id} op={op} />
                        ))}
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
        </Command>
    </Shell>
);

/** Groups driven by `items` on the root — each group renders its own collection. */
export const FromItems = () => (
    <Shell>
        <Command items={GROUPED} mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    {(group: (typeof GROUPED)[number]) => (
                        <CommandGroup key={group.value} items={group.items}>
                            <CommandGroupLabel>{group.value}</CommandGroupLabel>
                            <CommandCollection>
                                {(item: (typeof GROUPED)[number]["items"][number]) => (
                                    <CommandItem className="flex flex-row gap-2" key={item.id} value={item}>
                                        {group.value === "Pages" ? <PackageIcon className="size-4 text-muted-foreground" /> : <CalculatorIcon className="size-4 text-muted-foreground" />}
                                        <span className="flex-1">{item.label}</span>
                                        <span className="text-muted-foreground text-xs">{item.desc}</span>
                                    </CommandItem>
                                )}
                            </CommandCollection>
                        </CommandGroup>
                    )}
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

/** Pages only, with a map-heavy result set. */
export const PagesGroup = () => (
    <Shell>
        <Command defaultValue="stage" mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Pages</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="page:stages">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Stages</span>
                            <span className="text-muted-foreground text-xs">Enemy-pathing simulator</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="stage:1-7">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">1-7 — Amid the Chaos</span>
                            <span className="text-muted-foreground text-xs">6 sanity · LMD farming</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="stage:s4-1">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">S4-1 — Roaring Flare</span>
                            <span className="text-muted-foreground text-xs">18 sanity · Chip Catalyst</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);
