import { Badge, Button, Checkbox, Label, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle, SheetTrigger, Switch } from "frontend";
import { FilterIcon, SlidersHorizontalIcon } from "lucide-react";
import type { ReactNode } from "react";

// `Sheet` is the Base UI dialog root in side-panel clothing: it owns `open` /
// `onOpenChange` and nothing else renders without it.
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

const SectionLabel = ({ children }: { children?: ReactNode }) => <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{children}</span>;

// Right side, inset — the shape the birthdays tool uses for its filter form.
export const BirthdayFilters = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Filters</SheetTitle>
                    <p className="font-medium font-mono text-[11.5px] text-muted-foreground uppercase tracking-[0.08em]">37 of 312 match</p>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-5">
                    <div className="flex flex-col gap-1">
                        <SectionLabel>Nation</SectionLabel>
                        <CheckRow checked count={64} label="Rhodes Island" />
                        <CheckRow count={38} label="Kazimierz" />
                        <CheckRow checked count={41} label="Lungmen" />
                        <CheckRow count={27} label="Victoria" />
                        <CheckRow count={33} label="Columbia" />
                    </div>
                    <div className="flex items-center justify-between gap-3 rounded-lg border bg-muted/50 p-3">
                        <div className="flex flex-col gap-0.5">
                            <span className="font-medium text-sm">Owned only</span>
                            <span className="text-muted-foreground text-xs">Hide operators missing from your roster</span>
                        </div>
                        <Switch defaultChecked />
                    </div>
                </SheetPanel>
                <SheetFooter>
                    <Button variant="ghost">Reset</Button>
                    <SheetClose render={<Button />}>Show 37 results</SheetClose>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Left side — the operator list opens its filter rail from the start edge on
// mobile.
export const OperatorFilterRail = () => (
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
                <SheetPanel className="flex flex-col gap-5">
                    <div className="flex flex-col gap-2">
                        <SectionLabel>Rarity</SectionLabel>
                        <div className="flex flex-wrap gap-1.5">
                            <Badge>6★</Badge>
                            <Badge variant="outline">5★</Badge>
                            <Badge variant="outline">4★</Badge>
                            <Badge variant="outline">3★</Badge>
                        </div>
                    </div>
                    <div className="flex flex-col gap-1">
                        <SectionLabel>Class</SectionLabel>
                        <CheckRow checked count={51} label="Guard" />
                        <CheckRow count={44} label="Sniper" />
                        <CheckRow checked count={39} label="Caster" />
                        <CheckRow count={31} label="Medic" />
                        <CheckRow count={29} label="Defender" />
                    </div>
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Bottom side — a short action sheet anchored to the bottom edge.
export const BottomActionSheet = () => (
    <Stage>
        <Sheet open>
            <SheetPopup className="mx-auto max-w-lg" initialFocus={false} side="bottom" variant="inset">
                <SheetHeader>
                    <SheetTitle>Add 8-14 to the planner</SheetTitle>
                    <SheetDescription>Roaring Flare · 25 sanity · drops 2 Chip Catalysts on first clear.</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-2">
                    {[
                        { label: "Runs to plan", value: "12" },
                        { label: "Sanity cost", value: "300" },
                        { label: "Expected D32 Steel", value: "2.2" },
                    ].map((row) => (
                        <div className="flex items-center justify-between rounded-lg border px-3 py-2.5" key={row.label}>
                            <span className="text-muted-foreground text-sm">{row.label}</span>
                            <span className="font-mono font-semibold text-sm tabular-nums">{row.value}</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Cancel</SheetClose>
                    <Button>Add to plan</Button>
                </SheetFooter>
            </SheetPopup>
        </Sheet>
    </Stage>
);

// Closed — the root renders only its trigger until `open` flips.
export const Closed = () => (
    <div className="w-full max-w-xl rounded-xl border bg-card p-4">
        <div className="flex items-center justify-between gap-3">
            <div className="flex min-w-0 flex-col gap-0.5">
                <span className="font-heading font-semibold text-base">Operator birthdays</span>
                <span className="text-muted-foreground text-sm">312 operators · 37 in June</span>
            </div>
            <Sheet>
                <SheetTrigger render={<Button variant="outline" />}>
                    <SlidersHorizontalIcon />
                    Filters
                    <Badge className="ml-0.5" size="sm">
                        3
                    </Badge>
                </SheetTrigger>
                <SheetPopup className="w-[min(420px,100vw)]" side="right" variant="inset">
                    <SheetHeader>
                        <SheetTitle>Filters</SheetTitle>
                    </SheetHeader>
                </SheetPopup>
            </Sheet>
        </div>
    </div>
);
