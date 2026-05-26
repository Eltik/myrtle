import { SlidersHorizontal } from "lucide-react";
import * as React from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Sheet, SheetClose, SheetContent, SheetFooter, SheetHeader, SheetPanel, SheetTitle, SheetTrigger } from "#/components/ui/sheet";
import type { IBirthdayFilters } from "../types";
import { FilterControls } from "./FilterControls";

interface IFilterSheetProps {
    filters: IBirthdayFilters;
    onChange: (next: IBirthdayFilters) => void;
    nations: [string, string][];
    matched: number;
    total: number;
    /** Count of active facets, shown as a badge on the trigger. */
    activeCount: number;
    onReset: () => void;
}

/** Trigger button + side sheet holding the filter form; the trigger is hidden at `lg` and up. */
export function FilterSheet({ filters, onChange, nations, matched, total, activeCount, onReset }: IFilterSheetProps): React.ReactElement {
    const [open, setOpen] = React.useState(false);

    return (
        <Sheet open={open} onOpenChange={setOpen}>
            <SheetTrigger render={<Button variant="outline" className="w-full justify-center sm:w-auto lg:hidden" />}>
                <SlidersHorizontal />
                Filters
                {activeCount > 0 && (
                    <Badge size="sm" className="ml-0.5">
                        {activeCount}
                    </Badge>
                )}
            </SheetTrigger>
            <SheetContent side="right" variant="inset" className="w-[min(420px,100vw)]">
                <SheetHeader>
                    <SheetTitle>Filters</SheetTitle>
                    <p className="font-medium font-mono text-[11.5px] text-muted-foreground uppercase tracking-[0.08em]">
                        {matched} of {total} match
                    </p>
                </SheetHeader>
                <SheetPanel className="px-5 pt-1 pb-2">
                    <FilterControls filters={filters} onChange={onChange} nations={nations} />
                </SheetPanel>
                <SheetFooter>
                    <Button variant="ghost" onClick={onReset} disabled={activeCount === 0}>
                        Reset
                    </Button>
                    <SheetClose render={<Button />}>Show {matched} results</SheetClose>
                </SheetFooter>
            </SheetContent>
        </Sheet>
    );
}
