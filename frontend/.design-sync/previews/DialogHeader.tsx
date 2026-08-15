import { Badge, Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "frontend";
import { TriangleAlertIcon } from "lucide-react";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// The default header: title over description, padding collapsing when a
// `DialogPanel` follows it.
export const TitleAndDescription = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Edit list details</DialogTitle>
                    <DialogDescription>Name and description are shown on browse cards and on the public detail page.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="text-muted-foreground text-sm">Last edited 2 days ago by Dr. Ceobe · 1,284 placements</DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="submit">Save</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// A header row: icon badge beside a stacked title/description.
export const IconLed = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader className="flex-row items-start gap-3 pr-12">
                    <span className="inline-flex size-9 shrink-0 items-center justify-center rounded-full bg-warning/12 text-warning-foreground">
                        <TriangleAlertIcon aria-hidden="true" className="size-4.5" />
                    </span>
                    <div className="flex min-w-0 flex-col gap-1">
                        <DialogTitle>Depot import will overwrite counts</DialogTitle>
                        <DialogDescription>312 item quantities from your last sync are replaced by the values in this file. Planner goals are untouched.</DialogDescription>
                    </div>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="button">Import anyway</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// A detail header: mono kicker line above a larger title, with metadata badges.
export const WithMeta = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-lg">
                <DialogHeader className="pr-12">
                    <div className="flex flex-wrap items-center gap-x-2 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                        <span>Guard</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span>6★</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span>Soloblade</span>
                    </div>
                    <DialogTitle className="text-2xl">Mlynar</DialogTitle>
                    <DialogDescription>Soloblade Guard. Arts damage on his third skill ignores DEF and scales with talent stacks.</DialogDescription>
                    <div className="flex flex-wrap gap-1.5 pt-1">
                        <Badge variant="secondary">E2 90</Badge>
                        <Badge variant="secondary">S3 M3</Badge>
                        <Badge variant="outline">Module Y</Badge>
                    </div>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Open operator page</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
