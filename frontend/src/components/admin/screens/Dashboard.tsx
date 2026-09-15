import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowRightIcon, ExternalLinkIcon, RefreshCwIcon } from "lucide-react";
import { useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Skeleton } from "#/components/ui/skeleton";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions, formatResponseTimeMs, healthQueryOptions } from "#/lib/api/admin";
import { userQueryOptions } from "#/lib/api/user";
import { type IFormatters, type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { getSecretaryAvatarURL } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import { HCode, PageHead } from "../AdminShell";
import { StatTile, StatusDot, Timeline } from "../Primitives";
import type { messages as primitivesMessages } from "../Primitives.messages";
import type { messages } from "./Dashboard.messages";

/** The role labels in the breakdown chart are the ones `RoleBadge` declares. */
type DashT = TypedT<typeof messages & typeof primitivesMessages>;
type DashRichT = TypedRichT<typeof messages & typeof primitivesMessages>;

function SignedInAvatar({ user }: { user: IUserProfile | null }): React.ReactElement {
    const [failed, setFailed] = useState(false);
    const url = user ? getSecretaryAvatarURL({ secretary: user.secretary, secretary_skin_id: user.secretary_skin_id }) : null;
    return (
        <span className="relative inline-block size-8.5 shrink-0 overflow-hidden rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]">
            {url && !failed ? (
                <img src={url} alt="" loading="lazy" decoding="async" className="absolute inset-0 size-full object-cover" onError={() => setFailed(true)} />
            ) : (
                <span className="absolute inset-0 flex items-center justify-center font-bold text-[12px] text-white/90">{user?.nickname?.[0]?.toUpperCase() ?? "?"}</span>
            )}
        </span>
    );
}

function DashUserCell({ uid, fallbackName }: { uid: string; fallbackName: string }): React.ReactElement {
    const t: DashT = useT("admin");
    const profile = useQuery({ ...userQueryOptions(uid), retry: 0 });
    const u = profile.data;
    return (
        <span className="flex min-w-0 items-center gap-2">
            <span className="relative inline-block size-6 shrink-0 overflow-hidden rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]">
                {u ? <img src={getSecretaryAvatarURL({ secretary: u.secretary, secretary_skin_id: u.secretary_skin_id })} alt="" loading="lazy" className="absolute inset-0 size-full object-cover" onError={(e) => (e.currentTarget.style.display = "none")} /> : null}
            </span>
            <span className="flex min-w-0 flex-col leading-tight">
                <Link to="/user/$id" params={{ id: uid }} className="truncate font-medium hover:underline">
                    {u?.nickname ?? fallbackName}
                </Link>
                <span className="truncate font-mono text-[11px] text-muted-foreground">{t("dash.user.uid", { uid })}</span>
            </span>
        </span>
    );
}

function compactNumber(n: number, f: IFormatters): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, "")}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, "")}K`;
    return f.number(n);
}

export function Dashboard(): React.ReactElement {
    const t: DashT = useT("admin");
    const rt: DashRichT = useRichT("admin");
    const f = useFormatters();
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
                kicker={t("dash.kicker")}
                title={t("dash.title")}
                sub={rt("dash.sub", { stats: <HCode>GET /admin/stats</HCode>, health: <HCode>/health</HCode> })}
                action={
                    <>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => {
                                void statsQuery.refetch();
                                void healthQuery.refetch();
                            }}
                            disabled={statsQuery.isFetching || healthQuery.isFetching}
                            loading={statsQuery.isFetching || healthQuery.isFetching}
                        >
                            <RefreshCwIcon />
                            {t("dash.refresh")}
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
                            {t("dash.viewPublicStats")}
                        </Button>
                    </>
                }
            />

            {statsQuery.isError ? (
                <div className="mb-4 rounded-2xl border border-destructive/32 bg-destructive/8 p-4 text-[13px] text-destructive-foreground">
                    <strong>{t("dash.statsError.title")}</strong> {rt("dash.statsError.body", { role: <span className="font-mono">tier_list_admin</span> })}
                </div>
            ) : null}

            <section className="mb-4 grid grid-cols-2 gap-3 sm:gap-3.5 lg:grid-cols-4">
                {statsQuery.isPending ? (
                    <>
                        <Skeleton className="h-31 rounded-2xl" />
                        <Skeleton className="h-31 rounded-2xl" />
                        <Skeleton className="h-31 rounded-2xl" />
                        <Skeleton className="h-31 rounded-2xl" />
                    </>
                ) : (
                    <>
                        <StatTile label={t("dash.tile.totalUsers")} value={compactNumber((stats?.usersByRole.user ?? 0) + (stats?.usersByRole.tierListEditor ?? 0) + (stats?.usersByRole.tierListAdmin ?? 0) + (stats?.usersByRole.translator ?? 0) + (stats?.usersByRole.superAdmin ?? 0), f)} color="var(--chart-1)" />
                        <StatTile label={t("dash.tile.rostersSynced")} value={compactNumber(totalRosters, f)} color="var(--chart-2)" />
                        <StatTile label={t("dash.tile.tierListsActive")} value={`${activeTierLists}`} unit={t("dash.tile.of", { count: totalTierLists })} color="var(--chart-2)" />
                        <StatTile label={t("dash.tile.placements")} value={compactNumber(totalPlacements, f)} color="var(--chart-4)" />
                    </>
                )}
            </section>

            <div className="grid min-w-0 grid-cols-1 gap-4.5 xl:grid-cols-[minmax(0,1fr)_320px]">
                <div className="grid min-w-0 gap-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">{t("dash.roles.title")}</CardTitle>
                            <CardDescription className="text-xs">{rt("dash.roles.desc", { column: <span className="font-mono">users.role</span> })}</CardDescription>
                            <CardAction>
                                <Button variant="ghost" size="sm" render={<Link to="/admin/users" />}>
                                    {t("dash.roles.manageUsers")} <ArrowRightIcon />
                                </Button>
                            </CardAction>
                        </CardHeader>
                        <CardContent>
                            {statsQuery.isPending ? (
                                <Skeleton className="h-24 w-full" />
                            ) : (
                                <RoleBreakdownBars
                                    rows={[
                                        { label: t("role.superAdmin"), count: stats?.usersByRole.superAdmin ?? 0, color: "var(--primary)" },
                                        { label: t("role.tierListAdmin"), count: stats?.usersByRole.tierListAdmin ?? 0, color: "oklch(0.7 0.16 84.4)" },
                                        { label: t("role.tierListEditor"), count: stats?.usersByRole.tierListEditor ?? 0, color: "oklch(0.55 0.15 184.7)" },
                                        { label: t("role.translator"), count: stats?.usersByRole.translator ?? 0, color: "oklch(0.62 0.17 305)" },
                                        { label: t("role.user"), count: stats?.usersByRole.user ?? 0, color: "oklch(0.7 0.005 285)" },
                                    ]}
                                />
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">{t("dash.recent.title")}</CardTitle>
                            <CardDescription className="text-xs">{rt("dash.recent.desc", { count: stats?.recentUsers.length ?? 0, endpoint: <HCode>GET /admin/stats</HCode> })}</CardDescription>
                        </CardHeader>
                        <CardContent className="pt-0">
                            {statsQuery.isPending ? (
                                <Skeleton className="h-40 w-full" />
                            ) : stats && stats.recentUsers.length > 0 ? (
                                <div className="overflow-x-auto">
                                    <table className="w-full border-collapse text-[13px]">
                                        <thead>
                                            <tr className="border-border border-b">
                                                <th className="px-2 py-2 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">{t("dash.th.doctor")}</th>
                                                <th className="px-2 py-2 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">{t("dash.th.server")}</th>
                                                <th className="hidden px-2 py-2 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em] sm:table-cell">{t("dash.th.level")}</th>
                                                <th className="px-2 py-2 text-right font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">{t("dash.th.joined")}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {stats.recentUsers.slice(0, 6).map((u) => (
                                                <tr key={u.uid} className="border-border border-b last:border-0">
                                                    <td className="min-w-0 max-w-0 px-2 py-2">
                                                        <DashUserCell uid={u.uid} fallbackName={u.nickname ?? "-"} />
                                                    </td>
                                                    <td className="px-2 py-2 font-mono text-muted-foreground">{serverIdToCode(u.serverId)}</td>
                                                    <td className="hidden px-2 py-2 tabular-nums sm:table-cell">{u.level ?? "-"}</td>
                                                    <td className="whitespace-nowrap px-2 py-2 text-right text-muted-foreground tabular-nums">{f.relativeShort(u.createdAt)}</td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            ) : (
                                <div className="px-1 py-6 text-center text-[13px] text-muted-foreground">{t("dash.recent.empty")}</div>
                            )}
                        </CardContent>
                    </Card>
                </div>

                <div className="grid gap-4">
                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">{t("dash.health.title")}</CardTitle>
                            <CardDescription className="text-xs">GET /health</CardDescription>
                            <CardAction>{health ? health.status === "ok" ? <Badge variant="success">{t("dash.health.healthy")}</Badge> : <Badge variant="warning">{t("dash.health.degraded")}</Badge> : null}</CardAction>
                        </CardHeader>
                        <CardContent className="pt-0">
                            {healthQuery.isPending ? (
                                <Skeleton className="h-24 w-full" />
                            ) : health ? (
                                <>
                                    <div className="flex items-center justify-between gap-3 pb-2 text-[12.5px]">
                                        <span className="truncate text-muted-foreground">{t("dash.health.postgres")}</span>
                                        <StatusDot state={health.database.status === "connected" ? "green" : "red"}>
                                            <span className="font-mono tabular-nums">{t("dash.health.ms", { ms: formatResponseTimeMs(health.database.responseTimeMs) })}</span>
                                        </StatusDot>
                                    </div>
                                    <div className="border-border border-t" />
                                    <div className="flex items-center justify-between gap-3 py-2 text-[12.5px]">
                                        <span className="min-w-0 truncate text-muted-foreground">
                                            {t("dash.health.cache")} <span className="text-foreground/70">{health.cache.backend}</span>
                                        </span>
                                        <StatusDot state={health.cache.status === "connected" ? "green" : "red"}>
                                            <span className="font-mono tabular-nums">{t("dash.health.ms", { ms: formatResponseTimeMs(health.cache.responseTimeMs) })}</span>
                                        </StatusDot>
                                    </div>
                                    <div className="border-border border-t" />
                                    <div className="flex items-center justify-between gap-3 py-2 text-[12.5px]">
                                        <span className="truncate text-muted-foreground">{t("dash.health.gameData")}</span>
                                        <StatusDot state="green">
                                            <span className="font-mono tabular-nums">{t("dash.health.ops", { count: f.number(totalOperators) })}</span>
                                        </StatusDot>
                                    </div>
                                    <div className="border-border border-t" />
                                    <div className="flex items-center justify-between gap-3 pt-2 text-[12.5px]">
                                        <span className="truncate text-muted-foreground">{t("dash.health.roundTrip")}</span>
                                        <span className="font-mono text-[12px] tabular-nums">{t("dash.health.ms", { ms: formatResponseTimeMs(health.responseTimeMs) })}</span>
                                    </div>
                                </>
                            ) : (
                                <div className="text-[12px] text-muted-foreground">{t("dash.health.unavailable")}</div>
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">{t("dash.signedIn.title")}</CardTitle>
                            {user?.uid ? (
                                <CardAction>
                                    <Button variant="ghost" size="sm" render={<Link to="/user/$id" params={{ id: user.uid }} target="_blank" />}>
                                        {t("dash.signedIn.profile")} <ExternalLinkIcon />
                                    </Button>
                                </CardAction>
                            ) : null}
                        </CardHeader>
                        <CardContent className="pt-0">
                            <div className="flex items-center gap-2.5">
                                <SignedInAvatar user={user} />
                                <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                                    <span className="truncate font-medium text-[13px]">{user?.nickname ?? "-"}</span>
                                    <span className="truncate font-mono text-[11.5px] text-muted-foreground">{t("dash.user.uid", { uid: user?.uid ?? "-" })}</span>
                                    <span className="truncate font-mono text-[11px] text-muted-foreground/80">{user?.role ?? "-"}</span>
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">{t("dash.quick.title")}</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                            <Timeline
                                items={[
                                    {
                                        when: t("dash.quick.when.manage"),
                                        what: (
                                            <Link to="/admin/permissions" className="hover:underline">
                                                {t("dash.quick.permissions")}
                                            </Link>
                                        ),
                                        who: t("dash.quick.permissions.who"),
                                    },
                                    {
                                        when: t("dash.quick.when.manage"),
                                        what: (
                                            <Link to="/admin/official-tier-lists" className="hover:underline">
                                                {t("dash.quick.official")}
                                            </Link>
                                        ),
                                        who: t("dash.quick.official.who"),
                                    },
                                    {
                                        when: t("dash.quick.when.manage"),
                                        what: (
                                            <Link to="/admin/operator-notes" className="hover:underline">
                                                {t("dash.quick.notes")}
                                            </Link>
                                        ),
                                        who: t("dash.quick.notes.who"),
                                    },
                                    {
                                        when: t("dash.quick.when.operate"),
                                        what: (
                                            <Link to="/admin/health" className="hover:underline">
                                                {t("dash.quick.health")}
                                            </Link>
                                        ),
                                        who: t("dash.quick.health.who"),
                                    },
                                    {
                                        when: t("dash.quick.when.operate"),
                                        what: (
                                            <Link to="/admin/audit" className="hover:underline">
                                                {t("dash.quick.audit")}
                                            </Link>
                                        ),
                                        who: t("dash.quick.audit.who"),
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
    const f = useFormatters();
    const max = Math.max(...rows.map((r) => r.count), 1);
    return (
        <div className="grid gap-3">
            {rows.map((r) => (
                <div key={r.label}>
                    <div className="mb-1 flex items-center justify-between">
                        <span className="font-mono text-[12px]">{r.label}</span>
                        <span className="font-mono text-[11.5px] text-muted-foreground tabular-nums">{f.number(r.count)}</span>
                    </div>
                    <div className="h-1.5 overflow-hidden rounded-[3px] bg-muted">
                        <div className="h-full" style={{ width: `${Math.max(2, (r.count / max) * 100)}%`, background: r.color }} />
                    </div>
                </div>
            ))}
        </div>
    );
}
