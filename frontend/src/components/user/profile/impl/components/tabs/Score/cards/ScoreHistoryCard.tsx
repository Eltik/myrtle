import { TrendingUp } from "lucide-react";
import { useMemo } from "react";
import { CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { IScoreHistoryPoint, IUserScore } from "#/lib/api/user";
import { CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../../Stats/primitives";
import { toPct } from "../helpers";
import { gradeColor } from "../palette";

interface IProps {
    score: IUserScore;
    history: IScoreHistoryPoint[] | null | undefined;
    isLoading: boolean;
}

interface IChartPoint {
    ts: number;
    pct: number;
    rank: number | null;
}

const DATE_FMT = new Intl.DateTimeFormat("en-US", { month: "short", day: "numeric" });

/**
 * Total score over time, one point per leaderboard snapshot plus the live
 * score. Snapshots are taken by the leaderboard job, so the series thickens
 * on its own as the account ages.
 */
export function ScoreHistoryCard({ score, history, isLoading }: IProps) {
    const color = gradeColor(score.grade);

    const points = useMemo<IChartPoint[]>(() => {
        const snaps: IChartPoint[] = (history ?? []).filter((p) => p.total_score !== null).map((p) => ({ ts: new Date(p.taken_at).getTime(), pct: toPct(p.total_score), rank: p.rank_global }));
        // The live score is the freshest point - append it unless a snapshot
        // is already newer (regrades can land between snapshots).
        const liveTs = new Date(score.calculated_at).getTime();
        if (!Number.isNaN(liveTs) && (snaps.length === 0 || liveTs > snaps[snaps.length - 1].ts)) {
            snaps.push({ ts: liveTs, pct: toPct(score.total_score), rank: null });
        }
        return snaps.filter((p) => !Number.isNaN(p.ts)).sort((a, b) => a.ts - b.ts);
    }, [history, score]);

    return (
        <StatCard className="sm:col-span-2" color={color}>
            <div className={`${CARD_PADDING} flex flex-col gap-4`}>
                <div className="flex items-center justify-between">
                    <Kicker icon={TrendingUp} label="Score history" />
                    {points.length > 1 && <TrendChip points={points} />}
                </div>
                {points.length < 2 ? (
                    <p className="rounded-md border border-border/40 border-dashed bg-muted/15 px-3 py-2 text-[11px] text-muted-foreground">{isLoading ? "Loading history…" : "History builds one point per leaderboard snapshot - check back after the next one."}</p>
                ) : (
                    <div className="h-44">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={points} margin={{ top: 4, right: 8, bottom: 0, left: -18 }}>
                                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" strokeOpacity={0.4} vertical={false} />
                                <XAxis dataKey="ts" type="number" scale="time" domain={["dataMin", "dataMax"]} tickFormatter={(ts: number) => DATE_FMT.format(ts)} tick={{ fontSize: 10, fill: "var(--muted-foreground)" }} tickLine={false} axisLine={false} minTickGap={40} />
                                <YAxis domain={[(min: number) => Math.max(0, Math.floor(min - 2)), (max: number) => Math.min(100, Math.ceil(max + 2))]} tickFormatter={(v: number) => `${v}%`} tick={{ fontSize: 10, fill: "var(--muted-foreground)" }} tickLine={false} axisLine={false} width={46} />
                                <Tooltip
                                    content={({ active, payload }) => {
                                        const p = active && payload?.[0]?.payload ? (payload[0].payload as IChartPoint) : null;
                                        if (!p) return null;
                                        return (
                                            <div className="rounded-md border border-border/60 bg-popover px-2.5 py-1.5 text-[11px] shadow-md">
                                                <div className="font-mono text-muted-foreground tabular-nums">{DATE_FMT.format(p.ts)}</div>
                                                <div className="font-semibold tabular-nums" style={{ color }}>
                                                    {p.pct.toFixed(2)}%
                                                </div>
                                                {p.rank !== null && <div className="text-muted-foreground tabular-nums">#{p.rank.toLocaleString()} global</div>}
                                            </div>
                                        );
                                    }}
                                />
                                <Line type="monotone" dataKey="pct" stroke={color} strokeWidth={2} dot={{ r: 2.5, fill: color, strokeWidth: 0 }} activeDot={{ r: 4 }} isAnimationActive={false} />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                )}
            </div>
        </StatCard>
    );
}

function TrendChip({ points }: { points: IChartPoint[] }) {
    const delta = points[points.length - 1].pct - points[0].pct;
    const up = delta >= 0;
    return (
        <span className={`${KICKER_TEXT} font-mono tabular-nums ${up ? "text-emerald-500" : "text-red-400"}`}>
            {up ? "▲" : "▼"} {Math.abs(delta).toFixed(2)} pts over {points.length} points
        </span>
    );
}
