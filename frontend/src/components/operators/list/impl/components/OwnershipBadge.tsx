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
    /** Rarity hue for the dot. Decorative: the percentage text carries the value. */
    color: string;
    className?: string;
}

/** Compact pill showing a community rate for an operator; the exact figure and
 *  its denominator are in the native title here and in visible text in the
 *  hover preview, and the counts are in the accessible name.
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
    // The counts go in the name too: the title reaches mouse users only.
    const ariaLabel = isE2 ? t("ownership.e2.aria", { pct: label, e2Owners: f.number(info.e2Owners), owners: f.number(info.owners) }) : t("ownership.owned.aria", { pct: label, owners: f.number(info.owners) });

    return (
        // The text is foreground, the rarity hue only a dot: as text it measured 1.38 (5★) to 2.42 (6★)
        // on the light page.
        <span role="img" className={cn("inline-flex items-center gap-1 rounded-full bg-background/85 px-1.5 py-px font-mono font-semibold text-[9px] text-foreground uppercase tabular-nums tracking-wider shadow-sm", className)} title={title} aria-label={ariaLabel}>
            <span aria-hidden="true" className="size-1.5 shrink-0 rounded-full" style={{ backgroundColor: color }} />
            {label}
        </span>
    );
});
