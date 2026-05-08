import { useState } from "react";

import { cn } from "#/lib/utils";

import type { IUserProfile } from "#/types/user";

interface IStatCardProps {
    kicker: string;
    value: string | number | null;
    sub: string;
    accent?: boolean;
    live?: boolean;
    compactOnMobile?: boolean;
    unformatted?: boolean;
    onClick?: () => void;
}

function formatValue(value: string | number | null) {
    if (value === null || value === undefined) return "-";
    if (typeof value === "number") return value.toLocaleString();
    return value;
}

function formatCompact(value: string | number | null) {
    if (value === null || value === undefined) return "-";
    if (typeof value !== "number") return value;
    const abs = Math.abs(value);
    const sign = value < 0 ? "-" : "";
    if (abs >= 1_000_000) return `${sign}${(abs / 1_000_000).toFixed(1)}mil`;
    if (abs >= 1_000) return `${sign}${(abs / 1_000).toFixed(1)}k`;
    return `${sign}${abs}`;
}

function StatCard({ kicker, value, sub, accent, live, compactOnMobile, unformatted, onClick }: IStatCardProps) {
    const interactive = typeof onClick === "function";
    const className = cn(
        "group relative flex flex-col overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl text-left",
        "shadow-[0_1px_2px_rgb(0_0_0/0.04)] transition-all duration-200",
        "hover:border-foreground/15 hover:shadow-[0_8px_24px_-12px_rgb(0_0_0/0.15)]",
        interactive && "cursor-pointer select-none",
    );
    const style = {
        padding: "clamp(0.75rem, 1.2vw + 0.5rem, 1.25rem)",
        gap: "clamp(0.375rem, 0.4vw + 0.25rem, 0.625rem)",
    };
    const inner = (
        <>
            {accent && <div aria-hidden className="pointer-events-none absolute inset-x-0 -top-px h-px bg-linear-to-r from-transparent via-primary/60 to-transparent" />}
            <div className="flex items-center gap-1.5 sm:gap-2">
                {live && (
                    <span className="relative inline-flex h-1.5 w-1.5 shrink-0">
                        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-70" />
                        <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-emerald-500" />
                    </span>
                )}
                <span
                    className="truncate font-mono font-medium uppercase tracking-wider text-muted-foreground sm:tracking-widest"
                    style={{
                        fontSize: "clamp(0.625rem, 0.25vw + 0.55rem, 0.6875rem)",
                        lineHeight: 1.2,
                    }}
                >
                    {kicker}
                </span>
            </div>
            <div
                className={cn("font-sans font-bold tracking-tight tabular-nums whitespace-nowrap", accent ? "text-primary" : "text-foreground")}
                style={{
                    fontSize: "clamp(1.125rem, 1.6vw + 0.75rem, 1.875rem)",
                    lineHeight: 1.05,
                }}
            >
                {unformatted ? (
                    (value ?? "-")
                ) : compactOnMobile ? (
                    <>
                        <span className="sm:hidden">{formatCompact(value)}</span>
                        <span className="hidden sm:inline">{formatValue(value)}</span>
                    </>
                ) : (
                    formatValue(value)
                )}
            </div>
            <p
                className="line-clamp-2 text-muted-foreground"
                style={{
                    fontSize: "clamp(0.6875rem, 0.25vw + 0.6rem, 0.75rem)",
                    lineHeight: 1.45,
                }}
            >
                {sub}
            </p>
        </>
    );

    if (interactive) {
        return (
            <button className={className} onClick={onClick} style={style} type="button">
                {inner}
            </button>
        );
    }
    return (
        <div className={className} style={style}>
            {inner}
        </div>
    );
}

interface IStatStripProps {
    profile: IUserProfile;
    rosterCount?: number;
}

export function StatStrip({ profile, rosterCount }: IStatStripProps) {
    const [lmdRaw, setLmdRaw] = useState(false);
    return (
        <div className="grid grid-cols-2 md:grid-cols-4" style={{ gap: "clamp(0.5rem, 0.6vw + 0.375rem, 0.875rem)" }}>
            <StatCard kicker="Operators" value={rosterCount ?? profile.operator_count} sub="Unique units in roster" live />
            <StatCard kicker="Skins" value={profile.skin_count} sub="Outfits collected" />
            <StatCard kicker="Items" value={profile.item_count} sub="Inventory entries" />
            {/* troll you can click on it */}
            <StatCard kicker="LMD" value={profile.lmd} sub="LMD Available" compactOnMobile={!lmdRaw} unformatted={lmdRaw} onClick={() => setLmdRaw((v) => !v)} />
        </div>
    );
}
