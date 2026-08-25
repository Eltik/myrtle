import { Trophy } from "lucide-react";
import type { IPlayerStanding, IUserScore } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../../Stats/primitives";
import { fastestSection, formatCalculatedAt, GRADE_LADDER, nextGradeStep, weightShare } from "../helpers";
import { gradeColor } from "../palette";

interface IOverallGradeCardProps {
    score: IUserScore;
    standing: IPlayerStanding | null | undefined;
}

export function OverallGradeCard({ score, standing }: IOverallGradeCardProps) {
    const color = gradeColor(score.grade);
    const totalPct = Math.max(0, Math.min(100, score.total_score * 100));
    const calcAt = formatCalculatedAt(score.calculated_at);

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
                    {standing && <StandingRow standing={standing} />}
                    {calcAt && <span className="font-mono text-[10.5px] text-muted-foreground/60 tabular-nums">Calculated · {calcAt}</span>}
                </div>

                <div className="flex flex-col justify-center gap-4 sm:border-border/50 sm:border-l sm:pl-8">
                    <GradeLadder total={score.total_score} color={color} />
                    <NextGradeHint score={score} />
                </div>
            </div>
        </StatCard>
    );
}

/** Global/server rank, percentile, and weekly movement in one compact row. */
function StandingRow({ standing }: { standing: IPlayerStanding }) {
    const { player, percentile, rank_delta } = standing;
    const topPct = Math.max(percentile * 100, 0.1);
    return (
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] text-muted-foreground tabular-nums">
            {player.rank_global != null && (
                <span>
                    <span className="text-foreground/90">#{player.rank_global.toLocaleString()}</span> global
                </span>
            )}
            {player.rank_server != null && (
                <span>
                    <span className="text-foreground/90">#{player.rank_server.toLocaleString()}</span> {player.server}
                </span>
            )}
            <span>top {topPct < 1 ? topPct.toFixed(1) : Math.ceil(topPct)}%</span>
            {rank_delta != null && rank_delta !== 0 && <span className={rank_delta > 0 ? "text-emerald-500" : "text-red-400"}>{rank_delta > 0 ? `▲${rank_delta}` : `▼${Math.abs(rank_delta)}`} this week</span>}
        </div>
    );
}

/** The grade thresholds as a segmented track with the account's position. */
function GradeLadder({ total, color }: { total: number; color: string }) {
    const pos = Math.max(0, Math.min(100, total * 100));
    return (
        <div className="flex flex-col gap-1.5">
            <span className={KICKER_TEXT}>Grade ladder</span>
            <div className="relative">
                <div className="flex h-2 w-full overflow-hidden rounded-full bg-muted/30">
                    {GRADE_LADDER.map((g, i) => {
                        const next = GRADE_LADDER[i + 1];
                        const width = ((next ? next.min : 1) - g.min) * 100;
                        const reached = total >= g.min;
                        return <div key={g.grade} className="h-full border-background/70 border-r last:border-r-0" style={{ width: `${width}%`, background: reached ? gradeColor(g.grade) : "transparent", opacity: reached ? 0.55 : 1 }} />;
                    })}
                </div>
                {/* Position marker */}
                <div className="absolute -top-0.75 h-3.5 w-0.75 -translate-x-1/2 rounded-full" style={{ left: `${pos}%`, background: color, boxShadow: "0 0 0 2px var(--background)" }} />
            </div>
            <div className="relative h-3.5">
                {GRADE_LADDER.map((g) => (
                    <span key={g.grade} className={cn("absolute -translate-x-1/2 font-mono text-[9px] tabular-nums", total >= g.min ? "text-foreground/75" : "text-muted-foreground/55")} style={{ left: `${g.min * 100}%` }}>
                        {g.grade}
                    </span>
                ))}
            </div>
        </div>
    );
}

/** "X points to <grade>" plus the section where a point buys the most. */
function NextGradeHint({ score }: { score: IUserScore }) {
    const next = nextGradeStep(score.total_score);
    if (!next) return <p className="text-[11.5px] text-muted-foreground">Top of the ladder - nothing left to climb.</p>;
    const fastest = fastestSection(score);
    return (
        <p className="text-[11.5px] text-muted-foreground leading-relaxed">
            <span className="font-semibold text-foreground/90 tabular-nums">{next.pointsAway.toFixed(1)} points</span> to{" "}
            <span className="font-semibold" style={{ color: gradeColor(next.grade) }}>
                {next.grade}
            </span>
            {fastest && (
                <>
                    {" "}
                    - fastest through <span className="font-medium text-foreground/85">{fastest.label}</span>, worth {weightShare(fastest.weight).toFixed(0)}% of the grade with headroom left
                </>
            )}
            .
        </p>
    );
}
