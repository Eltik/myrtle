import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ExternalLinkIcon, RefreshCwIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions, healthQueryOptions } from "#/lib/api/admin";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { HCode, PageHead } from "../AdminShell";
import { CardKV } from "../Primitives";
import type { messages } from "./Settings.messages";

interface IGameDataTileProps {
    label: string;
    value: number | undefined;
}

function GameDataTile({ label, value }: IGameDataTileProps): React.ReactElement {
    const f = useFormatters();
    return (
        <div className="flex flex-col gap-1 rounded-lg border border-border bg-background/40 p-3">
            <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">{label}</span>
            <span className="font-semibold text-[18px] tabular-nums leading-none">{value != null ? f.number(value) : "-"}</span>
        </div>
    );
}

export function AdminSettings(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    const rt: TypedRichT<typeof messages> = useRichT("admin");
    const f = useFormatters();
    const { user, isAuthenticated } = useAuth();
    const healthQuery = useQuery(healthQueryOptions());
    const statsQuery = useQuery(adminStatsQueryOptions(isAuthenticated));

    const gd = statsQuery.data?.gameData;

    return (
        <>
            <PageHead kicker={t("settings.kicker")} title={t("settings.title")} sub={t("settings.sub")} />

            <div className="grid grid-cols-1 gap-4.5 lg:grid-cols-2">
                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">{t("settings.probe.title")}</CardTitle>
                        <CardDescription className="text-xs">{rt("settings.probe.desc", { endpoint: <HCode>GET /health</HCode> })}</CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        {healthQuery.isPending ? (
                            <Skeleton className="h-32 w-full" />
                        ) : healthQuery.data ? (
                            <>
                                <CardKV k={t("settings.kv.cacheBackend")} v={healthQuery.data.cache.backend} />
                                <CardKV k={t("settings.kv.cacheStatus")} v={healthQuery.data.cache.status} />
                                <CardKV k={t("settings.kv.databaseStatus")} v={healthQuery.data.database.status} />
                                <CardKV k={t("settings.kv.serviceStatus")} v={healthQuery.data.status} />
                                <CardKV k={t("settings.kv.probeTimestamp")} v={healthQuery.data.timestamp} />
                            </>
                        ) : (
                            <div className="text-[13px] text-muted-foreground">{t("settings.probe.failed")}</div>
                        )}
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">{t("settings.user.title")}</CardTitle>
                        <CardDescription className="text-xs">{t("settings.user.desc")}</CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        <CardKV k={t("settings.kv.nickname")} v={user?.nickname ?? "-"} />
                        <CardKV k={t("settings.kv.uid")} v={user?.uid ?? "-"} />
                        <CardKV k={t("settings.kv.server")} v={user?.server ?? "-"} />
                        <CardKV
                            k={t("settings.kv.role")}
                            v={
                                <>
                                    {user?.role ?? "-"}
                                    {user?.role === "super_admin" ? (
                                        <Badge variant="success" className="ml-2">
                                            {t("settings.badge.super")}
                                        </Badge>
                                    ) : null}
                                </>
                            }
                        />
                        <CardKV k={t("settings.kv.totalScore")} v={user?.total_score != null ? f.number(user.total_score) : "-"} />
                        <div className="mt-3 flex items-center gap-2">
                            <Button variant="outline" size="sm" render={<Link to="/settings" />}>
                                {t("settings.accountSettings")}
                            </Button>
                            {user?.uid ? (
                                <Button variant="outline" size="sm" render={<Link to="/user/$id" params={{ id: user.uid }} target="_blank" />}>
                                    <ExternalLinkIcon />
                                    {t("settings.publicProfile")}
                                </Button>
                            ) : null}
                        </div>
                    </CardContent>
                </Card>
            </div>

            <div className="h-4" />

            <Card>
                <CardHeader>
                    <CardTitle className="text-sm">{t("settings.gameData.title")}</CardTitle>
                    <CardDescription className="text-xs">{rt("settings.gameData.desc", { dir: <HCode>GAME_DATA_DIR</HCode>, endpoint: <HCode>GET /admin/stats</HCode> })}</CardDescription>
                    <CardAction>
                        <Button variant="outline" size="sm" onClick={() => statsQuery.refetch()} disabled={statsQuery.isFetching}>
                            <RefreshCwIcon className={cn(statsQuery.isFetching && "animate-spin")} />
                            {t("settings.refresh")}
                        </Button>
                    </CardAction>
                </CardHeader>
                <CardContent className="pt-0">
                    {statsQuery.isPending ? (
                        <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4 lg:grid-cols-7">
                            {Array.from({ length: 7 }).map((_, i) => (
                                // biome-ignore lint/suspicious/noArrayIndexKey: fixed skeleton grid
                                <Skeleton key={i} className="h-16 rounded-lg" />
                            ))}
                        </div>
                    ) : statsQuery.isError ? (
                        <div className="rounded-lg border border-destructive/32 bg-destructive/8 p-3 text-[13px] text-destructive-foreground">{t("settings.gameData.error")}</div>
                    ) : gd ? (
                        <>
                            <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4 lg:grid-cols-7">
                                <GameDataTile label={t("settings.tile.operators")} value={gd.operators} />
                                <GameDataTile label={t("settings.tile.skills")} value={gd.skills} />
                                <GameDataTile label={t("settings.tile.modules")} value={gd.modules} />
                                <GameDataTile label={t("settings.tile.skins")} value={gd.skins} />
                                <GameDataTile label={t("settings.tile.stages")} value={gd.stages} />
                                <GameDataTile label={t("settings.tile.zones")} value={gd.zones} />
                                <GameDataTile label={t("settings.tile.enemies")} value={gd.enemies} />
                            </div>
                            <div className="mt-3.5 flex flex-wrap items-center justify-between gap-2 border-border border-t pt-3 text-[12px] text-muted-foreground">
                                <span>
                                    {rt("settings.snapshotComputed", { when: <span className="font-medium text-foreground">{f.relativeShort(statsQuery.data?.computedAt)}</span> })}
                                    <span className="ml-1.5 opacity-70">{t("settings.cachedFor")}</span>
                                </span>
                                <span>
                                    {t("settings.placementsAcross", {
                                        placements: statsQuery.data?.tierLists.totalPlacements != null ? f.number(statsQuery.data.tierLists.totalPlacements) : "-",
                                        lists: statsQuery.data?.tierLists.total ?? "-",
                                    })}
                                </span>
                            </div>
                        </>
                    ) : (
                        <div className="text-[13px] text-muted-foreground">{t("settings.gameData.empty")}</div>
                    )}
                </CardContent>
            </Card>
        </>
    );
}
