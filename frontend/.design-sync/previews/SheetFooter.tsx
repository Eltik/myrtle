import { Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import type { ReactNode } from "react";

// `SheetFooter` is the action row pinned under the panel. `variant="default"`
// gives it a top border and a muted bar; `variant="bare"` drops both and lets
// the actions sit on the panel background.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const Body = () => (
    <SheetPanel className="flex flex-col gap-2">
        {[
            { label: "Bipolar Nanoflake", value: "6 / 12" },
            { label: "D32 Steel", value: "5 / 7" },
            { label: "Crystalline Electronic Unit", value: "4 / 1" },
        ].map((row) => (
            <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={row.label}>
                <span className="truncate text-sm">{row.label}</span>
                <span className="font-mono text-muted-foreground text-xs tabular-nums">{row.value}</span>
            </div>
        ))}
    </SheetPanel>
);

// Default — bordered muted bar, actions aligned to the end.
export const Bordered = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Mlynar — E2 90 plan</SheetTitle>
                    <SheetDescription>Remaining materials after your depot.</SheetDescription>
                </SheetHeader>
                <Body />
                <SheetFooter>
                    <Button variant="ghost">Reset</Button>
                    <SheetClose render={<Button />}>Save plan</SheetClose>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// `variant="bare"` — no border, no bar; for sheets where the actions should
// read as part of the content.
export const Bare = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Mlynar — E2 90 plan</SheetTitle>
                    <SheetDescription>Remaining materials after your depot.</SheetDescription>
                </SheetHeader>
                <Body />
                <SheetFooter variant="bare">
                    <Button variant="ghost">Reset</Button>
                    <SheetClose render={<Button />}>Save plan</SheetClose>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// A destructive pair — the safe action is the close, so the ring never lands on
// the dangerous one.
export const DestructivePair = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Delete "Global 6★ PvE"?</SheetTitle>
                    <SheetDescription>46 placements and the public share link are removed. This cannot be undone.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-0">
                    {[
                        { label: "Placements", value: "46" },
                        { label: "Published", value: "12 May" },
                        { label: "Views", value: "1,284" },
                        { label: "Forks", value: "7" },
                    ].map((row) => (
                        <div className="flex items-center justify-between border-b py-2.5 last:border-b-0" key={row.label}>
                            <span className="text-muted-foreground text-sm">{row.label}</span>
                            <span className="font-mono text-sm tabular-nums">{row.value}</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Cancel</SheetClose>
                    <Button variant="destructive">Delete list</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// A single full-width action — the footer stacks on narrow panels.
export const SingleAction = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="mx-auto max-w-md" initialFocus={false} side="bottom" variant="inset">
                <SheetHeader>
                    <SheetTitle>Sanity restored</SheetTitle>
                    <SheetDescription>You are back to 135/135. 4 Originite Prime remaining.</SheetDescription>
                </SheetHeader>
                <SheetFooter>
                    <SheetClose render={<Button />}>Got it</SheetClose>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);
