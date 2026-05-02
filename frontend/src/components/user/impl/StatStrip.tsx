import { cn } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";

interface IStatCardProps {
    kicker: string;
    value: string | number | null;
    sub: string;
    accent?: boolean;
    live?: boolean;
}

function formatValue(value: string | number | null) {
    if (value === null || value === undefined) return "—";
    if (typeof value === "number") return value.toLocaleString();
    return value;
}

function StatCard({ kicker, value, sub, accent, live }: IStatCardProps) {
    return (
        <div className={cn("group relative flex flex-col gap-2 overflow-hidden rounded-2xl border border-border bg-card p-5", "shadow-[0_1px_2px_rgb(0_0_0/0.04)] transition-all duration-200", "hover:border-foreground/15 hover:shadow-[0_8px_24px_-12px_rgb(0_0_0/0.15)]")}>
            {accent && <div aria-hidden className="pointer-events-none absolute inset-x-0 -top-px h-px bg-linear-to-r from-transparent via-primary/60 to-transparent" />}
            <div className="flex items-center gap-2">
                {live && (
                    <span className="relative inline-flex h-1.5 w-1.5 shrink-0">
                        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-70" />
                        <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-emerald-500" />
                    </span>
                )}
                <span className="font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground">{kicker}</span>
            </div>
            <div className={cn("font-sans text-3xl font-bold leading-none tracking-tight tabular-nums", accent ? "text-primary" : "text-foreground")}>{formatValue(value)}</div>
            <p className="text-xs leading-relaxed text-muted-foreground">{sub}</p>
        </div>
    );
}

interface IStatStripProps {
    profile: IUserProfile;
    rosterCount?: number;
}

export function StatStrip({ profile, rosterCount }: IStatStripProps) {
    return (
        <div className="grid grid-cols-2 gap-3.5 md:grid-cols-4">
            <StatCard kicker="Operators" value={rosterCount ?? profile.operator_count} sub="Unique units in roster" live />
            <StatCard kicker="Skins" value={profile.skin_count} sub="Outfits collected" />
            <StatCard kicker="Items" value={profile.item_count} sub="Inventory entries" />
            <StatCard kicker="LMD" value={profile.lmd} sub="Available currency" />
        </div>
    );
}
