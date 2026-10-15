import type { IBadgeProps } from "#/components/ui/badge";
import type { UserRole } from "#/lib/api/admin/types";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./Users.messages";

type BadgeVariant = NonNullable<IBadgeProps["variant"]>;

/** Roles in the order the sheet's role buttons list them (design `ROLES`). */
export const ROLE_ORDER: readonly UserRole[] = ["user", "translator", "tier_list_editor", "tier_list_admin", "super_admin"];

const ROLE_VARIANT: Record<UserRole, BadgeVariant> = {
    super_admin: "default",
    tier_list_admin: "success",
    tier_list_editor: "info",
    translator: "warning",
    user: "outline",
};

export function roleVariant(role: string | null | undefined): BadgeVariant {
    return ROLE_VARIANT[role as UserRole] ?? "outline";
}

/** "View" / "Edit" / "Publish" / "Admin". Shared with Home. */
export function useLevelLabel(): (level: string) => string {
    const t: TypedT<typeof messages> = useT("admin");
    return (level) => {
        switch (level) {
            case "view":
                return t("users.level.view");
            case "publish":
                return t("users.level.publish");
            case "admin":
                return t("users.level.admin");
            default:
                return t("users.level.edit");
        }
    };
}

/** The sentence under the role buttons explaining what a role can do. */
export function useRoleDescription(): (role: string) => string {
    const t: TypedT<typeof messages> = useT("admin");
    return (role) => {
        switch (role) {
            case "super_admin":
                return t("users.role.desc.superAdmin");
            case "tier_list_admin":
                return t("users.role.desc.tierListAdmin");
            case "tier_list_editor":
                return t("users.role.desc.tierListEditor");
            case "translator":
                return t("users.role.desc.translator");
            default:
                return t("users.role.desc.user");
        }
    };
}
