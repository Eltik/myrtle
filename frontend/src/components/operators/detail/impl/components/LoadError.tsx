import { Button } from "#/components/ui/button";
import type { messages as routeErrorMessages } from "#/components/ui/route-error.messages";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";

/**
 * A section's failed fetch, said as a failure and not as an empty state, with the query's own
 * refetch behind the button. "No voice data available" read the same whether the operator has no
 * lines or the request died, and only one of those is worth retrying.
 */
export function LoadError({ message, onRetry, className }: { message: string; onRetry: () => void; className?: string }) {
    const t: TypedT<typeof routeErrorMessages> = useT("common");
    return (
        <div className={cn("flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border bg-card px-4 py-3", className)} role="alert">
            <p className="text-foreground text-sm">{message}</p>
            <Button onClick={onRetry} size="sm" variant="outline">
                {t("routeError.retry")}
            </Button>
        </div>
    );
}
