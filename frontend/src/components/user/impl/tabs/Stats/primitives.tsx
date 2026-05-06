import type { ElementType, ReactNode } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";

export const KICKER_TEXT = "font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em] text-muted-foreground";
export const CARD_PADDING = "p-4 sm:p-5";

function barGradient(color: string): string {
    return `linear-gradient(to right, color-mix(in oklch, ${color} 55%, transparent), ${color})`;
}

function lineGradient(color: string | readonly string[]): string {
    if (typeof color === "string") {
        return `linear-gradient(to right, transparent, color-mix(in oklch, ${color} 70%, transparent), transparent)`;
    }
    const stops = color.map((c) => `color-mix(in oklch, ${c} 70%, transparent)`).join(", ");
    return `linear-gradient(to right, transparent, ${stops}, transparent)`;
}

interface IStatCardProps {
    children: ReactNode;
    color: string | readonly string[];
    className?: string;
}

export function StatCard({ children, color, className }: IStatCardProps) {
    return (
        <div className={cn("relative overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl", "shadow-[0_1px_2px_rgb(0_0_0/0.04)] transition-colors duration-200 hover:border-foreground/15", className)}>
            <div aria-hidden className="pointer-events-none absolute inset-x-0 top-0 z-10 h-px" style={{ background: lineGradient(color) }} />
            {children}
        </div>
    );
}

export function Kicker({ icon: Icon, label }: { icon: ElementType; label: string }) {
    return (
        <div className="flex items-center gap-2">
            <Icon className="h-3.5 w-3.5 shrink-0 text-muted-foreground/60" />
            <span className={KICKER_TEXT}>{label}</span>
        </div>
    );
}

interface IBarProps {
    pct: number;
    color: string;
    thin?: boolean;
    dim?: boolean;
}

export function Bar({ pct, color, thin, dim }: IBarProps) {
    return (
        <div className={cn("w-full overflow-hidden rounded-full bg-muted/50", thin ? "h-1" : "h-1.5")}>
            <div
                className="h-full rounded-full transition-[width] duration-700 ease-out"
                style={{
                    width: `${Math.min(pct, 100)}%`,
                    background: barGradient(color),
                    opacity: dim ? 0.5 : 1,
                }}
            />
        </div>
    );
}

interface ITileProps {
    value: ReactNode;
    sub: string;
    color: string;
    tooltip?: string;
}

export function Tile({ value, sub, color, tooltip }: ITileProps) {
    const inner = (
        <div className="flex cursor-default flex-col items-center gap-1.5 rounded-lg border border-border/40 bg-muted/20 p-3 text-center transition-colors hover:bg-muted/35">
            <span className="font-bold tabular-nums leading-none" style={{ fontSize: "1.375rem", letterSpacing: "-0.02em", color }}>
                {value}
            </span>
            <span className={KICKER_TEXT}>{sub}</span>
        </div>
    );
    if (!tooltip) return inner;
    return (
        <Tooltip>
            <TooltipTrigger render={inner} />
            <TooltipContent sideOffset={5}>
                <p>{tooltip}</p>
            </TooltipContent>
        </Tooltip>
    );
}

interface IMetricRowProps {
    label: string;
    value: ReactNode;
    pct: number;
    color: string;
}

export function MetricRow({ label, value, pct, color }: IMetricRowProps) {
    return (
        <div className="space-y-2">
            <div className="flex items-center justify-between">
                <span className={KICKER_TEXT}>{label}</span>
                <span className="font-mono text-[10.5px] tabular-nums text-muted-foreground">{value}</span>
            </div>
            <Bar color={color} pct={pct} />
        </div>
    );
}
