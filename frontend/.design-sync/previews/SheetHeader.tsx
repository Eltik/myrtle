import { Badge, Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import { FilterIcon, XIcon } from "lucide-react";
import type { ReactNode } from "react";

// `SheetHeader` is the padded title block at the leading edge of the panel. It
// tightens its bottom padding automatically when the sheet also has a
// `SheetPanel`, so header + panel read as one column.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// Title + description, the default pairing.
export const TitleAndDescription = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Randomizer settings</SheetTitle>
                    <SheetDescription>Constrain the roll by class, rarity, owned operators or stage availability.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {["Classes · all 8", "Rarity · 5★ and 6★", "Roster · owned only", "Stages · cleared only"].map((row) => (
                        <div className="rounded-lg border px-3 py-2.5 text-sm" key={row}>
                            {row}
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
                    <Button>Roll a squad</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// A mono counter under the title instead of prose — the birthdays tool's shape.
export const WithResultCounter = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Filters</SheetTitle>
                    <p className="font-medium font-mono text-[11.5px] text-muted-foreground uppercase tracking-[0.08em]">37 of 312 match</p>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { name: "Amiya", date: "1 Dec" },
                        { name: "Texas", date: "4 Jun" },
                        { name: "Skadi", date: "24 Jun" },
                        { name: "Mlynar", date: "22 Jun" },
                    ].map((op) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={op.name}>
                            <span className="text-sm">{op.name}</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">{op.date}</span>
                        </div>
                    ))}
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// A compact header carrying its own inline action next to the title.
export const CompactWithAction = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-80" initialFocus={false} showCloseButton={false} side="left" variant="inset">
                <SheetHeader className="pb-4">
                    <div className="flex items-center justify-between gap-3">
                        <SheetTitle className="flex items-center gap-2 text-base">
                            <FilterIcon aria-hidden="true" className="size-4 text-muted-foreground" />
                            Filters
                            <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 font-mono font-semibold text-[10px] text-primary-foreground">3</span>
                        </SheetTitle>
                        <SheetClose className="inline-flex items-center gap-1 font-medium text-[11.5px] text-muted-foreground">
                            <XIcon aria-hidden="true" className="size-2.5" />
                            Clear all
                        </SheetClose>
                    </div>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    <div className="flex flex-wrap gap-1.5">
                        <Badge>6★</Badge>
                        <Badge variant="outline">5★</Badge>
                        <Badge variant="outline">4★</Badge>
                    </div>
                    {["Guard", "Caster", "Medic"].map((row) => (
                        <div className="rounded-lg border px-3 py-2 text-sm" key={row}>
                            {row}
                        </div>
                    ))}
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// A header carrying an eyebrow above the title, for detail sheets.
export const WithEyebrow = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right">
                <SheetHeader>
                    <div className="flex flex-wrap items-center gap-x-2 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                        <span>Stage</span>
                        <span aria-hidden className="opacity-40">
                            ·
                        </span>
                        <span>Chapter 8</span>
                    </div>
                    <SheetTitle>8-14 — Roaring Flare</SheetTitle>
                    <SheetDescription>25 sanity · recommended average level E2 40.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { name: "Chip Catalyst", rate: "First clear ×2" },
                        { name: "Bipolar Nanoflake", rate: "18.4%" },
                        { name: "Orirock Cluster", rate: "24.1%" },
                    ].map((drop) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={drop.name}>
                            <span className="truncate text-sm">{drop.name}</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">{drop.rate}</span>
                        </div>
                    ))}
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    </Stage>
);
