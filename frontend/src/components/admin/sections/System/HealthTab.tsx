import { useQuery } from "@tanstack/react-query";
import { StatTile, StatusDot, skeletons } from "#/components/admin/Primitives";
import { Card, CardDescription, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { adminStatsQueryOptions, formatResponseTimeMs, healthQueryOptions } from "#/lib/api/admin";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { type HealthSummary, healthDot, summarizeHealth } from "./health";
import type { messages } from "./System.messages";

type SystemT = TypedT<typeof messages>;

interface IHealthCopy {
    status: (summary: HealthSummary) => string;
    /** Empty for the states that need no explanation. */
    detail: (summary: HealthSummary) => string;
}

/** The status line and its explanation, shared with the Check now toast. */
export function useHealthCopy(): IHealthCopy {
    const t: SystemT = useT("admin");
    return {
        status: (summary) => {
            switch (summary) {
                case "checking":
                    return t("system.health.status.checking");
                case "healthy":
                    return t("system.health.status.healthy");
                case "cacheDown":
                    return t("system.health.status.cacheDown");
                case "databaseDown":
                    return t("system.health.status.databaseDown");
                case "bothDown":
                    return t("system.health.status.bothDown");
                case "degraded":
                    return t("system.health.status.degraded");
                case "unreachable":
                    return t("system.health.status.unreachable");
            }
        },
        detail: (summary) => {
            switch (summary) {
                case "checking":
                case "healthy":
                    return "";
                case "cacheDown":
                    return t("system.health.detail.cacheDown");
                case "databaseDown":
                    return t("system.health.detail.databaseDown");
                case "bothDown":
                    return t("system.health.detail.bothDown");
                case "degraded":
                    return t("system.health.detail.degraded");
                case "unreachable":
                    return t("system.health.detail.unreachable");
            }
        },
    };
}

/** Label over value, the design's figure cell in the two data cards. */
function Figure({ label, value }: { label: string; value: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex min-w-0 flex-col gap-0.5">
            <span className="text-[12px] text-muted-foreground">{label}</span>
            <span className="font-semibold text-[16px] tabular-nums">{value}</span>
        </div>
    );
}

export function HealthTab({ enabled }: { enabled: boolean }): React.ReactElement {
    const t: SystemT = useT("admin");
    const f = useFormatters();
    const copy = useHealthCopy();
    const healthQuery = useQuery({ ...healthQueryOptions(), enabled });
    const statsQuery = useQuery({ ...adminStatsQueryOptions(enabled), enabled });

    const h = healthQuery.data;
    const s = statsQuery.data;
    // A refetch that fails after a good probe still means the backend is unreachable now.
    const summary = summarizeHealth(healthQuery.isError ? undefined : h, healthQuery.isError);
    const detail = copy.detail(summary);
    const cacheBackend = h ? (h.cache.backend === "redis" ? t("system.health.cacheBackend.redis") : t("system.health.cacheBackend.memory")) : "-";
    const serviceColor = (up: boolean | undefined) => (up === false ? "var(--destructive)" : "var(--success)");

    const siteData = s
        ? [
              { label: t("system.health.site.rosters"), value: s.rosters.total },
              { label: t("system.health.site.tierListsActive"), value: s.tierLists.active },
              { label: t("system.health.site.tierListsTotal"), value: s.tierLists.total },
              { label: t("system.health.site.versions"), value: s.tierLists.totalVersions },
              { label: t("system.health.site.placements"), value: s.tierLists.totalPlacements },
          ]
        : [];
    const gameData = s
        ? [
              { label: t("system.health.game.operators"), value: s.gameData.operators },
              { label: t("system.health.game.skills"), value: s.gameData.skills },
              { label: t("system.health.game.modules"), value: s.gameData.modules },
              { label: t("system.health.game.skins"), value: s.gameData.skins },
              { label: t("system.health.game.stages"), value: s.gameData.stages },
              { label: t("system.health.game.zones"), value: s.gameData.zones },
              { label: t("system.health.game.enemies"), value: s.gameData.enemies },
          ]
        : [];

    const statsBody = (rows: { label: string; value: number }[], grid: string, count: number): React.ReactNode => {
        if (statsQuery.isPending) {
            return <div className={cn("grid gap-3.5", grid)}>{skeletons(count, "h-10.5")}</div>;
        }
        if (!s) return <p className="text-[12.5px] text-muted-foreground">{t("system.health.statsError")}</p>;
        return (
            <div className={cn("grid gap-3.5", grid)}>
                {rows.map((r) => (
                    <Figure key={r.label} label={r.label} value={f.number(r.value)} />
                ))}
            </div>
        );
    };

    return (
        <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-1">
                <div className="flex flex-wrap items-center gap-3">
                    <StatusDot state={healthDot(summary)} pulse={summary !== "unreachable"}>
                        {copy.status(summary)}
                    </StatusDot>
                    {healthQuery.dataUpdatedAt > 0 ? <span className="whitespace-nowrap text-[12.5px] text-muted-foreground">{t("system.health.checked", { when: f.relativeShort(healthQuery.dataUpdatedAt) })}</span> : null}
                </div>
                {detail ? <p className="text-[12.5px] text-muted-foreground">{detail}</p> : null}
            </div>

            <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
                {healthQuery.isPending ? (
                    skeletons(3, "h-24 rounded-2xl")
                ) : (
                    <>
                        <StatTile label={t("system.health.tile.database")} value={formatResponseTimeMs(h?.database.responseTimeMs)} unit={t("system.health.unit.ms")} color={serviceColor(h ? h.database.status === "connected" : undefined)} />
                        <StatTile label={t("system.health.tile.cache", { backend: cacheBackend })} value={formatResponseTimeMs(h?.cache.responseTimeMs)} unit={t("system.health.unit.ms")} color={serviceColor(h ? h.cache.status === "connected" : undefined)} />
                        <StatTile label={t("system.health.tile.roundTrip")} value={formatResponseTimeMs(h?.responseTimeMs)} unit={t("system.health.unit.ms")} color="var(--success)" />
                    </>
                )}
            </div>

            <div className="grid grid-cols-1 gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.6fr)]">
                <Card>
                    <CardHeader>
                        <CardTitle>{t("system.health.site.title")}</CardTitle>
                    </CardHeader>
                    <CardPanel>{statsBody(siteData, "grid-cols-2", 5)}</CardPanel>
                </Card>
                <Card>
                    <CardHeader>
                        <CardTitle>{t("system.health.game.title")}</CardTitle>
                        <CardDescription>{t("system.health.game.desc")}</CardDescription>
                    </CardHeader>
                    <CardPanel>{statsBody(gameData, "grid-cols-2 md:grid-cols-4", 7)}</CardPanel>
                </Card>
            </div>
        </div>
    );
}
