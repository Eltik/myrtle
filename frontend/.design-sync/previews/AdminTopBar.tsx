import { AdminStatTile, AdminTopBar, PageHead } from "frontend";

const noop = () => {};

export function DashboardCrumbs() {
    return <AdminTopBar crumbs={[{ label: "myrtle.moe", href: "https://myrtle.moe" }, { label: "Admin", to: "/admin" }, { label: "Dashboard" }]} onOpenSidebar={noop} />;
}

export function DeepCrumbs() {
    return <AdminTopBar crumbs={[{ label: "Admin", to: "/admin" }, { label: "Official tier lists", to: "/admin/official-tier-lists" }, { label: "Endgame DPS rankings" }]} onOpenSidebar={noop} />;
}

export function SingleCrumb() {
    return <AdminTopBar crumbs={[{ label: "Health & cache" }]} onOpenSidebar={noop} />;
}

export function AbovePageContent() {
    return (
        <div className="flex min-h-[520px] flex-col">
            <AdminTopBar crumbs={[{ label: "Admin", to: "/admin" }, { label: "Operate", to: "/admin" }, { label: "Health & cache" }]} onOpenSidebar={noop} />
            <div className="px-4 pt-5 sm:px-6">
                <PageHead kicker="Operate" title="Health &amp; cache" sub="Live probe of the Rust backend — GET /health and the public /stats snapshot." />
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-3.5">
                    <AdminStatTile label="Postgres" value="14" unit="ms" color="var(--chart-2)" />
                    <AdminStatTile label="Cache · redis" value="3.2" unit="ms" color="var(--chart-1)" />
                    <AdminStatTile label="Round-trip" value="184" unit="ms" color="var(--chart-4)" />
                </div>
            </div>
        </div>
    );
}
