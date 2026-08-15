import { Badge, Button, Checkbox, Label, Select, SelectItem, SelectPopup, SelectTrigger, SelectValue, Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetPanel, SheetTitle, Switch } from "frontend";
import type { ReactNode } from "react";

// `SheetContent` is the shadcn-compatible alias for `SheetPopup` — same
// component, same props. This is the composition the birthdays tool ships.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const SectionLabel = ({ children }: { children?: ReactNode }) => <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{children}</span>;

const CheckRow = ({ label, count, checked }: { label: string; count: number; checked?: boolean }) => (
    <Label className="flex w-full items-center justify-between gap-3 py-1.5">
        <span className="flex items-center gap-2.5">
            <Checkbox defaultChecked={checked} />
            {label}
        </span>
        <span className="font-mono text-muted-foreground text-xs tabular-nums">{count}</span>
    </Label>
);

// The birthdays filter sheet: header counter, scrollable form, bordered footer
// whose confirm is a `SheetClose`.
export const BirthdayFilters = () => (
    <Stage>
        <Sheet open>
            <SheetContent className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>Filters</SheetTitle>
                    <p className="font-medium font-mono text-[11.5px] text-muted-foreground uppercase tracking-[0.08em]">37 of 312 match</p>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-5">
                    <div className="flex flex-col gap-2">
                        <SectionLabel>Month</SectionLabel>
                        <Select defaultValue="June">
                            <SelectTrigger className="w-full">
                                <SelectValue placeholder="Any month" />
                            </SelectTrigger>
                            <SelectPopup>
                                <SelectItem value="May">May</SelectItem>
                                <SelectItem value="June">June</SelectItem>
                                <SelectItem value="July">July</SelectItem>
                            </SelectPopup>
                        </Select>
                    </div>
                    <div className="flex flex-col gap-1">
                        <SectionLabel>Nation</SectionLabel>
                        <CheckRow checked count={64} label="Rhodes Island" />
                        <CheckRow count={38} label="Kazimierz" />
                        <CheckRow checked count={41} label="Lungmen" />
                        <CheckRow count={27} label="Victoria" />
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
            </SheetContent>
        </Sheet>
    </Stage>
);

// A detail sheet instead of a form — same alias, `variant="default"` so the
// panel meets the end edge.
export const OperatorDetail = () => (
    <Stage>
        <Sheet open>
            <SheetContent className="max-w-sm" initialFocus={false} side="right">
                <SheetHeader>
                    <div className="flex items-center gap-3">
                        <img alt="" className="size-12 shrink-0 rounded-lg border bg-muted object-cover" src="https://api.myrtle.moe/api/avatar/char_4064_mlynar" />
                        <div className="flex min-w-0 flex-col gap-1">
                            <SheetTitle className="text-base">Mlynar</SheetTitle>
                            <span className="truncate text-muted-foreground text-sm">6★ Guard · Soldier · Sarkaz</span>
                        </div>
                    </div>
                    <div className="flex flex-wrap gap-1.5 pt-1">
                        <Badge variant="outline">Nuker</Badge>
                        <Badge variant="outline">Survival</Badge>
                        <Badge variant="secondary">Owned</Badge>
                    </div>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-0">
                    {[
                        { label: "Promotion", value: "E2 90" },
                        { label: "Trust", value: "200%" },
                        { label: "Skill masteries", value: "M3 / M3 / M3" },
                        { label: "Module", value: "GUA-Y · Stage 3" },
                        { label: "ATK / DEF", value: "1,050 / 505" },
                        { label: "Deploy cost", value: "26 DP" },
                    ].map((row) => (
                        <div className="flex items-center justify-between border-b py-2.5 last:border-b-0" key={row.label}>
                            <span className="text-muted-foreground text-sm">{row.label}</span>
                            <span className="font-mono text-sm tabular-nums">{row.value}</span>
                        </div>
                    ))}
                </SheetPanel>
                <SheetFooter>
                    <SheetClose render={<Button variant="outline" />}>Close</SheetClose>
                    <Button>Open full profile</Button>
                </SheetFooter>
            </SheetContent>
        </Sheet>
    </Stage>
);
