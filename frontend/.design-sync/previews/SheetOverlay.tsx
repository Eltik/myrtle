import { Badge, Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import type { ReactNode } from "react";

// `SheetOverlay` is the shadcn-compatible alias for `SheetBackdrop`. Every
// `SheetPopup` mounts one, so these stories show it over the page it dims.
const DepotBehind = ({ children }: { children?: ReactNode }) => (
    <div className="relative min-h-[520px] w-full">
        <div className="flex flex-col gap-4 p-1">
            <div className="flex items-baseline justify-between gap-3">
                <h2 className="font-heading font-semibold text-xl">Depot</h2>
                <Badge variant="secondary">187 material types</Badge>
            </div>
            <div className="grid grid-cols-3 gap-2">
                {[
                    { name: "Orirock Cube", qty: 184 },
                    { name: "Bipolar Nanoflake", qty: 12 },
                    { name: "D32 Steel", qty: 7 },
                    { name: "Polymerization Prep.", qty: 9 },
                    { name: "Crystalline Circuit", qty: 21 },
                    { name: "Grindstone Pentahydrate", qty: 4 },
                ].map((i) => (
                    <div className="flex flex-col gap-1 rounded-lg border bg-card px-3 py-2.5" key={i.name}>
                        <span className="truncate text-sm">{i.name}</span>
                        <span className="font-mono font-semibold text-lg tabular-nums">{i.qty}</span>
                    </div>
                ))}
            </div>
        </div>
        {children}
    </div>
);

// Over the depot grid — the blur is what separates the panel from the page.
export const OverDepot = () => (
    <DepotBehind>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <div className="flex flex-wrap items-center gap-x-2 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                        <span>Material</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span>Tier 5</span>
                    </div>
                    <SheetTitle>D32 Steel</SheetTitle>
                    <SheetDescription>Consumed by Elite 2 promotions for six-star Guards and Defenders.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { label: "On hand", value: "7" },
                        { label: "Planned", value: "12" },
                        { label: "Short by", value: "5" },
                    ].map((row) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={row.label}>
                            <span className="text-muted-foreground text-sm">{row.label}</span>
                            <span className="font-mono font-semibold text-sm tabular-nums">{row.value}</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
                    <Button>Add to planner</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </DepotBehind>
);

// From the bottom edge — the overlay still covers the full page while the
// panel only claims the last rows.
export const OverDepotFromBottom = () => (
    <DepotBehind>
        <Sheet open>
            <SheetPopup className="mx-auto max-w-lg" initialFocus={false} side="bottom" variant="inset">
                <SheetHeader>
                    <SheetTitle>Edit depot count</SheetTitle>
                    <SheetDescription>D32 Steel · used by 6 of your 14 planned upgrades.</SheetDescription>
                </SheetHeader>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Cancel</SheetClose>
                    <Button>Save count</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </DepotBehind>
);
