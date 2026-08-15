import { Badge, Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "frontend";
import type { ReactNode } from "react";

// `DialogOverlay` is the shadcn-compatible alias for `DialogBackdrop`; every
// `DialogPopup` mounts one, so these stories show what it looks like over the
// page it dims.
const PageBehind = ({ children }: { children?: ReactNode }) => (
    <div className="relative min-h-[520px] w-full">
        <div className="flex flex-col gap-4 p-1">
            <div className="flex items-baseline justify-between gap-3">
                <h2 className="font-heading font-semibold text-xl">Depot</h2>
                <span className="font-mono text-muted-foreground text-xs tabular-nums">312 items</span>
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

export const OverDepot = () => (
    <PageBehind>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <div className="flex flex-wrap items-center gap-x-2 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                        <span>Material</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span>5★</span>
                    </div>
                    <DialogTitle>Bipolar Nanoflake</DialogTitle>
                    <DialogDescription>A refined nanomaterial used in Elite 2 promotions and top-tier skill masteries.</DialogDescription>
                </DialogHeader>
                <DialogPanel>
                    <div className="flex flex-wrap gap-1.5">
                        <Badge variant="outline">Factory</Badge>
                        <Badge variant="outline">Store</Badge>
                        <Badge variant="secondary">CE-6 drop</Badge>
                    </div>
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Plan 4 more</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </PageBehind>
);

export const DestructiveConfirm = () => (
    <PageBehind>
        <Dialog open>
            <DialogPopup className="max-w-sm">
                <DialogHeader>
                    <DialogTitle>Clear depot?</DialogTitle>
                    <DialogDescription>All 312 recorded item counts will be set to zero. Your planner goals are kept.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="button" variant="destructive">
                        Clear depot
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </PageBehind>
);
