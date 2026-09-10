import { memo } from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import type { IChoiceShare } from "../useCommunityDefaults";

interface ICommunitySharePillProps {
    /** `undefined` when this option drew no picks, or when the whole
     *  distribution is below the reporting floor. Both render nothing. */
    share: IChoiceShare | undefined;
    /** Size of the cohort the share is drawn from, for the tooltip. */
    total: number;
    /** What the denominator is, in the reader's terms. The two differ and the
     *  difference matters: skills are measured over E2 owners, modules only
     *  over owners who actually have one equipped. */
    cohort: string;
    className?: string;
}

/**
 * The share of the community that picked one skill or module, as a compact
 * percentage with the raw counts on hover.
 *
 * Renders nothing rather than "0%" when there is no figure. A zero here would
 * be read as "nobody picked this", but the actual meaning is "we cannot say" -
 * either the operator's cohort is under the floor of 50 that keeps a rare
 * operator's numbers from describing a handful of identifiable accounts, or
 * nobody has synced a roster with this option set.
 */
export const CommunitySharePill = memo(function CommunitySharePill({ share, total, cohort, className }: ICommunitySharePillProps) {
    if (!share || total <= 0) return null;

    const pct = share.share * 100;
    // Whole numbers below 10% would collapse 0.4% and 4.4% into "0%" and "4%";
    // one decimal under 10 keeps small shares distinguishable without making
    // the common case noisy.
    const label = pct < 10 ? `${pct.toFixed(1)}%` : `${Math.round(pct)}%`;
    const isTop = share.rank === 0;

    return (
        <Tooltip>
            <TooltipTrigger render={<span className={cn("inline-flex shrink-0 items-center rounded-full px-1.5 py-px font-mono font-semibold text-[10px] tabular-nums leading-none tracking-wider", isTop ? "bg-primary/15 text-primary" : "bg-secondary text-muted-foreground", className)}>{label}</span>} />
            <TooltipPopup side="top" sideOffset={6}>
                {share.users.toLocaleString()} of {total.toLocaleString()} {cohort}
                {isTop ? " - the most common choice" : ""}
            </TooltipPopup>
        </Tooltip>
    );
});
