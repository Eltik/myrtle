import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { roomLabel } from "#/lib/base/catalog";
import { cn } from "#/lib/utils";
import { useBaseOptimizer } from "../base-context";
import { MoraleTimeline } from "./MoraleTimeline";

const num = (value: number) => Math.round(value).toLocaleString();
const dec = (value: number) => (Number.isInteger(value) ? String(value) : value.toFixed(2));

function Delta({ from, to, suffix = "" }: { from: number; to: number; suffix?: string }) {
    const change = Math.round(to) - Math.round(from);
    if (change === 0) return <span className="text-muted-foreground">no change</span>;
    return (
        <span className={cn("font-semibold", change > 0 ? "text-emerald-400" : "text-destructive")}>
            {change > 0 ? "+" : "-"}
            {num(Math.abs(change))}
            {suffix}
        </span>
    );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
    return (
        <div className="flex items-baseline justify-between gap-4 py-1">
            <span className="text-[11.5px] text-muted-foreground">{label}</span>
            <span className="font-mono text-[11.5px] tabular-nums">{children}</span>
        </div>
    );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
    return (
        <section className="flex flex-col">
            <h3 className="mb-1 font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{title}</h3>
            <div className="divide-y divide-border/60">{children}</div>
        </section>
    );
}

export function StatsForNerds() {
    const [open, setOpen] = useState(false);
    const api = useBaseOptimizer();

    const evaluation = api.evaluation;
    if (!evaluation) return null;

    const { assignment, power, sustain, dorms } = evaluation;
    const baseline = api.proposal?.baseline;
    const planned = api.proposal?.proposal;
    const diffs = (api.proposal?.room_diffs ?? []).filter((d) => d.before.join() !== d.after.join());
    const rotation = api.rotation?.rotation;
    const timeline = rotation?.sustainability?.timeline ?? [];

    const depleting = sustain.filter((entry) => entry.lasts_hours !== null);
    const tireless = sustain.length - depleting.length;
    const shortest = depleting.length > 0 ? Math.min(...depleting.map((entry) => entry.lasts_hours ?? 0)) : null;

    return (
        <Collapsible onOpenChange={setOpen} open={open}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-xl border border-border bg-card px-4 py-2.5 transition-colors hover:bg-muted/40">
                <span className="font-medium text-[13px]">Stats for Nerds</span>
                <ChevronDown className={cn("size-4 text-muted-foreground transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>

            <CollapsibleContent>
                <div className="mt-2 grid gap-x-8 gap-y-5 rounded-xl border border-border bg-card px-4 py-3 md:grid-cols-2">
                    <Section title="Output">
                        <Row label="Production efficiency">{Math.round(assignment.total_production_efficiency)}%</Row>
                        <Row label="LMD / day">{num(assignment.yield_lmd_per_day)}</Row>
                        <Row label="EXP / day">{num(assignment.yield_exp_per_day)}</Row>
                        <Row label="Total value (LMD-eq)">{num(assignment.yield_total_value)}</Row>
                    </Section>

                    <Section title="Power">
                        <Row label="Generated">{num(power.generated)}</Row>
                        <Row label="Consumed">{num(power.consumed)}</Row>
                        <Row label="Net">
                            <span className={cn("font-semibold", power.net < 0 && "text-destructive")}>
                                {power.net > 0 ? "+" : ""}
                                {num(power.net)}
                            </span>
                        </Row>
                    </Section>

                    <Section title="Dormitories">
                        <Row label="Rooms">{dorms.count}</Row>
                        <Row label="Summed levels">{dorms.total_levels}</Row>
                        <Row label="Beds">{dorms.total_capacity}</Row>
                        <Row label="Sim recovery / hour">{dec(dorms.recovery_per_hour)}</Row>
                    </Section>

                    <Section title="Crew">
                        <Row label="Stationed">{sustain.length}</Row>
                        <Row label="Never deplete">{tireless}</Row>
                        {shortest !== null && <Row label="Shortest morale">{Math.round(shortest)}h</Row>}
                        {rotation?.sustained && rotation.sustained.length > 0 && <Row label="Run 24/7 (Fiammetta)">{rotation.sustained.map((o) => o.name).join(", ")}</Row>}
                    </Section>

                    {baseline && planned && (
                        <Section title="Optimized vs your base">
                            <Row label="Production efficiency">
                                {Math.round(baseline.total_production_efficiency)}% &rarr; {Math.round(planned.total_production_efficiency)}% <Delta from={baseline.total_production_efficiency} suffix="%" to={planned.total_production_efficiency} />
                            </Row>
                            <Row label="LMD / day">
                                {num(baseline.yield_lmd_per_day)} &rarr; {num(planned.yield_lmd_per_day)} <Delta from={baseline.yield_lmd_per_day} to={planned.yield_lmd_per_day} />
                            </Row>
                            <Row label="EXP / day">
                                {num(baseline.yield_exp_per_day)} &rarr; {num(planned.yield_exp_per_day)} <Delta from={baseline.yield_exp_per_day} to={planned.yield_exp_per_day} />
                            </Row>
                            <Row label="Total value (LMD-eq)">
                                {num(baseline.yield_total_value)} &rarr; {num(planned.yield_total_value)} <Delta from={baseline.yield_total_value} to={planned.yield_total_value} />
                            </Row>
                            <Row label="Rooms restaffed">{diffs.length}</Row>
                        </Section>
                    )}

                    {rotation?.sustainability && (
                        <Section title="Rotation">
                            <Row label="Verdict">{rotation.sustainability.verdict === "holds_up" ? "Holds up" : "Depletes"}</Row>
                            <Row label="Simulated">{Math.round(rotation.sustainability.horizon_hours / 24)}d</Row>
                            <Row label="Operators depleting">{rotation.sustainability.depleted.length}</Row>
                            <Row label="Dorm overflow at peak">{rotation.sustainability.dorm_overflow}</Row>
                        </Section>
                    )}

                    {dorms.per_dorm && dorms.per_dorm.length > 0 && (
                        <div className="md:col-span-2">
                            <Section title="Dormitories - best first, game rates">
                                {dorms.per_dorm.map((dorm) => (
                                    <div className="flex items-baseline gap-3 py-1 text-[11.5px]" key={dorm.slot_id}>
                                        <span className="min-w-0 flex-1 truncate text-muted-foreground">{dorm.slot_id}</span>
                                        <span className="font-mono tabular-nums">Lv {dorm.level}</span>
                                        <span className="w-16 text-right font-mono text-muted-foreground tabular-nums">{dorm.capacity} beds</span>
                                        <span className="w-16 text-right font-mono tabular-nums">{dec(dorm.recovery_per_hour)}/h</span>
                                        <span className="w-24 text-right font-mono text-muted-foreground tabular-nums">{dorm.occupant_aura_per_hour > 0 ? `+${dec(dorm.occupant_aura_per_hour)} aura` : dorm.occupant_single_per_hour > 0 ? `+${dec(dorm.occupant_single_per_hour)} single` : "-"}</span>
                                    </div>
                                ))}
                            </Section>
                        </div>
                    )}

                    {timeline.length > 0 && (
                        <div className="md:col-span-2">
                            <Section title="Morale over time">
                                <MoraleTimeline catalog={api.catalog} timeline={timeline} />
                            </Section>
                        </div>
                    )}

                    {diffs.length > 0 && (
                        <div className="md:col-span-2">
                            <Section title="Per-room changes">
                                {diffs.map((diff) => (
                                    <div className="flex items-baseline gap-3 py-1 text-[11.5px]" key={diff.slot_id}>
                                        <span className="min-w-0 flex-1 truncate">{roomLabel(diff.room_type, api.catalog)}</span>
                                        <span className="font-mono text-muted-foreground tabular-nums">
                                            {Math.round(diff.efficiency_before)}% &rarr; {Math.round(diff.efficiency_after)}%
                                        </span>
                                        <span className="w-24 text-right font-mono tabular-nums">
                                            <Delta from={diff.efficiency_before} suffix="%" to={diff.efficiency_after} />
                                        </span>
                                    </div>
                                ))}
                            </Section>
                        </div>
                    )}
                </div>
            </CollapsibleContent>
        </Collapsible>
    );
}
