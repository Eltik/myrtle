import { useQuery } from "@tanstack/react-query";
import { Link, useRouterState } from "@tanstack/react-router";
import { ActivityIcon, ChevronDownIcon, FileTextIcon, LayoutDashboardIcon, ListOrderedIcon, type LucideIcon, SettingsIcon, ShieldIcon, UsersIcon, ZapIcon } from "lucide-react";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions } from "#/lib/api/admin";
import { operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { browseTierListsQueryOptions } from "#/lib/api/tier-lists";

interface INavItem {
    to: string;
    label: string;
    icon: LucideIcon;
    count?: number;
}

function NavRow({ item, active }: { item: INavItem; active: boolean }): React.ReactElement {
    const Icon = item.icon;
    return (
        <Link
            to={item.to}
            className={
                active
                    ? "flex w-full items-center gap-2.5 rounded-lg border border-primary/20 bg-primary/10 px-2.5 py-1.5 text-left font-medium text-[13px] text-primary transition-colors"
                    : "flex w-full items-center gap-2.5 rounded-lg border border-transparent px-2.5 py-1.5 text-left font-medium text-[13px] text-sidebar-foreground transition-colors hover:bg-sidebar-accent"
            }
        >
            <Icon className="size-4 shrink-0 opacity-85" strokeWidth={1.9} />
            <span className="truncate">{item.label}</span>
            {item.count != null ? <span className={`ml-auto font-mono text-[10.5px] ${active ? "text-primary" : "text-muted-foreground"}`}>{item.count.toLocaleString()}</span> : null}
        </Link>
    );
}

export function AdminSidebar(): React.ReactElement {
    const { user, isAuthenticated } = useAuth();
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const isActive = (to: string) => (to === "/admin" ? pathname === "/admin" : pathname.startsWith(to));

    // Live counts
    const statsQuery = useQuery(adminStatsQueryOptions(isAuthenticated));
    const tierListsQuery = useQuery(browseTierListsQueryOptions());
    const notesQuery = useQuery(operatorNotesListQueryOptions());

    const totalUsers = statsQuery.data ? (statsQuery.data.usersByRole.user ?? 0) + (statsQuery.data.usersByRole.tierListEditor ?? 0) + (statsQuery.data.usersByRole.tierListAdmin ?? 0) + (statsQuery.data.usersByRole.superAdmin ?? 0) : undefined;
    const officialCount = tierListsQuery.data ? tierListsQuery.data.filter((tl) => tl.listType === "official").length : undefined;
    const tierListCount = tierListsQuery.data?.length;
    const notesCount = notesQuery.data?.length;

    const manage: INavItem[] = [
        { to: "/admin", label: "Dashboard", icon: LayoutDashboardIcon },
        { to: "/admin/users", label: "Users", icon: UsersIcon, count: totalUsers },
        { to: "/admin/official-tier-lists", label: "Official tier lists", icon: ListOrderedIcon, count: officialCount },
        { to: "/admin/permissions", label: "Tier Lists", icon: ShieldIcon, count: tierListCount },
        { to: "/admin/operator-notes", label: "Operator notes", icon: FileTextIcon, count: notesCount },
    ];

    const operate: INavItem[] = [
        { to: "/admin/health", label: "Health & cache", icon: ZapIcon },
        { to: "/admin/audit", label: "Audit log", icon: ActivityIcon },
        { to: "/admin/settings", label: "Settings", icon: SettingsIcon },
    ];

    return (
        <aside className="sticky top-0 flex h-screen flex-col border-sidebar-border border-r bg-sidebar">
            <div className="flex items-center gap-2.5 px-4.5 py-4">
                <img src="/logo/bust_transparent.png" alt="myrtle" className="size-7 shrink-0 object-contain" />
                <span className="font-semibold text-[15px] leading-none tracking-[-0.01em]">myrtle.moe</span>
                <span className="inline-flex h-4 items-center rounded-[5px] border border-border bg-muted px-1.5 font-medium font-mono text-[10px] text-muted-foreground leading-none">v3</span>
                <span className="font-medium font-mono text-[10px] text-primary leading-none tracking-wide">[admin]</span>
            </div>

            <nav className="flex flex-1 flex-col gap-px overflow-auto p-2">
                <div className="px-3 pt-3.5 pb-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Manage</div>
                {manage.map((it) => (
                    <NavRow key={it.to} item={it} active={isActive(it.to)} />
                ))}
                <div className="px-3 pt-3.5 pb-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Operate</div>
                {operate.map((it) => (
                    <NavRow key={it.to} item={it} active={isActive(it.to)} />
                ))}
            </nav>

            <div className="border-sidebar-border border-t p-2.5">
                <div className="flex items-center gap-2.5 rounded-[10px] border border-sidebar-border bg-card px-2.5 py-2">
                    <span className="block size-6.5 shrink-0 rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]" />
                    <div className="flex min-w-0 flex-1 flex-col gap-px">
                        <span className="truncate font-semibold text-[12px] leading-tight">{user?.nickname ?? "Eltik"}</span>
                        <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.06em]">{user?.role ?? "super_admin"}</span>
                    </div>
                    <ChevronDownIcon className="size-3.5 opacity-60" strokeWidth={1.9} />
                </div>
                <div className="flex items-center justify-end pt-1.5">
                    <Link to="/" className="text-[11px] text-muted-foreground hover:text-foreground">
                        ← back to site
                    </Link>
                </div>
            </div>
        </aside>
    );
}
