import type * as React from "react";
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
        <div className="relative rounded-2xl border border-border bg-card p-4.5 shadow-xs/5 before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-2xl)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] dark:before:shadow-[0_-1px_--theme(--color-white/6%)]">
            <div className="font-mono font-medium text-[10.5px] uppercase tracking-[0.1em] text-muted-foreground">{label}</div>
            <div className="mt-2.5 font-bold font-sans text-[26px] leading-none tracking-[-0.02em] tabular-nums">
                {value}
                {unit ? <span className="ml-1 font-mono font-medium text-[12px] text-muted-foreground">{unit}</span> : null}
            </div>
            {delta != null ? (
                <div className={cn("mt-2 inline-flex items-center gap-1 font-mono font-medium text-[11px]", deltaDir === "up" ? "text-emerald-500" : "text-destructive-foreground")}>
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
            <span className={cn("size-[7px] shrink-0 rounded-full", colorMap[state], pulse && "animate-pulse")} />
            {children}
        </span>
    );
}

interface IHitBarProps {
    value: number;
    width?: number;
}

export function HitBar({ value, width = 60 }: IHitBarProps): React.ReactElement {
    const color = value >= 95 ? "#10b981" : value >= 85 ? "#f59e0b" : "oklch(0.577 0.245 27.325)";
    return (
        <span className="inline-flex items-center gap-2">
            <span className="inline-block h-1.5 overflow-hidden rounded-[3px] bg-muted" style={{ width }}>
                <span className="block h-full" style={{ width: `${value}%`, background: color }} />
            </span>
            <span className="font-mono font-medium text-[11.5px] tabular-nums text-muted-foreground">{value}%</span>
        </span>
    );
}

interface ICardKVProps {
    k: string;
    v: React.ReactNode;
}

export function CardKV({ k, v }: ICardKVProps): React.ReactElement {
    return (
        <div className="flex items-center justify-between border-border border-b py-2 last:border-0">
            <span className="font-mono text-[12.5px]">{k}</span>
            <span className="font-mono text-[12.5px] text-muted-foreground">{v}</span>
        </div>
    );
}

export function Kicker({ children }: { children: React.ReactNode }): React.ReactElement {
    return <span className="font-bold text-[11px] uppercase tracking-[0.22em] text-primary">{children}</span>;
}

export function MonoSection({ children }: { children: React.ReactNode }): React.ReactElement {
    return <span className="font-mono font-medium text-[11px] uppercase tracking-[0.1em] text-muted-foreground">{children}</span>;
}

interface IRoleBadgeProps {
    role: string;
}

export function RoleBadge({ role }: IRoleBadgeProps): React.ReactElement {
    if (role === "super_admin") return <span className="inline-flex h-[18px] items-center rounded-[4px] bg-primary px-1.5 font-medium text-[11px] leading-none text-primary-foreground">super_admin</span>;
    if (role === "tier_list_admin")
        return (
            <span className="inline-flex h-[18px] items-center gap-1 rounded-[4px] bg-success/8 px-1.5 font-medium text-[11px] leading-none text-success-foreground">
                <span className="size-[6px] rounded-full bg-success-foreground/85" />
                tier_list_admin
            </span>
        );
    if (role === "tier_list_editor")
        return (
            <span className="inline-flex h-[18px] items-center gap-1 rounded-[4px] bg-info/8 px-1.5 font-medium text-[11px] leading-none text-info-foreground">
                <span className="size-[6px] rounded-full bg-info-foreground/85" />
                tier_list_editor
            </span>
        );
    return <span className="font-mono text-[12px] text-muted-foreground">user</span>;
}

interface ILevelBadgeProps {
    level: "View" | "Edit" | "Publish" | "Admin";
}

export function LevelBadge({ level }: ILevelBadgeProps): React.ReactElement {
    if (level === "Admin") return <span className="inline-flex h-[18px] items-center rounded-[4px] bg-primary px-1.5 font-medium text-[11px] leading-none text-primary-foreground">Admin</span>;
    if (level === "Publish")
        return (
            <span className="inline-flex h-[18px] items-center gap-1 rounded-[4px] bg-success/8 px-1.5 font-medium text-[11px] leading-none text-success-foreground">
                <span className="size-[6px] rounded-full bg-success-foreground/85" />
                Publish
            </span>
        );
    if (level === "Edit")
        return (
            <span className="inline-flex h-[18px] items-center gap-1 rounded-[4px] bg-info/8 px-1.5 font-medium text-[11px] leading-none text-info-foreground">
                <span className="size-[6px] rounded-full bg-info-foreground/85" />
                Edit
            </span>
        );
    return <span className="inline-flex h-[18px] items-center rounded-[4px] border border-input bg-background px-1.5 font-medium text-[11px] leading-none text-foreground">View</span>;
}

interface ITimelineProps {
    items: ITimelineItem[];
}

export interface ITimelineItem {
    when: string;
    what: React.ReactNode;
    who?: React.ReactNode;
    muted?: boolean;
}

export function Timeline({ items }: ITimelineProps): React.ReactElement {
    return (
        <div className="relative pl-4.5">
            <div className="absolute top-1 bottom-1 left-1 w-0.5 rounded bg-border" />
            {items.map((it, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: timeline items are append-only and positional
                <div key={i} className="relative pb-3.5 last:pb-0">
                    <span className={cn("absolute top-1 left-[-18px] size-2.5 rounded-full border-2 bg-card shadow-[0_0_0_3px_var(--background)]", it.muted ? "border-border" : "border-primary")} />
                    <div className="font-mono font-medium text-[10.5px] uppercase tracking-[0.06em] text-muted-foreground">{it.when}</div>
                    <div className="mt-1 font-medium text-[13px] leading-[1.4]">{it.what}</div>
                    {it.who ? <div className="mt-0.5 text-[12px] leading-[1.4] text-muted-foreground">{it.who}</div> : null}
                </div>
            ))}
        </div>
    );
}
