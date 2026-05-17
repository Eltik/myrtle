import type { IGameDataStats, IRostersStats, ITierListStats } from "#/lib/api/stats";
import { formatNumber, formatNumberCompact } from "#/lib/utils";

type IconName = "users" | "skill" | "module" | "skin" | "zone" | "enemy" | "tiers" | "sync";

function CatalogIcon({ name }: { name: IconName }) {
    const p = { fill: "none", stroke: "currentColor", strokeWidth: 1.9, strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
    switch (name) {
        case "users":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
                    <circle cx="9" cy="7" r="4" />
                    <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
                    <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                </svg>
            );
        case "skill":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="m12 2 2.39 4.84L20 8l-4 3.9L17 18l-5-2.6L7 18l1-6.1L4 8l5.61-1.16L12 2z" />
                </svg>
            );
        case "module":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <rect x="4" y="4" width="16" height="16" rx="3" />
                    <path d="M9 9h6v6H9z" />
                </svg>
            );
        case "skin":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M5 7c2-3 5-4 7-4s5 1 7 4l-3 2v10H8V9L5 7z" />
                </svg>
            );
        case "zone":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M12 21s-7-4.5-7-11a7 7 0 0 1 14 0c0 6.5-7 11-7 11z" />
                    <circle cx="12" cy="10" r="2.5" />
                </svg>
            );
        case "enemy":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M3 4h18l-2 6a7 7 0 0 1-14 0L3 4z" />
                    <path d="M9 14v5M15 14v5" />
                </svg>
            );
        case "tiers":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <rect x="3" y="3.5" width="18" height="5" rx="1.2" />
                    <rect x="3" y="9.5" width="13" height="5" rx="1.2" />
                    <rect x="3" y="15.5" width="8" height="5" rx="1.2" />
                </svg>
            );
        case "sync":
            return (
                <svg viewBox="0 0 24 24" {...p} aria-hidden="true">
                    <path d="M21 12a9 9 0 0 0-15-6.7L3 8" />
                    <path d="M3 4v4h4" />
                    <path d="M3 12a9 9 0 0 0 15 6.7l3-2.7" />
                    <path d="M21 20v-4h-4" />
                </svg>
            );
    }
}

interface ICatalogTile {
    icon: IconName;
    label: string;
    value: string;
    meta?: string;
}

function CatalogCell({ icon, label, value, meta }: ICatalogTile) {
    return (
        <div className="relative flex flex-col gap-2 rounded-xl border border-border bg-card px-4 py-3.5">
            <span className="inline-flex h-7 w-7 items-center justify-center rounded-lg border border-border bg-muted/70 text-primary [&>svg]:h-4 [&>svg]:w-4">
                <CatalogIcon name={icon} />
            </span>
            <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{label}</span>
            <span className="font-bold font-sans text-[24px] text-foreground tabular-nums leading-none tracking-tight">{value}</span>
            {meta ? <span className="font-medium font-mono text-[11px] text-muted-foreground leading-[1.2]">{meta}</span> : null}
        </div>
    );
}

interface ICatalogGridProps {
    gameData: IGameDataStats;
    tierLists: ITierListStats;
    rosters: IRostersStats;
}

export function CatalogGrid({ gameData, tierLists, rosters }: ICatalogGridProps) {
    const tiles: ICatalogTile[] = [
        { icon: "users", label: "Operators", value: formatNumber(gameData.operators), meta: "playable + alters" },
        { icon: "skill", label: "Skills", value: formatNumber(gameData.skills), meta: "across all archetypes" },
        { icon: "module", label: "Modules", value: formatNumber(gameData.modules), meta: "stage-3 unlocked" },
        { icon: "skin", label: "Skins", value: formatNumber(gameData.skins), meta: "elite & seasonal" },
        { icon: "zone", label: "Stages", value: formatNumber(gameData.stages), meta: `across ${formatNumber(gameData.zones)} zones` },
        { icon: "enemy", label: "Enemies", value: formatNumber(gameData.enemies), meta: "indexed" },
        { icon: "tiers", label: "Tier lists", value: `${formatNumber(tierLists.active)}/${formatNumber(tierLists.total)}`, meta: `${formatNumberCompact(tierLists.totalPlacements)} placements` },
        { icon: "sync", label: "Rosters", value: formatNumberCompact(rosters.total), meta: "Yostar-linked doctors" },
    ];

    return (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
            {tiles.map((t) => (
                <CatalogCell key={t.label} {...t} />
            ))}
        </div>
    );
}
