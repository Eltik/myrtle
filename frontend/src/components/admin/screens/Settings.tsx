import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ExternalLinkIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { healthQueryOptions } from "#/lib/api/admin";
import { HCode, PageHead } from "../AdminShell";
import { CardKV } from "../Primitives";

export function AdminSettings(): React.ReactElement {
    const { user } = useAuth();
    const healthQuery = useQuery(healthQueryOptions());

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
                        <CardTitle className="text-sm">Signed-in operator</CardTitle>
                        <CardDescription className="text-xs">Your current JWT session.</CardDescription>
                    </CardHeader>
                    <CardContent className="pt-0">
                        <CardKV k="Nickname" v={user?.nickname ?? "—"} />
                        <CardKV k="UID" v={user?.uid ?? "—"} />
                        <CardKV k="Server" v={user?.server ?? "—"} />
                        <CardKV
                            k="Role"
                            v={
                                <>
                                    {user?.role ?? "—"}
                                    {user?.role === "super_admin" ? (
                                        <Badge variant="success" className="ml-2">
                                            super
                                        </Badge>
                                    ) : null}
                                </>
                            }
                        />
                        <CardKV k="Total score" v={user?.total_score?.toLocaleString() ?? "—"} />
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
                    <CardTitle className="text-sm">Environment (reference)</CardTitle>
                    <CardDescription className="text-xs">
                        These values live in <HCode>.env</HCode> on the backend and require a redeploy to change.
                    </CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <CardKV
                        k="DATABASE_URL"
                        v={
                            <span className="text-muted-foreground">
                                in <span className="font-mono">backend/.env</span>
                            </span>
                        }
                    />
                    <CardKV
                        k="REDIS_URL"
                        v={
                            <span className="text-muted-foreground">
                                in <span className="font-mono">backend/.env</span>
                            </span>
                        }
                    />
                    <CardKV k="GAME_DATA_DIR" v={<span className="text-muted-foreground">../assets/output/gamedata/excel</span>} />
                    <CardKV k="ASSETS_DIR" v={<span className="text-muted-foreground">../assets/output</span>} />
                    <CardKV
                        k="JWT_SECRET"
                        v={
                            <>
                                ••••••••••••••••{" "}
                                <Badge variant="outline" className="ml-2">
                                    secret
                                </Badge>
                            </>
                        }
                    />
                    <CardKV
                        k="SERVICE_KEY"
                        v={
                            <>
                                ••••••••••••••••{" "}
                                <Badge variant="outline" className="ml-2">
                                    secret
                                </Badge>
                            </>
                        }
                    />
                </CardContent>
            </Card>
        </>
    );
}
