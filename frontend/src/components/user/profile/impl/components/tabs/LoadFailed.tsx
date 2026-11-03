import { RotateCwIcon } from "lucide-react";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./LoadFailed.messages";

interface ILoadFailedProps {
    /** The tab's own sentence for what failed, already translated. */
    message: string;
    onRetry: () => void;
}

/**
 * What a profile tab shows when its query fails, in place of the empty or loading state that a
 * failure used to fall through to. Same shape as the Showcase tab's failure card.
 */
export function LoadFailed({ message, onRetry }: ILoadFailedProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("user");
    return (
        <div role="alert" className="flex flex-col items-center gap-3 rounded-2xl border border-border bg-card px-8 py-12 text-center">
            <p className="m-0 text-muted-foreground text-sm">{message}</p>
            <Button type="button" variant="outline" size="sm" onClick={onRetry}>
                <RotateCwIcon aria-hidden="true" />
                {t("profile.loadFailed.retry")}
            </Button>
        </div>
    );
}
