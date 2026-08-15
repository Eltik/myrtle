import { Command, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, Kbd, OperatorAvatar } from "frontend";
import { MapIcon, PackageIcon } from "lucide-react";

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

/** The input alone — the top row of the palette, before anything is typed. */
export const SearchField = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
        </Command>
    </Shell>
);

export const WithQuery = () => (
    <Shell>
        <Command defaultValue="mlyn" mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Operators</CommandGroupLabel>
                        <OperatorRow op={OPERATORS[0]} />
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
        </Command>
    </Shell>
);

export const InPalette = () => (
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

/** Custom placeholder for a scoped palette — one of the planner's pickers. */
export const ScopedPlaceholder = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Add an operator to the planner…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Not yet planned</CommandGroupLabel>
                        {OPERATORS.map((op) => (
                            <OperatorRow key={op.id} op={op} />
                        ))}
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
        </Command>
    </Shell>
);
