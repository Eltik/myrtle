import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { roomLabel } from "#/lib/base/catalog";
import { cn } from "#/lib/utils";
import { useBaseOptimizer } from "../base-context";

const num = (value: number) => Math.round(value).toLocaleString();

function hoursLabel(hours: number): string {
    if (hours >= 48) return `${(hours / 24).toFixed(1)}d`;
    return `${hours.toFixed(1)}h`;
}

/** Wall-clock time `hours` from now, in the viewer's locale. */
function clockAfter(hours: number): string {
    const at = new Date(Date.now() + hours * 3_600_000);
    const sameDay = at.getDate() === new Date().getDate();
    const time = at.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
    return sameDay ? time : `${at.toLocaleDateString(undefined, { weekday: "short" })} ${time}`;
}

function productLabel(room: { room_type: string; formula_type: string | null }): string {
    if (room.room_type === "TRADING") return "LMD orders";
    if (room.formula_type === "F_GOLD") return "Pure Gold";
    if (room.formula_type === "F_EXP") return "Battle Records";
    return "Unconfigured";
}

/**
 * The deep dive: check-in economics (when to log in, what a lazier cadence
 * loses), per-facility output with buffer fill times, and the rotation
 * simulation's depletion events. Everything here is server-scored - this
 * panel only formats it.
 */
export function DeepDive() {
    const [open, setOpen] = useState(true);
    const [cadence, setCadence] = useState("12");
    const api = useBaseOptimizer();
    const evaluation = api.evaluation;
    if (!evaluation) return null;

    const claim = evaluation.claim;
    const rooms = evaluation.assignment.rooms.filter((r) => r.fill_hours !== undefined);
    const sustainability = api.rotation?.rotation.sustainability;
    const interval = claim?.intervals.find((i) => String(i.hours) === cadence);
    const losses = interval ? [interval.lost_lmd_per_day >= 1 ? `−${num(interval.lost_lmd_per_day)} LMD` : null, interval.lost_gold_per_day >= 0.1 ? `−${interval.lost_gold_per_day.toFixed(1)} gold` : null, interval.lost_exp_per_day >= 1 ? `−${num(interval.lost_exp_per_day)} EXP` : null].filter(Boolean) : [];

    return (
        <Collapsible onOpenChange={setOpen} open={open}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-xl border border-border bg-card px-4 py-2.5 transition-colors hover:bg-muted/40">
                <span className="font-medium text-[13px]">Deep Dive</span>
                <ChevronDown className={cn("size-4 text-muted-foreground transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>

            <CollapsibleContent>
                <div className="mt-2 flex flex-col gap-4 rounded-xl border border-border bg-card px-4 py-3">
                    {claim && (
                        <section className="flex flex-col gap-2">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">Check-in economics</h3>
                            <div className="flex flex-wrap items-center justify-between gap-3">
                                <div className="flex flex-col gap-0.5">
                                    <span className="text-[12px]">
                                        Log in before <span className="font-semibold text-foreground">{clockAfter(claim.next_full_hours)}</span> to lose nothing
                                    </span>
                                    <span className="text-[11px] text-muted-foreground">First room stalls {hoursLabel(claim.next_full_hours)} after a claim - buffers below.</span>
                                </div>
                                <div className="flex items-center gap-2">
                                    <ToggleGroup
                                        aria-label="Check-in cadence"
                                        onValueChange={(next: string[]) => {
                                            if (next[0]) setCadence(next[0]);
                                        }}
                                        value={[cadence]}
                                    >
                                        {claim.intervals.map((i) => (
                                            <ToggleGroupItem key={i.hours} size="sm" value={String(i.hours)}>
                                                {i.hours}h
                                            </ToggleGroupItem>
                                        ))}
                                    </ToggleGroup>
                                    <span className={cn("font-mono text-[11.5px] tabular-nums", losses.length > 0 ? "text-destructive" : "text-muted-foreground")}>{losses.length > 0 ? `${losses.join(" · ")} /day` : "nothing lost"}</span>
                                </div>
                            </div>
                        </section>
                    )}

                    {rooms.length > 0 && (
                        <section className="flex flex-col gap-2">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">Output by facility</h3>
                            <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
                                {rooms.map((room) => {
                                    const daily = room.room_type === "TRADING" ? `${num(room.yield_lmd_per_day)} LMD` : room.formula_type === "F_GOLD" ? `${room.yield_gold_per_day.toFixed(1)} gold` : `${num(room.yield_exp_per_day)} EXP`;
                                    const overflows = room.fill_hours !== undefined && interval !== undefined && room.fill_hours < interval.hours;
                                    return (
                                        <div className="flex flex-col gap-1 rounded-lg border border-border/60 bg-muted/10 px-3 py-2" key={room.slot_id}>
                                            <div className="flex items-baseline justify-between gap-2">
                                                <span className="min-w-0 truncate font-medium text-[12px]">{roomLabel(room.room_type, api.catalog)}</span>
                                                <span className="shrink-0 text-[10px] text-muted-foreground uppercase tracking-wide">{productLabel(room)}</span>
                                            </div>
                                            <div className="flex items-baseline justify-between gap-2">
                                                <span className="font-mono font-semibold text-[13px] tabular-nums">
                                                    {daily}
                                                    <span className="font-normal text-[10px] text-muted-foreground"> /day</span>
                                                </span>
                                                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{Math.round(room.total_efficiency)}%</span>
                                            </div>
                                            <span className={cn("text-[10.5px]", overflows ? "text-destructive" : "text-muted-foreground")}>
                                                full in {room.fill_hours === undefined ? "-" : hoursLabel(room.fill_hours)} · {room.capacity} {room.room_type === "TRADING" ? "orders" : "items"}
                                                {overflows && " - overflows at this cadence"}
                                            </span>
                                        </div>
                                    );
                                })}
                            </div>
                        </section>
                    )}

                    {sustainability && (
                        <section className="flex flex-col gap-1.5">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">Rotation events · {Math.round(sustainability.horizon_hours / 24)} simulated days</h3>
                            {sustainability.depleted.length === 0 ? (
                                <span className="text-[11.5px] text-muted-foreground">Nobody runs dry - every operator survives the recommended rotation.</span>
                            ) : (
                                <div className="flex flex-col divide-y divide-border/60">
                                    {sustainability.depleted.map((event) => (
                                        <div className="flex items-baseline gap-3 py-1 text-[11.5px]" key={`${event.operator.operator_id}:${event.at_hours}`}>
                                            <span className="w-24 shrink-0 font-mono text-muted-foreground tabular-nums">
                                                Day {Math.floor(event.at_hours / 24) + 1}, {(event.at_hours % 24).toFixed(1)}h
                                            </span>
                                            <span className="min-w-0 flex-1 truncate">
                                                {event.operator.name} <span className="text-muted-foreground">runs dry in {roomLabel(event.room_type, api.catalog)}</span>
                                            </span>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </section>
                    )}
                </div>
            </CollapsibleContent>
        </Collapsible>
    );
}
