import { Badge, Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "frontend";
import type { ReactNode } from "react";

// The backdrop is only legible over real page content — this is the stage it
// dims and blurs. `DialogPopup` mounts one automatically.
const PageBehind = ({ children }: { children?: ReactNode }) => (
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

export const OverPageContent = () => (
    <PageBehind>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>8-14 — Roaring Flare</DialogTitle>
                    <DialogDescription>Recommended average level E2 40. First clear drops 2 Chip Catalysts and 1 Bipolar Nanoflake.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Add to planner</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </PageBehind>
);

export const OverPageContentCompact = () => (
    <PageBehind>
        <Dialog open>
            <DialogPopup className="max-w-sm">
                <DialogHeader>
                    <DialogTitle>Reset stage progress?</DialogTitle>
                    <DialogDescription>Clear counts and 3-star flags for all 14 stages in Chapter 8 will be wiped from your account.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="button" variant="destructive">
                        Reset chapter
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </PageBehind>
);
