import { AdminKicker, AdminSidebar, AdminStatTile, Card, CardContent, CardHeader, CardTitle, HCode, PageHead, StatusDot, Timeline } from "frontend";

// The admin rail. It is `fixed` + backdrop below `lg` and a sticky in-flow
// column at `lg` and up, so the stories mirror `AdminShell`'s own composition —
// `<div className="flex min-h-screen">` with the rail first and the screen
// beside it. At the 900px capture viewport that reads as the drawer over a
// dimmed page (the mobile state); at 1280 it reads as the desktop rail.
//
// The nav counts come from `adminStatsQueryOptions` / `browseTierListsQueryOptions`
// and the footer chip from `useAuth()`, all stubbed in a preview — so the rows
// render countless and the account chip reads "Guest".

const noop = () => {};

function Stage({ children }: { children: React.ReactNode }) {
    return <div className="flex min-h-[520px] w-full bg-background text-foreground">{children}</div>;
}

export function OverOverviewPage() {
    return (
        <Stage>
            <AdminSidebar open onClose={noop} />
            <main className="flex min-w-0 flex-1 flex-col px-4 pt-4 pb-10 sm:px-6">
                <PageHead
                    kicker="Overview"
                    title="Dashboard"
                    sub={
                        <>
                            Service-level signals from the Rust backend — surfaced verbatim from <HCode>GET /admin/stats</HCode>.
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
                            ]}
                        />
                    </CardContent>
                </Card>
            </main>
        </Stage>
    );
}

export function OverHealthPage() {
    return (
        <Stage>
            <AdminSidebar open onClose={noop} />
            <main className="flex min-w-0 flex-1 flex-col px-4 pt-4 pb-10 sm:px-6">
                <PageHead
                    kicker="Operate"
                    title="Health &amp; cache"
                    sub={
                        <>
                            Live probe of the Rust backend — <HCode>GET /health</HCode> and the public <HCode>/stats</HCode> snapshot.
                        </>
                    }
                />
                <div className="mb-3">
                    <AdminKicker>Probes</AdminKicker>
                </div>
                <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
                    {[
                        { label: "Postgres", value: "8", tone: "green" as const, note: "connected" },
                        { label: "Cache · gamedata", value: "1", tone: "green" as const, note: "warm" },
                        { label: "Round-trip", value: "42", tone: "amber" as const, note: "degraded" },
                    ].map((p) => (
                        <div key={p.label} className="rounded-2xl border border-border bg-card px-3.5 py-3">
                            <div className="mb-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{p.label}</div>
                            <div className="font-semibold text-[22px] leading-none tracking-[-0.02em]">
                                {p.value}
                                <span className="ml-1 font-mono text-[11px] text-muted-foreground">ms</span>
                            </div>
                            <div className="mt-2 text-muted-foreground">
                                <StatusDot state={p.tone}>{p.note}</StatusDot>
                            </div>
                        </div>
                    ))}
                </div>
            </main>
        </Stage>
    );
}
