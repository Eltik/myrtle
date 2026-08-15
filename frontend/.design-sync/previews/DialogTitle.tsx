import { Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "frontend";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// The default title: Geist heading face, `text-xl`, and the popup's accessible
// name.
export const Default = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Tier settings</DialogTitle>
                    <DialogDescription>Rename tiers, recolour them, or change how many operators fit per row.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="text-muted-foreground text-sm">6 tiers · S through F · 8 per row</DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="submit">Save settings</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// Scaled up for detail surfaces, with a mono kicker line above it.
export const LargeWithKicker = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-lg">
                <DialogHeader className="pr-12">
                    <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Chapter 8 · Main theme</span>
                    <DialogTitle className="text-2xl">Roaring Flare</DialogTitle>
                    <DialogDescription>14 stages and 3 challenge modes. First-clear rewards include 2 Chip Catalysts.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Browse stages</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// A long title wraps rather than clipping — the corner close button is cleared
// with padding on the header.
export const LongTitle = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader className="pr-12">
                    <DialogTitle>Grandmaster of the Corrupting Heart — operator record</DialogTitle>
                    <DialogDescription>Skadi's alter form, recorded at E2 90 with Module SPC-Y and all three skills at Mastery 3.</DialogDescription>
                </DialogHeader>
                <DialogFooter variant="bare">
                    <DialogClose render={<Button type="button" variant="ghost" />}>Close</DialogClose>
                    <Button type="button">Open record</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
