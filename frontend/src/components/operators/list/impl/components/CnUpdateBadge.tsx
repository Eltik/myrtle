import { memo, useMemo } from "react";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { summarizeChanges } from "#/lib/operators/cn-changes";
import { cn } from "#/lib/utils";
import type { OperatorChange } from "#/types/generated/OperatorChange";
import type { messages } from "./CnUpdateBadge.messages";

/** The changes as one readable list ("new module and stats"), in the reader's language. */
export function useCnChangeSummary(changes: readonly OperatorChange[]): string {
    const t: TypedT<typeof messages> = useT("operators");
    const locale = useLocale();
    return useMemo(() => {
        const labels = summarizeChanges(changes).map(({ kind, count }) => t(`cnUpdates.kind.${kind}`, { count }));
        return new Intl.ListFormat(locale, { type: "conjunction" }).format(labels);
    }, [changes, locale, t]);
}

interface ICnUpdateBadgeProps {
    changes: readonly OperatorChange[];
    className?: string;
}

/** Pill marking an operator whose CN data differs from this server's; what changed is in the native title, as the ownership pill does it. */
export const CnUpdateBadge = memo(function CnUpdateBadge({ changes, className }: ICnUpdateBadgeProps) {
    const t: TypedT<typeof messages> = useT("operators");
    const summary = useCnChangeSummary(changes);
    const title = t("cnUpdates.badge.title", { changes: summary });
    return (
        <span role="img" className={cn("inline-flex items-center rounded-full bg-background/85 px-1.5 py-px font-mono font-semibold text-[9px] text-primary uppercase tracking-wider shadow-sm ring-1 ring-primary/30", className)} title={title} aria-label={title}>
            {t("cnUpdates.badge")}
        </span>
    );
});
