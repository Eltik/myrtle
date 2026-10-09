import { Badge } from "#/components/ui/badge";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { EntryStatus } from "./model";
import type { messages } from "./Translations.messages";

/** Done (secondary) / Out of date (warning) / Missing (outline), as in the design's `ST` table. */
export function StatusBadge({ status, size }: { status: EntryStatus; size?: "sm" }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    switch (status) {
        case "translated":
            return (
                <Badge variant="secondary" size={size}>
                    {t("translations.status.translated")}
                </Badge>
            );
        case "stale":
            return (
                <Badge variant="warning" size={size}>
                    {t("translations.status.stale")}
                </Badge>
            );
        case "untranslated":
            return (
                <Badge variant="outline" size={size}>
                    {t("translations.status.untranslated")}
                </Badge>
            );
    }
}
