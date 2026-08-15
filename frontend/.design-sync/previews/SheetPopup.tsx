import { Badge, Button, Checkbox, Label, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle, Switch } from "frontend";
import { FilterIcon, TriangleAlertIcon } from "lucide-react";
import type { ReactNode } from "react";

// `SheetPopup` (aliased `SheetContent`) is the whole overlay in one component:
// it mounts the portal, the backdrop, the viewport and the corner close button,
// and takes `side` × `variant` for placement.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const CheckRow = ({ label, count, checked }: { label: string; count: number; checked?: boolean }) => (
    <Label className="flex w-full items-center justify-between gap-3 py-1.5">
        <span className="flex items-center gap-2.5">
            <Checkbox defaultChecked={checked} />
            {label}
        </span>
        <span className="font-mono text-muted-foreground text-xs tabular-nums">{count}</span>
    </Label>
);

// The default: end edge, full height, flush border.
export const SideRight = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right">
                <SheetHeader>
                    <SheetTitle>Filters</SheetTitle>
                    <SheetDescription>37 of 312 operators match.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-1">
                    <CheckRow checked count={51} label="Guard" />
                    <CheckRow count={44} label="Sniper" />
                    <CheckRow checked count={39} label="Caster" />
                    <CheckRow count={31} label="Medic" />
                </SheetPanel>
                <SheetFooter>
                    <Button variant="ghost">Reset</Button>
                    <SheetClose render={<Button />}>Show 37 results</SheetClose>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Start edge, `inset` — the operator list's mobile filter rail.
export const SideLeftInset = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-80" initialFocus={false} side="left" variant="inset">
                <SheetHeader className="pb-4">
                    <SheetTitle className="flex items-center gap-2 text-base">
                        <FilterIcon aria-hidden="true" className="size-4 text-muted-foreground" />
                        Filters
                        <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 font-mono font-semibold text-[10px] text-primary-foreground">2</span>
                    </SheetTitle>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-4">
                    <div className="flex flex-wrap gap-1.5">
                        <Badge>6★</Badge>
                        <Badge variant="outline">5★</Badge>
                        <Badge variant="outline">4★</Badge>
                    </div>
                    <div className="flex flex-col gap-1">
                        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Class</span>
                        <CheckRow checked count={51} label="Guard" />
                        <CheckRow count={44} label="Sniper" />
                        <CheckRow count={39} label="Caster" />
                        <CheckRow count={31} label="Medic" />
                        <CheckRow count={29} label="Defender" />
                    </div>
                    <div className="flex items-center justify-between gap-3 rounded-lg border bg-muted/50 p-3">
                        <span className="font-medium text-sm">Owned only</span>
                        <Switch defaultChecked />
                    </div>
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Bottom edge, `inset` — a centred action sheet that keeps its content height.
export const SideBottomInset = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="mx-auto max-w-lg" initialFocus={false} side="bottom" variant="inset">
                <SheetHeader>
                    <SheetTitle>Pull summary — Lone Trail</SheetTitle>
                    <SheetDescription>10 pulls · 1 six-star · pity counter reset at 47.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { name: "Mlynar", rarity: "6★" },
                        { name: "Heidi", rarity: "5★" },
                        { name: "Texas", rarity: "5★" },
                    ].map((pull) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={pull.name}>
                            <span className="text-sm">{pull.name}</span>
                            <Badge variant={pull.rarity === "6★" ? "default" : "secondary"}>{pull.rarity}</Badge>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
                    <Button>Save to history</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Top edge — a banner-shaped sheet for a site-wide notice.
export const SideTop = () => (
    <Stage>
        <Sheet open>
            <SheetPopup initialFocus={false} side="top">
                <SheetHeader>
                    <SheetTitle className="flex items-center gap-2 text-base">
                        <TriangleAlertIcon aria-hidden="true" className="size-4 text-muted-foreground" />
                        Drop rates are 3 days stale
                    </SheetTitle>
                    <SheetDescription>Penguin Stats last responded on 12 May. Efficiency numbers may be off for Chapter 8.</SheetDescription>
                </SheetHeader>
                <SheetFooter variant="bare">
                    <SheetClose render={<Button variant="outline" />}>Dismiss</SheetClose>
                    <Button>Retry sync</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// `showCloseButton={false}` drops the corner dismiss — for sheets whose footer
// already owns every exit.
export const WithoutCloseButton = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} showCloseButton={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Leave the editor?</SheetTitle>
                    <SheetDescription>4 placements in "Global 6★ PvE" have not been saved to the draft yet.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-0">
                    {[
                        { op: "Mlynar", move: "A → S" },
                        { op: "Muelsyse", move: "B → S" },
                        { op: "Texas the Omertosa", move: "unplaced → A" },
                        { op: "Eyjafjalla", move: "S → A" },
                    ].map((change) => (
                        <div className="flex items-center justify-between gap-3 border-b py-2.5 last:border-b-0" key={change.op}>
                            <span className="min-w-0 truncate text-sm">{change.op}</span>
                            <span className="shrink-0 font-mono text-muted-foreground text-xs">{change.move}</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Keep editing</SheetClose>
                    <Button variant="destructive">Discard changes</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);
