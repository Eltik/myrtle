import { Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import { ChevronLeftIcon } from "lucide-react";
import type { ReactNode } from "react";

// `SheetClose` dismisses the sheet from anywhere inside it. `SheetPopup`
// already mounts one in the top corner; these stories cover the extra ones you
// place yourself.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// The common shape: a confirm/dismiss pair where the dismiss is the close.
export const InFooter = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Filters</SheetTitle>
                    <SheetDescription>37 of 312 operators match the current facets.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {["Nation · Lungmen", "Class · Guard, Caster", "Rarity · 6★"].map((row) => (
                        <div className="rounded-lg border px-3 py-2 text-sm" key={row}>
                            {row}
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <Button variant="ghost">Reset</Button>
                    <SheetClose render={<Button />}>Show 37 results</SheetClose>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// A back-style close in the header, with the corner button turned off so the
// sheet has exactly one exit.
export const AsHeaderBack = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} showCloseButton={false} side="left" variant="inset">
                <SheetHeader className="gap-3">
                    <SheetClose className="self-start" render={<Button size="sm" variant="ghost" />}>
                        <ChevronLeftIcon />
                        All stages
                    </SheetClose>
                    <SheetTitle>8-14 — Roaring Flare</SheetTitle>
                    <SheetDescription>Chapter 8 · 25 sanity · recommended average level E2 40.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { name: "Chip Catalyst", rate: "First clear ×2" },
                        { name: "Bipolar Nanoflake", rate: "18.4%" },
                        { name: "Orirock Cluster", rate: "24.1%" },
                    ].map((drop) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={drop.name}>
                            <span className="truncate text-sm">{drop.name}</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">{drop.rate}</span>
                        </div>
                    ))}
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Both actions dismiss: the destructive one is a plain button that closes on
// submit, the safe one is the close.
export const WithDestructiveSibling = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Clear this tier list?</SheetTitle>
                    <SheetDescription>All 46 placements in "Global 6★ PvE" are removed. The list itself stays.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-0">
                    {[
                        { tier: "S", count: 12 },
                        { tier: "A", count: 18 },
                        { tier: "B", count: 9 },
                        { tier: "C", count: 7 },
                    ].map((t) => (
                        <div className="flex items-center justify-between gap-3 border-b py-2.5 last:border-b-0" key={t.tier}>
                            <span className="font-heading font-semibold text-sm">Tier {t.tier}</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">{t.count} operators</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Keep placements</SheetClose>
                    <Button variant="destructive">Clear list</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);
