export type UserRole = "user" | "tier_list_editor" | "tier_list_admin" | "super_admin";

export type TierListPermissionLevel = "view" | "edit" | "publish" | "admin";

export function isTierListAdmin(role: string | null | undefined): boolean {
    return role === "tier_list_admin" || role === "super_admin";
}

export function isSuperAdmin(role: string | null | undefined): boolean {
    return role === "super_admin";
}

export function isAnyAdminRole(role: string | null | undefined): boolean {
    return role === "tier_list_editor" || role === "tier_list_admin" || role === "super_admin";
}
