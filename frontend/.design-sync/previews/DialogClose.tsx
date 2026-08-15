import { Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "frontend";
import { ArrowLeftIcon } from "lucide-react";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// `DialogClose` is a headless dismiss button — `render` swaps in whichever
// Button variant the surface calls for.
export const FooterCancel = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Publish version 12</DialogTitle>
                    <DialogDescription>Readers will see this snapshot until you publish again. Draft edits stay private.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="submit">Publish</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// `showCloseButton` renders a `DialogClose` as a ghost icon button in the corner.
export const IconClose = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Orirock Cube</DialogTitle>
                    <DialogDescription>Tier 3 material. Crafted from 3 Orirock Cluster in any Factory.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="text-muted-foreground text-sm">
                    You hold <span className="font-mono font-semibold text-foreground tabular-nums">184</span> — enough for 6 more E2 promotions before the next Annihilation reset.
                </DialogPanel>
            </DialogPopup>
        </Dialog>
    </Stage>
);

export const DestructiveAndBack = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md" showCloseButton={false}>
                <DialogHeader>
                    <DialogTitle>Discard planner changes?</DialogTitle>
                    <DialogDescription>You added 4 goals for Mlynar and Ines that have not been saved. Leaving now drops them.</DialogDescription>
                </DialogHeader>
                <DialogFooter variant="bare">
                    <DialogClose render={<Button type="button" variant="ghost" />}>
                        <ArrowLeftIcon />
                        Keep editing
                    </DialogClose>
                    <DialogClose render={<Button type="button" variant="destructive" />}>Discard changes</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
