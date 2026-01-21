/**
 * Role-based access control for the admin panel
 */

export type AdminRole = "tier_list_editor" | "tier_list_admin" | "super_admin";

export const ADMIN_ROLES: AdminRole[] = ["tier_list_editor", "tier_list_admin", "super_admin"];

/**
 * Role hierarchy - higher number = more permissions
 */
export const ROLE_HIERARCHY: Record<AdminRole, number> = {
    tier_list_editor: 1,
    tier_list_admin: 2,
    super_admin: 3,
};

/**
 * Check if a string is a valid admin role
 */
export function isAdminRole(role: string | undefined): role is AdminRole {
    return role !== undefined && ADMIN_ROLES.includes(role as AdminRole);
}

/**
 * Check if user has at least the minimum required role
 */
export function hasMinRole(userRole: AdminRole | null, minRole: AdminRole): boolean {
    if (!userRole) return false;
    return ROLE_HIERARCHY[userRole] >= ROLE_HIERARCHY[minRole];
}

/**
 * Check if user can view/manage users (super_admin only)
 */
export function canManageUsers(role: AdminRole | null): boolean {
    return role === "super_admin";
}

/**
 * Check if user can create tier lists (any admin role)
 */
export function canCreateTierList(role: AdminRole | null): boolean {
    return hasMinRole(role, "tier_list_editor");
}

/**
 * Check if user can delete tier lists (tier_list_admin or higher)
 */
export function canDeleteTierList(role: AdminRole | null): boolean {
    return hasMinRole(role, "tier_list_admin");
}

/**
 * Check if user can toggle tier list active status (tier_list_admin or higher)
 */
export function canToggleTierListActive(role: AdminRole | null): boolean {
    return hasMinRole(role, "tier_list_admin");
}

/**
 * Check if user can moderate community tier lists (tier_list_admin or higher)
 */
export function canModerateTierList(role: AdminRole | null): boolean {
    return hasMinRole(role, "tier_list_admin");
}

/**
 * Check if user can review tier list reports (any admin role)
 */
export function canReviewReports(role: AdminRole | null): boolean {
    return hasMinRole(role, "tier_list_editor");
}
