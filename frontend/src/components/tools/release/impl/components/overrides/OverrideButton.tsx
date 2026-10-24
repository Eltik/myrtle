import { Pencil } from "lucide-react";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./OverrideButton.messages";

interface IOverrideButtonProps {
    /** The row's display name, for the accessible label. */
    name: string;
    /** The row already carries an override, so the button edits rather than adds. */
    exists: boolean;
    onClick: () => void;
}

/** The super-admin pencil beside a row's date. Callers render it only behind `canEdit`. */
export function OverrideButton({ name, exists, onClick }: IOverrideButtonProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    return (
        <Tooltip>
            <TooltipTrigger render={<Button type="button" variant="ghost" size="icon" className="text-muted-foreground" aria-label={exists ? t("release.override.button.edit", { name }) : t("release.override.button.add", { name })} onClick={onClick} />}>
                <Pencil />
            </TooltipTrigger>
            <TooltipPopup>{exists ? t("release.override.button.editTitle") : t("release.override.button.addTitle")}</TooltipPopup>
        </Tooltip>
    );
}

/**
 * A row's date block with the pencil beside it. Only an editor's rows get the wrapper; everyone
 * else's badge renders exactly as it did.
 */
export function BadgeWithOverride({ badge, button }: { badge: React.ReactNode; button: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex items-start gap-1 sm:justify-end">
            <div className="min-w-0 flex-1 sm:flex-initial">{badge}</div>
            {button}
        </div>
    );
}
