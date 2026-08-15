import { Command, CommandCollection, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, Kbd, OperatorAvatar } from "frontend";
import { CalculatorIcon, MapIcon } from "lucide-react";

interface IOperator {
    id: string;
    name: string;
    meta: string;
}

const OPERATORS: IOperator[] = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
];

const GROUPED = [
    { value: "Stages", items: [{ id: "1-7", label: "1-7 — Amid the Chaos", desc: "6 sanity · LMD" }, { id: "s4-1", label: "S4-1 — Roaring Flare", desc: "18 sanity · Chip Catalyst" }] },
    { value: "Tools", items: [{ id: "recruitment", label: "Recruitment calculator", desc: "Guaranteed tag combos" }, { id: "sanity", label: "Sanity calculator", desc: "Refresh and drop planning" }] },
];

const Shell = ({ children }: { children?: React.ReactNode }) => <div className="mx-auto flex w-full max-w-xl flex-col rounded-2xl border bg-popover text-popover-foreground shadow-lg">{children}</div>;

const Hints = () => (
    <CommandFooter>
        <span className="flex items-center gap-1">
            <Kbd>↑</Kbd>
            <Kbd>↓</Kbd> to navigate
        </span>
        <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
    </CommandFooter>
);

/** A flat `items` array rendered through one collection. */
export const FlatItems = () => (
    <Shell>
        <Command items={OPERATORS} mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Operators</CommandGroupLabel>
                        <CommandCollection>
                            {(op: IOperator) => (
                                <CommandItem className="flex flex-row gap-2" key={op.id} value={op}>
                                    <span aria-hidden="true" className="op-chip">
                                        <OperatorAvatar charId={op.id} name={op.name} />
                                    </span>
                                    <span className="flex-1 font-medium">{op.name}</span>
                                    <span className="text-muted-foreground text-xs">{op.meta}</span>
                                </CommandItem>
                            )}
                        </CommandCollection>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <Hints />
        </Command>
    </Shell>
);

/** Grouped `items` — the list renders groups, each collection its own rows. */
export const GroupedItems = () => (
    <Shell>
        <Command items={GROUPED} mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    {(group: (typeof GROUPED)[number]) => (
                        <CommandGroup items={group.items} key={group.value}>
                            <CommandGroupLabel>{group.value}</CommandGroupLabel>
                            <CommandCollection>
                                {(item: (typeof GROUPED)[number]["items"][number]) => (
                                    <CommandItem className="flex flex-row gap-2" key={item.id} value={item}>
                                        {group.value === "Stages" ? <MapIcon className="size-4 text-muted-foreground" /> : <CalculatorIcon className="size-4 text-muted-foreground" />}
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
