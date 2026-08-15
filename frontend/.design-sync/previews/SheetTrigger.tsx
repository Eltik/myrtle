import { Badge, Button, Sheet, SheetClose, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle, SheetTrigger } from "frontend";
import { SettingsIcon, SlidersHorizontalIcon, UserRoundIcon } from "lucide-react";
import type { ReactNode } from "react";

// The trigger is only visible while the sheet is closed, so these stories show
// the toolbars it actually lives in. It takes `render` to borrow any button.
const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const FiltersSheet = () => (
    <SheetPopup className="w-[min(420px,100vw)]" side="right" variant="inset">
        <SheetHeader>
            <SheetTitle>Filters</SheetTitle>
            <SheetDescription>Narrow the roster by class, rarity and nation.</SheetDescription>
        </SheetHeader>
        <SheetFooter>
            <Button variant="ghost">Reset</Button>
            <SheetClose render={<Button />}>Show 37 results</SheetClose>
        </SheetFooter>
    </SheetPopup>
);

// A list toolbar: one labelled trigger with a count badge, one icon-only.
export const InListToolbar = () => (
    <div className="w-full max-w-2xl rounded-xl border bg-card p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex min-w-0 flex-col gap-0.5">
                <span className="font-heading font-semibold text-base">Operators</span>
                <span className="text-muted-foreground text-sm">312 total · 37 shown</span>
            </div>
            <div className="flex items-center gap-2">
                <Sheet>
                    <SheetTrigger render={<Button variant="outline" />}>
                        <SlidersHorizontalIcon />
                        Filters
                        <Badge className="ml-0.5" size="sm">
                            3
                        </Badge>
                    </SheetTrigger>
                    <FiltersSheet />
                </Sheet>
                <Sheet>
                    <SheetTrigger aria-label="Randomizer settings" render={<Button size="icon" variant="ghost" />}>
                        <SettingsIcon />
                    </SheetTrigger>
                    <FiltersSheet />
                </Sheet>
            </div>
        </div>
    </div>
);

// `render` is not limited to buttons — a whole row can be the trigger.
export const AsRowTrigger = () => (
    <div className="flex w-full max-w-md flex-col gap-2">
        {[
            { name: "Mlynar", meta: "6★ Guard · E2 90 · M3/M3/M3" },
            { name: "Muelsyse", meta: "6★ Specialist · E2 60 · M3/M1/M1" },
            { name: "Texas the Omertosa", meta: "6★ Specialist · E2 45 · M3/-/-" },
        ].map((op) => (
            <Sheet key={op.name}>
                <SheetTrigger className="flex w-full items-center gap-3 rounded-lg border bg-card px-3 py-2.5 text-left">
                    <UserRoundIcon aria-hidden="true" className="size-4 shrink-0 text-muted-foreground" />
                    <span className="flex min-w-0 flex-col gap-0.5">
                        <span className="truncate font-medium text-sm">{op.name}</span>
                        <span className="truncate text-muted-foreground text-xs">{op.meta}</span>
                    </span>
                </SheetTrigger>
                <SheetPopup className="w-[min(420px,100vw)]" side="right" variant="inset">
                    <SheetHeader>
                        <SheetTitle>{op.name}</SheetTitle>
                    </SheetHeader>
                </SheetPopup>
            </Sheet>
        ))}
    </div>
);

// Opened — the trigger has handed off to the popup and is covered by the
// backdrop.
export const Opened = () => (
    <Stage>
        <div className="flex items-center justify-between gap-3 rounded-xl border bg-card p-4">
            <span className="font-heading font-semibold text-base">Operators</span>
            <Sheet open>
                <SheetTrigger render={<Button variant="outline" />}>
                    <SlidersHorizontalIcon />
                    Filters
                </SheetTrigger>
                <SheetPopup className="w-[min(420px,100vw)]" initialFocus={false} side="right" variant="inset">
                    <SheetHeader>
                        <SheetTitle>Filters</SheetTitle>
                        <SheetDescription>Narrow the roster by class, rarity and nation.</SheetDescription>
                    </SheetHeader>
                    <SheetPanel className="flex flex-col gap-2">
                        {["Guard · 51", "Sniper · 44", "Caster · 39", "Medic · 31"].map((row) => (
                            <div className="rounded-lg border px-3 py-2 text-sm" key={row}>
                                {row}
                            </div>
                        ))}
                    </SheetPanel>
                    <SheetFooter>
                        <Button variant="ghost">Reset</Button>
                        <SheetClose render={<Button />}>Show 37 results</SheetClose>
                    </SheetFooter>
                </SheetPopup>
            </Sheet>
        </div>
    </Stage>
);
