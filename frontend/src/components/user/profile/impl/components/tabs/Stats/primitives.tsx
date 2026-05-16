import { ArrowUpRight, ChevronDown } from "lucide-react";
import { type ElementType, type ReactNode, useState } from "react";
import { Collapsible, CollapsibleContent } from "#/components/ui/collapsible";
import { Dialog, DialogTrigger } from "#/components/ui/dialog";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipContent, TooltipTrigger } from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import type { IOperatorGapItem } from "./helpers";

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
                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{value}</span>
            </div>
            <Bar color={color} pct={pct} />
        </div>
    );
}

export interface IGapItem {
    key: string;
    label: string;
    value: number;
    color: string;
    tooltip?: string;
    /** Inline collapsible list — best for compact, scannable detail. */
    details?: IOperatorGapItem[];
    /** When provided, clicking the pill opens this node inside a dialog instead
     *  of expanding inline. Use for richer drill-down experiences (large images,
     *  multi-column layouts, etc.). Mutually exclusive with `details`. */
    dialogContent?: ReactNode;
}

interface IGapListProps {
    items: IGapItem[];
    label?: string;
}

export function GapList({ items, label = "To Go" }: IGapListProps) {
    const [openKey, setOpenKey] = useState<string | null>(null);
    const [lastKey, setLastKey] = useState<string | null>(null);
    const visible = items.filter((item) => item.value > 0);

    if (visible.length === 0) {
        return (
            <div className="flex items-center gap-1.5">
                <span aria-hidden className="h-1 w-1 rounded-full bg-emerald-500/70" />
                <span className={cn(KICKER_TEXT, "text-emerald-500/80")}>All Complete</span>
            </div>
        );
    }

    const handleToggle = (key: string) => {
        setOpenKey((cur) => {
            const next = cur === key ? null : key;
            if (next) setLastKey(next);
            return next;
        });
    };

    const displayKey = openKey ?? lastKey;
    const display = displayKey ? visible.find((i) => i.key === displayKey) : null;
    const isOpen = openKey !== null;

    return (
        <div className="space-y-2.5">
            <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1">
                <span className={KICKER_TEXT}>{label}</span>
                {visible.map((item) => {
                    if (item.dialogContent) {
                        return (
                            <Dialog key={item.key}>
                                <DialogTrigger aria-label={`Show ${item.value} ${item.label}`} className={pillClasses(false)}>
                                    <GapPillContent item={item} mode="dialog" />
                                </DialogTrigger>
                                {item.dialogContent}
                            </Dialog>
                        );
                    }
                    const canCollapse = (item.details?.length ?? 0) > 0;
                    return <GapPill expanded={openKey === item.key} item={item} key={item.key} mode={canCollapse ? "collapse" : "static"} onToggle={canCollapse ? () => handleToggle(item.key) : undefined} />;
                })}
            </div>
            <Collapsible onOpenChange={(open) => !open && setOpenKey(null)} open={isOpen}>
                <CollapsibleContent>{display?.details ? <GapDetailsPanel color={display.color} items={display.details} title={display.label} /> : null}</CollapsibleContent>
            </Collapsible>
        </div>
    );
}

function pillClasses(expanded: boolean): string {
    return cn("inline-flex cursor-pointer items-center gap-1.5 rounded-md px-1.5 py-0.5 transition-colors hover:bg-muted/50", expanded && "bg-muted/60");
}

interface IGapPillContentProps {
    item: IGapItem;
    mode: "collapse" | "dialog";
    expanded?: boolean;
}

function GapPillContent({ item, mode, expanded = false }: IGapPillContentProps) {
    return (
        <>
            <span aria-hidden className="h-1 w-1 shrink-0 rounded-full" style={{ background: item.color }} />
            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                <span className="font-semibold" style={{ color: item.color }}>
                    {item.value.toLocaleString()}
                </span>{" "}
                {item.label}
            </span>
            {mode === "collapse" && <ChevronDown aria-hidden className={cn("h-2.5 w-2.5 text-muted-foreground/50 transition-transform duration-200", expanded && "rotate-180")} />}
            {mode === "dialog" && <ArrowUpRight aria-hidden className="h-2.5 w-2.5 text-muted-foreground/50" />}
        </>
    );
}

interface IGapPillProps {
    item: IGapItem;
    expanded: boolean;
    mode: "static" | "collapse";
    onToggle?: () => void;
}

function GapPill({ item, expanded, mode, onToggle }: IGapPillProps) {
    if (mode === "static") {
        const staticNode = (
            <span className="inline-flex cursor-default items-center gap-1.5 rounded-md">
                <span aria-hidden className="h-1 w-1 shrink-0 rounded-full" style={{ background: item.color }} />
                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                    <span className="font-semibold" style={{ color: item.color }}>
                        {item.value.toLocaleString()}
                    </span>{" "}
                    {item.label}
                </span>
            </span>
        );
        if (!item.tooltip) return staticNode;
        return (
            <Tooltip>
                <TooltipTrigger render={staticNode} />
                <TooltipContent sideOffset={5}>
                    <p>{item.tooltip}</p>
                </TooltipContent>
            </Tooltip>
        );
    }

    const trigger = (
        <button aria-expanded={expanded} aria-label={`Show ${item.value} ${item.label}`} className={pillClasses(expanded)} onClick={onToggle} type="button">
            <GapPillContent expanded={expanded} item={item} mode="collapse" />
        </button>
    );

    if (!item.tooltip) return trigger;
    return (
        <Tooltip>
            <TooltipTrigger render={trigger} />
            <TooltipContent sideOffset={5}>
                <p>{item.tooltip}</p>
            </TooltipContent>
        </Tooltip>
    );
}

const MAX_RENDERED_DETAILS = 100;

interface IGapDetailsPanelProps {
    title: string;
    color: string;
    items: IOperatorGapItem[];
}

function GapDetailsPanel({ title, color, items }: IGapDetailsPanelProps) {
    const rendered = items.length > MAX_RENDERED_DETAILS ? items.slice(0, MAX_RENDERED_DETAILS) : items;
    const overflow = items.length - rendered.length;

    return (
        <div className="rounded-lg border border-border/60 bg-muted/15">
            <div className="flex items-center justify-between border-border/40 border-b px-3 py-1.5">
                <span className={KICKER_TEXT}>{title}</span>
                <span className="font-mono text-[10px] text-muted-foreground/70 tabular-nums">
                    <span className="font-semibold" style={{ color }}>
                        {items.length.toLocaleString()}
                    </span>{" "}
                    total
                </span>
            </div>
            <ul className="max-h-64 space-y-px overflow-y-auto p-1.5">
                {rendered.map((d) => (
                    <li className="flex items-center gap-2 rounded-md px-1.5 py-1 hover:bg-muted/40" key={d.id}>
                        <span className="flex h-6 w-6 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted/60 font-semibold text-[10px]">
                            <OperatorAvatar charId={d.charId} name={d.name} />
                        </span>
                        <span className="min-w-0 flex-1 truncate font-medium text-xs">{d.name}</span>
                        {d.sub && <span className="shrink-0 font-mono text-[10px] text-muted-foreground/80 tabular-nums">{d.sub}</span>}
                    </li>
                ))}
                {overflow > 0 && <li className="px-1.5 pt-1 text-center font-mono text-[10px] text-muted-foreground/60">+ {overflow.toLocaleString()} more (not rendered)</li>}
            </ul>
        </div>
    );
}
