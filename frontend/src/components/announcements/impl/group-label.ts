import { useCallback } from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../Announcements.messages";

/**
 * A board group's display name. The groups come from the board itself; the two it uses today
 * (`ACTIVITY`, `SYSTEM`, 2026-10-11) have translated names, and any other shows as itself,
 * title-cased, so a new group appears without a code change.
 */
export function useGroupLabel(): (group: string) => string {
    const t: TypedT<typeof messages> = useT("announcements");
    return useCallback(
        (group: string) => {
            switch (group) {
                case "ACTIVITY":
                    return t("group.activity");
                case "SYSTEM":
                    return t("group.system");
                default:
                    return group.charAt(0).toUpperCase() + group.slice(1).toLowerCase();
            }
        },
        [t],
    );
}
