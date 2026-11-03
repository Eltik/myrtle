import { RotateCwIcon } from "lucide-react";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./LoadFailed.messages";

interface ILoadFailedProps {
    /** The caller's own sentence for what failed, already translated. */
    message: string;
    onRetry: () => void;
}

/**
 * A failure line with a retry, for a gacha panel whose query failed. Without it a failed
 * query fell through to the panel's empty state, or to rows that silently lost their names.
 * The server's own error text is not shown: it is rarely readable to a player.
 */
export function LoadFailed({ message, onRetry }: ILoadFailedProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("gacha");
    return (
        <div role="alert" className="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-2xl border border-border bg-card px-5 py-4">
            <p className="m-0 font-sans text-muted-foreground text-sm">{message}</p>
            <Button type="button" variant="outline" size="sm" onClick={onRetry}>
                <RotateCwIcon aria-hidden="true" />
                {t("loadFailed.retry")}
            </Button>
        </div>
    );
}
