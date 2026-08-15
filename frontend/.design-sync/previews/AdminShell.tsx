import { AdminKicker, AdminLevelBadge, AdminOperatorNotes, AdminShell, AdminStatTile, Badge, Button, Card, CardContent, CardHeader, CardTitle, HCode, PageHead, RoleBadge, Timeline } from "frontend";
import { ExternalLinkIcon, PlusIcon, RefreshCwIcon } from "lucide-react";

const grants = [
    { uid: "10289471", nickname: "Kyostinv", level: "Admin" as const, granted: "3mo ago" },
    { uid: "42118900", nickname: "Dr. Reisen", level: "Publish" as const, granted: "6w ago" },
    { uid: "77004512", nickname: "Ceylonade", level: "Edit" as const, granted: "12d ago" },
];

export function DashboardPage() {
    return (
        <AdminShell crumbs={[{ label: "myrtle.moe", href: "https://myrtle.moe" }, { label: "Admin", to: "/admin" }, { label: "Dashboard" }]}>
            <PageHead
                kicker="Overview"
                title="Dashboard"
                sub={
                    <>
                        Service-level signals from the Rust backend — surfaced verbatim from <HCode>GET /admin/stats</HCode> and <HCode>/health</HCode>.
                    </>
                }
                action={
                    <>
                        <Button variant="outline" size="sm">
                            <RefreshCwIcon />
                            Refresh
                        </Button>
                        <Button variant="outline" size="sm">
                            <ExternalLinkIcon />
                            View public stats
                        </Button>
                    </>
                }
            />
            <section className="mb-4 grid grid-cols-2 gap-3 sm:grid-cols-4 sm:gap-3.5">
                <AdminStatTile label="Total users" value="18.4K" color="var(--chart-1)" />
                <AdminStatTile label="Rosters synced" value="12.9K" color="var(--chart-2)" />
                <AdminStatTile label="Tier lists · active" value="37" unit="of 214" color="var(--chart-2)" />
                <AdminStatTile label="Tier-list placements" value="41.6K" color="var(--chart-4)" />
            </section>
            <Card>
                <CardHeader>
                    <CardTitle className="text-sm">Recent admin activity</CardTitle>
                </CardHeader>
                <CardContent className="pt-0">
                    <Timeline
                        items={[
                            { when: "4m ago", what: "Edited `pros` on Mlynar", who: "Kyostinv · super_admin" },
                            { when: "1h ago", what: "Published version 14 of /endgame-dps", who: "Dr. Reisen · tier_list_admin" },
                            { when: "yesterday", what: "Granted Edit on /cc12-sandbox-picks", who: "Kyostinv · super_admin" },
                        ]}
                    />
                </CardContent>
            </Card>
        </AdminShell>
    );
}

export function DeepBreadcrumbPage() {
    return (
        <AdminShell crumbs={[{ label: "Admin", to: "/admin" }, { label: "Manage", to: "/admin" }, { label: "Official tier lists", to: "/admin/official-tier-lists" }, { label: "Endgame DPS rankings" }]}>
            <PageHead
                kicker="Manage · official tier lists"
                title="Endgame DPS rankings"
                sub={
                    <>
                        Public at <HCode>/tier-lists/endgame-dps</HCode> · 6 tiers · 96 placements · flair <Badge variant="success">Official</Badge>
                    </>
                }
                action={
                    <Button size="sm">
                        <PlusIcon />
                        Grant access
                    </Button>
                }
            />
            <div className="mb-3">
                <AdminKicker>Per-list grants</AdminKicker>
            </div>
            <div className="relative overflow-hidden rounded-2xl border border-border bg-card">
                <table className="w-full border-collapse text-[13px]">
                    <thead>
                        <tr>
                            {["Doctor", "Global role", "Access", "Granted"].map((h) => (
                                <th key={h} className="border-border border-b px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                    {h}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {grants.map((g, i) => (
                            <tr key={g.uid} className="border-border border-b last:border-0">
                                <td className="px-3.5 py-2.5">
                                    <span className="font-medium">{g.nickname}</span>
                                    <span className="ml-1.5 font-mono text-[11.5px] text-muted-foreground">UID {g.uid}</span>
                                </td>
                                <td className="px-3.5 py-2.5">
                                    <RoleBadge role={i === 0 ? "super_admin" : i === 1 ? "tier_list_admin" : "tier_list_editor"} />
                                </td>
                                <td className="px-3.5 py-2.5">
                                    <AdminLevelBadge level={g.level} />
                                </td>
                                <td className="whitespace-nowrap px-3.5 py-2.5 text-muted-foreground tabular-nums">{g.granted}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </AdminShell>
    );
}

export function WrappingAScreen() {
    return (
        <AdminShell crumbs={[{ label: "Admin", to: "/admin" }, { label: "Manage", to: "/admin" }, { label: "Operator notes" }]}>
            <AdminOperatorNotes />
        </AdminShell>
    );
}
