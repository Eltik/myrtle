import { Camera, ChevronDown, Copy, X } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import type { IEvaluateResponse } from "#/lib/api/base";
import { roomLabel } from "#/lib/base/catalog";
import { cn } from "#/lib/utils";
import { useBaseOptimizer } from "../base-context";
import { TileTooltip } from "../board/tile/components/TileTooltip";

const num = (value: number) => Math.round(value).toLocaleString();

/** The comparable numbers of one evaluated draft, for the snapshot diff. */
interface ISnapshot {
    efficiency: number;
    lmd: number;
    exp: number;
    gold: number;
    tradingCapacity: number;
    powerNet: number;
    dormRecovery: number;
    nextFullHours: number | null;
}

function snapshotOf(evaluation: IEvaluateResponse): ISnapshot {
    const rooms = evaluation.assignment.rooms;
    return {
        efficiency: evaluation.assignment.total_production_efficiency,
        lmd: evaluation.assignment.yield_lmd_per_day,
        exp: evaluation.assignment.yield_exp_per_day,
        gold: rooms.reduce((sum, r) => sum + r.yield_gold_per_day, 0),
        tradingCapacity: rooms.filter((r) => r.room_type === "TRADING").reduce((sum, r) => sum + (r.capacity ?? 0), 0),
        powerNet: evaluation.power.net,
        dormRecovery: evaluation.dorms.recovery_per_hour,
        nextFullHours: evaluation.claim?.next_full_hours ?? null,
    };
}

function DiffRow({ label, from, to, format, betterLow }: { label: string; from: number; to: number; format: (v: number) => string; betterLow?: boolean }) {
    const delta = to - from;
    const meaningful = Math.abs(delta) > (Math.abs(from) + Math.abs(to)) * 1e-6 + 1e-9;
    const improved = betterLow ? delta < 0 : delta > 0;
    return (
        <div className="flex items-baseline justify-between gap-4 py-1">
            <span className="text-[11.5px] text-muted-foreground">{label}</span>
            <span className="font-mono text-[11.5px] tabular-nums">
                {format(from)} &rarr; {format(to)}{" "}
                {meaningful ? (
                    <span className={cn("font-semibold", improved ? "text-emerald-400" : "text-destructive")}>
                        {delta > 0 ? "+" : "−"}
                        {format(Math.abs(delta))}
                    </span>
                ) : (
                    <span className="text-muted-foreground">no change</span>
                )}
            </span>
        </div>
    );
}

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
    const [period, setPeriod] = useState<"day" | "week" | "month" | "year">("day");
    const periodMult = { day: 1, week: 7, month: 30, year: 365 }[period];
    const periodLabel = { day: "/day", week: "/week", month: "/month", year: "/year" }[period];
    const [snapshot, setSnapshot] = useState<ISnapshot | null>(null);
    const api = useBaseOptimizer();
    const evaluation = api.evaluation;
    if (!evaluation) return null;
    const live = snapshotOf(evaluation);

    const claim = evaluation.claim;
    // Who runs dry soonest FROM THEIR REAL CURRENT BAR (as of the last sync).
    const atRisk = evaluation.sustain
        .filter((e) => e.lasts_hours !== null && e.morale !== undefined)
        .sort((a, b) => (a.lasts_hours ?? 0) - (b.lasts_hours ?? 0))
        .slice(0, 3);
    const rooms = evaluation.assignment.rooms.filter((r) => r.fill_hours !== undefined);
    const sustainability = api.rotation?.rotation.sustainability;
    const interval = claim?.intervals.find((i) => String(i.hours) === cadence);
    const losses = interval
        ? [interval.lost_lmd_per_day >= 1 ? `−${num(interval.lost_lmd_per_day * periodMult)} LMD` : null, interval.lost_gold_per_day >= 0.1 ? `−${(interval.lost_gold_per_day * periodMult).toFixed(1)} gold` : null, interval.lost_exp_per_day >= 1 ? `−${num(interval.lost_exp_per_day * periodMult)} EXP` : null].filter(
              Boolean,
          )
        : [];

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
                                    <span className="text-[11px] text-muted-foreground">
                                        First room stalls {hoursLabel(claim.next_full_hours)} after a claim - buffers below.{" "}
                                        <TileTooltip label={<span className="block max-w-64">Order sizes are modeled conservatively (small orders, fast turnover), so real deadlines can be later than shown - never earlier. Trading posts are assumed gold-supplied.</span>}>
                                            <span className="cursor-help underline decoration-dotted underline-offset-2">conservative</span>
                                        </TileTooltip>
                                    </span>
                                    {atRisk.length > 0 && (
                                        <span className="text-[11px]">
                                            <span className="text-muted-foreground">At risk from their current bar{evaluation.morale_synced_hours_ago !== undefined && ` (projected from your sync ${hoursLabel(evaluation.morale_synced_hours_ago)} ago)`}: </span>
                                            {atRisk.map((e, i) => (
                                                <span key={e.operator_id}>
                                                    {i > 0 && <span className="text-muted-foreground"> · </span>}
                                                    {e.name} <span className={cn("font-mono tabular-nums", (e.lasts_hours ?? 99) < 12 ? "text-destructive" : "text-muted-foreground")}>{hoursLabel(e.lasts_hours ?? 0)}</span>
                                                </span>
                                            ))}
                                        </span>
                                    )}
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
                                    <input
                                        aria-label="Custom check-in cadence in hours"
                                        className="w-14 rounded-md border border-border bg-transparent px-1.5 py-0.5 text-center font-mono text-[11px] tabular-nums placeholder:text-muted-foreground/50"
                                        inputMode="numeric"
                                        onChange={(e) => {
                                            const v = Number(e.target.value);
                                            if (Number.isFinite(v) && v >= 1 && v <= 168) {
                                                api.setClaimIntervalHours(v);
                                                setCadence(String(v));
                                            } else if (e.target.value === "") {
                                                api.setClaimIntervalHours(undefined);
                                            }
                                        }}
                                        placeholder="h"
                                    />
                                    <ToggleGroup
                                        aria-label="Loss period"
                                        onValueChange={(next: string[]) => {
                                            if (next[0]) setPeriod(next[0] as typeof period);
                                        }}
                                        value={[period]}
                                    >
                                        {(["day", "week", "month", "year"] as const).map((pp) => (
                                            <ToggleGroupItem key={pp} size="sm" value={pp}>
                                                {pp[0].toUpperCase()}
                                            </ToggleGroupItem>
                                        ))}
                                    </ToggleGroup>
                                    <span className={cn("font-mono text-[11.5px] tabular-nums", losses.length > 0 ? "text-destructive" : "text-muted-foreground")}>{losses.length > 0 ? `${losses.join(" · ")} ${periodLabel}` : "nothing lost"}</span>
                                </div>
                            </div>
                        </section>
                    )}

                    {(evaluation.dorms.per_dorm ?? []).some((d) => (d.comfort_upside_per_hour ?? 0) > 0.05) && (
                        <section className="flex flex-col gap-1">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">Furniture upside</h3>
                            {(evaluation.dorms.per_dorm ?? [])
                                .filter((d) => (d.comfort_upside_per_hour ?? 0) > 0.05)
                                .map((d) => (
                                    <span className="text-[11.5px]" key={d.slot_id}>
                                        <span className="text-muted-foreground">Dormitory Lv{d.level} ambience</span>{" "}
                                        <span className="font-mono tabular-nums">
                                            {d.comfort}/{d.comfort_limit}
                                        </span>
                                        <span className="text-muted-foreground"> - </span>
                                        <span className="font-mono font-semibold text-emerald-400 tabular-nums">+{(d.comfort_upside_per_hour ?? 0).toFixed(2)}/h</span>
                                        <span className="text-muted-foreground"> recovery sitting in the furniture shop.</span>
                                    </span>
                                ))}
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

                    <section className="flex flex-col gap-1.5">
                        <div className="flex items-center justify-between gap-3">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">Compare</h3>
                            <div className="flex items-center gap-1.5">
                                <Button
                                    onClick={() => {
                                        const rot = api.rotation?.rotation;
                                        const lines: string[] = [`Base plan - ${num(evaluation.assignment.yield_lmd_per_day)} LMD/day, ${num(evaluation.assignment.yield_exp_per_day)} EXP/day`];
                                        for (const shift of rot?.shifts ?? []) {
                                            lines.push(`\nShift ${shift.index}`);
                                            for (const r of shift.rooms.filter((r) => r.active && r.recommended.length > 0)) {
                                                lines.push(`  ${roomLabel(r.room_type, api.catalog)}: ${r.recommended.map((o) => o.name).join(", ")}`);
                                            }
                                        }
                                        void navigator.clipboard.writeText(lines.join("\n"));
                                    }}
                                    size="sm"
                                    variant="ghost"
                                >
                                    <Copy />
                                    Copy plan
                                </Button>
                                <Button onClick={() => setSnapshot(live)} size="sm" variant="outline">
                                    <Camera />
                                    {snapshot ? "Re-snapshot" : "Snapshot"}
                                </Button>
                                {snapshot && (
                                    <Button aria-label="Clear snapshot" onClick={() => setSnapshot(null)} size="icon-sm" variant="ghost">
                                        <X />
                                    </Button>
                                )}
                            </div>
                        </div>
                        {snapshot ? (
                            <div className="flex flex-col divide-y divide-border/60">
                                <DiffRow format={(v) => `${Math.round(v)}%`} from={snapshot.efficiency} label="Production efficiency" to={live.efficiency} />
                                <DiffRow format={num} from={snapshot.lmd} label="LMD / day" to={live.lmd} />
                                <DiffRow format={num} from={snapshot.exp} label="EXP / day" to={live.exp} />
                                <DiffRow format={(v) => v.toFixed(1)} from={snapshot.gold} label="Pure Gold / day" to={live.gold} />
                                <DiffRow format={(v) => String(Math.round(v))} from={snapshot.tradingCapacity} label="Trading capacity" to={live.tradingCapacity} />
                                <DiffRow format={(v) => String(Math.round(v))} from={snapshot.powerNet} label="Power net" to={live.powerNet} />
                                <DiffRow format={(v) => `${v.toFixed(2)}/h`} from={snapshot.dormRecovery} label="Dorm recovery" to={live.dormRecovery} />
                                {snapshot.nextFullHours !== null && live.nextFullHours !== null && <DiffRow format={(v) => hoursLabel(v)} from={snapshot.nextFullHours} label="First room stalls" to={live.nextFullHours} />}
                            </div>
                        ) : (
                            <span className="text-[11.5px] text-muted-foreground">Snapshot the current numbers, then edit the board or run the optimizer - the difference tracks live.</span>
                        )}
                    </section>

                    {api.rotationLoading && !sustainability && (
                        <section className="flex flex-col gap-1">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">Simulated totals &amp; rotation events</h3>
                            <span className="animate-pulse text-[11.5px] text-muted-foreground">Simulating the rotation - this is the slow part…</span>
                        </section>
                    )}

                    {sustainability?.facilities && sustainability.facilities.length > 0 && (
                        <section className="flex flex-col gap-1.5">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">Simulated totals · {Math.round(sustainability.horizon_hours / 24)} days under the rotation</h3>
                            <div className="flex flex-col divide-y divide-border/60">
                                {sustainability.facilities.map((f) => {
                                    const produced = f.room_type === "TRADING" ? `${num(f.lmd)} LMD` : f.formula_type === "F_GOLD" ? `${f.gold.toFixed(1)} gold` : f.formula_type === "F_EXP" ? `${num(f.exp)} EXP` : "-";
                                    return (
                                        <div className="flex items-baseline gap-3 py-1 text-[11.5px]" key={f.slot_id}>
                                            <span className="min-w-0 flex-1 truncate">{roomLabel(f.room_type, api.catalog)}</span>
                                            <span className="font-mono tabular-nums">{produced}</span>
                                            <span className={cn("w-24 text-right font-mono tabular-nums", f.idle_hours > 0.05 ? "text-destructive" : "text-muted-foreground")}>{f.idle_hours > 0.05 ? `${f.idle_hours.toFixed(1)}h idle` : "no idle"}</span>
                                        </div>
                                    );
                                })}
                            </div>
                        </section>
                    )}

                    {evaluation.unrotated && (
                        <section className="flex flex-col gap-1">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">Without rotating · {Math.round(evaluation.unrotated.horizon_hours / 24)} days</h3>
                            {evaluation.unrotated.depleted.length === 0 ? (
                                <span className="text-[11.5px] text-muted-foreground">Your stationed crews hold up even with no swaps at all.</span>
                            ) : (
                                <span className="text-[11.5px]">
                                    <span className="font-semibold text-destructive">{evaluation.unrotated.depleted.length} operators run dry</span>
                                    <span className="text-muted-foreground">
                                        {" "}
                                        if you never swap - first at {(evaluation.unrotated.depleted[0]?.at_hours ?? 0).toFixed(1)}h ({evaluation.unrotated.depleted[0]?.operator.name}). Rotate or lose the hours.
                                    </span>
                                </span>
                            )}
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
