import { useQuery } from "@tanstack/react-query";
import { RefreshCwIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions, healthQueryOptions } from "#/lib/api/admin";
import { HCode, PageHead } from "../AdminShell";
import { StatusDot } from "../Primitives";

export function Health(): React.ReactElement {
    const { isAuthenticated } = useAuth();
    const healthQuery = useQuery(healthQueryOptions());
    const statsQuery = useQuery(adminStatsQueryOptions(isAuthenticated));

    const h = healthQuery.data;
    const s = statsQuery.data;

    return (
        <>
            <PageHead
                kicker="Operate"
                title="Health & cache"
                sub={
                    <>
                        Live probe of the Rust backend — <HCode>GET /health</HCode> and the public <HCode>/stats</HCode> snapshot.
                    </>
                }
                action={
                    <Button variant="outline" size="sm" loading={healthQuery.isFetching} onClick={() => healthQuery.refetch()}>
                        <RefreshCwIcon />
                        Re-probe
                    </Button>
                }
            />

            {healthQuery.isPending ? (
                <Skeleton className="mb-4 h-32" />
            ) : (
                <div className="mb-4 grid grid-cols-3 gap-3.5">
                    <Card>
                        <CardContent className="p-4.5">
                            <div className="font-mono font-medium text-[10.5px] uppercase tracking-[0.1em] text-muted-foreground">Postgres</div>
                            <div className="mt-2.5 font-bold text-[26px] leading-none tracking-[-0.02em] tabular-nums">
                                {h?.database.responseTimeMs ?? "—"}
                                <span className="ml-1 font-mono font-medium text-[12px] text-muted-foreground">ms</span>
                            </div>
                            <div className="mt-2 flex items-center gap-2">
                                <StatusDot state={h?.database.status === "connected" ? "green" : "red"} pulse>
                                    <span className="font-mono">{h?.database.status ?? "unknown"}</span>
                                </StatusDot>
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4.5">
                            <div className="font-mono font-medium text-[10.5px] uppercase tracking-[0.1em] text-muted-foreground">Cache · {h?.cache.backend ?? "—"}</div>
                            <div className="mt-2.5 font-bold text-[26px] leading-none tracking-[-0.02em] tabular-nums">
                                {h?.cache.responseTimeMs ?? "—"}
                                <span className="ml-1 font-mono font-medium text-[12px] text-muted-foreground">ms</span>
                            </div>
                            <div className="mt-2 flex items-center gap-2">
                                <StatusDot state={h?.cache.status === "connected" ? "green" : "red"} pulse>
                                    <span className="font-mono">{h?.cache.status ?? "unknown"}</span>
                                </StatusDot>
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="p-4.5">
                            <div className="font-mono font-medium text-[10.5px] uppercase tracking-[0.1em] text-muted-foreground">Round-trip</div>
                            <div className="mt-2.5 font-bold text-[26px] leading-none tracking-[-0.02em] tabular-nums">
                                {h?.responseTimeMs ?? "—"}
                                <span className="ml-1 font-mono font-medium text-[12px] text-muted-foreground">ms</span>
                            </div>
                            <div className="mt-2 flex items-center gap-2">{h?.status === "ok" ? <Badge variant="success">healthy</Badge> : h ? <Badge variant="warning">degraded</Badge> : null}</div>
                        </CardContent>
                    </Card>
                </div>
            )}

            <div className="grid grid-cols-1 gap-4.5 lg:grid-cols-2">
                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">Game data</CardTitle>
                        <CardDescription className="text-xs">
                            Backed by the in-memory <HCode>Arc&lt;GameData&gt;</HCode>. Counts from <HCode>GET /stats</HCode>.
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        {statsQuery.isPending ? (
                            <Skeleton className="h-40 w-full" />
                        ) : s ? (
                            <>
                                <KV k="Operators" v={s.gameData.operators.toLocaleString()} />
                                <KV k="Skills" v={s.gameData.skills.toLocaleString()} />
                                <KV k="Modules" v={s.gameData.modules.toLocaleString()} />
                                <KV k="Skins" v={s.gameData.skins.toLocaleString()} />
                                <KV k="Stages" v={s.gameData.stages.toLocaleString()} />
                                <KV k="Zones" v={s.gameData.zones.toLocaleString()} />
                                <KV k="Enemies" v={s.gameData.enemies.toLocaleString()} last />
                            </>
                        ) : (
                            <div className="text-[12.5px] text-muted-foreground">/stats endpoint unavailable.</div>
                        )}
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">Stats snapshot</CardTitle>
                        <CardDescription className="text-xs">Computed at {s ? new Date(s.computedAt).toLocaleString() : "—"}.</CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        {statsQuery.isPending ? (
                            <Skeleton className="h-40 w-full" />
                        ) : s ? (
                            <>
                                <KV k="Tier lists · total" v={s.tierLists.total.toLocaleString()} />
                                <KV k="Tier lists · active" v={s.tierLists.active.toLocaleString()} />
                                <KV k="Tier list versions" v={s.tierLists.totalVersions.toLocaleString()} />
                                <KV k="Tier list placements" v={s.tierLists.totalPlacements.toLocaleString()} />
                                <KV k="Rosters synced" v={s.rosters.total.toLocaleString()} last />
                            </>
                        ) : (
                            <div className="text-[12.5px] text-muted-foreground">/stats endpoint unavailable.</div>
                        )}
                    </CardContent>
                </Card>
            </div>

            <div className="h-4" />

            <Card>
                <CardHeader>
                    <CardTitle className="text-sm">Probe timestamp</CardTitle>
                    <CardDescription className="text-xs">Auto-refreshes every 30s.</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <div className="font-mono text-[12.5px] text-muted-foreground">{h?.timestamp ?? "—"}</div>
                </CardContent>
            </Card>
        </>
    );
}

function KV({ k, v, last }: { k: string; v: string; last?: boolean }): React.ReactElement {
    return (
        <div className={`flex items-center justify-between py-1.5 ${last ? "" : "border-b border-border"}`}>
            <span className="text-[12.5px] text-muted-foreground">{k}</span>
            <span className="font-mono text-[12.5px] tabular-nums">{v}</span>
        </div>
    );
}
