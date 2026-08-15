import { Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "frontend";
import type { ReactNode } from "react";

// `SheetDescription` is the muted sub-line under the title, wired to the
// popup's `aria-describedby`. It carries the one sentence that tells the doctor
// what the sheet will change.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

// One sentence of context under the title.
export const Default = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Import roster</SheetTitle>
                    <SheetDescription>Paste a Krooster share link and we will match operators by id — nothing is written until you confirm.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { label: "Matched", value: "268" },
                        { label: "New to your roster", value: "31" },
                        { label: "Unrecognised ids", value: "2" },
                    ].map((row) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={row.label}>
                            <span className="text-muted-foreground text-sm">{row.label}</span>
                            <span className="font-mono font-semibold text-sm tabular-nums">{row.value}</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Cancel</SheetClose>
                    <Button>Import 268</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Inline emphasis — a number the doctor should not miss, still inside the
// muted line.
export const WithInlineEmphasis = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="max-w-sm" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Sanity plan for this week</SheetTitle>
                    <SheetDescription>
                        <span className="font-medium text-foreground">1,260 sanity</span> across 4 stages — about 3 days of natural regen plus 2 Originite Prime.
                    </SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { code: "8-14", runs: 12 },
                        { code: "S8-2", runs: 6 },
                        { code: "CE-6", runs: 4 },
                        { code: "1-7", runs: 18 },
                    ].map((row) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={row.code}>
                            <span className="font-mono font-semibold text-sm tabular-nums">{row.code}</span>
                            <span className="text-muted-foreground text-sm">{row.runs} runs</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter variant="bare">
                    <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
                    <Button>Start route</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Short and dense — a bottom sheet where the description is the only body copy.
export const InBottomSheet = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="mx-auto max-w-lg" initialFocus={false} side="bottom" variant="inset">
                <SheetHeader>
                    <SheetTitle>Reset all filters?</SheetTitle>
                    <SheetDescription>3 facets are active. Clearing them brings the list back to all 312 operators.</SheetDescription>
                </SheetHeader>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Keep filters</SheetClose>
                    <Button>Reset</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);
