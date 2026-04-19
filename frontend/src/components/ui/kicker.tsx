import type * as React from "react";
import { cn } from "#/lib/utils";

export function Kicker({ className, ...props }: React.ComponentProps<"span">): React.ReactElement {
    return <span className={cn("inline-block text-[0.69rem] font-bold tracking-[0.22em] uppercase text-primary mb-2.5", className)} data-slot="kicker" {...props} />;
}
