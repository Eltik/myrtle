import { Command, CommandDialog, CommandDialogPopup, CommandEmpty, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { ChartColumnIcon, MapIcon, TrophyIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
];

const OperatorRow = ({ op }: { op: (typeof OPERATORS)[number] }) => (
    <CommandItem className="flex flex-row gap-2" value={`operator:${op.id}`}>
        <span aria-hidden="true" className="op-chip">
            <OperatorAvatar charId={op.id} name={op.name} />
        </span>
        <span className="flex-1 font-medium">{op.name}</span>
        <span className="text-muted-foreground text-xs">{op.meta}</span>
    </CommandItem>
);

const PageBehind = () => (
    <div className="mx-auto w-full max-w-2xl">
        <div className="flex items-center justify-between border-b pb-3">
            <span className="font-semibold text-lg">Operators</span>
            <span className="text-muted-foreground text-sm">312 operators · EN server</span>
        </div>
        <div className="mt-4 grid grid-cols-4 gap-3">
            {OPERATORS.map((op) => (
                <div className="rounded-lg border bg-card p-3" key={op.id}>
                    <span aria-hidden="true" className="op-chip">
                        <OperatorAvatar charId={op.id} name={op.name} />
                    </span>
                    <div className="mt-2 truncate font-medium text-sm">{op.name}</div>
                    <div className="text-muted-foreground text-xs">{op.meta}</div>
                </div>
            ))}
        </div>
    </div>
);

const Hints = () => (
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
);

/** The palette open over the page — how ⌘K renders across the site. */
export const Open = () => (
    <div>
        <PageBehind />
        <CommandDialog defaultOpen>
            <CommandDialogPopup>
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
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);

/** Query with no hits — the popup keeps its height floor. */
export const NoResults = () => (
    <div>
        <PageBehind />
        <CommandDialog defaultOpen>
            <CommandDialogPopup>
                <Command defaultValue="wisadel e3" items={[]} mode="none">
                    <CommandInput placeholder="Search operators, pages, tools…" />
                    <CommandPanel>
                        <CommandList>
                            <CommandEmpty>No results found.</CommandEmpty>
                        </CommandList>
                    </CommandPanel>
                    <Hints />
                </Command>
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);

/** A scoped palette — the planner's operator picker, no footer. */
export const ScopedPicker = () => (
    <div>
        <PageBehind />
        <CommandDialog defaultOpen>
            <CommandDialogPopup>
                <Command mode="none">
                    <CommandInput placeholder="Add an operator to the planner…" />
                    <CommandPanel>
                        <CommandList>
                            <CommandGroup>
                                <CommandGroupLabel>Owned, not yet planned</CommandGroupLabel>
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
                            </CommandGroup>
                        </CommandList>
                    </CommandPanel>
                </Command>
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);
