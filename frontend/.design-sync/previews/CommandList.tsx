import { Command, CommandEmpty, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { CalculatorIcon, ChartColumnIcon, ListTodoIcon, MapIcon, PackageIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
    { id: "char_1035_wisdel", name: "Wiš'adel", meta: "6★ · Sniper" },
    { id: "char_4133_logos", name: "Logos", meta: "6★ · Caster" },
    { id: "char_179_cgbird", name: "Nightingale", meta: "6★ · Medic" },
    { id: "char_293_thorns", name: "Thorns", meta: "6★ · Guard" },
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

export const Results = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Operators</CommandGroupLabel>
                        {OPERATORS.slice(0, 3).map((op) => (
                            <OperatorRow key={op.id} op={op} />
                        ))}
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

const STAGES = [
    { code: "1-7 — Amid the Chaos", meta: "6 sanity · LMD farming" },
    { code: "S4-1 — Roaring Flare", meta: "18 sanity · Chip Catalyst" },
    { code: "S4-6 — Ashen Trail", meta: "21 sanity · Grindstone" },
    { code: "4-8 — Cliffside Watch", meta: "18 sanity · Manganese Ore" },
    { code: "6-16 — Lungmen Downtown", meta: "21 sanity · Orirock Cluster" },
    { code: "7-18 — Frozen Bastion", meta: "21 sanity · RMA70-12" },
    { code: "10-8 — Twilight Wharf", meta: "21 sanity · Crystalline Circuit" },
    { code: "12-17 — Sea of Ashes", meta: "21 sanity · Bipolar Nanoflake" },
];

/** A long result set — the list scrolls inside the panel's fixed height. */
export const Scrollable = () => (
    <Shell>
        <Command defaultValue="sanity" mode="none">
            <CommandInput placeholder="Jump to a stage…" />
            <CommandPanel className="min-h-0" style={{ height: 240 }}>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Stages</CommandGroupLabel>
                        {STAGES.map((stage) => (
                            <CommandItem className="flex flex-row gap-2" key={stage.code} value={`stage:${stage.code}`}>
                                <MapIcon className="size-4 text-muted-foreground" />
                                <span className="flex-1">{stage.code}</span>
                                <span className="text-muted-foreground text-xs">{stage.meta}</span>
                            </CommandItem>
                        ))}
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

export const Empty = () => (
    <Shell>
        <Command defaultValue="annihilation 12" items={[]} mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandEmpty>No results found.</CommandEmpty>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

export const PagesOnly = () => (
    <Shell>
        <Command defaultValue="pl" mode="none">
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
                        <CommandItem className="flex flex-row gap-2" value="tool:planner">
                            <ListTodoIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Operator planner</span>
                            <span className="text-muted-foreground text-xs">Promotions, skills, modules</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);
