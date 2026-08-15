import { Command, CommandDialog, CommandDialogPopup, CommandFooter, CommandGroup, CommandGroupLabel, CommandInput, CommandItem, CommandList, CommandPanel, CommandSeparator, Kbd, OperatorAvatar } from "frontend";
import { MapIcon } from "lucide-react";

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
            {OPERATORS.slice(0, 4).map((op) => (
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

/** The viewport parks the palette near the top of the screen, not centred. */
export const TopAligned = () => (
    <div>
        <PageBehind />
        <CommandDialog defaultOpen>
            <CommandDialogPopup>
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
                            </CommandGroup>
                        </CommandList>
                    </CommandPanel>
                    <Hints />
                </Command>
            </CommandDialogPopup>
        </CommandDialog>
    </div>
);

/** A long result set — the popup stops at its max height inside the viewport. */
export const TallResults = () => (
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
                                <CommandGroupLabel>Stages</CommandGroupLabel>
                                <CommandItem className="flex flex-row gap-2" value="stage:s4-1">
                                    <MapIcon className="size-4 text-muted-foreground" />
                                    <span className="flex-1">S4-1 — Roaring Flare</span>
                                    <span className="text-muted-foreground text-xs">18 sanity</span>
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
