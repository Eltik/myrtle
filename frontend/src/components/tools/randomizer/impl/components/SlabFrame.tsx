import type * as React from "react";
import { cn } from "#/lib/utils";

interface ISlabFrameProps {
    index: string;
    kicker: string;
    children: React.ReactNode;
    className?: string;
    accent?: "primary" | "lagoon" | "palm";
}

const ACCENT_LINE: Record<NonNullable<ISlabFrameProps["accent"]>, string> = {
    primary: "before:bg-primary/70",
    lagoon: "before:bg-[var(--lagoon)]",
    palm: "before:bg-[var(--palm)]",
};

/**
 * A "dossier slab" — a horizontal card with a numbered vertical rail on the left,
 * an interior content area, and a hairline accent stripe. Distinct from the old
 * glassmorphic gradient cards: flat surface, sharp typography, monospace numbering.
 */
export function SlabFrame({ index, kicker, children, className, accent = "primary" }: ISlabFrameProps): React.ReactElement {
    return (
        <section className={cn("relative overflow-hidden rounded-2xl border border-border/70 bg-card text-card-foreground shadow-xs/5", `before:absolute before:inset-y-0 before:left-0 before:w-[3px] ${ACCENT_LINE[accent]} before:content-['']`, className)} data-slot="slab-frame">
            <div className="flex min-w-0">
                <aside className="relative flex w-12 shrink-0 flex-col items-center justify-between border-r border-border/60 bg-muted/30 py-4 sm:w-14 sm:py-5">
                    <span className="font-mono text-[11px] tracking-[0.18em] text-muted-foreground/80 sm:text-[12px]">{index}</span>
                    <span className="font-mono [writing-mode:vertical-rl] rotate-180 text-[10px] uppercase tracking-[0.22em] text-foreground/70 sm:text-[11px]">{kicker}</span>
                    <span className="font-mono text-[11px] tracking-[0.18em] text-muted-foreground/40 sm:text-[12px]">— —</span>
                </aside>
                <div className="min-w-0 flex-1 p-4 sm:p-6">{children}</div>
            </div>
        </section>
    );
}
