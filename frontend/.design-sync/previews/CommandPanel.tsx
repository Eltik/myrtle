import { Command, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { CalculatorIcon, MapIcon, PackageIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
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

/** Panel followed by a footer: the panel keeps a square bottom edge. */
export const WithFooter = () => (
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

/** Last child in the palette: the panel rounds off its own bottom corners. */
export const WithoutFooter = () => (
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
                </CommandList>
            </CommandPanel>
        </Command>
    </Shell>
);

/** A fixed-height panel — the results scroll, the input and footer do not. */
export const FixedHeight = () => (
    <Shell>
        <Command defaultValue="calc" mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel style={{ height: 180 }}>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Tools</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="tool:recruitment">
                            <CalculatorIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Recruitment calculator</span>
                            <span className="text-muted-foreground text-xs">Guaranteed tag combos</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="tool:sanity">
                            <PackageIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">Sanity calculator</span>
                            <span className="text-muted-foreground text-xs">Refresh and drop planning</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <CommandFooter>
                <span className="flex items-center gap-1">
                    <Kbd>esc</Kbd> to close
                </span>
                <span className="ml-auto font-mono text-muted-foreground/60">2 of 14 tools</span>
            </CommandFooter>
        </Command>
    </Shell>
);
