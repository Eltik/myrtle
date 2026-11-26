import { Kicker } from "#/components/ui/kicker";
import type { IGameDataStats, IRostersStats } from "#/lib/api/stats";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./CatalogGrid.messages";

type IconName = "users" | "skill" | "module" | "skin" | "zone" | "enemy" | "sync";

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
    id: IconName;
    label: string;
    value: string;
    meta?: string;
    className?: string;
}

function CatalogCell({ id, label, value, meta, className }: ICatalogTile) {
    return (
        <div className={cn("flex flex-col gap-2 bg-card px-4 py-3.5", className)}>
            <dt className="flex flex-col gap-2">
                <span className="inline-flex h-7 w-7 items-center justify-center rounded-lg border border-border bg-muted/70 text-primary [&>svg]:h-4 [&>svg]:w-4" aria-hidden="true">
                    <CatalogIcon name={id} />
                </span>
                <Kicker>{label}</Kicker>
            </dt>
            <dd className="m-0 font-bold font-sans text-[24px] text-foreground tabular-nums leading-none tracking-tight">{value}</dd>
            {meta ? <dd className="m-0 font-medium font-mono text-[11px] text-muted-foreground leading-[1.2]">{meta}</dd> : null}
        </div>
    );
}

interface ICatalogGridProps {
    gameData: IGameDataStats;
    rosters: IRostersStats;
}

export function CatalogGrid({ gameData, rosters }: ICatalogGridProps) {
    const t: TypedT<typeof messages> = useT("stats");
    const f = useFormatters();

    const tiles: ICatalogTile[] = [
        { id: "users", label: t("catalog.operators"), value: f.number(gameData.operators), meta: t("catalog.operators.meta") },
        { id: "skill", label: t("catalog.skills"), value: f.number(gameData.skills), meta: t("catalog.skills.meta") },
        { id: "module", label: t("catalog.modules"), value: f.number(gameData.modules), meta: t("catalog.modules.meta") },
        { id: "skin", label: t("catalog.skins"), value: f.number(gameData.skins), meta: t("catalog.skins.meta") },
        { id: "zone", label: t("catalog.stages"), value: f.number(gameData.stages), meta: t("catalog.stages.meta", { count: f.number(gameData.zones) }) },
        { id: "enemy", label: t("catalog.enemies"), value: f.number(gameData.enemies), meta: t("catalog.enemies.meta") },
        // Tier lists have their own section below; Rosters takes the freed slot so the seventh tile closes the row.
        { id: "sync", label: t("catalog.rosters"), value: f.compact(rosters.total), meta: t("catalog.rosters.meta"), className: "col-span-2" },
    ];

    return (
        <dl className="m-0 grid grid-cols-2 gap-px overflow-hidden rounded-2xl border border-border bg-border lg:grid-cols-4">
            {tiles.map((tile) => (
                <CatalogCell key={tile.id} {...tile} />
            ))}
        </dl>
    );
}
