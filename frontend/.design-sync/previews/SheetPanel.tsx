import { Button, Checkbox, Label, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle, Slider, Switch } from "frontend";
import type { ReactNode } from "react";

// `SheetPanel` is the scrollable middle of the sheet: it wraps its children in
// a `ScrollArea` and takes the height left over by the header and footer, so a
// long list scrolls while the action row stays pinned.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const MATERIALS = [
    { name: "Bipolar Nanoflake", need: 6, have: 12 },
    { name: "D32 Steel", need: 5, have: 7 },
    { name: "Crystalline Electronic Unit", need: 4, have: 1 },
    { name: "Polymerization Preparation", need: 8, have: 9 },
    { name: "White Horse Kohl", need: 12, have: 3 },
    { name: "Manganese Trihydrate", need: 9, have: 14 },
    { name: "Orirock Concentration", need: 21, have: 18 },
    { name: "Grindstone Pentahydrate", need: 6, have: 4 },
    { name: "Optimized Device", need: 3, have: 5 },
    { name: "Keton Colloid", need: 7, have: 2 },
    { name: "Sugar Lump", need: 11, have: 6 },
    { name: "Aketon", need: 5, have: 13 },
    { name: "Polyester Lump", need: 8, have: 8 },
    { name: "Incandescent Alloy Block", need: 4, have: 0 },
    { name: "Transmuted Salt Agglomerate", need: 6, have: 3 },
    { name: "RMA70-24", need: 9, have: 11 },
];

// A list longer than the panel — it scrolls under the fade while the footer
// stays put.
export const ScrollingList = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Mlynar — E2 90 · S3 M3</SheetTitle>
                    <SheetDescription>Remaining materials after your depot.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-0">
                    {MATERIALS.map((m) => (
                        <div className="flex items-center justify-between gap-3 border-b py-2.5 last:border-b-0" key={m.name}>
                            <span className="min-w-0 truncate text-sm">{m.name}</span>
                            <span className={`shrink-0 font-mono text-xs tabular-nums ${m.have < m.need ? "text-destructive" : "text-muted-foreground"}`}>
                                {m.have} / {m.need}
                            </span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <Button variant="ghost">Reset</Button>
                    <SheetClose render={<Button />}>Save plan</SheetClose>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Form controls instead of rows — the panel is just the padded body.
export const FormControls = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Randomizer settings</SheetTitle>
                    <SheetDescription>Constrain the roll before rolling a squad.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-5">
                    <div className="flex flex-col gap-2">
                        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Squad size</span>
                        <Slider defaultValue={8} max={12} min={1} />
                    </div>
                    <div className="flex flex-col gap-1">
                        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Rarity</span>
                        {[
                            { label: "6★", checked: true },
                            { label: "5★", checked: true },
                            { label: "4★", checked: false },
                        ].map((r) => (
                            <Label className="flex w-full items-center gap-2.5 py-1.5" key={r.label}>
                                <Checkbox defaultChecked={r.checked} />
                                {r.label}
                            </Label>
                        ))}
                    </div>
                    <div className="flex items-center justify-between gap-3 rounded-lg border bg-muted/50 p-3">
                        <div className="flex flex-col gap-0.5">
                            <span className="font-medium text-sm">Owned only</span>
                            <span className="text-muted-foreground text-xs">Roll from your imported roster</span>
                        </div>
                        <Switch defaultChecked />
                    </div>
                </SheetPanel>
                <SheetFooter>
                    <Button variant="ghost">Reset</Button>
                    <SheetClose render={<Button />}>Roll a squad</SheetClose>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// `scrollFade={false}` turns off the mask at the panel edges — use it when the
// content already ends on a border.
export const WithoutScrollFade = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="left" variant="inset">
                <SheetHeader>
                    <SheetTitle>Chapter 8 — Roaring Flare</SheetTitle>
                    <SheetDescription>14 stages · 4 still unclear.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2" scrollFade={false}>
                    {[
                        { code: "8-7", name: "Ashes", sanity: 21 },
                        { code: "8-8", name: "Sundered Ground", sanity: 21 },
                        { code: "8-9", name: "Dust Settles", sanity: 21 },
                        { code: "8-10", name: "Cold Comfort", sanity: 21 },
                        { code: "8-11", name: "Endless Night", sanity: 21 },
                        { code: "8-12", name: "Long Shadow", sanity: 21 },
                        { code: "8-14", name: "Roaring Flare", sanity: 25 },
                        { code: "S8-1", name: "Broken Wall", sanity: 18 },
                        { code: "S8-2", name: "Gravel Road", sanity: 18 },
                    ].map((s) => (
                        <div className="flex items-center gap-3 rounded-lg border px-3 py-2.5" key={s.code}>
                            <span className="font-mono font-semibold text-sm tabular-nums">{s.code}</span>
                            <span className="min-w-0 flex-1 truncate text-sm">{s.name}</span>
                            <span className="font-mono text-muted-foreground text-xs tabular-nums">{s.sanity}</span>
                        </div>
                    ))}
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    </Stage>
);
