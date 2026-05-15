import { Trophy } from "lucide-react";
import type { IUserScore } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { Bar, CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../../Stats/primitives";
import { formatCalculatedAt, formatPct, SUBSCORES, toPct } from "../helpers";
import { gradeColor } from "../palette";

interface IOverallGradeCardProps {
    score: IUserScore;
}

export function OverallGradeCard({ score }: IOverallGradeCardProps) {
    const color = gradeColor(score.grade);
    const totalPct = toPct(score.total_score);
    const calcAt = formatCalculatedAt(score.calculated_at);

    const topThree = [...SUBSCORES].sort((a, b) => (score[b.key] ?? 0) - (score[a.key] ?? 0)).slice(0, 3);

    return (
        <StatCard className="sm:col-span-2" color={color}>
            <div className={cn("grid gap-6 sm:grid-cols-[auto_1fr] sm:gap-8", CARD_PADDING)}>
                <div className="flex flex-col items-start gap-3 sm:gap-4">
                    <Kicker icon={Trophy} label="Overall Grade" />
                    <div className="flex items-baseline gap-3">
                        <span
                            className="font-bold tabular-nums leading-none"
                            style={{
                                fontSize: "clamp(4.5rem, 8vw + 1rem, 7rem)",
                                letterSpacing: "-0.06em",
                                color,
                            }}
                        >
                            {score.grade ?? "-"}
                        </span>
                        <div className="flex flex-col gap-0.5">
                            <span className="font-bold text-2xl tabular-nums leading-none" style={{ color }}>
                                {totalPct.toFixed(1)}%
                            </span>
                            <span className={KICKER_TEXT}>composite</span>
                        </div>
                    </div>
                    {calcAt && <span className="font-mono text-[10.5px] text-muted-foreground/60 tabular-nums">Calculated · {calcAt}</span>}
                </div>

                <div className="flex flex-col justify-center gap-3 sm:border-border/50 sm:border-l sm:pl-8">
                    <span className={KICKER_TEXT}>Strongest categories</span>
                    <div className="flex flex-col gap-3">
                        {topThree.map((sub) => {
                            const pct = toPct(score[sub.key]);
                            return (
                                <div className="space-y-1.5" key={sub.key}>
                                    <div className="flex items-center justify-between gap-3">
                                        <div className="flex min-w-0 items-center gap-2">
                                            <sub.icon className="h-3.5 w-3.5 shrink-0" style={{ color: sub.color }} />
                                            <span className="truncate font-semibold text-sm">{sub.label}</span>
                                        </div>
                                        <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{formatPct(score[sub.key])}</span>
                                    </div>
                                    <Bar color={sub.color} pct={pct} thin />
                                </div>
                            );
                        })}
                    </div>
                </div>
            </div>
        </StatCard>
    );
}
