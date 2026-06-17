import { memo } from "react";
import { cn, formatSharePct } from "#/lib/utils";
import type { IOperatorOwnershipInfo } from "../types";

interface IOwnershipBadgeProps {
    info: IOperatorOwnershipInfo;
    /** Accent color. The percentage text carries the meaning, so color is
     *  decorative - it never encodes the value on its own. */
    color: string;
    className?: string;
}

/** Compact pill showing the share of players who own an operator; the exact
 *  figure is available via the native title on hover. */
export const OwnershipBadge = memo(function OwnershipBadge({ info, color, className }: IOwnershipBadgeProps) {
    const pct = info.pct ?? 0;
    const label = formatSharePct(pct);
    return (
        <span
            role="img"
            className={cn("inline-flex items-center rounded-full bg-background/85 px-1.5 py-px font-mono font-semibold text-[9px] uppercase tabular-nums tracking-wider shadow-sm", className)}
            style={{ color }}
            title={`${info.owners.toLocaleString()} owners (${(pct * 100).toFixed(2)}% of imported players)`}
            aria-label={`Owned by ${label} of players`}
        >
            {label}
        </span>
    );
});
