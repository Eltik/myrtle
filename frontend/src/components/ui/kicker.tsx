import type * as React from "react";
import { cn } from "#/lib/utils";

/**
 * The small uppercase label over a card or section, in the one `.kicker` voice (mono,
 * muted). `tone="accent"` keeps the red for the rare label that has to pull the eye;
 * spent on every eyebrow, the red stopped marking anything.
 */
export function Kicker({ className, tone = "muted", ...props }: React.ComponentProps<"span"> & { tone?: "muted" | "accent" }): React.ReactElement {
    return <span className={cn("kicker inline-block", tone === "accent" && "text-primary", className)} data-slot="kicker" {...props} />;
}
