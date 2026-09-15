import { Trophy } from "lucide-react";
import type { IPlayerStanding, IUserScore } from "#/lib/api/user";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../../Stats/primitives";
import { fastestSection, formatCalculatedAt, GRADE_LADDER, nextGradeStep, weightShare } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { gradeColor } from "../palette";
import type { messages } from "./OverallGradeCard.messages";

/** Section names are declared once, in `helpers.messages.ts`. */
type GradeT = TypedT<typeof messages & typeof helperMessages>;
type GradeRichT = TypedRichT<typeof messages>;

interface IOverallGradeCardProps {
    score: IUserScore;
    standing: IPlayerStanding | null | undefined;
}

export function OverallGradeCard({ score, standing }: IOverallGradeCardProps) {
    const t: GradeT = useT("user");
    const f = useFormatters();
    const color = gradeColor(score.grade);
    const totalPct = Math.max(0, Math.min(100, score.total_score * 100));
    const calcAt = formatCalculatedAt(score.calculated_at, f);

    return (
        <StatCard className="sm:col-span-2" color={color}>
            <div className={cn("grid gap-6 sm:grid-cols-[auto_1fr] sm:gap-8", CARD_PADDING)}>
                <div className="flex flex-col items-start gap-3 sm:gap-4">
                    <Kicker icon={Trophy} label={t("score.overall.title")} />
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
                            <span className={KICKER_TEXT}>{t("score.overall.composite")}</span>
                        </div>
                    </div>
                    {standing && <StandingRow standing={standing} />}
                    {calcAt && <span className="font-mono text-[10.5px] text-muted-foreground/60 tabular-nums">{t("score.overall.calculated", { date: calcAt })}</span>}
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
    const t: GradeT = useT("user");
    const f = useFormatters();
    const { player, percentile, rank_delta } = standing;
    const topPct = Math.max(percentile * 100, 0.1);
    return (
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[11px] text-muted-foreground tabular-nums">
            {player.rank_global != null && (
                <span>
                    <span className="text-foreground/90">#{f.number(player.rank_global)}</span> {t("score.global")}
                </span>
            )}
            {player.rank_server != null && (
                <span>
                    <span className="text-foreground/90">#{f.number(player.rank_server)}</span> {player.server}
                </span>
            )}
            <span>{t("score.standing.top", { pct: topPct < 1 ? topPct.toFixed(1) : Math.ceil(topPct) })}</span>
            {rank_delta != null && rank_delta !== 0 && (
                <span className={rank_delta > 0 ? "text-emerald-500" : "text-red-400"}>
                    {rank_delta > 0 ? `▲${rank_delta}` : `▼${Math.abs(rank_delta)}`} {t("score.standing.thisWeek")}
                </span>
            )}
        </div>
    );
}

/** The grade thresholds as a segmented track with the account's position. */
function GradeLadder({ total, color }: { total: number; color: string }) {
    const t: GradeT = useT("user");
    const pos = Math.max(0, Math.min(100, total * 100));
    return (
        <div className="flex flex-col gap-1.5">
            <span className={KICKER_TEXT}>{t("score.ladder.title")}</span>
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
    const t: GradeT = useT("user");
    const rt: GradeRichT = useRichT("user");
    const next = nextGradeStep(score.total_score);
    if (!next) return <p className="text-[11.5px] text-muted-foreground">{t("score.ladder.top")}</p>;
    const fastest = fastestSection(score);
    return (
        <p className="text-[11.5px] text-muted-foreground leading-relaxed">
            {rt("score.next.step", {
                points: <span className="font-semibold text-foreground/90 tabular-nums">{t("score.next.points", { points: next.pointsAway.toFixed(1) })}</span>,
                grade: (
                    <span className="font-semibold" style={{ color: gradeColor(next.grade) }}>
                        {next.grade}
                    </span>
                ),
            })}
            {fastest && <> {rt("score.next.fastest", { section: <span className="font-medium text-foreground/85">{t(fastest.labelKey)}</span>, share: weightShare(fastest.weight).toFixed(0) })}</>}.
        </p>
    );
}
