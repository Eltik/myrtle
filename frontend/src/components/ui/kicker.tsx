import type * as React from "react";
import { cn } from "#/lib/utils";

export function Kicker({ className, ...props }: React.ComponentProps<"span">): React.ReactElement {
    return <span className={cn("mb-2.5 inline-block font-bold text-[0.69rem] text-primary uppercase tracking-[0.22em]", className)} data-slot="kicker" {...props} />;
}
