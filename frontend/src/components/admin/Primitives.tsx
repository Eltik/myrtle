import type * as React from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { cn } from "#/lib/utils";

interface IStatTileProps {
    label: string;
    value: string;
    unit?: string;
    delta?: string;
    deltaDir?: "up" | "down";
    color?: string;
    spark?: string;
}

export function StatTile({ label, value, unit, delta, deltaDir = "up", color = "var(--primary)", spark }: IStatTileProps): React.ReactElement {
    return (
        <div className="relative min-w-0 rounded-2xl border border-border bg-card p-3.5 shadow-xs/5 before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-2xl)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] sm:p-4.5 dark:before:shadow-[0_-1px_--theme(--color-white/6%)]">
            <div className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-tight tracking-[0.08em]">{label}</div>
            <div className="mt-2 flex flex-wrap items-baseline gap-x-1 font-bold font-sans text-[22px] tabular-nums leading-none tracking-[-0.02em] sm:mt-2.5 sm:text-[26px]">
                <span className="break-all">{value}</span>
                {unit ? <span className="font-medium font-mono text-[11px] text-muted-foreground sm:text-[12px]">{unit}</span> : null}
            </div>
            {delta != null ? (
                <div className={cn("mt-2 inline-flex items-center gap-1 font-medium font-mono text-[11px]", deltaDir === "up" ? "text-emerald-500" : "text-destructive-foreground")}>
                    {deltaDir === "up" ? "▴" : "▾"} {delta}
                </div>
            ) : null}
            {spark ? (
                <svg className="mt-2.5 block h-7 w-full" viewBox="0 0 200 28" preserveAspectRatio="none" aria-hidden>
                    <title>trend</title>
                    <polyline fill="none" stroke={color} strokeWidth="1.6" points={spark} />
                </svg>
            ) : null}
        </div>
    );
}

interface IStatusDotProps {
    state?: "green" | "amber" | "red";
    pulse?: boolean;
    children?: React.ReactNode;
}

export function StatusDot({ state = "green", pulse = false, children }: IStatusDotProps): React.ReactElement {
    const colorMap: Record<string, string> = {
        green: "bg-emerald-400 shadow-[0_0_0_3px_oklch(0.75_0.17_155/0.2)]",
        amber: "bg-amber-500 shadow-[0_0_0_3px_oklch(0.75_0.16_84/0.18)]",
        red: "bg-destructive shadow-[0_0_0_3px_oklch(0.577_0.245_27/0.2)]",
    };
    return (
        <span className="inline-flex items-center gap-1.5 font-medium text-[12px]">
            <span className={cn("size-1.75 shrink-0 rounded-full", colorMap[state], pulse && "animate-pulse")} />
            {children}
        </span>
    );
}

/**
 * Pins a table's last (row actions) column to the right edge of its scroller,
 * so the actions stay reachable when a narrow screen scrolls the table
 * sideways. The cell paints the card under it and repeats the row's hover and
 * selected tints (the same mixes `TableRow` uses), so it does not read as a
 * separate block.
 */
export const stickyActionsCell =
    "sticky right-0 z-1 bg-card shadow-[-8px_0_8px_-8px_var(--border)] [tr:hover>&]:bg-[color-mix(in_srgb,var(--background),var(--color-black)_2%)] [tr[data-state=selected]>&]:bg-[color-mix(in_srgb,var(--background),var(--color-black)_4%)] dark:[tr:hover>&]:bg-[color-mix(in_srgb,var(--background),var(--color-white)_2%)] dark:[tr[data-state=selected]>&]:bg-[color-mix(in_srgb,var(--background),var(--color-white)_4%)]";

/** The search-and-filters row across the top of a table card. */
export function AdminToolbar({ children }: { children?: React.ReactNode }): React.ReactElement {
    return <div className="flex flex-wrap items-center gap-3 border-border border-b p-3.5">{children}</div>;
}

/**
 * A centred muted line standing in for a panel's content (loading, empty,
 * failed). `className` carries the padding and leads the class list.
 */
export function PanelMessage({ className, children }: { className: string; children?: React.ReactNode }): React.ReactElement {
    return <div className={`${className} text-center text-[14px] text-muted-foreground`}>{children}</div>;
}

/** `count` loading placeholders of one shape. */
export function skeletons(count: number, className: string): React.ReactElement[] {
    return Array.from({ length: count }, (_, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: static placeholders
        <Skeleton key={i} className={className} />
    ));
}

/** A card header that stacks its action under the title on phones (`stackedCardAction` goes on the `CardAction`). */
export const stackedCardHeader = "max-sm:grid-cols-1!";
export const stackedCardAction = "max-sm:col-start-1 max-sm:row-span-1 max-sm:row-start-3 max-sm:mt-2 max-sm:justify-self-start";
