import { useQuery } from "@tanstack/react-query";
import { Link, useRouterState } from "@tanstack/react-router";
import { ActivityIcon, ChevronRightIcon, FileTextIcon, LanguagesIcon, LayoutDashboardIcon, ListOrderedIcon, type LucideIcon, SettingsIcon, ShieldIcon, UsersIcon, XIcon, ZapIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { useAuth } from "#/hooks/use-auth";
import { adminStatsQueryOptions, localesQueryOptions, translationProgressQueryOptions } from "#/lib/api/admin";
import { operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { browseTierListsQueryOptions } from "#/lib/api/tier-lists";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getSecretaryAvatarURL } from "#/lib/utils";
import type { messages } from "./AdminSidebar.messages";

type NavLabelKey = keyof typeof messages & string;

interface INavItem {
    to: string;
    labelKey: NavLabelKey;
    icon: LucideIcon;
    count?: number;
}

interface IAvatarUser {
    nickname: string | null;
    secretary: string | null;
    secretary_skin_id: string | null;
}

function UserBadgeAvatar({ user, size = 26 }: { user: IAvatarUser | null | undefined; size?: number }): React.ReactElement {
    const [failed, setFailed] = useState(false);
    const url = user ? getSecretaryAvatarURL({ secretary: user.secretary, secretary_skin_id: user.secretary_skin_id }) : null;
    return (
        <span className="relative inline-block shrink-0 overflow-hidden rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]" style={{ width: size, height: size }}>
            {url && !failed ? (
                <img src={url} alt="" loading="lazy" decoding="async" className="absolute inset-0 size-full object-cover" onError={() => setFailed(true)} />
            ) : (
                <span className="absolute inset-0 flex items-center justify-center font-bold text-[11px] text-white/90">{user?.nickname?.[0]?.toUpperCase() ?? "?"}</span>
            )}
        </span>
    );
}

function NavRow({ item, active, onNavigate }: { item: INavItem; active: boolean; onNavigate?: () => void }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    const f = useFormatters();
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
            <span className="truncate">{t(item.labelKey)}</span>
            {item.count != null ? <span className={`ml-auto font-mono text-[10.5px] ${active ? "text-primary" : "text-muted-foreground"}`}>{f.number(item.count)}</span> : null}
        </Link>
    );
}

interface IAdminSidebarProps {
    open: boolean;
    onClose: () => void;
}

export function AdminSidebar({ open, onClose }: IAdminSidebarProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    const { user, isAuthenticated } = useAuth();
    const pathname = useRouterState({ select: (s) => s.location.pathname });
    const isActive = (to: string) => (to === "/admin" ? pathname === "/admin" : pathname.startsWith(to));

    // Live counts
    const statsQuery = useQuery(adminStatsQueryOptions(isAuthenticated));
    const tierListsQuery = useQuery(browseTierListsQueryOptions(useGamedataServer()));
    const notesQuery = useQuery(operatorNotesListQueryOptions());
    const progressQuery = useQuery(translationProgressQueryOptions(isAuthenticated));
    const localesQuery = useQuery(localesQueryOptions(isAuthenticated));

    const totalUsers = statsQuery.data ? (statsQuery.data.usersByRole.user ?? 0) + (statsQuery.data.usersByRole.tierListEditor ?? 0) + (statsQuery.data.usersByRole.tierListAdmin ?? 0) + (statsQuery.data.usersByRole.translator ?? 0) + (statsQuery.data.usersByRole.superAdmin ?? 0) : undefined;
    const officialCount = tierListsQuery.data ? tierListsQuery.data.filter((tl) => tl.listType === "official").length : undefined;
    const tierListCount = tierListsQuery.data?.length;
    const notesCount = notesQuery.data?.length;

    // The nav count reads as a work queue rather than a catalog size: untranslated
    // plus stale. `/admin/i18n/progress` comes back ordered by locale code, so the
    // primary locale is the first row that isn't the source - the source is where
    // the keys come from and reports complete by definition.
    const sourceLocale = localesQuery.data?.find((l) => l.is_source)?.code ?? null;
    const primaryProgress = progressQuery.data?.find((p) => p.locale !== sourceLocale) ?? progressQuery.data?.[0];
    const translationBacklog = primaryProgress ? primaryProgress.total - primaryProgress.translated + primaryProgress.stale : undefined;

    const manage: INavItem[] = [
        { to: "/admin", labelKey: "sidebar.nav.dashboard", icon: LayoutDashboardIcon },
        { to: "/admin/users", labelKey: "sidebar.nav.users", icon: UsersIcon, count: totalUsers },
        { to: "/admin/official-tier-lists", labelKey: "sidebar.nav.officialTierLists", icon: ListOrderedIcon, count: officialCount },
        { to: "/admin/permissions", labelKey: "sidebar.nav.tierLists", icon: ShieldIcon, count: tierListCount },
        { to: "/admin/operator-notes", labelKey: "sidebar.nav.operatorNotes", icon: FileTextIcon, count: notesCount },
        { to: "/admin/translations", labelKey: "sidebar.nav.translations", icon: LanguagesIcon, count: translationBacklog },
    ];

    const operate: INavItem[] = [
        { to: "/admin/health", labelKey: "sidebar.nav.health", icon: ZapIcon },
        { to: "/admin/audit", labelKey: "sidebar.nav.audit", icon: ActivityIcon },
        { to: "/admin/settings", labelKey: "sidebar.nav.settings", icon: SettingsIcon },
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
            <button type="button" aria-label={t("sidebar.closeBackdrop")} onClick={onClose} className={cn("fixed inset-0 z-40 cursor-default bg-black/36 backdrop-blur-[2px] transition-opacity duration-200 lg:hidden", open ? "opacity-100" : "pointer-events-none opacity-0")} />

            <aside
                aria-hidden={!open}
                className={cn(
                    "fixed top-0 bottom-0 left-0 z-50 flex w-64 flex-col border-sidebar-border border-r bg-sidebar transition-transform duration-200 ease-out",
                    "lg:sticky lg:top-0 lg:z-auto lg:h-screen lg:w-64 lg:shrink-0 lg:translate-x-0 lg:transition-none",
                    open ? "translate-x-0 shadow-[8px_0_40px_oklch(0_0_0/0.18)]" : "-translate-x-full lg:shadow-none",
                )}
            >
                <div className="flex items-center gap-2.5 px-4 py-4">
                    <Link to="/admin" onClick={onClose} aria-label={t("sidebar.home")} className="-m-1 flex min-w-0 flex-1 items-center gap-2.5 rounded-lg p-1 transition-colors hover:bg-sidebar-accent">
                        <img src="/logo/bust_transparent.png" alt="" className="size-7 shrink-0 object-contain" />
                        <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                            <span className="truncate font-semibold text-[15px] leading-none tracking-[-0.01em]">myrtle.moe</span>
                            <span className="inline-flex items-center gap-1 font-medium font-mono text-[10px] text-muted-foreground leading-none tracking-[0.06em]">
                                <span className="text-primary uppercase">{t("sidebar.badge")}</span>
                                <span className="opacity-40">·</span>
                                <span className="uppercase">v3</span>
                            </span>
                        </div>
                    </Link>
                    <button type="button" aria-label={t("sidebar.closeMenu")} onClick={onClose} className="-mr-1 inline-flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-md text-foreground hover:bg-sidebar-accent lg:hidden">
                        <XIcon className="size-4" strokeWidth={1.9} />
                    </button>
                </div>

                <nav className="flex flex-1 flex-col gap-px overflow-auto p-2">
                    <div className="px-3 pt-3.5 pb-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{t("sidebar.group.manage")}</div>
                    {manage.map((it) => (
                        <NavRow key={it.to} item={it} active={isActive(it.to)} onNavigate={onClose} />
                    ))}
                    <div className="px-3 pt-3.5 pb-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{t("sidebar.group.operate")}</div>
                    {operate.map((it) => (
                        <NavRow key={it.to} item={it} active={isActive(it.to)} onNavigate={onClose} />
                    ))}
                </nav>

                <div className="border-sidebar-border border-t p-2.5">
                    <Link
                        to={user?.uid ? "/user/$id" : "/settings"}
                        params={user?.uid ? { id: user.uid } : undefined}
                        onClick={onClose}
                        aria-label={t("sidebar.profile")}
                        className="group flex items-center gap-2.5 rounded-[10px] border border-sidebar-border bg-card px-2.5 py-2 transition-colors hover:bg-sidebar-accent"
                    >
                        <UserBadgeAvatar user={user} size={26} />
                        <div className="flex min-w-0 flex-1 flex-col gap-px">
                            <span className="truncate font-semibold text-[12px] leading-tight">{user?.nickname ?? t("sidebar.guest")}</span>
                            <span className="truncate font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.06em]">{user?.role ?? "-"}</span>
                        </div>
                        <ChevronRightIcon className="size-3.5 opacity-60 transition-transform group-hover:translate-x-0.5" strokeWidth={1.9} />
                    </Link>
                    <div className="flex items-center justify-end pt-1.5">
                        <Link to="/" className="text-[11px] text-muted-foreground hover:text-foreground" onClick={onClose}>
                            {t("sidebar.backToSite")}
                        </Link>
                    </div>
                </div>
            </aside>
        </>
    );
}
