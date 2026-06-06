import { ChevronRight, Plus } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { OperatorPlannerDialog } from "./OperatorPlannerDialog";

/** Page wrapper for the operator planner tool. */
export function OperatorPlanner(): React.ReactElement {
    const [open, setOpen] = React.useState(false);

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-1.5rem))] py-4 pb-24 sm:w-[min(1400px,calc(100%-2rem))] sm:py-5 sm:pb-20">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                <span>Tools</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">Operator Planner</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <h1 className="m-0 font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">Operator Planner</h1>
                    <p className="mt-1.5 max-w-2xl font-sans text-[13px] text-muted-foreground leading-normal sm:text-[13.5px]">Plan your operator promotion, level, skill, and module targets.</p>
                </div>
            </div>

            <div className="mt-16 flex flex-col items-center justify-center gap-4 py-20 text-center sm:mt-24 sm:py-28">
                <Button size="xl" className="shadow-lg" onClick={() => setOpen(true)}>
                    <Plus className="size-5" />
                    Create new plan
                </Button>
            </div>

            <OperatorPlannerDialog open={open} onOpenChange={setOpen} />
        </div>
    );
}
