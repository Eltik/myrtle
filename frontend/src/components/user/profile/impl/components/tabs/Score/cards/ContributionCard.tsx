import { PieChart } from "lucide-react";
import type { IUserScore } from "#/lib/api/user";
import { CARD_PADDING, Kicker, StatCard } from "../../Stats/primitives";
import { SUBSCORES, toPct, weightShare } from "../helpers";
import { gradeColor } from "../palette";

interface IProps {
    score: IUserScore;
}

/**
 * Where the composite score comes from: one bar whose segments are each
 * section's maximum share of the grade (the weights made visible), filled by
 * how much of that share the account has earned. The legend reads
 * "earned / potential" per section, so the biggest untapped pools stand out.
 */
export function ContributionCard({ score }: IProps) {
    const rows = SUBSCORES.map((sub) => {
        const share = weightShare(sub.weight);
        const frac = toPct(score[sub.key]) / 100;
        return { sub, share, earned: share * frac };
    });

    return (
        <StatCard className="sm:col-span-2" color={gradeColor(score.grade)}>
            <div className={`${CARD_PADDING} flex flex-col gap-4`}>
                <Kicker icon={PieChart} label="Grade composition" />
                <div className="flex h-3 w-full overflow-hidden rounded-full border border-border/40 bg-muted/25">
                    {rows.map(({ sub, share, earned }) => (
                        <div key={sub.key} className="relative h-full border-background/60 border-r last:border-r-0" style={{ width: `${share}%` }} title={`${sub.label}: ${earned.toFixed(1)} of ${share.toFixed(1)} pts`}>
                            <div className="absolute inset-y-0 left-0" style={{ width: `${(earned / share) * 100}%`, background: sub.color, opacity: 0.85 }} />
                        </div>
                    ))}
                </div>
                <div className="grid grid-cols-2 gap-x-4 gap-y-1.5 sm:grid-cols-3">
                    {rows.map(({ sub, share, earned }) => (
                        <div key={sub.key} className="flex items-center justify-between gap-2 text-[11px]">
                            <span className="flex min-w-0 items-center gap-1.5">
                                <span className="size-2 shrink-0 rounded-full" style={{ background: sub.color }} />
                                <span className="truncate text-muted-foreground">{sub.label}</span>
                            </span>
                            <span className="font-mono text-foreground/85 tabular-nums">
                                {earned.toFixed(1)}
                                <span className="text-muted-foreground/60"> / {share.toFixed(1)}</span>
                            </span>
                        </div>
                    ))}
                </div>
            </div>
        </StatCard>
    );
}
