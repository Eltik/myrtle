import { Camera, ChevronDown, Copy, X } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import type { IEvaluateResponse } from "#/lib/api/base";
import { roomLabel } from "#/lib/base/catalog";
import { type IFormatters, type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages as panelMessages } from "../BasePanel.messages";
import { useBaseOptimizer } from "../base-context";
import { TileTooltip } from "../board/tile/components/TileTooltip";
import type { messages } from "./DeepDive.messages";

/** The daily LMD and EXP labels are declared with the panel's own headline figures. */
type DeepT = TypedT<typeof messages & typeof panelMessages>;
type DeepRichT = TypedRichT<typeof messages>;

/** A key in `DeepDive.messages.ts`, resolved through one of the tables below. */
type MessageKey = keyof typeof messages & string;

const PERIOD_SUFFIX: Record<Period, MessageKey> = {
    day: "profile.base.deep.period.day",
    week: "profile.base.deep.period.week",
    month: "profile.base.deep.period.month",
    year: "profile.base.deep.period.year",
};

const PERIOD_INITIAL: Record<Period, MessageKey> = {
    day: "profile.base.deep.period.dayInitial",
    week: "profile.base.deep.period.weekInitial",
    month: "profile.base.deep.period.monthInitial",
    year: "profile.base.deep.period.yearInitial",
};

type Period = "day" | "week" | "month" | "year";

const num = (value: number, f: IFormatters) => f.number(Math.round(value));

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
    const t: DeepT = useT("user");
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
                    <span className="text-muted-foreground">{t("profile.base.deep.noChange")}</span>
                )}
            </span>
        </div>
    );
}

function hoursLabel(hours: number, t: DeepT): string {
    if (hours >= 48) return t("profile.base.deep.days", { days: (hours / 24).toFixed(1) });
    return t("profile.base.deep.hours", { hours: hours.toFixed(1) });
}

/** Wall-clock time `hours` from now, in the page's locale. */
function clockAfter(hours: number, f: IFormatters): string {
    const at = new Date(Date.now() + hours * 3_600_000);
    const sameDay = at.getDate() === new Date().getDate();
    const time = f.time(at, { hour: "numeric", minute: "2-digit" });
    return sameDay ? time : `${f.date(at, { weekday: "short" })} ${time}`;
}

function productLabel(room: { room_type: string; formula_type: string | null }, t: DeepT): string {
    if (room.room_type === "TRADING") return t("profile.base.deep.product.lmd");
    if (room.formula_type === "F_GOLD") return t("profile.base.deep.product.gold");
    if (room.formula_type === "F_EXP") return t("profile.base.deep.product.exp");
    return t("profile.base.deep.product.none");
}

/**
 * The deep dive: check-in economics (when to log in, what a lazier cadence
 * loses), per-facility output with buffer fill times, and the rotation
 * simulation's depletion events. Everything here is server-scored - this
 * panel only formats it.
 */
export function DeepDive() {
    const t: DeepT = useT("user");
    const rt: DeepRichT = useRichT("user");
    const f = useFormatters();
    const [open, setOpen] = useState(true);
    const [cadence, setCadence] = useState("12");
    const [period, setPeriod] = useState<Period>("day");
    const periodMult = { day: 1, week: 7, month: 30, year: 365 }[period];
    const periodLabel = t(PERIOD_SUFFIX[period]);
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
    const stallRoom = rooms.length > 0 ? rooms.reduce((a, b) => ((a.fill_hours ?? 1e9) <= (b.fill_hours ?? 1e9) ? a : b)) : undefined;
    const sustainability = api.rotation?.rotation.sustainability;
    const interval = claim?.intervals.find((i) => String(i.hours) === cadence);
    const losses = interval
        ? [
              interval.lost_lmd_per_day >= 1 ? t("profile.base.deep.lost.lmd", { amount: num(interval.lost_lmd_per_day * periodMult, f) }) : null,
              interval.lost_gold_per_day >= 0.1 ? t("profile.base.deep.lost.gold", { amount: (interval.lost_gold_per_day * periodMult).toFixed(1) }) : null,
              interval.lost_exp_per_day >= 1 ? t("profile.base.deep.lost.exp", { amount: num(interval.lost_exp_per_day * periodMult, f) }) : null,
          ].filter(Boolean)
        : [];

    return (
        <Collapsible onOpenChange={setOpen} open={open}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-xl border border-border bg-card px-4 py-2.5 transition-colors hover:bg-muted/40">
                <span className="font-medium text-[13px]">{t("profile.base.deep.title")}</span>
                <ChevronDown className={cn("size-4 text-muted-foreground transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>

            <CollapsibleContent>
                <div className="mt-2 flex flex-col gap-4 rounded-xl border border-border bg-card px-4 py-3">
                    {claim && (
                        <section className="flex flex-col gap-2">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.deep.checkin")}</h3>
                            <div className="flex flex-wrap items-center justify-between gap-3">
                                <div className="flex flex-col gap-0.5">
                                    <span className="text-[12px]">{rt("profile.base.deep.logInBefore", { time: <span className="font-semibold text-foreground">{clockAfter(claim.next_full_hours, f)}</span> })}</span>
                                    <span className="text-[11px] text-muted-foreground">
                                        {t("profile.base.deep.stalls", { room: stallRoom ? roomLabel(stallRoom.room_type, api.catalog) : t("profile.base.deep.firstRoom"), duration: hoursLabel(claim.next_full_hours, t) })}{" "}
                                        <TileTooltip label={<span className="block max-w-64">{t("profile.base.deep.conservative.tooltip")}</span>}>
                                            <span className="cursor-help underline decoration-dotted underline-offset-2">{t("profile.base.deep.conservative")}</span>
                                        </TileTooltip>
                                    </span>
                                    {evaluation.drones && (
                                        <span className="text-[11px] text-muted-foreground">
                                            {evaluation.drones.full_in_hours != null ? (
                                                rt("profile.base.deep.dronesFilling", {
                                                    meter: (
                                                        <span className="font-mono tabular-nums">
                                                            {Math.round(evaluation.drones.current)}/{evaluation.drones.max}
                                                        </span>
                                                    ),
                                                    when: <span className="font-mono tabular-nums">{hoursLabel(evaluation.drones.full_in_hours, t)}</span>,
                                                })
                                            ) : (
                                                <>
                                                    {t("profile.base.deep.drones")}{" "}
                                                    <span className="font-mono tabular-nums">
                                                        {Math.round(evaluation.drones.current)}/{evaluation.drones.max}
                                                    </span>
                                                    <span className="text-destructive">{t("profile.base.deep.dronesFull")}</span>
                                                </>
                                            )}
                                        </span>
                                    )}
                                    {(evaluation.trainer_hints ?? []).length > 0 && (
                                        <span className="text-[11px]">
                                            <span className="text-muted-foreground">{t("profile.base.deep.trainers", { class: api.trainingClass })}</span>
                                            {(evaluation.trainer_hints ?? []).map((h, i) => (
                                                <span key={h.operator.operator_id}>
                                                    {i > 0 && <span className="text-muted-foreground"> · </span>}
                                                    {h.operator.name} <span className="font-mono text-muted-foreground tabular-nums">+{Math.round(h.value_pct)}%</span>
                                                </span>
                                            ))}
                                        </span>
                                    )}
                                    {atRisk.length > 0 && (
                                        <span className="text-[11px]">
                                            <span className="text-muted-foreground">{evaluation.morale_synced_hours_ago != null ? t("profile.base.deep.atRisk.synced", { ago: hoursLabel(evaluation.morale_synced_hours_ago, t) }) : t("profile.base.deep.atRisk")}</span>
                                            {atRisk.map((e, i) => (
                                                <span key={e.operator_id}>
                                                    {i > 0 && <span className="text-muted-foreground"> · </span>}
                                                    {e.name} <span className={cn("font-mono tabular-nums", (e.lasts_hours ?? 99) < 12 ? "text-destructive" : "text-muted-foreground")}>{hoursLabel(e.lasts_hours ?? 0, t)}</span>
                                                </span>
                                            ))}
                                        </span>
                                    )}
                                </div>
                                <div className="flex items-center gap-2">
                                    <ToggleGroup
                                        aria-label={t("profile.base.deep.cadence.aria")}
                                        onValueChange={(next: string[]) => {
                                            if (next[0]) setCadence(next[0]);
                                        }}
                                        value={[cadence]}
                                    >
                                        {claim.intervals.map((i) => (
                                            <ToggleGroupItem key={i.hours} size="sm" value={String(i.hours)}>
                                                {t("profile.base.deep.cadence.hours", { hours: i.hours })}
                                            </ToggleGroupItem>
                                        ))}
                                    </ToggleGroup>
                                    <input
                                        aria-label={t("profile.base.deep.cadence.custom")}
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
                                        placeholder={t("profile.base.deep.cadence.customPlaceholder")}
                                    />
                                    <ToggleGroup
                                        aria-label={t("profile.base.deep.period.aria")}
                                        onValueChange={(next: string[]) => {
                                            if (next[0]) setPeriod(next[0] as typeof period);
                                        }}
                                        value={[period]}
                                    >
                                        {(["day", "week", "month", "year"] as const).map((pp) => (
                                            <ToggleGroupItem key={pp} size="sm" value={pp}>
                                                {t(PERIOD_INITIAL[pp])}
                                            </ToggleGroupItem>
                                        ))}
                                    </ToggleGroup>
                                    <span className={cn("font-mono text-[11.5px] tabular-nums", losses.length > 0 ? "text-destructive" : "text-muted-foreground")}>{losses.length > 0 ? t("profile.base.deep.lost.suffix", { losses: losses.join(" · "), period: periodLabel }) : t("profile.base.deep.lost.nothing")}</span>
                                </div>
                            </div>
                        </section>
                    )}

                    {(evaluation.dorms.per_dorm ?? []).some((d) => (d.comfort_upside_per_hour ?? 0) > 0.05) && (
                        <section className="flex flex-col gap-1">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.deep.furniture")}</h3>
                            {(evaluation.dorms.per_dorm ?? [])
                                .filter((d) => (d.comfort_upside_per_hour ?? 0) > 0.05)
                                .map((d) => (
                                    <span className="text-[11.5px] text-muted-foreground" key={d.slot_id}>
                                        {rt("profile.base.deep.furniture.line", {
                                            level: d.level,
                                            meter: (
                                                <span className="font-mono text-foreground tabular-nums">
                                                    {d.comfort}/{d.comfort_limit}
                                                </span>
                                            ),
                                            rate: <span className="font-mono font-semibold text-emerald-400 tabular-nums">{t("profile.base.deep.furniture.rate", { rate: (d.comfort_upside_per_hour ?? 0).toFixed(2) })}</span>,
                                        })}
                                    </span>
                                ))}
                        </section>
                    )}

                    {rooms.length > 0 && (
                        <section className="flex flex-col gap-2">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.deep.outputByFacility")}</h3>
                            <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
                                {rooms.map((room) => {
                                    const daily =
                                        room.room_type === "TRADING"
                                            ? t("profile.base.deep.daily.lmd", { amount: num(room.yield_lmd_per_day, f) })
                                            : room.formula_type === "F_GOLD"
                                              ? t("profile.base.deep.daily.gold", { amount: room.yield_gold_per_day.toFixed(1) })
                                              : t("profile.base.deep.daily.exp", { amount: num(room.yield_exp_per_day, f) });
                                    const overflows = room.fill_hours != null && interval !== undefined && room.fill_hours < interval.hours;
                                    return (
                                        <div className="flex flex-col gap-1 rounded-lg border border-border/60 bg-muted/10 px-3 py-2" key={room.slot_id}>
                                            <div className="flex items-baseline justify-between gap-2">
                                                <span className="min-w-0 truncate font-medium text-[12px]">{roomLabel(room.room_type, api.catalog)}</span>
                                                <span className="shrink-0 text-[10px] text-muted-foreground uppercase tracking-wide">{productLabel(room, t)}</span>
                                            </div>
                                            <div className="flex items-baseline justify-between gap-2">
                                                <span className="font-mono font-semibold text-[13px] tabular-nums">
                                                    {daily}
                                                    <span className="font-normal text-[10px] text-muted-foreground">{t("profile.base.deep.perDay")}</span>
                                                </span>
                                                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{Math.round(room.total_efficiency)}%</span>
                                            </div>
                                            <span className={cn("text-[10.5px]", overflows ? "text-destructive" : "text-muted-foreground")}>
                                                {t("profile.base.deep.fullIn", { duration: room.fill_hours == null ? "-" : hoursLabel(room.fill_hours, t), count: room.capacity, unit: room.room_type === "TRADING" ? t("profile.base.deep.unit.orders") : t("profile.base.deep.unit.items") })}
                                                {overflows && t("profile.base.deep.overflows")}
                                            </span>
                                        </div>
                                    );
                                })}
                            </div>
                        </section>
                    )}

                    <section className="flex flex-col gap-1.5">
                        <div className="flex items-center justify-between gap-3">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.deep.compare")}</h3>
                            <div className="flex items-center gap-1.5">
                                <Button
                                    disabled={!api.rotation}
                                    onClick={() => {
                                        const rot = api.rotation?.rotation;
                                        const lines: string[] = [t("profile.base.deep.copy.header", { lmd: num(evaluation.assignment.yield_lmd_per_day, f), exp: num(evaluation.assignment.yield_exp_per_day, f) })];
                                        for (const shift of rot?.shifts ?? []) {
                                            lines.push(`\n${t("profile.base.deep.copy.shift", { n: shift.index })}`);
                                            for (const r of shift.rooms.filter((r) => r.active && r.recommended.length > 0)) {
                                                lines.push(`  ${t("profile.base.deep.copy.room", { room: roomLabel(r.room_type, api.catalog), operators: r.recommended.map((o) => o.name).join(", ") })}`);
                                            }
                                        }
                                        void navigator.clipboard.writeText(lines.join("\n"));
                                    }}
                                    size="sm"
                                    variant="ghost"
                                >
                                    <Copy />
                                    {t("profile.base.deep.copyPlan")}
                                </Button>
                                <Button onClick={() => setSnapshot(live)} size="sm" variant="outline">
                                    <Camera />
                                    {snapshot ? t("profile.base.deep.resnapshot") : t("profile.base.deep.snapshot")}
                                </Button>
                                {snapshot && (
                                    <Button aria-label={t("profile.base.deep.clearSnapshot")} onClick={() => setSnapshot(null)} size="icon-sm" variant="ghost">
                                        <X />
                                    </Button>
                                )}
                            </div>
                        </div>
                        {snapshot ? (
                            <div className="flex flex-col divide-y divide-border/60">
                                <DiffRow format={(v) => `${Math.round(v)}%`} from={snapshot.efficiency} label={t("profile.base.deep.diff.efficiency")} to={live.efficiency} />
                                <DiffRow format={(v) => num(v, f)} from={snapshot.lmd} label={t("profile.base.headline.lmd")} to={live.lmd} />
                                <DiffRow format={(v) => num(v, f)} from={snapshot.exp} label={t("profile.base.headline.exp")} to={live.exp} />
                                <DiffRow format={(v) => v.toFixed(1)} from={snapshot.gold} label={t("profile.base.deep.diff.gold")} to={live.gold} />
                                <DiffRow format={(v) => String(Math.round(v))} from={snapshot.tradingCapacity} label={t("profile.base.deep.diff.tradingCapacity")} to={live.tradingCapacity} />
                                <DiffRow format={(v) => String(Math.round(v))} from={snapshot.powerNet} label={t("profile.base.deep.diff.powerNet")} to={live.powerNet} />
                                <DiffRow format={(v) => `${v.toFixed(2)}/h`} from={snapshot.dormRecovery} label={t("profile.base.deep.diff.dormRecovery")} to={live.dormRecovery} />
                                {snapshot.nextFullHours !== null && live.nextFullHours !== null && <DiffRow format={(v) => hoursLabel(v, t)} from={snapshot.nextFullHours} label={t("profile.base.deep.diff.firstStall")} to={live.nextFullHours} />}
                            </div>
                        ) : (
                            <span className="text-[11.5px] text-muted-foreground">{t("profile.base.deep.snapshotHint")}</span>
                        )}
                    </section>

                    {api.rotationLoading && !sustainability && (
                        <section className="flex flex-col gap-1">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.deep.simulating.title")}</h3>
                            <span className="animate-pulse text-[11.5px] text-muted-foreground">{t("profile.base.deep.simulating")}</span>
                        </section>
                    )}

                    {sustainability?.facilities && sustainability.facilities.length > 0 && (
                        <section className="flex flex-col gap-1.5">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.deep.simulated.title", { days: Math.round(sustainability.horizon_hours / 24) })}</h3>
                            <div className="flex flex-col divide-y divide-border/60">
                                {sustainability.facilities.map((facility) => {
                                    const produced =
                                        facility.room_type === "TRADING"
                                            ? t("profile.base.deep.daily.lmd", { amount: num(facility.lmd, f) })
                                            : facility.formula_type === "F_GOLD"
                                              ? t("profile.base.deep.daily.gold", { amount: facility.gold.toFixed(1) })
                                              : facility.formula_type === "F_EXP"
                                                ? t("profile.base.deep.daily.exp", { amount: num(facility.exp, f) })
                                                : "-";
                                    return (
                                        <div className="flex items-baseline gap-3 py-1 text-[11.5px]" key={facility.slot_id}>
                                            <span className="min-w-0 flex-1 truncate">{roomLabel(facility.room_type, api.catalog)}</span>
                                            <span className="font-mono tabular-nums">{produced}</span>
                                            <span className={cn("w-24 text-right font-mono tabular-nums", facility.idle_hours > 0.05 ? "text-destructive" : "text-muted-foreground")}>
                                                {facility.idle_hours > 0.05 ? t("profile.base.deep.idle", { hours: facility.idle_hours.toFixed(1) }) : t("profile.base.deep.noIdle")}
                                            </span>
                                        </div>
                                    );
                                })}
                            </div>
                        </section>
                    )}

                    {evaluation.unrotated && (
                        <section className="flex flex-col gap-1">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.deep.unrotated.title", { days: Math.round(evaluation.unrotated.horizon_hours / 24) })}</h3>
                            {evaluation.unrotated.depleted.length === 0 ? (
                                <span className="text-[11.5px] text-muted-foreground">{t("profile.base.deep.unrotated.fine")}</span>
                            ) : (
                                <span className="text-[11.5px] text-muted-foreground">
                                    {rt("profile.base.deep.unrotated.line", {
                                        count: <span className="font-semibold text-destructive">{t("profile.base.deep.unrotated.dry", { count: evaluation.unrotated.depleted.length })}</span>,
                                        hours: (evaluation.unrotated.depleted[0]?.at_hours ?? 0).toFixed(1),
                                        name: evaluation.unrotated.depleted[0]?.operator.name,
                                    })}
                                </span>
                            )}
                        </section>
                    )}

                    {sustainability && (
                        <section className="flex flex-col gap-1.5">
                            <h3 className="font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("profile.base.deep.events.title", { days: Math.round(sustainability.horizon_hours / 24) })}</h3>
                            {sustainability.depleted.length === 0 ? (
                                <span className="text-[11.5px] text-muted-foreground">{t("profile.base.deep.events.none")}</span>
                            ) : (
                                <div className="flex flex-col divide-y divide-border/60">
                                    {sustainability.depleted.map((event) => (
                                        <div className="flex items-baseline gap-3 py-1 text-[11.5px]" key={`${event.operator.operator_id}:${event.at_hours}`}>
                                            <span className="w-24 shrink-0 font-mono text-muted-foreground tabular-nums">{t("profile.base.deep.events.when", { day: Math.floor(event.at_hours / 24) + 1, hours: (event.at_hours % 24).toFixed(1) })}</span>
                                            <span className="min-w-0 flex-1 truncate">
                                                {event.operator.name} <span className="text-muted-foreground">{t("profile.base.deep.events.runsDry", { room: roomLabel(event.room_type, api.catalog) })}</span>
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
