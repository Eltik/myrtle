import { Command, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { ChartColumnIcon, MapIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1035_wisdel", name: "Wiš'adel", meta: "6★ · Sniper" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
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

/** Plain section labels — one per result category. */
export const SectionLabels = () => (
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
                        <CommandGroupLabel>Tools</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="tool:dps">
                            <ChartColumnIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">DPS charts</span>
                            <span className="text-muted-foreground text-xs">Damage curves per skill</span>
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

/** Label carrying a match count on the trailing edge. */
export const WithCount = () => (
    <Shell>
        <Command defaultValue="mlyn" mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel className="flex items-center justify-between">
                            <span>Operators</span>
                            <span className="font-mono text-muted-foreground/60">3 of 312</span>
                        </CommandGroupLabel>
                        {OPERATORS.map((op) => (
                            <OperatorRow key={op.id} op={op} />
                        ))}
                    </CommandGroup>
                    <CommandSeparator />
                    <CommandGroup>
                        <CommandGroupLabel className="flex items-center justify-between">
                            <span>Stages</span>
                            <span className="font-mono text-muted-foreground/60">1 of 1,204</span>
                        </CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="stage:s4-1">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">S4-1 — Roaring Flare</span>
                            <span className="text-muted-foreground text-xs">18 sanity</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
        </Command>
    </Shell>
);
