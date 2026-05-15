import type { ReactNode } from "react";
import { cn } from "#/lib/utils";

// ─── Layout ─────────────────────────────────────────────────────────────────

export const PANEL_PADDING = "px-4 pb-4 sm:px-5 sm:pb-5";

// ─── Typography ─────────────────────────────────────────────────────────────
// One scale across every improvement panel. Keep these in sync; new sizes
// don't go inline - extend the scale here.

/** XS mono - tiny chips, level/state indicators, badges. */
export const TEXT_BADGE = "font-mono text-[10px] tabular-nums";
/** XS uppercase kicker - section headers, button labels, tag captions. */
export const TEXT_KICKER = "font-mono text-[10.5px] font-semibold uppercase tracking-[0.12em]";
/** SM - secondary descriptive text, subtitles. */
export const TEXT_META = "text-[10.5px] leading-snug";
/** Body - list rows, descriptions, default content size. */
export const TEXT_BODY = "text-[11.5px] leading-tight";

// ─── Color palette ──────────────────────────────────────────────────────────
// Shared rarity scheme so the Operator and Medal panels read with the same
// visual language. Hue picks roughly match Arknights' in-game color tiers
// (gold → purple → blue → gray).

const RARITY_HUE = {
    /** Highest tier - gold. */
    apex: "oklch(0.78 0.16 80)",
    /** Gold (5★ / T2.5 / T3). */
    gold: "oklch(0.74 0.16 80)",
    /** Purple (4★ / T2). */
    purple: "oklch(0.67 0.17 295)",
    /** Blue (3★ / T1.5). */
    blue: "oklch(0.65 0.13 220)",
    /** Muted gray (2★/1★ / T1). */
    muted: "oklch(0.58 0.04 285)",
} as const;

/** Operator rarity (1-6 ★). */
export function operatorRarityColor(rarity: number): string {
    if (rarity >= 6) return RARITY_HUE.apex;
    if (rarity === 5) return RARITY_HUE.gold;
    if (rarity === 4) return RARITY_HUE.purple;
    if (rarity === 3) return RARITY_HUE.blue;
    return RARITY_HUE.muted;
}

/** Medal rarity (T1 → T3D5). */
export function medalRarityColor(rarity: string): string {
    switch (rarity) {
        case "T3D5":
            return RARITY_HUE.apex;
        case "T3":
        case "T2D5":
            return RARITY_HUE.gold;
        case "T2":
            return RARITY_HUE.purple;
        case "T1D5":
            return RARITY_HUE.blue;
        default:
            return RARITY_HUE.muted;
    }
}

/** Urgency color for time-bound items (event medals running out). */
export const URGENT_COLOR = "oklch(0.65 0.22 30)";

// ─── Reusable primitives ────────────────────────────────────────────────────

/**
 * Mini-section header inside an expanded subscore panel.
 */
export function SectionHeader({ title, count, accent }: { title: string; count?: ReactNode; accent: string }) {
    return (
        <div className="flex items-center justify-between border-border/40 border-b pb-1.5">
            <span className={cn(TEXT_KICKER)} style={{ color: `color-mix(in oklch, ${accent} 60%, var(--foreground))` }}>
                {title}
            </span>
            {count !== undefined && <span className={cn(TEXT_BADGE, "rounded-md border border-border/40 bg-muted/30 px-1.5 py-0.5 text-muted-foreground")}>{count}</span>}
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
            <div className="flex items-center justify-between">
                <span className="text-[11px] text-muted-foreground">{label}</span>
                <span className={cn(TEXT_BADGE, "text-foreground/85")}>
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
    return <p className="rounded-md border border-border/40 border-dashed bg-muted/15 px-3 py-2 text-center text-[11px] text-muted-foreground">{children}</p>;
}

interface IPillProps {
    children: ReactNode;
    color?: string;
    className?: string;
}

export function Pill({ children, color, className }: IPillProps) {
    return (
        <span className={cn("inline-flex items-center gap-1 rounded-md border border-border/50 px-1.5 py-0.5", TEXT_KICKER, "text-muted-foreground/85", className)} style={color ? { background: `color-mix(in oklch, ${color} 8%, transparent)`, color: `color-mix(in oklch, ${color} 75%, var(--foreground))` } : undefined}>
            {children}
        </span>
    );
}

/**
 * Show-more / show-less toggle styled to match the rest of the panel chrome.
 */
export function ShowMoreButton({ onClick, label }: { onClick: () => void; label: string }) {
    return (
        <button type="button" onClick={onClick} className={cn("self-start", TEXT_KICKER, "text-muted-foreground transition-colors hover:text-foreground")}>
            {label}
        </button>
    );
}

/**
 * Stat tile: small card with a big number, a label below, optional sub-line.
 * Used for top-of-panel summary breakdowns.
 *
 * - Centered content (numbers + labels read as one stacked unit).
 * - `min-h` keeps every tile in a row the same height even if one sub-label wraps.
 * - Non-interactive tiles (no `onClick`) render dimmed so "0" doesn't look clickable.
 */
export function StatTile({ value, label, sub, accent, active, onClick }: { value: ReactNode; label: string; sub?: ReactNode; accent: string; active?: boolean; onClick?: () => void }) {
    const interactive = onClick !== undefined;
    const Wrapper = interactive ? "button" : "div";
    return (
        <Wrapper
            type={interactive ? "button" : undefined}
            onClick={onClick}
            aria-pressed={interactive ? active : undefined}
            className={cn("flex min-h-22 flex-col items-center justify-center gap-1 rounded-md border px-2 py-2.5 text-center transition-colors", active ? "border-foreground/30 bg-muted/40" : "border-border/40 bg-muted/15", interactive ? "cursor-pointer hover:border-foreground/25 hover:bg-muted/30" : "opacity-55")}
        >
            <span
                className="font-bold tabular-nums leading-none"
                style={{
                    fontSize: "1.5rem",
                    letterSpacing: "-0.02em",
                    color: `color-mix(in oklch, ${accent} 70%, var(--foreground))`,
                }}
            >
                {value}
            </span>
            <span className={cn(TEXT_KICKER, "text-foreground/75")}>{label}</span>
            {sub !== undefined && <span className={cn(TEXT_BADGE, "text-muted-foreground/70 leading-tight")}>{sub}</span>}
        </Wrapper>
    );
}
