import type { TierListPermissionLevel } from "#/lib/api/admin/types";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./TierLists.messages";

export type TierListsT = TypedT<typeof messages>;

/**
 * The display name of a grant level. A hook rather than a `(t, level)` helper
 * so its `t()` calls sit next to a `useT` binding, which is what the i18n
 * extractor counts as usage.
 */
export function useLevelLabel(): (level: TierListPermissionLevel) => string {
    const t: TierListsT = useT("admin");
    return (level) => {
        switch (level) {
            case "view":
                return t("tierLists.level.view");
            case "edit":
                return t("tierLists.level.edit");
            case "publish":
                return t("tierLists.level.publish");
            case "admin":
                return t("tierLists.level.admin");
        }
    };
}
