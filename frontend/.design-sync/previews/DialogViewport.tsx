import { Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle, Separator } from "frontend";
import type { ReactNode } from "react";

// The viewport is the `fixed inset-0` grid that places the popup — rows
// `1fr auto 3fr`, so a short dialog sits in the upper third and a tall one
// grows against `max-h-full`.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

export const ShortDialog = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-sm">
                <DialogHeader>
                    <DialogTitle>Sanity restored</DialogTitle>
                    <DialogDescription>Your sanity is back to 135/135. 4 Originite Prime remaining.</DialogDescription>
                </DialogHeader>
                <DialogFooter>
                    <DialogClose render={<Button type="button" />}>Got it</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

export const TallDialog = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Materials needed</DialogTitle>
                    <DialogDescription>Mlynar E2 90 · S3 M3 — remaining after depot</DialogDescription>
                </DialogHeader>
                <DialogPanel className="flex flex-col gap-0">
                    {[
                        { name: "Bipolar Nanoflake", need: 6, have: 12 },
                        { name: "D32 Steel", need: 5, have: 7 },
                        { name: "Crystalline Electronic Unit", need: 4, have: 1 },
                        { name: "Polymerization Preparation", need: 8, have: 9 },
                        { name: "White Horse Kohl", need: 12, have: 3 },
                        { name: "Manganese Trihydrate", need: 9, have: 14 },
                        { name: "Orirock Concentration", need: 22, have: 18 },
                        { name: "Grindstone Pentahydrate", need: 5, have: 4 },
                    ].map((m, i) => (
                        <div key={m.name}>
                            {i > 0 && <Separator />}
                            <div className="flex items-center justify-between gap-3 py-2.5">
                                <span className="min-w-0 flex-1 truncate text-sm">{m.name}</span>
                                <span className="font-mono text-sm tabular-nums">
                                    <span className={m.have >= m.need ? "text-success-foreground" : "text-destructive-foreground"}>{m.have}</span>
                                    <span className="text-muted-foreground"> / {m.need}</span>
                                </span>
                            </div>
                        </div>
                    ))}
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Farm plan</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
