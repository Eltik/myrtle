import { Command, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, Kbd, OperatorAvatar } from "frontend";
import { MapIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
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

const Results = () => (
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
);

/** The shipped footer: navigation keys, select key, attribution. */
export const KeyHints = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <Results />
            <CommandFooter>
                <span className="flex items-center gap-1">
                    <Kbd>↑</Kbd>
                    <Kbd>↓</Kbd> to navigate
                </span>
                <span className="flex items-center gap-1">
                    <Kbd>↵</Kbd> to select
                </span>
                <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
            </CommandFooter>
        </Command>
    </Shell>
);

/** Footer as a status bar — match count and the key that closes the palette. */
export const ResultCount = () => (
    <Shell>
        <Command defaultValue="mlyn" mode="none">
            <CommandInput placeholder="Search operators, pages, tools…" />
            <Results />
            <CommandFooter>
                <span className="font-mono">3 of 312 operators</span>
                <span className="ml-auto flex items-center gap-1">
                    <Kbd>esc</Kbd> to close
                </span>
            </CommandFooter>
        </Command>
    </Shell>
);

/** Scoped picker footer — what the highlighted row will do. */
export const ScopedAction = () => (
    <Shell>
        <Command mode="none">
            <CommandInput placeholder="Jump to a stage…" />
            <CommandPanel>
                <CommandList>
                    <CommandGroup>
                        <CommandGroupLabel>Stages</CommandGroupLabel>
                        <CommandItem className="flex flex-row gap-2" value="stage:s4-1">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">S4-1 — Roaring Flare</span>
                            <span className="text-muted-foreground text-xs">18 sanity</span>
                        </CommandItem>
                        <CommandItem className="flex flex-row gap-2" value="stage:1-7">
                            <MapIcon className="size-4 text-muted-foreground" />
                            <span className="flex-1">1-7 — Amid the Chaos</span>
                            <span className="text-muted-foreground text-xs">6 sanity</span>
                        </CommandItem>
                    </CommandGroup>
                </CommandList>
            </CommandPanel>
            <CommandFooter>
                <span className="flex items-center gap-1">
                    <Kbd>↵</Kbd> opens the pathing simulator
                </span>
                <span className="ml-auto font-mono text-muted-foreground/60">2 stages</span>
            </CommandFooter>
        </Command>
    </Shell>
);
