import { RotateCcw, Sparkles } from "lucide-react";
import { Button } from "#/components/ui/button";
import { TooltipProvider } from "#/components/ui/tooltip";
import type { IBoard } from "#/lib/base/board";
import { useBaseOptimizer } from "./base-context";
import { Board } from "./board/Board";
import { Headline } from "./controls/Headline";
import { PromotionToggle } from "./controls/PromotionToggle";
import { ShiftStrip } from "./controls/ShiftStrip";
import { SustainabilityBadge } from "./controls/SustainabilityBadge";
import { StatsForNerds } from "./stats/StatsForNerds";

export function BasePanel({ board }: { board: IBoard }) {
    const api = useBaseOptimizer();
    const totals = api.evaluation?.assignment;
    const power = api.evaluation?.power;
    const sustainability = api.rotation?.rotation.sustainability;
    const planned = api.proposal !== null;

    return (
        <TooltipProvider closeDelay={0} delay={350}>
            <div className="flex flex-col gap-3">
                <div className="flex flex-wrap items-end justify-between gap-3 rounded-xl border border-border bg-card px-4 py-3">
                    <div className="flex flex-wrap gap-6">
                        <Headline hint="Production rooms" label="Efficiency" value={totals ? `${Math.round(totals.total_production_efficiency)}%` : "-"} />
                        <Headline label="LMD / day" value={totals ? Math.round(totals.yield_lmd_per_day).toLocaleString() : "-"} />
                        <Headline label="EXP / day" value={totals ? Math.round(totals.yield_exp_per_day).toLocaleString() : "-"} />
                        <Headline hint={power ? `${power.generated} generated · ${power.consumed} drawn` : undefined} label="Power" value={power ? `${power.net > 0 ? "+" : ""}${power.net}` : "-"} />
                    </div>

                    <div className="flex items-center gap-2">
                        {(api.dirty || planned) && (
                            <Button onClick={api.reset} size="sm" variant="ghost">
                                <RotateCcw />
                                Reset to my base
                            </Button>
                        )}
                        <Button disabled={api.optimizing} onClick={() => api.runOptimize([])} size="sm">
                            <Sparkles />
                            {api.optimizing ? "Optimizing…" : planned ? "Re-optimize" : "Optimize"}
                        </Button>
                    </div>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="flex flex-wrap items-center gap-4">
                        <ShiftStrip />
                        <PromotionToggle />
                    </div>
                    {planned && sustainability && <SustainabilityBadge depletedCount={sustainability.depleted.length} dormOverflow={sustainability.dorm_overflow} horizonHours={sustainability.horizon_hours} verdict={sustainability.verdict} />}
                </div>

                {api.evaluationError && <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">This plan could not be scored: {api.evaluationError.message}</p>}
                {api.optimizeError && <p className="rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-[12px] text-destructive">The optimizer failed: {api.optimizeError.message}</p>}

                <div className="rounded-xl border border-border bg-card p-3">
                    <div className="relative overflow-x-auto">
                        <Board board={board} />
                    </div>
                </div>

                <StatsForNerds />
            </div>
        </TooltipProvider>
    );
}
