import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowRightIcon, ExternalLinkIcon, RefreshCwIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions, healthQueryOptions } from "#/lib/api/admin";
import { userQueryOptions } from "#/lib/api/user";
import { getSecretaryAvatarURL } from "#/lib/utils";
import { HCode, PageHead } from "../AdminShell";
import { StatTile, StatusDot, Timeline } from "../Primitives";

function DashUserCell({ uid, fallbackName }: { uid: string; fallbackName: string }): React.ReactElement {
    const profile = useQuery({ ...userQueryOptions(uid), retry: 0 });
    const u = profile.data;
    return (
        <span className="flex items-center gap-2">
            <span className="relative inline-block size-5.5 overflow-hidden rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]">
                {u ? <img src={getSecretaryAvatarURL({ secretary: u.secretary, secretary_skin_id: u.secretary_skin_id })} alt="" loading="lazy" className="absolute inset-0 size-full object-cover" onError={(e) => (e.currentTarget.style.display = "none")} /> : null}
            </span>
            <Link to="/user/$id" params={{ id: uid }} className="font-medium hover:underline">
                {u?.nickname ?? fallbackName}
            </Link>
            <span className="font-mono text-[11.5px] text-muted-foreground">UID {uid}</span>
        </span>
    );
}

function formatRelative(iso: string): string {
    const t = Date.parse(iso);
    if (!Number.isFinite(t)) return iso;
    const diff = (Date.now() - t) / 1000;
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)} min ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} h ago`;
    if (diff < 86400 * 30) return `${Math.floor(diff / 86400)} d ago`;
    return new Date(t).toLocaleDateString();
}

function compactNumber(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, "")}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, "")}K`;
    return n.toLocaleString();
}

export function Dashboard(): React.ReactElement {
    const { user, isAuthenticated } = useAuth();
    const statsQuery = useQuery(adminStatsQueryOptions(isAuthenticated));
    const healthQuery = useQuery(healthQueryOptions());

    const stats = statsQuery.data;
    const health = healthQuery.data;

    const totalTierLists = stats?.tierLists.total ?? 0;
    const activeTierLists = stats?.tierLists.active ?? 0;
    const totalPlacements = stats?.tierLists.totalPlacements ?? 0;
    const totalRosters = stats?.rosters.total ?? 0;
    const totalOperators = stats?.gameData.operators ?? 0;

    return (
        <>
            <PageHead
                kicker="Overview"
                title="Dashboard"
                sub={
                    <>
                        Service-level signals from the Rust backend - surfaced verbatim from <HCode>GET /admin/stats</HCode> and <HCode>/health</HCode>.
                    </>
                }
                action={
                    <>
                        <Button variant="outline" size="sm" onClick={() => statsQuery.refetch()} disabled={statsQuery.isFetching} loading={statsQuery.isFetching}>
                            <RefreshCwIcon />
                            Refresh
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            render={
                                // biome-ignore lint/a11y/useAnchorContent: Button's `render` merges children into the anchor at runtime
                                <a href="https://myrtle.moe/stats" target="_blank" rel="noreferrer" />
                            }
                        >
                            <ExternalLinkIcon />
                            View public stats
                        </Button>
                    </>
                }
            />

            {statsQuery.isError ? (
                <div className="mb-4 rounded-2xl border border-destructive/32 bg-destructive/8 p-4 text-[13px] text-destructive-foreground">
                    <strong>Failed to load /admin/stats.</strong> Your account may not have <span className="font-mono">tier_list_admin</span> or above.
                </div>
            ) : null}

            <section className="mb-4 grid grid-cols-4 gap-3.5">
                {statsQuery.isPending ? (
                    <>
                        <Skeleton className="h-31 rounded-2xl" />
                        <Skeleton className="h-31 rounded-2xl" />
                        <Skeleton className="h-31 rounded-2xl" />
                        <Skeleton className="h-31 rounded-2xl" />
                    </>
                ) : (
                    <>
                        <StatTile label="Total users" value={compactNumber((stats?.usersByRole.user ?? 0) + (stats?.usersByRole.tierListEditor ?? 0) + (stats?.usersByRole.tierListAdmin ?? 0) + (stats?.usersByRole.superAdmin ?? 0))} color="var(--chart-1)" />
                        <StatTile label="Rosters synced" value={compactNumber(totalRosters)} color="var(--chart-2)" />
                        <StatTile label="Tier lists · active" value={`${activeTierLists}`} unit={`of ${totalTierLists}`} color="var(--chart-2)" />
                        <StatTile label="Tier-list placements" value={compactNumber(totalPlacements)} color="var(--chart-4)" />
                    </>
                )}
            </section>

            <div className="grid grid-cols-1 gap-4.5 lg:grid-cols-[1fr_320px]">
                <div className="grid gap-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">Roles breakdown</CardTitle>
                            <CardDescription className="text-xs">
                                From the <span className="font-mono">users.role</span> column - global access rungs.
                            </CardDescription>
                            <CardAction>
                                <Button variant="ghost" size="sm" render={<Link to="/admin/users" />}>
                                    Manage users <ArrowRightIcon />
                                </Button>
                            </CardAction>
                        </CardHeader>
                        <CardContent>
                            {statsQuery.isPending ? (
                                <Skeleton className="h-24 w-full" />
                            ) : (
                                <RoleBreakdownBars
                                    rows={[
                                        { label: "super_admin", count: stats?.usersByRole.superAdmin ?? 0, color: "var(--primary)" },
                                        { label: "tier_list_admin", count: stats?.usersByRole.tierListAdmin ?? 0, color: "oklch(0.7 0.16 84.4)" },
                                        { label: "tier_list_editor", count: stats?.usersByRole.tierListEditor ?? 0, color: "oklch(0.55 0.15 184.7)" },
                                        { label: "user", count: stats?.usersByRole.user ?? 0, color: "oklch(0.7 0.005 285)" },
                                    ]}
                                />
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">Recently signed-up users</CardTitle>
                            <CardDescription className="text-xs">
                                Last {stats?.recentUsers.length ?? 0} Doctors to authenticate. From <HCode>GET /admin/stats</HCode>.
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="pt-0">
                            {statsQuery.isPending ? (
                                <Skeleton className="h-40 w-full" />
                            ) : stats && stats.recentUsers.length > 0 ? (
                                <table className="w-full border-collapse text-[13px]">
                                    <thead>
                                        <tr className="border-border border-b">
                                            <th className="px-2 py-2 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">Doctor</th>
                                            <th className="px-2 py-2 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">Server</th>
                                            <th className="px-2 py-2 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">Level</th>
                                            <th className="px-2 py-2 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">Joined</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {stats.recentUsers.slice(0, 6).map((u) => (
                                            <tr key={u.uid} className="border-border border-b last:border-0">
                                                <td className="px-2 py-2">
                                                    <DashUserCell uid={u.uid} fallbackName={u.nickname ?? "-"} />
                                                </td>
                                                <td className="px-2 py-2 font-mono">{serverIdToCode(u.serverId)}</td>
                                                <td className="px-2 py-2 tabular-nums">{u.level ?? "-"}</td>
                                                <td className="px-2 py-2 text-muted-foreground">{formatRelative(u.createdAt)}</td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            ) : (
                                <div className="px-1 py-6 text-center text-[13px] text-muted-foreground">No recent signups.</div>
                            )}
                        </CardContent>
                    </Card>
                </div>

                <div className="grid gap-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">Service health</CardTitle>
                            <CardDescription className="text-xs">GET /health</CardDescription>
                            <CardAction>{health ? health.status === "ok" ? <Badge variant="success">healthy</Badge> : <Badge variant="warning">degraded</Badge> : null}</CardAction>
                        </CardHeader>
                        <CardContent className="pt-0">
                            {healthQuery.isPending ? (
                                <Skeleton className="h-24 w-full" />
                            ) : health ? (
                                <>
                                    <div className="flex items-center justify-between pb-2">
                                        <span className="text-muted-foreground">Postgres</span>
                                        <StatusDot state={health.database.status === "connected" ? "green" : "red"}>
                                            <span className="font-mono">{health.database.responseTimeMs} ms</span>
                                        </StatusDot>
                                    </div>
                                    <div className="border-border border-t" />
                                    <div className="flex items-center justify-between py-2">
                                        <span className="text-muted-foreground">Cache · {health.cache.backend}</span>
                                        <StatusDot state={health.cache.status === "connected" ? "green" : "red"}>
                                            <span className="font-mono">{health.cache.responseTimeMs} ms</span>
                                        </StatusDot>
                                    </div>
                                    <div className="border-border border-t" />
                                    <div className="flex items-center justify-between py-2">
                                        <span className="text-muted-foreground">Game data</span>
                                        <StatusDot state="green">
                                            <span className="font-mono">{totalOperators} operators</span>
                                        </StatusDot>
                                    </div>
                                    <div className="border-border border-t" />
                                    <div className="flex items-center justify-between pt-2">
                                        <span className="text-muted-foreground">Round-trip</span>
                                        <span className="font-mono text-[12px]">{health.responseTimeMs} ms</span>
                                    </div>
                                </>
                            ) : (
                                <div className="text-[12px] text-muted-foreground">Health probe unavailable.</div>
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">Signed in as</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                            <div className="flex items-center gap-2.5">
                                <span className="block size-8.5 rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]" />
                                <div className="flex min-w-0 flex-1 flex-col">
                                    <span className="truncate font-medium text-[13px]">{user?.nickname ?? "-"}</span>
                                    <span className="font-mono text-[11.5px] text-muted-foreground">
                                        UID {user?.uid ?? "-"} · {user?.role ?? "-"}
                                    </span>
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">Quick links</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                            <Timeline
                                items={[
                                    {
                                        when: "manage",
                                        what: (
                                            <Link to="/admin/permissions" className="hover:underline">
                                                Tier list permissions
                                            </Link>
                                        ),
                                        who: "View / Edit / Publish / Admin",
                                    },
                                    {
                                        when: "manage",
                                        what: (
                                            <Link to="/admin/official-tier-lists" className="hover:underline">
                                                Official tier lists
                                            </Link>
                                        ),
                                        who: "Flair-tagged lists in the Official rail",
                                    },
                                    {
                                        when: "manage",
                                        what: (
                                            <Link to="/admin/operator-notes" className="hover:underline">
                                                Operator notes
                                            </Link>
                                        ),
                                        who: "Community guidance per operator",
                                    },
                                    {
                                        when: "operate",
                                        what: (
                                            <Link to="/admin/health" className="hover:underline">
                                                Health & cache
                                            </Link>
                                        ),
                                        who: "Redis + Postgres probes",
                                    },
                                    {
                                        when: "operate",
                                        what: (
                                            <Link to="/admin/audit" className="hover:underline">
                                                Audit log
                                            </Link>
                                        ),
                                        who: "Permission grants + note edits",
                                    },
                                ]}
                            />
                        </CardContent>
                    </Card>
                </div>
            </div>
        </>
    );
}

function serverIdToCode(id: number): string {
    if (id === 0) return "EN";
    if (id === 1) return "JP";
    if (id === 2) return "KR";
    if (id === 3) return "CN";
    return `srv${id}`;
}

function RoleBreakdownBars({ rows }: { rows: { label: string; count: number; color: string }[] }): React.ReactElement {
    const max = Math.max(...rows.map((r) => r.count), 1);
    return (
        <div className="grid gap-3">
            {rows.map((r) => (
                <div key={r.label}>
                    <div className="mb-1 flex items-center justify-between">
                        <span className="font-mono text-[12px]">{r.label}</span>
                        <span className="font-mono text-[11.5px] text-muted-foreground tabular-nums">{r.count.toLocaleString()}</span>
                    </div>
                    <div className="h-1.5 overflow-hidden rounded-[3px] bg-muted">
                        <div className="h-full" style={{ width: `${Math.max(2, (r.count / max) * 100)}%`, background: r.color }} />
                    </div>
                </div>
            ))}
        </div>
    );
}
