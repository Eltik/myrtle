import { Kicker } from "#/components/ui/kicker";
import type { IGameDataStats, IRostersStats } from "#/lib/api/stats";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./CatalogGrid.messages";

interface ICatalogTile {
    id: string;
    label: string;
    value: string;
    meta?: string;
    className?: string;
}

function CatalogCell({ label, value, meta, className }: ICatalogTile) {
    return (
        <div className={cn("flex flex-col gap-2 bg-card px-4 py-3.5", className)}>
            <dt>
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
