import { Badge, Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import type { ReactNode } from "react";

// The backdrop dims and blurs the page behind the sheet. `SheetPopup` mounts
// one for you, so it is only legible over real content — that is the stage
// these stories build.
const StageBehind = ({ children }: { children?: ReactNode }) => (
    <div className="relative min-h-[520px] w-full">
        <div className="flex flex-col gap-4 p-1">
            <div className="flex items-baseline justify-between gap-3">
                <h2 className="font-heading font-semibold text-xl">Chapter 8 — Roaring Flare</h2>
                <span className="font-mono text-muted-foreground text-xs tabular-nums">14 stages</span>
            </div>
            <div className="grid gap-2">
                {[
                    { code: "8-7", name: "Ashes", sanity: 21 },
                    { code: "8-9", name: "Dust Settles", sanity: 21 },
                    { code: "8-11", name: "Endless Night", sanity: 21 },
                    { code: "8-14", name: "Roaring Flare", sanity: 25 },
                    { code: "S8-2", name: "Gravel Road", sanity: 18 },
                ].map((s) => (
                    <div className="flex items-center gap-3 rounded-lg border bg-card px-3 py-2.5" key={s.code}>
                        <span className="font-mono font-semibold text-sm tabular-nums">{s.code}</span>
                        <span className="min-w-0 flex-1 truncate text-sm">{s.name}</span>
                        <Badge variant="secondary">{s.sanity} sanity</Badge>
                    </div>
                ))}
            </div>
        </div>
        {children}
    </div>
);

// Right side, inset — the backdrop covers the whole page and the panel floats
// inside the safe area.
export const BehindInsetSheet = () => (
    <StageBehind>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>8-14 — Roaring Flare</SheetTitle>
                    <SheetDescription>25 sanity · first clear drops 2 Chip Catalysts and 1 Bipolar Nanoflake.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { name: "Bipolar Nanoflake", rate: "18.4%" },
                        { name: "Orirock Cluster", rate: "24.1%" },
                        { name: "Manganese Ore", rate: "9.6%" },
                    ].map((drop) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={drop.name}>
                            <span className="truncate text-sm">{drop.name}</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">{drop.rate}</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
                    <Button>Add 12 runs</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </StageBehind>
);

// Flush variant — the panel meets the viewport edge, so only the start half of
// the backdrop stays visible.
export const BehindFlushSheet = () => (
    <StageBehind>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right">
                <SheetHeader>
                    <SheetTitle>Sanity plan</SheetTitle>
                    <SheetDescription>Everything queued for this week's Chapter 8 grind.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { code: "8-14", runs: 12 },
                        { code: "S8-2", runs: 6 },
                        { code: "CE-6", runs: 4 },
                    ].map((row) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={row.code}>
                            <span className="font-mono font-semibold text-sm tabular-nums">{row.code}</span>
                            <span className="text-muted-foreground text-sm">{row.runs} runs</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter variant="bare">
                    <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
                    <Button>Start route</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </StageBehind>
);
