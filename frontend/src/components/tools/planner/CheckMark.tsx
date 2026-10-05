import type * as React from "react";

import { cn } from "#/lib/utils";

/** The tick box at the start of a multi-select combobox row, shared by the operator and group pickers. */
export function CheckMark({ checked }: { checked: boolean }): React.ReactElement {
    return (
        <span className={cn("flex size-4.5 shrink-0 items-center justify-center rounded-sm border transition-colors sm:size-4", checked ? "border-primary bg-primary text-primary-foreground" : "border-input bg-background")}>
            {checked && (
                <svg aria-hidden="true" className="size-3 sm:size-2.5" fill="none" height="24" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="3" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg">
                    <path d="M5.252 12.7 10.2 18.63 18.748 5.37" />
                </svg>
            )}
        </span>
    );
}
