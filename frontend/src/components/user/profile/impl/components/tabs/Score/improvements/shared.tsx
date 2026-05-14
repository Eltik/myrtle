import type { ReactNode } from "react";
import { cn } from "#/lib/utils";

export const PANEL_PADDING = "px-4 pb-4 sm:px-5 sm:pb-5";
export const PANEL_TEXT = "font-mono text-[11px] tabular-nums";

/**
 * Mini-section header inside an expanded subscore panel.
 */
export function SectionHeader({ title, count, accent }: { title: string; count?: ReactNode; accent: string }) {
    return (
        <div className="flex items-center justify-between border-b border-border/40 pb-1.5">
            <span className="font-mono text-[10.5px] font-semibold uppercase tracking-[0.12em] text-muted-foreground/85" style={{ color: `color-mix(in oklch, ${accent} 60%, var(--foreground))` }}>
                {title}
            </span>
            {count !== undefined && <span className="rounded-md border border-border/40 bg-muted/30 px-1.5 py-0.5 font-mono text-[9.5px] font-semibold tabular-nums text-muted-foreground">{count}</span>}
        </div>
    );
}

/**
 * "X / Y" progress chip with a thin bar underneath.
 */
export function ProgressLine({ label, current, max, accent }: { label: string; current: number; max: number; accent: string }) {
    const pct = max > 0 ? Math.min(100, (current / max) * 100) : 0;
    return (
        <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between text-[11px]">
                <span className="text-muted-foreground">{label}</span>
                <span className="font-mono tabular-nums text-foreground/85">
                    {current.toLocaleString()} <span className="text-muted-foreground">/ {max.toLocaleString()}</span>
                </span>
            </div>
            <div className="h-1 w-full overflow-hidden rounded-full bg-muted/40">
                <div
                    className="h-full rounded-full transition-[width] duration-500 ease-out"
                    style={{
                        width: `${pct}%`,
                        background: `linear-gradient(to right, color-mix(in oklch, ${accent} 55%, transparent), ${accent})`,
                    }}
                />
            </div>
        </div>
    );
}

export function EmptyHint({ children }: { children: ReactNode }) {
    return <p className="rounded-md border border-dashed border-border/40 bg-muted/15 px-3 py-2 text-center text-[11px] text-muted-foreground">{children}</p>;
}

interface IPillProps {
    children: ReactNode;
    color?: string;
    className?: string;
}

export function Pill({ children, color, className }: IPillProps) {
    return (
        <span
            className={cn("inline-flex items-center gap-1 rounded-md border border-border/50 px-1.5 py-0.5 font-mono text-[9.5px] font-semibold uppercase tracking-wider tabular-nums text-muted-foreground/85", className)}
            style={color ? { background: `color-mix(in oklch, ${color} 8%, transparent)`, color: `color-mix(in oklch, ${color} 75%, var(--foreground))` } : undefined}
        >
            {children}
        </span>
    );
}
