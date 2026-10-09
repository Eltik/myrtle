export type UserRole = "user" | "tier_list_editor" | "tier_list_admin" | "translator" | "super_admin";

export type TierListPermissionLevel = "view" | "edit" | "publish" | "admin";

/**
 * The People tab's role filter, mirroring the backend's: `staff` is
 * super-admins, tier list admins and tier list editors; `translators` is the
 * translator role; `players` is plain `user`. In the tab's order.
 */
export const ADMIN_USER_ROLE_FILTERS = ["all", "staff", "translators", "players"] as const;
export type AdminUserRoleFilter = (typeof ADMIN_USER_ROLE_FILTERS)[number];

/** Server codes `GET /admin/users?server=` accepts, in the People tab's order. */
export const ADMIN_USER_SERVERS = ["en", "jp", "kr", "cn", "bili", "tw"] as const;
export type AdminUserServer = (typeof ADMIN_USER_SERVERS)[number];

export function isTierListAdmin(role: string | null | undefined): boolean {
    return role === "tier_list_admin" || role === "super_admin";
}

export function isSuperAdmin(role: string | null | undefined): boolean {
    return role === "super_admin";
}

/**
 * Gates tier-list authoring. Deliberately excludes `translator`: a translator
 * reaches the admin panel but must never inherit tier-list write access.
 * Mirrors the backend's `is_any_admin_role()`.
 */
export function isAnyAdminRole(role: string | null | undefined): boolean {
    return role === "tier_list_editor" || role === "tier_list_admin" || role === "super_admin";
}

/**
 * May load the admin panel at all - any role other than plain `user`. Mirrors
 * the backend's `can_access_admin_panel()`; this is the gate the `/admin`
 * route tree uses, not `isAnyAdminRole`.
 */
export function canAccessAdminPanel(role: string | null | undefined): boolean {
    return role != null && role !== "user";
}
