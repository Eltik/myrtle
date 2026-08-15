import { Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle, DialogTrigger } from "frontend";
import { PlusIcon, SettingsIcon, Trash2Icon } from "lucide-react";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const SettingsDialog = () => (
    <DialogPopup className="max-w-md">
        <DialogHeader>
            <DialogTitle>Tier settings</DialogTitle>
            <DialogDescription>Rename tiers, recolour them, or change how many operators fit per row.</DialogDescription>
        </DialogHeader>
        <DialogFooter>
            <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
            <Button type="submit">Save settings</Button>
        </DialogFooter>
    </DialogPopup>
);

// The trigger is only visible while the dialog is closed — this is the toolbar
// it actually lives in.
export const InToolbar = () => (
    <div className="w-full max-w-xl rounded-xl border bg-card p-4">
        <div className="flex items-center justify-between gap-3">
            <div className="flex min-w-0 flex-col gap-0.5">
                <span className="font-heading font-semibold text-base">My tier lists</span>
                <span className="text-muted-foreground text-sm">6 lists · 2 published</span>
            </div>
            <div className="flex items-center gap-2">
                <Dialog>
                    <DialogTrigger aria-label="Tier settings" render={<Button size="icon" variant="ghost" />}>
                        <SettingsIcon />
                    </DialogTrigger>
                    <SettingsDialog />
                </Dialog>
                <Dialog>
                    <DialogTrigger render={<Button />}>
                        <PlusIcon />
                        New list
                    </DialogTrigger>
                    <SettingsDialog />
                </Dialog>
            </div>
        </div>
    </div>
);

export const TriggerVariants = () => (
    <div className="flex flex-wrap items-center gap-2">
        <Dialog>
            <DialogTrigger render={<Button />}>
                <PlusIcon />
                New list
            </DialogTrigger>
            <SettingsDialog />
        </Dialog>
        <Dialog>
            <DialogTrigger render={<Button variant="outline" />}>Publish version</DialogTrigger>
            <SettingsDialog />
        </Dialog>
        <Dialog>
            <DialogTrigger render={<Button variant="ghost" />}>Import from depot</DialogTrigger>
            <SettingsDialog />
        </Dialog>
        <Dialog>
            <DialogTrigger render={<Button variant="destructive-outline" />}>
                <Trash2Icon />
                Delete
            </DialogTrigger>
            <SettingsDialog />
        </Dialog>
        <Dialog>
            <DialogTrigger render={<Button size="xs" variant="secondary" />}>Version history</DialogTrigger>
            <SettingsDialog />
        </Dialog>
    </div>
);

export const Opened = () => (
    <Stage>
        <div className="w-full max-w-xl rounded-xl border bg-card p-4">
            <div className="flex items-center justify-between gap-3">
                <span className="font-heading font-semibold text-base">Endgame DPS rankings</span>
                <Dialog defaultOpen>
                    <DialogTrigger render={<Button variant="outline" />}>
                        <SettingsIcon />
                        Tier settings
                    </DialogTrigger>
                    <SettingsDialog />
                </Dialog>
            </div>
        </div>
    </Stage>
);
