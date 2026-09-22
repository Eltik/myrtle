import { ChevronDown } from "lucide-react";
import { useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { roomLabel } from "#/lib/base/catalog";
import { type IFormatters, useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages as panelMessages } from "../BasePanel.messages";
import { useBaseOptimizer } from "../base-context";
import type { messages as sustainMessages } from "../controls/SustainabilityBadge.messages";
import { MoraleTimeline } from "./MoraleTimeline";
import type { messages } from "./StatsForNerds.messages";

/** The daily-output labels and the rotation verdict are declared with the panel and the badge that own them. */
type NerdsT = TypedT<typeof messages & typeof panelMessages & typeof sustainMessages>;

const num = (value: number, f: IFormatters) => f.number(Math.round(value));
const dec = (value: number) => (Number.isInteger(value) ? String(value) : value.toFixed(2));

function Delta({ from, to, suffix = "" }: { from: number; to: number; suffix?: string }) {
    const t: NerdsT = useT("user");
    const f = useFormatters();
    const change = Math.round(to) - Math.round(from);
    if (change === 0) return <span className="text-muted-foreground">{t("profile.base.nerds.noChange")}</span>;
    return (
        <span className={cn("font-semibold", change > 0 ? "text-emerald-400" : "text-destructive")}>
            {change > 0 ? "+" : "-"}
            {num(Math.abs(change), f)}
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
    const t: NerdsT = useT("user");
    const f = useFormatters();
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
                <span className="font-medium text-[13px]">{t("profile.base.nerds.title")}</span>
                <ChevronDown className={cn("size-4 text-muted-foreground transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>

            <CollapsibleContent>
                <div className="mt-2 grid gap-x-8 gap-y-5 rounded-xl border border-border bg-card px-4 py-3 md:grid-cols-2">
                    <Section title={t("profile.base.nerds.output")}>
                        <Row label={t("profile.base.nerds.productionEfficiency")}>{Math.round(assignment.total_production_efficiency)}%</Row>
                        <Row label={t("profile.base.headline.lmd")}>{num(assignment.yield_lmd_per_day, f)}</Row>
                        <Row label={t("profile.base.headline.exp")}>{num(assignment.yield_exp_per_day, f)}</Row>
                        <Row label={t("profile.base.nerds.totalValue")}>{num(assignment.yield_total_value, f)}</Row>
                    </Section>

                    <Section title={t("profile.base.nerds.power")}>
                        <Row label={t("profile.base.nerds.generated")}>{num(power.generated, f)}</Row>
                        <Row label={t("profile.base.nerds.consumed")}>{num(power.consumed, f)}</Row>
                        <Row label={t("profile.base.nerds.net")}>
                            <span className={cn("font-semibold", power.net < 0 && "text-destructive")}>
                                {power.net > 0 ? "+" : ""}
                                {num(power.net, f)}
                            </span>
                        </Row>
                    </Section>

                    <Section title={t("profile.base.nerds.dormitories")}>
                        <Row label={t("profile.base.nerds.rooms")}>{dorms.count}</Row>
                        <Row label={t("profile.base.nerds.summedLevels")}>{dorms.total_levels}</Row>
                        <Row label={t("profile.base.nerds.beds")}>{dorms.total_capacity}</Row>
                        <Row label={t("profile.base.nerds.simRecovery")}>{dec(dorms.recovery_per_hour)}</Row>
                    </Section>

                    <Section title={t("profile.base.nerds.crew")}>
                        <Row label={t("profile.base.nerds.stationed")}>{sustain.length}</Row>
                        <Row label={t("profile.base.nerds.neverDeplete")}>{tireless}</Row>
                        {shortest !== null && <Row label={t("profile.base.nerds.shortestMorale")}>{t("profile.base.nerds.hours", { hours: Math.round(shortest) })}</Row>}
                        {rotation?.sustained && rotation.sustained.length > 0 && <Row label={t("profile.base.nerds.sustained")}>{rotation.sustained.map((o) => o.name).join(", ")}</Row>}
                    </Section>

                    {baseline && planned && (
                        <Section title={t("profile.base.nerds.comparison")}>
                            <Row label={t("profile.base.nerds.productionEfficiency")}>
                                {Math.round(baseline.total_production_efficiency)}% &rarr; {Math.round(planned.total_production_efficiency)}% <Delta from={baseline.total_production_efficiency} suffix="%" to={planned.total_production_efficiency} />
                            </Row>
                            <Row label={t("profile.base.headline.lmd")}>
                                {num(baseline.yield_lmd_per_day, f)} &rarr; {num(planned.yield_lmd_per_day, f)} <Delta from={baseline.yield_lmd_per_day} to={planned.yield_lmd_per_day} />
                            </Row>
                            <Row label={t("profile.base.headline.exp")}>
                                {num(baseline.yield_exp_per_day, f)} &rarr; {num(planned.yield_exp_per_day, f)} <Delta from={baseline.yield_exp_per_day} to={planned.yield_exp_per_day} />
                            </Row>
                            <Row label={t("profile.base.nerds.totalValue")}>
                                {num(baseline.yield_total_value, f)} &rarr; {num(planned.yield_total_value, f)} <Delta from={baseline.yield_total_value} to={planned.yield_total_value} />
                            </Row>
                            <Row label={t("profile.base.nerds.roomsRestaffed")}>{diffs.length}</Row>
                        </Section>
                    )}

                    {rotation?.sustainability && (
                        <Section title={t("profile.base.nerds.rotation")}>
                            <Row label={t("profile.base.nerds.verdict")}>{rotation.sustainability.verdict === "holds_up" ? t("profile.base.sustain.holds") : t("profile.base.sustain.depletes")}</Row>
                            <Row label={t("profile.base.nerds.simulated")}>{t("profile.base.nerds.days", { days: Math.round(rotation.sustainability.horizon_hours / 24) })}</Row>
                            <Row label={t("profile.base.nerds.depleting")}>{rotation.sustainability.depleted.length}</Row>
                            <Row label={t("profile.base.nerds.dormOverflow")}>{rotation.sustainability.dorm_overflow}</Row>
                        </Section>
                    )}

                    {dorms.per_dorm && dorms.per_dorm.length > 0 && (
                        <div className="md:col-span-2">
                            <Section title={t("profile.base.nerds.dormList")}>
                                {dorms.per_dorm.map((dorm) => (
                                    <div className="flex items-baseline gap-3 py-1 text-[11.5px]" key={dorm.slot_id}>
                                        <span className="min-w-0 flex-1 truncate text-muted-foreground">{dorm.slot_id}</span>
                                        <span className="font-mono tabular-nums">{t("profile.base.nerds.dormLevel", { level: dorm.level })}</span>
                                        <span className="w-16 text-right font-mono text-muted-foreground tabular-nums">{t("profile.base.nerds.dormBeds", { count: dorm.capacity })}</span>
                                        <span className="w-16 text-right font-mono tabular-nums">{t("profile.base.nerds.dormRecovery", { rate: dec(dorm.recovery_per_hour) })}</span>
                                        <span className="w-24 text-right font-mono text-muted-foreground tabular-nums">
                                            {dorm.occupant_aura_per_hour > 0 ? t("profile.base.nerds.dormAura", { rate: dec(dorm.occupant_aura_per_hour) }) : dorm.occupant_single_per_hour > 0 ? t("profile.base.nerds.dormSingle", { rate: dec(dorm.occupant_single_per_hour) }) : "-"}
                                        </span>
                                    </div>
                                ))}
                            </Section>
                        </div>
                    )}

                    {timeline.length > 0 && (
                        <div className="md:col-span-2">
                            <Section title={t("profile.base.nerds.moraleOverTime")}>
                                <MoraleTimeline catalog={api.catalog} timeline={timeline} />
                            </Section>
                        </div>
                    )}

                    {diffs.length > 0 && (
                        <div className="md:col-span-2">
                            <Section title={t("profile.base.nerds.perRoomChanges")}>
                                {diffs.map((diff) => (
                                    <div className="flex items-baseline gap-3 py-1 text-[11.5px]" key={diff.slot_id}>
                                        <span className="min-w-0 flex-1 truncate">{roomLabel(diff.room_type, api.catalog)}</span>
                                        <span className="font-mono text-muted-foreground tabular-nums">
                                            {Math.round(diff.efficiency_before)}% &rarr; {Math.round(diff.efficiency_after)}%
                                        </span>
                                        <span className="w-24 text-right font-mono tabular-nums">
                                            {/* Every listed room has a changed crew; a dormitory or an
                                                equal-output swap still reads 0% -> 0%, and "no change"
                                                there hid the plan's dorm seating from a player copying
                                                it 1:1. */}
                                            {Math.round(diff.efficiency_before) === Math.round(diff.efficiency_after) ? <span className="text-muted-foreground">{t("profile.base.nerds.crewChanged")}</span> : <Delta from={diff.efficiency_before} suffix="%" to={diff.efficiency_after} />}
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
