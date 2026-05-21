import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ExternalLinkIcon, RefreshCwIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions, healthQueryOptions } from "#/lib/api/admin";
import { cn, formatRelativeShort } from "#/lib/utils";
import { HCode, PageHead } from "../AdminShell";
import { CardKV } from "../Primitives";

interface IGameDataTileProps {
    label: string;
    value: number | undefined;
}

function GameDataTile({ label, value }: IGameDataTileProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-1 rounded-lg border border-border bg-background/40 p-3">
            <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">{label}</span>
            <span className="font-semibold text-[18px] tabular-nums leading-none">{value != null ? value.toLocaleString() : "-"}</span>
        </div>
    );
}

export function AdminSettings(): React.ReactElement {
    const { user, isAuthenticated } = useAuth();
    const healthQuery = useQuery(healthQueryOptions());
    const statsQuery = useQuery(adminStatsQueryOptions(isAuthenticated));

    const gd = statsQuery.data?.gameData;

    return (
        <>
            <PageHead kicker="Operate" title="Settings" sub="Service-wide configuration. Most values live in env vars and require a redeploy." />

            <div className="grid grid-cols-1 gap-4.5 lg:grid-cols-2">
                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">Runtime probe</CardTitle>
                        <CardDescription className="text-xs">
                            Live data from <HCode>GET /health</HCode>.
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        {healthQuery.isPending ? (
                            <Skeleton className="h-32 w-full" />
                        ) : healthQuery.data ? (
                            <>
                                <CardKV k="Cache backend" v={healthQuery.data.cache.backend} />
                                <CardKV k="Cache status" v={healthQuery.data.cache.status} />
                                <CardKV k="Database status" v={healthQuery.data.database.status} />
                                <CardKV k="Service status" v={healthQuery.data.status} />
                                <CardKV k="Probe timestamp" v={healthQuery.data.timestamp} />
                            </>
                        ) : (
                            <div className="text-[13px] text-muted-foreground">Probe failed.</div>
                        )}
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">Signed-in User</CardTitle>
                        <CardDescription className="text-xs">Your current session.</CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        <CardKV k="Nickname" v={user?.nickname ?? "-"} />
                        <CardKV k="UID" v={user?.uid ?? "-"} />
                        <CardKV k="Server" v={user?.server ?? "-"} />
                        <CardKV
                            k="Role"
                            v={
                                <>
                                    {user?.role ?? "-"}
                                    {user?.role === "super_admin" ? (
                                        <Badge variant="success" className="ml-2">
                                            super
                                        </Badge>
                                    ) : null}
                                </>
                            }
                        />
                        <CardKV k="Total score" v={user?.total_score?.toLocaleString() ?? "-"} />
                        <div className="mt-3 flex items-center gap-2">
                            <Button variant="outline" size="sm" render={<Link to="/settings" />}>
                                Account settings
                            </Button>
                            {user?.uid ? (
                                <Button variant="outline" size="sm" render={<Link to="/user/$id" params={{ id: user.uid }} target="_blank" />}>
                                    <ExternalLinkIcon />
                                    Public profile
                                </Button>
                            ) : null}
                        </div>
                    </CardContent>
                </Card>
            </div>

            <div className="h-4" />

            <Card>
                <CardHeader>
                    <CardTitle className="text-sm">Loaded game data</CardTitle>
                    <CardDescription className="text-xs">
                        Snapshot of what the backend currently has resident from <HCode>GAME_DATA_DIR</HCode>. Counts come from <HCode>GET /admin/stats</HCode> - they reflect the live in-memory dataset, so any drift here means the asset import is out of date and a redeploy or asset refresh is needed.
                    </CardDescription>
                    <CardAction>
                        <Button variant="outline" size="sm" onClick={() => statsQuery.refetch()} disabled={statsQuery.isFetching}>
                            <RefreshCwIcon className={cn(statsQuery.isFetching && "animate-spin")} />
                            Refresh
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
                        <div className="rounded-lg border border-destructive/32 bg-destructive/8 p-3 text-[13px] text-destructive-foreground">Failed to load /admin/stats. Your account may not have tier_list_admin or above.</div>
                    ) : gd ? (
                        <>
                            <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4 lg:grid-cols-7">
                                <GameDataTile label="Operators" value={gd.operators} />
                                <GameDataTile label="Skills" value={gd.skills} />
                                <GameDataTile label="Modules" value={gd.modules} />
                                <GameDataTile label="Skins" value={gd.skins} />
                                <GameDataTile label="Stages" value={gd.stages} />
                                <GameDataTile label="Zones" value={gd.zones} />
                                <GameDataTile label="Enemies" value={gd.enemies} />
                            </div>
                            <div className="mt-3.5 flex flex-wrap items-center justify-between gap-2 border-border border-t pt-3 text-[12px] text-muted-foreground">
                                <span>
                                    Snapshot computed <span className="font-medium text-foreground">{formatRelativeShort(statsQuery.data?.computedAt)}</span>
                                    <span className="ml-1.5 opacity-70">(cached for ~60s)</span>
                                </span>
                                <span>
                                    {statsQuery.data?.tierLists.totalPlacements.toLocaleString() ?? "-"} placements across {statsQuery.data?.tierLists.total ?? "-"} tier lists
                                </span>
                            </div>
                        </>
                    ) : (
                        <div className="text-[13px] text-muted-foreground">No data.</div>
                    )}
                </CardContent>
            </Card>
        </>
    );
}
