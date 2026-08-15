import { Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "frontend";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// `variant="default"` — the muted, top-bordered action bar flush with the
// popup's bottom corners.
export const Default = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Publish version 12</DialogTitle>
                    <DialogDescription>Readers see this snapshot until you publish again.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="text-muted-foreground text-sm">Since v11: 6 operators moved, 2 tiers renamed.</DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="submit">Publish</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// `variant="bare"` — no border or fill, for short dialogs where a bar would
// out-weigh the content.
export const Bare = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-sm">
                <DialogHeader>
                    <DialogTitle>Goal added</DialogTitle>
                    <DialogDescription>Mlynar E2 90 · S3 M3 is now tracked in your planner.</DialogDescription>
                </DialogHeader>
                <DialogFooter variant="bare">
                    <DialogClose render={<Button type="button" variant="ghost" />}>Close</DialogClose>
                    <Button type="button">Open planner</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// Destructive confirmation, plus a secondary note pinned to the start of the bar.
export const DestructiveWithNote = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Delete 3 plans?</DialogTitle>
                    <DialogDescription>Goals for Skadi the Corrupting Heart, Ines and Texas the Omertosa will be removed from the planner.</DialogDescription>
                </DialogHeader>
                <DialogFooter className="sm:justify-between">
                    <span className="self-center font-mono text-muted-foreground text-xs tabular-nums max-sm:hidden">3 selected</span>
                    <div className="flex flex-col-reverse gap-2 sm:flex-row">
                        <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                        <Button type="button" variant="destructive">
                            Delete plans
                        </Button>
                    </div>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
