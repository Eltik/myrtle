import { CircleCheck, RotateCcw, Sparkles, TriangleAlert } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import type { IBoard } from "#/lib/base/layout";
import { SegmentedTabs } from "../SegmentedTabs";
import { Board } from "./Board";
import { useOptimizerApi } from "./optimizer-context";

function Headline({ label, value, hint }: { label: string; value: string; hint?: string }) {
    return (
        <div className="flex flex-col gap-0.5">
            <span className="text-[10px] text-muted-foreground uppercase tracking-wider">{label}</span>
            <span className="font-mono font-semibold text-[15px] tabular-nums">{value}</span>
            {hint && <span className="text-[10px] text-muted-foreground">{hint}</span>}
        </div>
    );
}

export function BoardView({ board }: { board: IBoard }) {
    const api = useOptimizerApi();
    const totals = api.evaluation?.assignment;
    const power = api.evaluation?.power;
    const sustainability = api.rotation?.rotation.sustainability;

    const shiftCount = api.rotation?.shift_count ?? api.shiftCount;
    const planned = api.rotation !== null;

    return (
        <div className="flex flex-col gap-3">
            <div className="flex flex-wrap items-end justify-between gap-3 rounded-xl border border-border bg-card px-4 py-3">
                <div className="flex flex-wrap gap-6">
                    <Headline hint="Production rooms" label="Efficiency" value={totals ? `${Math.round(totals.total_production_efficiency)}%` : "-"} />
                    <Headline hint="LMD produced" label="LMD / day" value={totals ? Math.round(totals.yield_lmd_per_day).toLocaleString() : "-"} />
                    <Headline hint="EXP produced" label="EXP / day" value={totals ? Math.round(totals.yield_exp_per_day).toLocaleString() : "-"} />
                    <Headline hint={power ? `${power.generated} generated · ${power.consumed} drawn` : undefined} label="Power" value={power ? `${power.net > 0 ? "+" : ""}${power.net}` : "-"} />
                </div>

                <div className="flex items-center gap-2">
                    {planned && (
                        <Button onClick={api.discardPlan} size="sm" variant="ghost">
                            <RotateCcw />
                            Show mine
                        </Button>
                    )}
                    <Button disabled={api.optimizing} onClick={() => api.runOptimize([])} size="sm">
                        <Sparkles />
                        {api.optimizing ? "Optimizing…" : planned ? "Re-optimize" : "Optimize"}
                    </Button>
                </div>
            </div>

            {shiftCount > 0 && (
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <SegmentedTabs
                        active={api.viewShift === null ? "now" : String(api.viewShift)}
                        label="Rotation shifts"
                        onChange={(id) => api.setViewShift(id === "now" ? null : Number(id))}
                        segments={[{ id: "now", label: "Stationed now" }, ...Array.from({ length: shiftCount }, (_, i) => ({ id: String(i + 1), label: `Shift ${i + 1}` }))]}
                    />
                    {planned && sustainability && <SustainabilityBadge depletedCount={sustainability.depleted.length} dormOverflow={sustainability.dorm_overflow} horizonHours={sustainability.horizon_hours} verdict={sustainability.verdict} />}
                </div>
            )}

            {api.evaluationError && <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">This plan could not be scored: {api.evaluationError.message}</p>}
            {api.optimizeError && <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">The optimizer failed: {api.optimizeError.message}</p>}

            <div className="rounded-xl border border-border bg-card p-3">
                <div className="relative overflow-x-auto">
                    <Board board={board} />
                </div>
            </div>
        </div>
    );
}

function SustainabilityBadge({ verdict, depletedCount, dormOverflow, horizonHours }: { verdict: string; depletedCount: number; dormOverflow: number; horizonHours: number }) {
    const holds = verdict === "holds_up";
    const days = Math.round(horizonHours / 24);

    return (
        <div className="flex items-center gap-2">
            <Badge className="gap-1" variant={holds ? "secondary" : "destructive"}>
                {holds ? <CircleCheck className="h-3 w-3" /> : <TriangleAlert className="h-3 w-3" />}
                {holds ? "Holds up" : "Depletes"}
            </Badge>
            <p className="text-[11px] text-muted-foreground">
                {holds ? `Nobody runs dry over ${days} simulated day${days === 1 ? "" : "s"}.` : `${depletedCount} operator${depletedCount === 1 ? "" : "s"} hit zero morale within ${days} day${days === 1 ? "" : "s"}.`}
                {dormOverflow > 0 ? ` Dorms are ${dormOverflow} bed${dormOverflow === 1 ? "" : "s"} short at peak.` : ""}
            </p>
        </div>
    );
}
