import { memo } from "react";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IOperatorOwnershipInfo, StatMetric } from "../types";
import type { messages } from "./OwnershipBadge.messages";

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
    const t: TypedT<typeof messages> = useT("operators");
    const f = useFormatters();
    const isE2 = metric === "e2";
    if (isE2 && info.e2Pct == null) return null;

    const pct = (isE2 ? info.e2Pct : info.pct) ?? 0;
    const label = f.percent(pct);
    const exact = (pct * 100).toFixed(2);
    const title = isE2 ? t("ownership.e2.title", { e2Owners: f.number(info.e2Owners), owners: f.number(info.owners), exact }) : t("ownership.owned.title", { owners: f.number(info.owners), exact });
    const ariaLabel = isE2 ? t("ownership.e2.aria", { pct: label }) : t("ownership.owned.aria", { pct: label });

    return (
        <span role="img" className={cn("inline-flex items-center rounded-full bg-background/85 px-1.5 py-px font-mono font-semibold text-[9px] uppercase tabular-nums tracking-wider shadow-sm", className)} style={{ color }} title={title} aria-label={ariaLabel}>
            {label}
        </span>
    );
});
