import { Badge, Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import type { ReactNode } from "react";

// The portal is why the panel escapes clipping ancestors: this card is
// `overflow-hidden` with its own stacking context, and the sheet still paints
// over the whole page. `SheetPopup` mounts the portal for you — pass
// `portalProps` to retarget it.
const ClippedCard = ({ children }: { children?: ReactNode }) => (
    <div className="relative min-h-[520px] w-full">
        <div className="relative isolate w-full max-w-xl overflow-hidden rounded-xl border bg-card">
            <div className="flex items-center justify-between gap-3 border-b px-4 py-3">
                <span className="font-heading font-semibold text-base">Operator pool</span>
                <Badge variant="secondary">42 unplaced</Badge>
            </div>
            <div className="grid grid-cols-2 gap-2 p-4">
                {["Mlynar", "Skadi the Corrupting Heart", "Muelsyse", "Texas the Omertosa", "Ines", "Eyjafjalla"].map((op) => (
                    <div className="truncate rounded-lg border bg-background px-3 py-2 text-sm" key={op}>
                        {op}
                    </div>
                ))}
            </div>
        </div>
        {children}
    </div>
);

// The sheet is portalled to the body, so the card's `overflow-hidden` never
// clips it.
export const AboveClippedContent = () => (
    <ClippedCard>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Place Mlynar</SheetTitle>
                    <SheetDescription>Pick a tier. Drag him afterwards to reorder within the row.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { tier: "S", count: 12 },
                        { tier: "A", count: 18 },
                        { tier: "B", count: 9 },
                        { tier: "C", count: 7 },
                    ].map((t) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={t.tier}>
                            <span className="font-heading font-semibold text-sm">Tier {t.tier}</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">{t.count} operators</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Cancel</SheetClose>
                    <Button>Place in S</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </ClippedCard>
);

// From the bottom edge over the same clipped card — the portal keeps the panel
// above every stacking context on the page.
export const BottomAboveClippedContent = () => (
    <ClippedCard>
        <Sheet open>
            <SheetPopup className="mx-auto max-w-lg" initialFocus={false} side="bottom" variant="inset">
                <SheetHeader>
                    <SheetTitle>42 operators still unplaced</SheetTitle>
                    <SheetDescription>Auto-place puts each of them in the tier your last published list used.</SheetDescription>
                </SheetHeader>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Not now</SheetClose>
                    <Button>Auto-place all</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </ClippedCard>
);
