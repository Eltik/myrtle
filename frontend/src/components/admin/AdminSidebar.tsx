import { useQuery } from "@tanstack/react-query";
import { Link, useRouterState } from "@tanstack/react-router";
import { ActivityIcon, ChevronDownIcon, FileTextIcon, LayoutDashboardIcon, ListOrderedIcon, type LucideIcon, SettingsIcon, ShieldIcon, UsersIcon, XIcon, ZapIcon } from "lucide-react";
import { useEffect } from "react";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions } from "#/lib/api/admin";
import { operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { browseTierListsQueryOptions } from "#/lib/api/tier-lists";
import { cn } from "#/lib/utils";

interface INavItem {
    to: string;
    label: string;
    icon: LucideIcon;
    count?: number;
}

function NavRow({ item, active, onNavigate }: { item: INavItem; active: boolean; onNavigate?: () => void }): React.ReactElement {
    const Icon = item.icon;
    return (
        <Link
            to={item.to}
            onClick={onNavigate}
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

interface IAdminSidebarProps {
    open: boolean;
    onClose: () => void;
}

export function AdminSidebar({ open, onClose }: IAdminSidebarProps): React.ReactElement {
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

    // Close the mobile drawer on Escape and lock body scroll while open.
    useEffect(() => {
        if (!open) return;
        const onKey = (e: KeyboardEvent) => {
            if (e.key === "Escape") onClose();
        };
        document.addEventListener("keydown", onKey);
        const prevOverflow = document.body.style.overflow;
        document.body.style.overflow = "hidden";
        return () => {
            document.removeEventListener("keydown", onKey);
            document.body.style.overflow = prevOverflow;
        };
    }, [open, onClose]);

    return (
        <>
            {/* Mobile backdrop */}
            <button type="button" aria-label="Close sidebar" onClick={onClose} className={cn("fixed inset-0 z-40 cursor-default bg-black/36 backdrop-blur-[2px] transition-opacity duration-200 lg:hidden", open ? "opacity-100" : "pointer-events-none opacity-0")} />

            <aside
                aria-hidden={!open}
                className={cn(
                    "fixed top-0 bottom-0 left-0 z-50 flex w-64 flex-col border-sidebar-border border-r bg-sidebar transition-transform duration-200 ease-out",
                    "lg:sticky lg:top-0 lg:z-auto lg:h-screen lg:w-64 lg:shrink-0 lg:translate-x-0 lg:transition-none",
                    open ? "translate-x-0 shadow-[8px_0_40px_oklch(0_0_0/0.18)]" : "-translate-x-full lg:shadow-none",
                )}
            >
                <div className="flex items-center gap-2.5 px-4 py-4">
                    <img src="/logo/bust_transparent.png" alt="myrtle" className="size-7 shrink-0 object-contain" />
                    <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                        <span className="truncate font-semibold text-[15px] leading-none tracking-[-0.01em]">myrtle.moe</span>
                        <span className="inline-flex items-center gap-1 font-medium font-mono text-[10px] text-muted-foreground leading-none tracking-[0.06em]">
                            <span className="text-primary uppercase">admin</span>
                            <span className="opacity-40">·</span>
                            <span className="uppercase">v3</span>
                        </span>
                    </div>
                    <button type="button" aria-label="Close menu" onClick={onClose} className="-mr-1 inline-flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-md text-foreground hover:bg-sidebar-accent lg:hidden">
                        <XIcon className="size-4" strokeWidth={1.9} />
                    </button>
                </div>

                <nav className="flex flex-1 flex-col gap-px overflow-auto p-2">
                    <div className="px-3 pt-3.5 pb-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Manage</div>
                    {manage.map((it) => (
                        <NavRow key={it.to} item={it} active={isActive(it.to)} onNavigate={onClose} />
                    ))}
                    <div className="px-3 pt-3.5 pb-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Operate</div>
                    {operate.map((it) => (
                        <NavRow key={it.to} item={it} active={isActive(it.to)} onNavigate={onClose} />
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
        </>
    );
}
