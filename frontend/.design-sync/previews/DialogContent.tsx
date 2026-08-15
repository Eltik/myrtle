import { Button, Checkbox, Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogTitle, Separator } from "frontend";
import { DownloadIcon } from "lucide-react";
import type { ReactNode } from "react";

// `DialogContent` is the shadcn-compatible alias for `DialogPopup` — same
// portal, backdrop, viewport and corner close button.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

export const ExportOptions = () => (
    <Stage>
        <Dialog open>
            <DialogContent className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Export tier list</DialogTitle>
                    <DialogDescription>Renders a PNG at 2× scale using the current tier colours.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="flex flex-col gap-3.5">
                    {[
                        { id: "export-names", label: "Operator names", desc: "Draw the name under each portrait.", checked: true },
                        { id: "export-watermark", label: "Myrtle watermark", desc: "Small mark in the bottom-right corner.", checked: true },
                        { id: "export-empty", label: "Include empty tiers", desc: "Keep rows with no operators placed.", checked: false },
                    ].map((opt) => (
                        <div className="flex items-start gap-3" key={opt.id}>
                            <Checkbox className="mt-0.5" defaultChecked={opt.checked} id={opt.id} />
                            <div className="flex min-w-0 flex-col gap-0.5">
                                <label className="font-medium text-sm" htmlFor={opt.id}>
                                    {opt.label}
                                </label>
                                <span className="text-muted-foreground text-xs">{opt.desc}</span>
                            </div>
                        </div>
                    ))}
                    <Separator />
                    <div className="flex items-center justify-between gap-3">
                        <span className="text-muted-foreground text-xs">Saved as</span>
                        <span className="truncate font-mono text-xs">endgame-dps-rankings-v12.png</span>
                    </div>
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                    <Button type="button">
                        <DownloadIcon />
                        Export PNG
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    </Stage>
);

export const Compact = () => (
    <Stage>
        <Dialog open>
            <DialogContent className="max-w-sm">
                <DialogHeader>
                    <DialogTitle>Share link copied</DialogTitle>
                    <DialogDescription>myrtle.moe/tier-lists/endgame-dps-rankings — anyone with this link can view version 12.</DialogDescription>
                </DialogHeader>
                <DialogFooter variant="bare">
                    <DialogClose render={<Button type="button" />}>Done</DialogClose>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    </Stage>
);
