import { CircleAlertIcon, RotateCwIcon } from "lucide-react";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./route-error.messages";

export interface ILoadFailedProps {
    /** The caller's own sentence for what failed ("Couldn't load <thing>."), already translated. Never the server's text. */
    message: React.ReactNode;
    /** The failed query's refetch. Without it the line is a notice with no button. */
    onRetry?: () => void;
    /**
     * `inline`: one row with no panel chrome, for a failure inside a card or section that already
     * draws its own border. `block`: a centred bordered panel standing in for a whole tab or area.
     */
    layout?: "inline" | "block";
    className?: string;
}

/**
 * The one shape for a failed fetch, said as a failure and not as an empty state, with the query's
 * own refetch behind the button. Five copies of this grew apart before it was shared.
 */
export function LoadFailed({ message, onRetry, layout = "block", className }: ILoadFailedProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("common");
    const retry = onRetry ? (
        <Button type="button" variant="outline" size="sm" onClick={onRetry}>
            <RotateCwIcon aria-hidden="true" />
            {t("routeError.retry")}
        </Button>
    ) : null;

    if (layout === "inline") {
        return (
            <div role="alert" className={cn("flex flex-wrap items-center gap-x-3 gap-y-2", className)}>
                <CircleAlertIcon aria-hidden="true" className="size-4 shrink-0 text-muted-foreground" />
                <p className="m-0 min-w-0 flex-1 text-foreground text-sm">{message}</p>
                {retry}
            </div>
        );
    }

    return (
        <div role="alert" className={cn("flex flex-col items-center gap-3 rounded-2xl border border-border bg-card px-6 py-8 text-center", className)}>
            <CircleAlertIcon aria-hidden="true" className="size-5 text-muted-foreground" />
            <p className="m-0 max-w-prose text-foreground text-sm">{message}</p>
            {retry}
        </div>
    );
}
