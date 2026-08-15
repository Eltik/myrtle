import { Badge, Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "frontend";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const Stat = ({ kicker, value, tone }: { kicker: string; value: string; tone?: "short" | "muted" }) => (
    <div className="flex flex-col gap-1.5 rounded-lg border bg-muted/50 p-3">
        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{kicker}</span>
        <span className={`font-bold text-lg tabular-nums leading-none ${tone === "short" ? "text-destructive-foreground" : tone === "muted" ? "text-muted-foreground" : "text-foreground"}`}>{value}</span>
    </div>
);

export const Default = () => (
    <Stage>
        <Dialog open>
            <DialogPopup>
                <DialogHeader>
                    <DialogTitle>Publish version 12</DialogTitle>
                    <DialogDescription>Everyone with the share link sees this snapshot. Your draft stays editable.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="text-muted-foreground text-sm">
                    Since v11: <span className="text-foreground">6 operators moved</span>, 2 tiers renamed, 1 tier added.
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="submit">Publish</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// `showCloseButton={false}` drops the corner dismiss — use it when the footer
// already owns every exit.
export const WithoutCloseButton = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-sm" showCloseButton={false}>
                <DialogHeader>
                    <DialogTitle>Leave the editor?</DialogTitle>
                    <DialogDescription>4 placements have not been saved to the draft yet.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Keep editing</DialogClose>
                    <Button type="button" variant="destructive">
                        Leave
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// A wider popup for detail surfaces — `className` overrides the `max-w-lg` default.
export const Wide = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-2xl">
                <DialogHeader>
                    <div className="flex flex-wrap items-center gap-x-2 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                        <span>Material</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span>5★</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span>Tier 5</span>
                    </div>
                    <DialogTitle className="text-2xl">D32 Steel</DialogTitle>
                    <DialogDescription>Rhodes Island's densest alloy stock. Consumed by Elite 2 promotions for six-star Guards and Defenders.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="flex flex-col gap-5">
                    <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                        <Stat kicker="On hand" value="7" />
                        <Stat kicker="Planned" value="12" />
                        <Stat kicker="Short by" tone="short" value="5" />
                        <Stat kicker="Sanity cost" tone="muted" value="1,260" />
                    </div>
                    <div className="flex flex-col gap-2.5">
                        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Obtain channels</span>
                        <div className="flex flex-wrap gap-1.5">
                            <Badge variant="outline">Factory</Badge>
                            <Badge variant="outline">Store</Badge>
                            <Badge variant="outline">Annihilation</Badge>
                            <Badge variant="secondary">Event exchange</Badge>
                        </div>
                    </div>
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Add to planner</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
