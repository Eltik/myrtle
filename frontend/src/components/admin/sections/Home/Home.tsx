import { useQuery } from "@tanstack/react-query";
import { StatTile } from "#/components/admin/Primitives";
import { type IAdminAccess, useAdminAccess } from "#/components/admin/shell/access";
import { PageHead } from "#/components/admin/shell/PageHead";
import type { IAdminHomeSearch } from "#/components/admin/shell/search";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions } from "#/lib/api/admin";
import { useFormatters, useT } from "#/lib/i18n";
import { FocusCards } from "./FocusCards";
import { InboxCard } from "./InboxCard";
import { AccessCard, RecentCard } from "./SideCards";
import type { HomeT } from "./useHomeLabels";

export interface IHomeProps {
    search: IAdminHomeSearch;
}

/** Home (spec 2.1): the staff Inbox, or "My work" for a translator or editor. */
export default function Home(_props: IHomeProps): React.ReactElement {
    const t: HomeT = useT("admin");
    const { user } = useAuth();
    const access = useAdminAccess();
    const { staff } = access;
    const sub = access.role === "translator" ? t("home.head.sub.translator") : access.role === "tier_list_editor" ? t("home.head.sub.tierListEditor") : undefined;

    return (
        <>
            <PageHead kicker={t("home.head.kicker", { nickname: user?.nickname ?? "" })} title={staff ? t("home.head.titleStaff") : t("home.head.titleOwn")} sub={staff ? undefined : sub} />
            <div className="flex flex-col gap-4">
                {staff ? <StatTiles /> : null}
                <div className="grid items-start gap-4 lg:grid-cols-[minmax(0,1.7fr)_minmax(0,1fr)]">
                    <div className="flex min-w-0 flex-col gap-4">{staff ? <InboxCard access={access} /> : <FocusCards access={access} />}</div>
                    <SideColumn access={access} />
                </div>
            </div>
        </>
    );
}

function SideColumn({ access }: { access: IAdminAccess }): React.ReactElement {
    return (
        <div className="flex min-w-0 flex-col gap-4">
            <AccessCard access={access} />
            <RecentCard access={access} />
        </div>
    );
}

/** The four staff tiles (design `stats`). */
function StatTiles(): React.ReactElement {
    const t: HomeT = useT("admin");
    const f = useFormatters();
    const { isAuthenticated } = useAuth();
    const access = useAdminAccess();
    const statsQuery = useQuery({ ...adminStatsQueryOptions(isAuthenticated), enabled: isAuthenticated && access.staff });
    const stats = statsQuery.data;
    const byRole = stats?.usersByRole;
    const totalUsers = byRole ? byRole.user + byRole.translator + byRole.tierListEditor + byRole.tierListAdmin + byRole.superAdmin : undefined;
    const staffCount = byRole && totalUsers !== undefined ? totalUsers - byRole.user : undefined;
    const dash = "—";

    return (
        <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
            <StatTile label={t("home.tile.players")} value={totalUsers === undefined ? dash : f.compact(totalUsers)} unit={staffCount === undefined ? undefined : t("home.tile.players.unit", { count: f.number(staffCount) })} color="var(--chart-1)" />
            <StatTile label={t("home.tile.rosters")} value={stats ? f.compact(stats.rosters.total) : dash} unit={stats && totalUsers ? t("home.tile.rosters.unit", { percent: f.percent(stats.rosters.total / totalUsers) }) : undefined} color="var(--chart-2)" />
            <StatTile label={t("home.tile.tierLists")} value={stats ? f.number(stats.tierLists.active) : dash} unit={stats ? t("home.tile.tierLists.unit", { total: f.number(stats.tierLists.total) }) : undefined} color="var(--chart-3)" />
            <StatTile label={t("home.tile.placements")} value={stats ? f.compact(stats.tierLists.totalPlacements) : dash} unit={t("home.tile.placements.unit")} color="var(--chart-4)" />
        </div>
    );
}
