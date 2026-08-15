import { Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import type { ReactNode } from "react";

// The viewport is the `fixed inset-0` layer that decides where the panel lands:
// a flex row for `left`/`right`, a two-row grid for `top`/`bottom`, plus the
// `sm:p-4` safe area the `inset` variant floats inside. `SheetPopup` mounts it
// from the `side` and `variant` props.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const Body = () => (
    <SheetPanel className="flex flex-col gap-2">
        {[
            { label: "Sanity", value: "135 / 135" },
            { label: "Trust today", value: "+18" },
            { label: "Base LMD/day", value: "8,240" },
        ].map((row) => (
            <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={row.label}>
                <span className="text-muted-foreground text-sm">{row.label}</span>
                <span className="font-mono font-semibold text-sm tabular-nums">{row.value}</span>
            </div>
        ))}
    </SheetPanel>
);

// `side="right"` — the viewport becomes a flex row justified to the end edge,
// so the panel runs the full height flush against the border.
export const EndAligned = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right">
                <SheetHeader>
                    <SheetTitle>Today at Rhodes Island</SheetTitle>
                    <SheetDescription>Sanity, trust and base output since the 04:00 reset.</SheetDescription>
                </SheetHeader>
                <Body />
            </SheetPopup>
        </Sheet>
    </Stage>
);

// `side="left"` — the same flex viewport justified to the start edge, the
// navigation rail position.
export const StartAligned = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-xs" initialFocus={false} side="left">
                <SheetHeader>
                    <SheetTitle>Today at Rhodes Island</SheetTitle>
                    <SheetDescription>Sanity, trust and base output since the 04:00 reset.</SheetDescription>
                </SheetHeader>
                <Body />
            </SheetPopup>
        </Sheet>
    </Stage>
);

// `side="bottom"` — a `1fr auto` grid with a gutter on top, so the panel keeps
// its content height instead of stretching.
export const BottomAligned = () => (
    <Stage>
        <Sheet open>
            <SheetPopup initialFocus={false} side="bottom">
                <SheetHeader>
                    <SheetTitle>Today at Rhodes Island</SheetTitle>
                    <SheetDescription>Sanity is full and 3 base rooms need a shift change.</SheetDescription>
                </SheetHeader>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Dismiss</SheetClose>
                    <Button>Open planner</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// `variant="inset"` adds the safe-area padding, so the panel floats with
// rounded corners instead of meeting the edge.
export const InsetSafeArea = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Today at Rhodes Island</SheetTitle>
                    <SheetDescription>Sanity, trust and base output since the 04:00 reset.</SheetDescription>
                </SheetHeader>
                <Body />
            </SheetPopup>
        </Sheet>
    </Stage>
);
