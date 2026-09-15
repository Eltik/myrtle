import { useQuery } from "@tanstack/react-query";
import { RefreshCwIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions, formatResponseTimeMs, healthQueryOptions } from "#/lib/api/admin";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { HCode, PageHead } from "../AdminShell";
import { StatusDot } from "../Primitives";
import type { messages } from "./Health.messages";

export function Health(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    const rt: TypedRichT<typeof messages> = useRichT("admin");
    const f = useFormatters();
    const { isAuthenticated } = useAuth();
    const healthQuery = useQuery(healthQueryOptions());
    const statsQuery = useQuery(adminStatsQueryOptions(isAuthenticated));

    const h = healthQuery.data;
    const s = statsQuery.data;

    return (
        <>
            <PageHead
                kicker={t("health.kicker")}
                title={t("health.title")}
                sub={rt("health.sub", { health: <HCode>GET /health</HCode>, stats: <HCode>/stats</HCode> })}
                action={
                    <Button
                        variant="outline"
                        size="sm"
                        loading={healthQuery.isFetching || statsQuery.isFetching}
                        disabled={healthQuery.isFetching || statsQuery.isFetching}
                        onClick={() => {
                            void healthQuery.refetch();
                            void statsQuery.refetch();
                        }}
                    >
                        <RefreshCwIcon />
                        {t("health.reprobe")}
                    </Button>
                }
            />

            {healthQuery.isPending ? (
                <Skeleton className="mb-4 h-32" />
            ) : (
                <div className="mb-4 grid grid-cols-1 gap-3 sm:grid-cols-3 sm:gap-3.5">
                    <Card>
                        <CardContent className="p-4.5">
                            <div className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">{t("health.tile.postgres")}</div>
                            <div className="mt-2.5 font-bold text-[26px] tabular-nums leading-none tracking-[-0.02em]">
                                {formatResponseTimeMs(h?.database.responseTimeMs)}
                                <span className="ml-1 font-medium font-mono text-[12px] text-muted-foreground">{t("health.ms")}</span>
                            </div>
                            <div className="mt-2 flex items-center gap-2">
                                <StatusDot state={h?.database.status === "connected" ? "green" : "red"} pulse>
                                    <span className="font-mono">{h?.database.status ?? t("health.status.unknown")}</span>
                                </StatusDot>
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4.5">
                            <div className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">{t("health.tile.cache", { backend: h?.cache.backend ?? "-" })}</div>
                            <div className="mt-2.5 font-bold text-[26px] tabular-nums leading-none tracking-[-0.02em]">
                                {formatResponseTimeMs(h?.cache.responseTimeMs)}
                                <span className="ml-1 font-medium font-mono text-[12px] text-muted-foreground">{t("health.ms")}</span>
                            </div>
                            <div className="mt-2 flex items-center gap-2">
                                <StatusDot state={h?.cache.status === "connected" ? "green" : "red"} pulse>
                                    <span className="font-mono">{h?.cache.status ?? t("health.status.unknown")}</span>
                                </StatusDot>
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4.5">
                            <div className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-widest">{t("health.tile.roundTrip")}</div>
                            <div className="mt-2.5 font-bold text-[26px] tabular-nums leading-none tracking-[-0.02em]">
                                {formatResponseTimeMs(h?.responseTimeMs)}
                                <span className="ml-1 font-medium font-mono text-[12px] text-muted-foreground">{t("health.ms")}</span>
                            </div>
                            <div className="mt-2 flex items-center gap-2">{h?.status === "ok" ? <Badge variant="success">{t("health.badge.healthy")}</Badge> : h ? <Badge variant="warning">{t("health.badge.degraded")}</Badge> : null}</div>
                        </CardContent>
                    </Card>
                </div>
            )}

            <div className="grid grid-cols-1 gap-4.5 lg:grid-cols-2">
                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">{t("health.gameData.title")}</CardTitle>
                        <CardDescription className="text-xs">{rt("health.gameData.desc", { type: <HCode>Arc&lt;GameData&gt;</HCode>, endpoint: <HCode>GET /stats</HCode> })}</CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        {statsQuery.isPending ? (
                            <Skeleton className="h-40 w-full" />
                        ) : s ? (
                            <>
                                <KV k={t("health.kv.operators")} v={f.number(s.gameData.operators)} />
                                <KV k={t("health.kv.skills")} v={f.number(s.gameData.skills)} />
                                <KV k={t("health.kv.modules")} v={f.number(s.gameData.modules)} />
                                <KV k={t("health.kv.skins")} v={f.number(s.gameData.skins)} />
                                <KV k={t("health.kv.stages")} v={f.number(s.gameData.stages)} />
                                <KV k={t("health.kv.zones")} v={f.number(s.gameData.zones)} />
                                <KV k={t("health.kv.enemies")} v={f.number(s.gameData.enemies)} last />
                            </>
                        ) : (
                            <div className="text-[12.5px] text-muted-foreground">{t("health.statsUnavailable")}</div>
                        )}
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">{t("health.snapshot.title")}</CardTitle>
                        <CardDescription className="text-xs">{t("health.snapshot.desc", { at: s ? f.date(s.computedAt, { year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric" }) : "-" })}</CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        {statsQuery.isPending ? (
                            <Skeleton className="h-40 w-full" />
                        ) : s ? (
                            <>
                                <KV k={t("health.kv.tierListsTotal")} v={f.number(s.tierLists.total)} />
                                <KV k={t("health.kv.tierListsActive")} v={f.number(s.tierLists.active)} />
                                <KV k={t("health.kv.tierListVersions")} v={f.number(s.tierLists.totalVersions)} />
                                <KV k={t("health.kv.tierListPlacements")} v={f.number(s.tierLists.totalPlacements)} />
                                <KV k={t("health.kv.rostersSynced")} v={f.number(s.rosters.total)} last />
                            </>
                        ) : (
                            <div className="text-[12.5px] text-muted-foreground">{t("health.statsUnavailable")}</div>
                        )}
                    </CardContent>
                </Card>
            </div>

            <div className="h-4" />

            <Card>
                <CardHeader>
                    <CardTitle className="text-sm">{t("health.probe.title")}</CardTitle>
                    <CardDescription className="text-xs">{t("health.probe.desc")}</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <div className="font-mono text-[12.5px] text-muted-foreground">{h?.timestamp ?? "-"}</div>
                </CardContent>
            </Card>
        </>
    );
}

function KV({ k, v, last }: { k: string; v: string; last?: boolean }): React.ReactElement {
    return (
        <div className={`flex items-center justify-between py-1.5 ${last ? "" : "border-border border-b"}`}>
            <span className="text-[12.5px] text-muted-foreground">{k}</span>
            <span className="font-mono text-[12.5px] tabular-nums">{v}</span>
        </div>
    );
}
