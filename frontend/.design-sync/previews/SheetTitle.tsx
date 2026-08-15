import { Badge, Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import { FilterIcon } from "lucide-react";
import type { ReactNode } from "react";

// `SheetTitle` renders the accessible name of the sheet in the heading face at
// `text-xl`. It is a Base UI `Dialog.Title`, so it is wired to the popup's
// `aria-labelledby` — every sheet should have one.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// The default size, paired with a description.
export const Default = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Randomizer settings</SheetTitle>
                    <SheetDescription>Constrain the roll by class, rarity, owned operators or stage availability.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {["Classes · all 8", "Rarity · 5★ and 6★", "Roster · owned only"].map((row) => (
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

// An icon and a count chip inside the title — it accepts arbitrary children,
// not just a string.
export const WithIconAndCount = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-80" initialFocus={false} side="left" variant="inset">
                <SheetHeader className="pb-4">
                    <SheetTitle className="flex items-center gap-2 text-base">
                        <FilterIcon aria-hidden="true" className="size-4 text-muted-foreground" />
                        Filters
                        <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 font-mono font-semibold text-[10px] text-primary-foreground">3</span>
                    </SheetTitle>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-3">
                    <div className="flex flex-wrap gap-1.5">
                        <Badge>6★</Badge>
                        <Badge variant="outline">5★</Badge>
                        <Badge variant="outline">4★</Badge>
                    </div>
                    {["Guard · 51", "Caster · 39", "Medic · 31"].map((row) => (
                        <div className="rounded-lg border px-3 py-2 text-sm" key={row}>
                            {row}
                        </div>
                    ))}
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// A long title wraps rather than truncating — the close button sits clear of it
// because the header reserves the corner.
export const LongTitle = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Skadi the Corrupting Heart — mastery plan</SheetTitle>
                    <SheetDescription>S3 M3 needs 8 more Bipolar Nanoflake at your current depot.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { label: "S1", value: "M3" },
                        { label: "S2", value: "M3" },
                        { label: "S3", value: "M1 → M3" },
                    ].map((row) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={row.label}>
                            <span className="font-mono text-sm">{row.label}</span>
                            <span className="text-muted-foreground text-sm">{row.value}</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
                    <Button>Queue masteries</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);
