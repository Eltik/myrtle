import { memo } from "react";
import { cn, formatSharePct } from "#/lib/utils";
import type { IOperatorOwnershipInfo, StatMetric } from "../types";

interface IOwnershipBadgeProps {
    info: IOperatorOwnershipInfo;
    /** Which community number to show. `owned` is the share of all sharing
     *  players; `e2` is the share OF OWNERS who promoted to E2. */
    metric: StatMetric;
    /** Accent color. The percentage text carries the meaning, so color is
     *  decorative - it never encodes the value on its own. */
    color: string;
    className?: string;
}

/** Compact pill showing a community rate for an operator; the exact figure and
 *  its denominator are available via the native title on hover.
 *
 *  Renders nothing on the `e2` metric when the operator has no conversion rate
 *  to report - nobody owns them, or they cannot be promoted to E2 at all. An
 *  empty slot says "not applicable"; a 0% pill would claim players looked at
 *  them and declined. */
export const OwnershipBadge = memo(function OwnershipBadge({ info, metric, color, className }: IOwnershipBadgeProps) {
    const isE2 = metric === "e2";
    if (isE2 && info.e2Pct == null) return null;

    const pct = (isE2 ? info.e2Pct : info.pct) ?? 0;
    const label = formatSharePct(pct);
    const title = isE2 ? `${info.e2Owners.toLocaleString()} of ${info.owners.toLocaleString()} owners at E2 (${(pct * 100).toFixed(2)}%)` : `${info.owners.toLocaleString()} owners (${(pct * 100).toFixed(2)}% of imported players)`;
    const ariaLabel = isE2 ? `E2'd by ${label} of owners` : `Owned by ${label} of players`;

    return (
        <span role="img" className={cn("inline-flex items-center rounded-full bg-background/85 px-1.5 py-px font-mono font-semibold text-[9px] uppercase tabular-nums tracking-wider shadow-sm", className)} style={{ color }} title={title} aria-label={ariaLabel}>
            {label}
        </span>
    );
});
