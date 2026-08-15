import { Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "frontend";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// One supporting sentence under the title, in muted small text.
export const Default = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Import depot</DialogTitle>
                    <DialogDescription>Paste the JSON exported from your game account to sync 312 item counts.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="button">Import</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// Inline emphasis: the description carries the destructive detail, so the risky
// noun is promoted to foreground weight.
export const WithEmphasis = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Delete tier list?</DialogTitle>
                    <DialogDescription>
                        <span className="font-medium text-foreground">Endgame DPS rankings</span> and all of its tiers, placements and stats will be permanently removed. Anyone with the link will see a 404. This cannot be undone.
                    </DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="button" variant="destructive">
                        Delete list
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// A longer, multi-paragraph description still reads at `text-sm` with relaxed
// leading.
export const Long = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-lg">
                <DialogHeader className="pr-12">
                    <DialogTitle>How the DPS calculator works</DialogTitle>
                    <DialogDescription className="leading-relaxed">
                        Damage is simulated frame by frame against a synthetic enemy with the DEF and RES you pick, including talent stacks, module bonuses and skill uptime over a 90-second window.
                    </DialogDescription>
                    <DialogDescription className="leading-relaxed">Buffs from other operators are not modelled. Numbers assume the operator is at E2 90 with the listed mastery, and are rounded to the nearest whole hit.</DialogDescription>
                </DialogHeader>
                <DialogFooter variant="bare">
                    <DialogClose render={<Button type="button" variant="ghost" />}>Close</DialogClose>
                    <Button type="button">Open calculator</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
