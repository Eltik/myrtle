import type { ISustainability } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { TEXT_META } from "../shared";
import { MiniChip, TEXT_TINY } from "./parts";
import { roomLabel } from "./roomColors";

/**
 * The rotation's morale-simulation verdict, shown above the shift poster: a
 * quiet green line when the week-long sim holds up, an amber banner naming the
 * operators that run dry (with when and where) if it doesn't. The plan is a
 * recommendation - this is the evidence it survives its own rhythm.
 */
export function SustainabilityBanner({ sim }: { sim: ISustainability }) {
    const days = Math.round(sim.horizon_hours / 24);
    if (sim.verdict === "holds_up" && sim.dorm_overflow === 0) {
        return (
            <p className={cn(TEXT_META, "flex items-center gap-1.5 text-emerald-500/85")}>
                <span aria-hidden>✓</span>
                Simulated over {days} days of the login rhythm - every operator's morale holds up.
            </p>
        );
    }
    return (
        <div className={cn(TEXT_META, "flex flex-col gap-1.5 rounded-md border border-amber-500/40 bg-amber-500/10 px-2 py-1.5 text-amber-500/90")}>
            {sim.depleted.length > 0 && (
                <>
                    <p>
                        ⚠ Simulated over {days} days, {sim.depleted.length === 1 ? "one operator runs" : `${sim.depleted.length} operators run`} out of morale mid-shift - their base skills stop working until they rest. Consider swapping them at login or upgrading dorms.
                    </p>
                    <div className="flex flex-wrap gap-1">
                        {sim.depleted.map((d) => (
                            <MiniChip
                                key={d.operator.operator_id}
                                op={d.operator}
                                suffix={<span className={cn("font-mono text-amber-500/80 tabular-nums", TEXT_TINY)}>~{Math.round(d.at_hours)}h</span>}
                                tip={
                                    <p>
                                        {d.operator.name} runs dry about {Math.round(d.at_hours)} hours in, working the {roomLabel(d.room_type)} - their drain outpaces the 24h-block rhythm.
                                    </p>
                                }
                            />
                        ))}
                    </div>
                </>
            )}
            {sim.dorm_overflow > 0 && (
                <p>
                    ⚠ At peak, {sim.dorm_overflow} resting {sim.dorm_overflow === 1 ? "operator doesn't" : "operators don't"} fit in your dormitories and recover nothing that shift.
                </p>
            )}
        </div>
    );
}
