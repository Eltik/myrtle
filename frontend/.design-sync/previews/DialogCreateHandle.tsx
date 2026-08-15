import { Button, Dialog, DialogClose, DialogCreateHandle, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle, DialogTrigger } from "frontend";
import { Trash2Icon } from "lucide-react";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// One handle lets many triggers drive a single dialog instance and hand it a
// payload — no `open` state per row.
const deleteListHandle = DialogCreateHandle<{ name: string; placements: number }>();

const LISTS = [
    { name: "Endgame DPS rankings", placements: 1284, version: "v12" },
    { name: "CC#14 must-haves", placements: 96, version: "v3" },
    { name: "Low-rarity carries", placements: 412, version: "draft" },
];

const ListRows = () => (
    <div className="w-full max-w-xl overflow-hidden rounded-xl border bg-card">
        <div className="flex items-center justify-between gap-3 border-b px-4 py-3">
            <span className="font-heading font-semibold text-base">My tier lists</span>
            <span className="font-mono text-muted-foreground text-xs tabular-nums">3 lists</span>
        </div>
        {LISTS.map((list) => (
            <div className="flex items-center gap-3 border-b px-4 py-3 last:border-b-0" key={list.name}>
                <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                    <span className="truncate font-medium text-sm">{list.name}</span>
                    <span className="font-mono text-muted-foreground text-xs tabular-nums">
                        {list.placements.toLocaleString()} placements · {list.version}
                    </span>
                </div>
                <DialogTrigger aria-label={`Delete ${list.name}`} handle={deleteListHandle} payload={{ name: list.name, placements: list.placements }} render={<Button size="icon" variant="ghost" />}>
                    <Trash2Icon />
                </DialogTrigger>
            </div>
        ))}
    </div>
);

// Closed: three rows, three triggers, one shared dialog mounted once.
export const SharedAcrossRows = () => (
    <div className="flex w-full flex-col gap-4">
        <ListRows />
        <Dialog handle={deleteListHandle}>
            {(payload: { name: string; placements: number } | undefined) => (
                <DialogPopup className="max-w-md">
                    <DialogHeader>
                        <DialogTitle>Delete tier list?</DialogTitle>
                        <DialogDescription>
                            <span className="font-medium text-foreground">{payload?.name ?? "This list"}</span> and its {(payload?.placements ?? 0).toLocaleString()} placements will be permanently removed.
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" />}>Cancel</DialogClose>
                        <Button type="button" variant="destructive">
                            Delete list
                        </Button>
                    </DialogFooter>
                </DialogPopup>
            )}
        </Dialog>
    </div>
);

// The same handle-driven dialog in its open state. Which row opened it is a
// runtime payload, so this resting shot shows the fallback copy.
export const OpenFromHandle = () => (
    <Stage>
        <ListRows />
        <Dialog defaultOpen handle={deleteListHandle}>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Delete tier list?</DialogTitle>
                    <DialogDescription>
                        <span className="font-medium text-foreground">CC#14 must-haves</span> and its 96 placements will be permanently removed. Anyone with the link will see a 404. This cannot be undone.
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
