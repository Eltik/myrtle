import { AlertTriangle, BatteryWarning, BedDouble } from "lucide-react";
import { cn } from "#/lib/utils";
import { isProduction, roomLabel } from "../layout";
import type { OptimizerApi } from "../use-optimizer";

/**
 * The whole-base readout: what the draft produces per day, whether it has the
 * power to run, and who runs out of morale first.
 */
export function ReportPanel({ api }: { api: OptimizerApi }) {
    const evaluation = api.evaluation;

    if (!evaluation) {
        return <p className="text-muted-foreground text-sm">{api.evaluating ? "Scoring layout…" : "No evaluation yet."}</p>;
    }

    const { assignment, power, sustain, dorms } = evaluation;
    const rooms = [...assignment.rooms].filter((r) => isProduction(r.room_type)).sort((a, b) => b.total_efficiency - a.total_efficiency);

    // The shortest endurance is the one that decides how long the base runs
    // unattended, so it leads.
    const weakest = [...sustain].filter((s) => s.lasts_hours != null).sort((a, b) => (a.lasts_hours ?? 0) - (b.lasts_hours ?? 0));

    return (
        <div className="flex flex-col gap-4">
            <dl className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                <Stat label="LMD / day" value={Math.round(assignment.yield_lmd_per_day).toLocaleString()} />
                <Stat label="EXP / day" value={Math.round(assignment.yield_exp_per_day).toLocaleString()} />
                <Stat label="Total value / day" value={Math.round(assignment.yield_total_value).toLocaleString()} />
                <Stat label="Power" value={`${power.net >= 0 ? "+" : ""}${power.net}`} warn={power.net < 0} />
            </dl>

            {power.net < 0 ? (
                <p className="flex items-center gap-1.5 rounded-md border border-destructive/40 bg-destructive/10 p-2 text-destructive text-xs">
                    <BatteryWarning className="h-3.5 w-3.5 shrink-0" />
                    This layout draws {Math.abs(power.net)} more power than it generates — the game will not let you run it.
                </p>
            ) : null}

            {dorms.count > 0 ? (
                <section>
                    <h4 className="mb-1.5 flex items-center gap-1.5 font-semibold text-muted-foreground text-xs uppercase tracking-wide">
                        <BedDouble className="h-3 w-3" /> Dormitories
                    </h4>
                    <dl className="grid grid-cols-3 gap-2">
                        <Stat label="Rest capacity" value={`${dorms.total_capacity}`} />
                        <Stat label="Recovery / h" value={dorms.recovery_per_hour.toFixed(2)} />
                        <Stat label="Total levels" value={`${dorms.total_levels}`} />
                    </dl>
                    <p className="mt-1.5 text-[10px] text-muted-foreground">Dorms produce nothing directly — they set how fast workers recover and how many can rest at once, and their levels scale every dorm-sensitive base skill.</p>
                </section>
            ) : null}

            {rooms.length > 0 ? (
                <section>
                    <h4 className="mb-1.5 font-semibold text-muted-foreground text-xs uppercase tracking-wide">Production rooms</h4>
                    <ul className="flex flex-col divide-y divide-border rounded-md border border-border">
                        {rooms.map((room) => (
                            <li key={room.slot_id}>
                                <button type="button" onClick={() => api.setSelectedSlotId(room.slot_id)} className="flex w-full items-center justify-between gap-2 p-2 text-left transition-colors hover:bg-muted">
                                    <span className="min-w-0 truncate text-foreground text-xs">{roomLabel(room.room_type, api.catalog)}</span>
                                    <span className="shrink-0 font-mono text-[11px] text-muted-foreground">
                                        {Math.round(room.total_efficiency)}% · {Math.round(room.yield_lmd_per_day + room.yield_exp_per_day).toLocaleString()}/d
                                    </span>
                                </button>
                            </li>
                        ))}
                    </ul>
                </section>
            ) : null}

            {weakest.length > 0 ? (
                <section>
                    <h4 className="mb-1.5 flex items-center gap-1.5 font-semibold text-muted-foreground text-xs uppercase tracking-wide">
                        <AlertTriangle className="h-3 w-3" /> Shortest endurance
                    </h4>
                    <ul className="flex flex-col gap-1">
                        {weakest.slice(0, 5).map((entry) => (
                            <li key={entry.operator_id} className="flex items-center justify-between gap-2 text-xs">
                                <span className="min-w-0 truncate text-foreground">{entry.name}</span>
                                <span className="shrink-0 font-mono text-[11px] text-muted-foreground">{entry.lasts_hours?.toFixed(1)}h</span>
                            </li>
                        ))}
                    </ul>
                </section>
            ) : null}
        </div>
    );
}

function Stat({ label, value, warn = false }: { label: string; value: string; warn?: boolean }) {
    return (
        <div className="rounded-md border border-border bg-background p-2">
            <dt className="text-[10px] text-muted-foreground uppercase tracking-wide">{label}</dt>
            <dd className={cn("font-mono font-semibold text-sm", warn ? "text-destructive" : "text-foreground")}>{value}</dd>
        </div>
    );
}
