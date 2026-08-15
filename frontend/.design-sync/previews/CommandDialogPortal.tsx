import { Command, CommandDialog, CommandDialogPopup, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { CalculatorIcon, MapIcon } from "lucide-react";

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_1012_skadi2", name: "Skadi the Corrupting Heart", meta: "6★ · Supporter" },
    { id: "char_249_mlyss", name: "Muelsyse", meta: "6★ · Vanguard" },
    { id: "char_1028_texas2", name: "Texas the Omertosa", meta: "6★ · Specialist" },
];

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
        <span className="ml-auto font-mono text-muted-foreground/60">powered by COSS UI</span>
    </CommandFooter>
);

/**
 * The palette is portalled out of the page it was declared in, so the operator
 * grid's stacking context can never clip it.
 */
export const AbovePageContent = () => (
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
                                    <CommandItem className="flex flex-row gap-2" key={op.id} value={`operator:${op.id}`}>
                                        <span aria-hidden="true" className="op-chip">
                                            <OperatorAvatar charId={op.id} name={op.name} />
                                        </span>
                                        <span className="flex-1 font-medium">{op.name}</span>
                                        <span className="text-muted-foreground text-xs">{op.meta}</span>
                                    </CommandItem>
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
                    <Hints />
                </Command>
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);

/** `portalProps` reaches the portal through the popup — kept mounted so the
 * stage index isn't re-fetched every time the palette closes. */
export const KeepMounted = () => (
    <div>
        <PageBehind />
        <CommandDialog defaultOpen>
            <CommandDialogPopup portalProps={{ keepMounted: true }}>
                <Command defaultValue="s4" mode="none">
                    <CommandInput placeholder="Jump to a stage…" />
                    <CommandPanel>
                        <CommandList>
                            <CommandGroup>
                                <CommandGroupLabel>Stages</CommandGroupLabel>
                                <CommandItem className="flex flex-row gap-2" value="stage:s4-1">
                                    <MapIcon className="size-4 text-muted-foreground" />
                                    <span className="flex-1">S4-1 — Roaring Flare</span>
                                    <span className="text-muted-foreground text-xs">18 sanity · Chip Catalyst</span>
                                </CommandItem>
                                <CommandItem className="flex flex-row gap-2" value="stage:s4-6">
                                    <MapIcon className="size-4 text-muted-foreground" />
                                    <span className="flex-1">S4-6 — Ashen Trail</span>
                                    <span className="text-muted-foreground text-xs">21 sanity · Grindstone</span>
                                </CommandItem>
                                <CommandItem className="flex flex-row gap-2" value="tool:sanity">
                                    <CalculatorIcon className="size-4 text-muted-foreground" />
                                    <span className="flex-1">Sanity calculator</span>
                                    <span className="text-muted-foreground text-xs">Refresh and drop planning</span>
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
