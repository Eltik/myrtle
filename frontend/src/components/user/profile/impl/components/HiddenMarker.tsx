import { EyeOffIcon } from "lucide-react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./HiddenMarker.messages";

/**
 * The eye-off mark the owner sees on a header number they hid from visitors: a tooltip on
 * hover, the same words as its accessible name for a screen reader. Not focusable, like
 * the site's other informational tooltips.
 */
export function HiddenMarker({ className }: { className?: string }) {
    const t: TypedT<typeof messages> = useT("user");
    const label = t("profile.header.hiddenMarker");
    return (
        <Tooltip>
            <TooltipTrigger
                render={(triggerProps) => (
                    <span {...triggerProps} role="img" aria-label={label} className={cn("inline-flex shrink-0 items-center text-muted-foreground/80 transition-colors hover:text-foreground", className)}>
                        <EyeOffIcon aria-hidden="true" className="size-3.5" />
                    </span>
                )}
            />
            <TooltipPopup>{label}</TooltipPopup>
        </Tooltip>
    );
}
