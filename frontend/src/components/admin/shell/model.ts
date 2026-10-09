/**
 * The admin shell's pure model: sections, their URLs, the role -> access
 * rules (spec section 0) and the nav-count arithmetic. No React, no API
 * clients, so routes and tests can import it without pulling in the session.
 */
import type { IBadgeProps } from "#/components/ui/badge";
import { isAnyAdminRole, isSuperAdmin, isTierListAdmin, type TierListPermissionLevel } from "#/lib/api/admin/types";

export const ADMIN_SECTIONS = ["home", "users", "tierlists", "notes", "translations", "system"] as const;
export type AdminSectionId = (typeof ADMIN_SECTIONS)[number];

export type SectionAccess = Record<AdminSectionId, boolean>;

const LEVEL_RANK: Record<TierListPermissionLevel, number> = { view: 0, edit: 1, publish: 2, admin: 3 };

/** `view < edit < publish < admin`; an unknown level ranks below `view`. */
export function levelRank(level: string): number {
    return LEVEL_RANK[level as TierListPermissionLevel] ?? -1;
}

/** `level` is `min` or better. */
export function atLeast(level: string, min: TierListPermissionLevel): boolean {
    return levelRank(level) >= levelRank(min);
}

/** Every grant level, lowest first. */
export const GRANT_LEVELS = ["view", "edit", "publish", "admin"] as const satisfies readonly TierListPermissionLevel[];

const LEVEL_VARIANT: Record<TierListPermissionLevel, NonNullable<IBadgeProps["variant"]>> = { admin: "default", publish: "success", edit: "info", view: "outline" };

/** The badge colour of a grant level; an unknown level reads as `fallback`. */
export function levelVariant(level: string, fallback: NonNullable<IBadgeProps["variant"]> = "outline"): NonNullable<IBadgeProps["variant"]> {
    return LEVEL_VARIANT[level as TierListPermissionLevel] ?? fallback;
}

export interface ILocaleGrant {
    code: string;
    level: TierListPermissionLevel;
}

/**
 * The caller's own locale grants, one per locale at the highest level held
 * there; rows with an unknown level are dropped. Sorted by locale code.
 */
export function bestLocaleGrants(rows: readonly { locale: string; permission: string }[]): ILocaleGrant[] {
    const best = new Map<string, TierListPermissionLevel>();
    for (const row of rows) {
        if (levelRank(row.permission) < 0) continue;
        const held = best.get(row.locale);
        if (held === undefined || levelRank(row.permission) > levelRank(held)) best.set(row.locale, row.permission as TierListPermissionLevel);
    }
    return [...best.entries()].sort(([a], [b]) => a.localeCompare(b)).map(([code, level]) => ({ code, level }));
}

/**
 * Which sections a user may open, from their role and how many grants they
 * hold (design `can`). Super-admins reach Translations without waiting for
 * the locale list: they hold every locale at `admin` by definition.
 */
export function computeSectionAccess(role: string | null | undefined, tierListGrantCount: number, localeGrantCount: number): SectionAccess {
    const staff = isTierListAdmin(role);
    return {
        home: true,
        users: staff,
        system: staff,
        notes: staff || role === "tier_list_editor",
        tierlists: staff || tierListGrantCount > 0,
        translations: isSuperAdmin(role) || localeGrantCount > 0,
    };
}

/**
 * The route-level gate, from the role alone (grants are not in the router
 * context). `false` = the role can never reach the section, redirect home.
 * `true` = it may, or may once its grants load: the section component handles
 * an editor or grant holder who turns out to hold none.
 */
export function roleMayReachSection(section: AdminSectionId, role: string | null | undefined): boolean {
    switch (section) {
        case "home":
            return true;
        case "users":
        case "system":
            return isTierListAdmin(role);
        case "notes":
            return isAnyAdminRole(role);
        case "tierlists":
            // Like translations: a per-list grant opens it for any role, and
            // the section sends a user without grants home itself.
            return true;
        case "translations":
            // A locale grant, not a role, opens it, and any role may hold
            // one. The section sends a user without grants home itself.
            return true;
    }
}

export type AdminSectionPath = "/admin" | "/admin/users" | "/admin/tier-lists" | "/admin/operator-notes" | "/admin/translations" | "/admin/system";

export const SECTION_PATHS: Record<AdminSectionId, AdminSectionPath> = {
    home: "/admin",
    users: "/admin/users",
    tierlists: "/admin/tier-lists",
    notes: "/admin/operator-notes",
    translations: "/admin/translations",
    system: "/admin/system",
};

/** The section a pathname belongs to; anything unrecognised under `/admin` is Home. */
export function sectionForPath(pathname: string): AdminSectionId {
    const path = pathname.replace(/\/+$/, "");
    for (const [id, to] of Object.entries(SECTION_PATHS) as [AdminSectionId, AdminSectionPath][]) {
        if (id !== "home" && (path === to || path.startsWith(`${to}/`))) return id;
    }
    return "home";
}

export type ChangeKind = "added" | "cleared" | "changed";

/** Added when there was nothing before, cleared when there is nothing after, else changed. Whitespace counts as nothing. */
export function changeKind(oldValue: string | null, newValue: string | null): ChangeKind {
    const hadOld = !!oldValue?.trim();
    const hasNew = !!newValue?.trim();
    if (!hadOld && hasNew) return "added";
    if (hadOld && !hasNew) return "cleared";
    return "changed";
}

export interface ILocaleProgress {
    locale: string;
    total: number;
    translated: number;
    stale: number;
}

type ProgressCounts = Pick<ILocaleProgress, "total" | "translated" | "stale">;

/** Untranslated strings in a locale: everything not yet translated. */
export function missingCount(p: ProgressCounts): number {
    return Math.max(0, p.total - p.translated);
}

/** Share of a locale that is translated and current, floored to one decimal (design `pct`). An empty catalog counts as complete. */
export function currentPercent(p: ProgressCounts): number {
    if (p.total <= 0) return 100;
    return Math.max(0, Math.floor(((p.translated - p.stale) / p.total) * 1000) / 10);
}

/**
 * Stale + untranslated strings across the locales this user can edit (design
 * `trTodo`). A super-admin's count covers public locales only; a translator's
 * covers every locale they hold `edit` or better on, public or not.
 */
export function translationsTodo(access: { isSuper: boolean; myLoc: readonly ILocaleGrant[] }, progress: readonly ILocaleProgress[], publicLocales: ReadonlySet<string>): number {
    let todo = 0;
    for (const grant of access.myLoc) {
        if (!atLeast(grant.level, "edit")) continue;
        if (access.isSuper && !publicLocales.has(grant.code)) continue;
        const p = progress.find((row) => row.locale === grant.code);
        if (!p) continue;
        todo += p.stale + missingCount(p);
    }
    return todo;
}
