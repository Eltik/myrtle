import { memo } from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import type { LevelBucket } from "#/types/generated/LevelBucket";

interface ILevelBreakdownProps {
    /** Always four buckets, levels 0 through 3, zero-filled by the backend. */
    buckets: LevelBucket[];
    total: number;
    /** Heading, e.g. "Community mastery". */
    title: string;
    /** Label per level, index 0 to 3. Level 0 is spelled out ("No mastery",
     *  "Not unlocked") rather than abbreviated, because "M0" reads as a rank
     *  someone attained instead of the absence of one. */
    labels: readonly [string, string, string, string];
    /** Completes "N% have ...", the headline above the rows. */
    summary: string;
    /** Names the cohort, e.g. "E2 owners". */
    cohort: string;
    /** Level the viewer is at. `null` when signed out, not an owner, or below
     *  E2, which is the only cohort these numbers count. */
    ownLevel?: number | null;
    className?: string;
}

/**
 * Where people stop investing: one row per level, each with its own bar.
 *
 * Deliberately NOT a stacked bar. Stacking four segments into one strip forces
 * the reader to match a colour to a label somewhere else, which makes the two
 * levels people actually ask about (M1 versus M2) the hardest pair to tell
 * apart, and they are routinely 3% of the cohort each. One row per level puts
 * every label beside its own magnitude, so nothing is carried by colour alone.
 *
 * Level 0 is a row like any other, because it is not missing data. It is the
 * commonest stopping point there is, and dropping it would rescale the rest to
 * read as though everyone had invested.
 */
export const LevelBreakdown = memo(function LevelBreakdown({ buckets, total, title, labels, summary, cohort, ownLevel, className }: ILevelBreakdownProps) {
    if (total <= 0 || buckets.length === 0) return null;

    // Anything past level 0 counts as invested. This is the number the strip is
    // usually asked for, and stating it stops "26% at M3" being misread as
    // "26% mastered it".
    const invested = buckets.filter((b) => b.level > 0).reduce((sum, b) => sum + b.users, 0);
    const investedPct = (invested / total) * 100;

    const fmtPct = (pct: number) => (pct > 0 && pct < 10 ? pct.toFixed(1) : String(Math.round(pct)));

    return (
        <div className={cn("rounded-lg border border-border bg-secondary/20 p-3", className)}>
            <div className="mb-2 flex flex-wrap items-baseline justify-between gap-x-3 gap-y-0.5">
                <span className="font-medium text-foreground text-xs">{title}</span>
                <span className="font-mono text-[10px] text-muted-foreground tabular-nums">
                    {total.toLocaleString()} {cohort}
                </span>
            </div>

            <p className="mb-2.5 text-muted-foreground text-xs">
                <span className="font-semibold text-foreground tabular-nums">{fmtPct(investedPct)}%</span> have {summary}
            </p>

            <div className="grid grid-cols-[minmax(64px,auto)_1fr_auto] items-center gap-x-2.5 gap-y-1.5">
                {buckets.map((bucket) => {
                    const pct = (bucket.users / total) * 100;
                    const isOwn = ownLevel != null && ownLevel === bucket.level;
                    const isNone = bucket.level === 0;
                    return (
                        <Tooltip key={bucket.level}>
                            <TooltipTrigger
                                render={
                                    <div className="col-span-3 grid grid-cols-subgrid items-center rounded-sm">
                                        <span className={cn("truncate text-[11px]", isNone ? "text-muted-foreground" : "text-foreground", isOwn && "font-semibold")}>
                                            {labels[bucket.level] ?? `L${bucket.level}`}
                                            {isOwn && <span className="ml-1 font-normal text-[9px] text-primary uppercase tracking-wider">you</span>}
                                        </span>

                                        <span className="h-1.5 w-full overflow-hidden rounded-full bg-border/70">
                                            {bucket.users > 0 && (
                                                <span
                                                    className={cn("block h-full rounded-full", isNone ? "bg-muted-foreground/45" : bucket.level === 3 ? "bg-primary" : "bg-primary/55")}
                                                    // Width is the share of the
                                                    // WHOLE cohort, the same
                                                    // quantity the percentage
                                                    // beside it names, so bar
                                                    // length and number cannot
                                                    // disagree. The floor is a
                                                    // pixel minimum rather than
                                                    // a percentage one: it keeps
                                                    // a 2% bucket legible as a
                                                    // bar instead of a dot,
                                                    // without restating the
                                                    // proportion as a lie.
                                                    style={{ width: `${pct}%`, minWidth: "0.625rem" }}
                                                />
                                            )}
                                        </span>

                                        <span className="flex items-baseline gap-1.5 font-mono text-[10px] tabular-nums">
                                            <span className={cn("w-9 text-right", isNone ? "text-muted-foreground" : "text-foreground")}>{fmtPct(pct)}%</span>
                                            <span className="w-11 text-right text-muted-foreground">{bucket.users.toLocaleString()}</span>
                                        </span>
                                    </div>
                                }
                            />
                            <TooltipPopup side="top" sideOffset={6}>
                                {bucket.users.toLocaleString()} of {total.toLocaleString()} {cohort} ({fmtPct(pct)}%)
                                {isOwn ? " - where you are" : ""}
                            </TooltipPopup>
                        </Tooltip>
                    );
                })}
            </div>
        </div>
    );
});
