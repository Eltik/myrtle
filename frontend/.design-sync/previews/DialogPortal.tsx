import { Badge, Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "frontend";
import type { ReactNode } from "react";

// The portal is why the popup escapes clipping ancestors: this card is
// `overflow-hidden` with its own stacking context, and the dialog still paints
// over the whole page. `DialogPopup` mounts the portal for you (pass
// `portalProps` to retarget it).
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

export const AboveClippedContent = () => (
    <ClippedCard>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Pick a tier</DialogTitle>
                    <DialogDescription>Mlynar will be placed in the tier you choose. Drag him afterwards to reorder within the row.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="button">Place in S</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </ClippedCard>
);

export const OverPageChrome = () => (
    <ClippedCard>
        <Dialog open>
            <DialogPopup className="max-w-sm">
                <DialogHeader>
                    <DialogTitle>Remove from pool?</DialogTitle>
                    <DialogDescription>Muelsyse goes back to the unranked pool. Nothing else in the list changes.</DialogDescription>
                </DialogHeader>
                <DialogFooter variant="bare">
                    <DialogClose render={<Button type="button" variant="ghost" />}>Cancel</DialogClose>
                    <Button type="button" variant="destructive">
                        Remove
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </ClippedCard>
);
