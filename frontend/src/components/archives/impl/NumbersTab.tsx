import { useQuery } from "@tanstack/react-query";
import { ChevronDownIcon } from "lucide-react";
import type * as React from "react";
import { useMemo, useState } from "react";
import { Bar, BarChart, Tooltip as ChartTooltip, ResponsiveContainer, XAxis } from "recharts";
import { Button } from "#/components/ui/button";
import { Kicker } from "#/components/ui/kicker";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { archiveDemographicsQueryOptions } from "#/lib/api/archives";
import { type IFormatters, useFormatters, useGamedataServer } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import type { ArchiveCount } from "#/types/generated/ArchiveCount";
import type { ArchiveDemographics } from "#/types/generated/ArchiveDemographics";
import type { ArchiveDistribution } from "#/types/generated/ArchiveDistribution";
import type { ArchiveInfectionRow } from "#/types/generated/ArchiveInfectionRow";
import type { ArchivePerson } from "#/types/generated/ArchivePerson";
import type { ArchiveScaleStep } from "#/types/generated/ArchiveScaleStep";
import { type ArchivesT, AXIS_KEY, BigStat, CLEAR_COLOR, INFECTED_COLOR, INFECTION_KEY, Panel, PersonChip, peopleById, SectionHead, STEP_KEY, StatStrip, TabMessage, useArchivesT } from "./shared";

/** Rows a long list shows before "Show all". */
const LIST_SHOWN = 14;
/** Names listed at a distribution's low or high end before "and N others". */
const NAMES_SHOWN = 3;
/** The ratings note names the fewest steps that cover at least this share. */
const RATING_COVER = 0.95;

const ROMAN = ["I", "II", "III", "IV", "V"] as const;

export function NumbersTab({ onOpenFile }: { onOpenFile: (id: string) => void }): React.ReactElement {
    const t = useArchivesT();
    const f = useFormatters();
    const server = useGamedataServer();
    const { data, isLoading, isError } = useQuery(archiveDemographicsQueryOptions(server));
    if (isError) return <TabMessage>{t("page.error")}</TabMessage>;
    if (isLoading || !data) return <TabMessage>{t("page.loading")}</TabMessage>;
    return <Numbers d={data} t={t} f={f} onOpenFile={onOpenFile} />;
}

function Numbers({ d, t, f, onOpenFile }: { d: ArchiveDemographics; t: ArchivesT; f: IFormatters; onOpenFile: (id: string) => void }): React.ReactElement {
    const people = useMemo(() => peopleById(d.people), [d.people]);
    const sections = [
        { id: "arc-physical", title: t("numbers.s1.title"), blurb: t("numbers.s1.blurb") },
        { id: "arc-origin", title: t("numbers.s2.title"), blurb: t("numbers.s2.blurb") },
        { id: "arc-originium", title: t("numbers.s3.title"), blurb: t("numbers.s3.blurb") },
        { id: "arc-assessment", title: t("numbers.s4.title"), blurb: t("numbers.s4.blurb") },
        { id: "arc-paperwork", title: t("numbers.s5.title"), blurb: t("numbers.s5.blurb") },
    ];
    const unit = {
        cm: (v: number) => `${f.number(v)}cm`,
        years: (v: number) => t("unit.years", { count: v }),
        ul: (v: number) => `${f.number(v)}u/L`,
        pct: (v: number) => `${f.number(v)}%`,
    };
    const manufactured = sumCounts(d.manufactured);

    return (
        <div className="flex flex-col gap-6">
            <header className="flex flex-col gap-3">
                <Kicker className="mb-0">{t("numbers.kicker")}</Kicker>
                <p className="m-0 max-w-3xl text-[13.5px] text-muted-foreground leading-normal">{t("numbers.lede", { total: f.number(d.total) })}</p>
                <StatStrip>
                    <BigStat label={t("numbers.stat.files")} value={f.number(d.total)} grow />
                    <BigStat label={t("numbers.stat.races")} value={f.number(d.race.length)} grow />
                    <BigStat label={t("numbers.stat.birthplaces")} value={f.number(d.birthplace.length)} grow />
                    <BigStat label={t("numbers.stat.prefixes")} value={f.number(d.prefixes.length)} grow />
                </StatStrip>
                <nav aria-label={t("numbers.jump")} className="flex flex-wrap gap-1.5">
                    {sections.map((s, i) => (
                        <a key={s.id} href={`#${s.id}`} className="inline-flex pointer-coarse:min-h-11 items-center gap-1.5 rounded-md border border-border bg-card px-2.5 py-1 text-[12px] text-foreground no-underline hover:bg-accent">
                            <span className="font-bold font-mono text-primary">{ROMAN[i]}</span>
                            {s.title}
                        </a>
                    ))}
                </nav>
            </header>

            <Section index={0} {...sections[0]}>
                <div className="grid grid-cols-2 gap-4 max-[900px]:grid-cols-1">
                    <DistributionCard title={t("numbers.height")} dist={d.height} people={people} format={unit.cm} t={t} f={f} onOpenFile={onOpenFile} />
                    <DistributionCard title={t("numbers.combat")} dist={d.combat} people={people} format={unit.years} t={t} f={f} onOpenFile={onOpenFile} />
                </div>
                <HeightByRace d={d} t={t} f={f} format={unit.cm} />
            </Section>

            <Section index={1} {...sections[1]}>
                <div className="grid grid-cols-3 gap-4 max-[1100px]:grid-cols-2 max-[700px]:grid-cols-1">
                    <Panel title={t("numbers.race")}>
                        <BarList rows={d.race} total={d.total} t={t} f={f} />
                    </Panel>
                    <Panel title={t("numbers.birthplace")}>
                        <BarList rows={d.birthplace} total={d.total} t={t} f={f} />
                    </Panel>
                    <Panel title={t("numbers.gender")}>
                        <BarList rows={withNotRecorded(genderRows(d), d.total, t)} total={d.total} t={t} f={f} />
                    </Panel>
                </div>
            </Section>

            <Section index={2} {...sections[2]}>
                <div className="grid grid-cols-2 gap-4 max-[900px]:grid-cols-1">
                    <DistributionCard title={t("numbers.blood")} dist={d.blood} people={people} format={unit.ul} t={t} f={f} onOpenFile={onOpenFile} />
                    <DistributionCard title={t("numbers.assimilation")} dist={d.assimilation} people={people} format={unit.pct} t={t} f={f} onOpenFile={onOpenFile} />
                </div>
                <div className="grid grid-cols-3 gap-4 max-[1100px]:grid-cols-1">
                    <Panel title={t("numbers.infection")} footer={t("numbers.infectionNote")}>
                        <BarList rows={d.infection.map((r) => ({ value: t(INFECTION_KEY[r.value as keyof typeof INFECTION_KEY] ?? "infection.unclear"), count: r.count }))} total={d.total} t={t} f={f} />
                    </Panel>
                    <Panel title={t("numbers.infectionByBirthplace")} footer={t("numbers.infectionByNote", { threshold: d.minGroup })}>
                        <InfectionBars rows={d.infectionByBirthplace} t={t} f={f} />
                    </Panel>
                    <Panel title={t("numbers.infectionByRace")} footer={t("numbers.infectionByNote", { threshold: d.minGroup })}>
                        <InfectionBars rows={d.infectionByRace} t={t} f={f} />
                    </Panel>
                </div>
            </Section>

            <Section index={3} {...sections[3]}>
                <div className="grid grid-cols-[2fr_1fr] gap-4 max-[1000px]:grid-cols-1">
                    <ExamCard d={d} t={t} f={f} />
                    <RatingsCard d={d} t={t} f={f} />
                </div>
            </Section>

            <Section index={4} {...sections[4]}>
                <div className="grid grid-cols-3 gap-4 max-[1100px]:grid-cols-2 max-[700px]:grid-cols-1">
                    <Panel title={t("numbers.months")}>
                        <BarList rows={d.birthMonths.map((count, i) => ({ value: f.date(new Date(2000, i, 1), { month: "short" }), count }))} total={d.dated} t={t} f={f} />
                    </Panel>
                    <Panel title={t("numbers.days")} footer={t("numbers.daysNote", { dated: f.number(d.dated) })}>
                        <BarList rows={d.birthDays.map((x) => ({ value: f.date(new Date(2000, x.month - 1, x.day), { month: "short", day: "numeric" }), count: x.count }))} total={d.dated} t={t} f={f} />
                    </Panel>
                    <Panel title={t("numbers.manufactured")}>
                        <div className="flex items-baseline gap-2">
                            <span className="font-bold text-[34px] text-foreground tabular-nums leading-none">{f.number(manufactured)}</span>
                            <span className="text-[12px] text-muted-foreground">{t("numbers.manufacturedTotal", { total: f.number(d.total) })}</span>
                        </div>
                        <BarList rows={d.manufactured} total={manufactured} t={t} f={f} />
                    </Panel>
                </div>
                <Prefixes d={d} people={people} t={t} f={f} onOpenFile={onOpenFile} />
            </Section>
        </div>
    );
}

/** Gender bars: the server's own word for each gender (女, 여, Female), else the file's word as written. */
function genderRows(d: ArchiveDemographics): ArchiveCount[] {
    return d.gender.map((g) => ({ value: g.value, count: g.count }));
}

/** A scale step's label: the word this server's files print for it, else the site's translation of its key. */
function stepWord(step: ArchiveScaleStep, t: ArchivesT): string {
    return step.word ?? t(STEP_KEY[step.key]);
}

function sumCounts(rows: readonly ArchiveCount[]): number {
    return rows.reduce((s, r) => s + r.count, 0);
}

function withNotRecorded(rows: ArchiveCount[], total: number, t: ArchivesT): ArchiveCount[] {
    const missing = total - sumCounts(rows);
    return missing > 0 ? [...rows, { value: t("numbers.notRecorded"), count: missing }].sort((a, b) => b.count - a.count) : rows;
}

function Section({ id, index, title, blurb, children }: { id: string; index: number; title: string; blurb: string; children: React.ReactNode }): React.ReactElement {
    return (
        <section id={id} className="flex scroll-mt-20 flex-col gap-4">
            <SectionHead index={ROMAN[index]} title={title} blurb={blurb} />
            {children}
        </section>
    );
}

/** The first {@link LIST_SHOWN} of `rows` until the reader asks for all of them. */
function useShowAll<T>(rows: readonly T[]): { shown: readonly T[]; all: boolean; toggle: () => void } {
    const [all, setAll] = useState(false);
    return { shown: all ? rows : rows.slice(0, LIST_SHOWN), all, toggle: () => setAll((v) => !v) };
}

/** "Show all N" / "Show fewer" under a list longer than {@link LIST_SHOWN}; nothing for a shorter one. */
function ShowAllButton({ count, all, onToggle, t }: { count: number; all: boolean; onToggle: () => void; t: ArchivesT }): React.ReactElement | null {
    if (count <= LIST_SHOWN) return null;
    return (
        <Button size="xs" variant="ghost" className="self-start" onClick={onToggle}>
            {all ? t("numbers.showFewer") : t("numbers.showAll", { count })}
        </Button>
    );
}

/** A colour key entry: a small square, then its label. */
function Swatch({ color, className, children }: { color?: string; className?: string; children: React.ReactNode }): React.ReactElement {
    return (
        <span className="inline-flex items-center gap-1">
            <span className={cn("size-2 rounded-sm", className)} style={color ? { background: color } : undefined} />
            {children}
        </span>
    );
}

function BarList({ rows, total, t, f }: { rows: ArchiveCount[]; total: number; t: ArchivesT; f: IFormatters }): React.ReactElement {
    const { shown, all, toggle } = useShowAll(rows);
    const max = Math.max(1, ...rows.map((r) => r.count));
    return (
        <div className="flex flex-col gap-1">
            {shown.map((r) => (
                <div key={r.value} className="grid grid-cols-[minmax(0,7.5rem)_1fr_auto_3rem] items-center gap-2 text-[12.5px]">
                    <span className="truncate text-foreground" title={r.value}>
                        {r.value}
                    </span>
                    <span className="h-2 overflow-hidden rounded-sm bg-muted">
                        <span className="block h-full rounded-sm bg-primary" style={{ width: `${(r.count / max) * 100}%` }} />
                    </span>
                    <span className="text-right text-foreground tabular-nums">{f.number(r.count)}</span>
                    <span className="text-right text-muted-foreground tabular-nums">{total > 0 ? f.percent(r.count / total) : ""}</span>
                </div>
            ))}
            <ShowAllButton count={rows.length} all={all} onToggle={toggle} t={t} />
        </div>
    );
}

function names(ids: string[], people: Map<string, ArchivePerson>, t: ArchivesT): string {
    const list = ids.slice(0, NAMES_SHOWN).map((id) => people.get(id)?.name ?? id);
    const rest = ids.length - list.length;
    return rest > 0 ? t("numbers.andOthers", { names: list.join(", "), count: rest }) : list.join(", ");
}

function DistributionCard({ title, dist, people, format, t, f, onOpenFile }: { title: string; dist: ArchiveDistribution; people: Map<string, ArchivePerson>; format: (v: number) => string; t: ArchivesT; f: IFormatters; onOpenFile: (id: string) => void }): React.ReactElement {
    const data = dist.buckets.map((b) => ({ label: format(b.from), from: format(b.from), to: format(b.to), count: b.count }));
    const ends = (label: string, value: number, ids: string[]) => (
        <div className="flex min-w-0 flex-col gap-0.5">
            <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{label}</span>
            <span className="font-semibold text-[18px] text-foreground tabular-nums">{format(value)}</span>
            {ids.length === 1 ? (
                <PersonChip person={people.get(ids[0])} onOpen={onOpenFile} className="self-start" />
            ) : ids.length > 1 ? (
                <Tooltip>
                    <TooltipTrigger render={<span className="truncate text-[12px] text-muted-foreground" />}>{names(ids, people, t)}</TooltipTrigger>
                    <TooltipPopup className="max-w-80">{ids.map((id) => people.get(id)?.name ?? id).join(", ")}</TooltipPopup>
                </Tooltip>
            ) : null}
        </div>
    );
    return (
        <Panel title={title} footer={t("numbers.recordedFor", { count: dist.counted })}>
            <div className="h-36 w-full">
                <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={data} margin={{ top: 4, right: 0, bottom: 0, left: 0 }} barCategoryGap={1}>
                        <XAxis dataKey="label" interval="preserveStartEnd" tick={{ fontSize: 10, fill: "var(--muted-foreground)" }} tickLine={false} axisLine={{ stroke: "var(--border)" }} />
                        <ChartTooltip
                            cursor={{ fill: "var(--accent)" }}
                            content={({ active, payload }) => {
                                const row = active ? (payload?.[0]?.payload as (typeof data)[number] | undefined) : undefined;
                                return row ? <div className="rounded-md border border-border bg-popover px-2 py-1 text-[12px] text-foreground shadow-sm">{t("numbers.histogramLabel", { from: row.from, to: row.to, count: f.number(row.count) })}</div> : null;
                            }}
                        />
                        <Bar dataKey="count" fill="var(--primary)" isAnimationActive={false} />
                    </BarChart>
                </ResponsiveContainer>
            </div>
            <div className="grid grid-cols-3 gap-3">
                {ends(t("numbers.lowest"), dist.min, dist.low)}
                <div className="flex flex-col gap-0.5">
                    <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{t("numbers.average")}</span>
                    <span className="font-semibold text-[18px] text-foreground tabular-nums">{format(dist.mean)}</span>
                </div>
                {ends(t("numbers.highest"), dist.max, dist.high)}
            </div>
        </Panel>
    );
}

function HeightByRace({ d, t, f, format }: { d: ArchiveDemographics; t: ArchivesT; f: IFormatters; format: (v: number) => string }): React.ReactElement | null {
    const rows = d.heightByRace;
    if (rows.length === 0) return null;
    // The track spans the lowest to the highest average, rounded out to whole centimetres.
    const lo = Math.floor(Math.min(...rows.map((r) => r.mean)));
    const hi = Math.ceil(Math.max(...rows.map((r) => r.mean)));
    const span = Math.max(1, hi - lo);
    return (
        <Panel title={t("numbers.heightByRace")} footer={t("numbers.heightByRaceNote", { min: format(lo), max: format(hi), threshold: d.minGroup })}>
            <div className="grid grid-cols-3 gap-x-6 gap-y-1.5 max-[1000px]:grid-cols-2 max-[600px]:grid-cols-1">
                {rows.map((r) => (
                    <div key={r.value} className="grid grid-cols-[minmax(0,6rem)_1fr_auto_2rem] items-center gap-2 text-[12.5px]">
                        <span className="truncate text-foreground">{r.value}</span>
                        <span className="relative h-1 rounded-full bg-muted">
                            <span className="absolute top-1/2 size-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary" style={{ left: `${((r.mean - lo) / span) * 100}%` }} />
                        </span>
                        <span className="text-right text-foreground tabular-nums">{format(r.mean)}</span>
                        <span className="text-right text-muted-foreground tabular-nums">{f.number(r.counted)}</span>
                    </div>
                ))}
            </div>
        </Panel>
    );
}

function InfectionBars({ rows, t, f }: { rows: ArchiveInfectionRow[]; t: ArchivesT; f: IFormatters }): React.ReactElement {
    const { shown, all, toggle } = useShowAll(rows);
    return (
        <div className="flex flex-col gap-1">
            <div className="mb-1 flex gap-3 text-[11px] text-muted-foreground">
                <Swatch color={INFECTED_COLOR}>{t("infection.infected")}</Swatch>
                <Swatch color={CLEAR_COLOR}>{t("infection.clear")}</Swatch>
            </div>
            {shown.map((r) => (
                <div key={r.value} className="grid grid-cols-[minmax(0,6.5rem)_1fr_auto_3rem] items-center gap-2 text-[12.5px]">
                    <span className="truncate text-foreground">{r.value}</span>
                    <span className="flex h-2 overflow-hidden rounded-sm bg-muted">
                        <span className="block h-full" style={{ width: `${(r.infected / r.total) * 100}%`, background: INFECTED_COLOR }} />
                        <span className="block h-full flex-1" />
                        <span className="block h-full" style={{ width: `${(r.clear / r.total) * 100}%`, background: CLEAR_COLOR }} />
                    </span>
                    <span className="text-right text-muted-foreground tabular-nums">
                        {f.number(r.infected)}/{f.number(r.total)}
                    </span>
                    <span className="text-right text-foreground tabular-nums">{f.percent(r.infected / r.total)}</span>
                </div>
            ))}
            <ShowAllButton count={rows.length} all={all} onToggle={toggle} t={t} />
        </div>
    );
}

/** One shade per step, lowest darkest, from muted to full primary. */
function stepShade(step: number, steps: number): string {
    const pct = Math.round(15 + (85 * step) / Math.max(1, steps - 1));
    return `color-mix(in oklab, var(--primary) ${pct}%, var(--muted))`;
}

function ExamCard({ d, t, f }: { d: ArchiveDemographics; t: ArchivesT; f: IFormatters }): React.ReactElement {
    return (
        <Panel title={t("numbers.exam")} footer={t("numbers.examNote")}>
            <div className="flex flex-col gap-2">
                {d.exam.map((col) => {
                    const top = col.steps.reduce((best, n, i) => (n > col.steps[best] ? i : best), 0);
                    return (
                        <div key={col.axis} className="grid grid-cols-[5.5rem_1fr_auto] items-center gap-2 text-[12.5px]">
                            <span className="text-foreground">{t(AXIS_KEY[col.axis])}</span>
                            <span className="flex h-3 overflow-hidden rounded-sm bg-muted">
                                {col.steps.map((n, i) =>
                                    n > 0 && d.scale[i] ? (
                                        <Tooltip key={d.scale[i].key}>
                                            <TooltipTrigger render={<span className="block h-full" style={{ width: `${(n / d.total) * 100}%`, background: stepShade(i, d.scale.length) }} />} />
                                            <TooltipPopup>{`${stepWord(d.scale[i], t)}: ${f.number(n)}`}</TooltipPopup>
                                        </Tooltip>
                                    ) : null,
                                )}
                            </span>
                            <span className="text-right text-muted-foreground tabular-nums">
                                {d.scale[top] ? stepWord(d.scale[top], t) : ""} {f.percent(col.steps[top] / d.total)}
                            </span>
                        </div>
                    );
                })}
            </div>
            <div className="flex flex-wrap gap-x-3 gap-y-1 text-[11px] text-muted-foreground">
                {d.scale.map((step, i) => (
                    <Swatch key={step.key} color={stepShade(i, d.scale.length)}>
                        {stepWord(step, t)}
                    </Swatch>
                ))}
                <Swatch className="bg-muted">{t("numbers.unrecorded")}</Swatch>
            </div>
        </Panel>
    );
}

function RatingsCard({ d, t, f }: { d: ArchiveDemographics; t: ArchivesT; f: IFormatters }): React.ReactElement {
    const perStep = d.scale.map((_, i) => d.exam.reduce((s, col) => s + (col.steps[i] ?? 0), 0));
    const total = perStep.reduce((s, n) => s + n, 0);
    const ranked = [...perStep].sort((a, b) => b - a);
    let covered = 0;
    let top = 0;
    while (top < ranked.length && covered < total * RATING_COVER) {
        covered += ranked[top];
        top += 1;
    }
    const last = d.scale.length - 1;
    return (
        <Panel title={t("numbers.ratings")} footer={t("numbers.ratingsNote", { steps: d.scale.length, top, share: total > 0 ? f.percent(covered / total) : "", topWord: d.scale[last] ? stepWord(d.scale[last], t) : "", topCount: perStep[last] ?? 0 })}>
            <div className="flex items-baseline gap-2">
                <span className="font-bold text-[34px] text-foreground tabular-nums leading-none">{f.number(total)}</span>
                <span className="text-[12px] text-muted-foreground">{t("numbers.ratingsTotal", { files: f.number(d.total) })}</span>
            </div>
            <BarList rows={d.scale.map((step, i) => ({ value: stepWord(step, t), count: perStep[i] }))} total={total} t={t} f={f} />
        </Panel>
    );
}

function Prefixes({ d, people, t, f, onOpenFile }: { d: ArchiveDemographics; people: Map<string, ArchivePerson>; t: ArchivesT; f: IFormatters; onOpenFile: (id: string) => void }): React.ReactElement {
    const { shown, all, toggle } = useShowAll(d.prefixes);
    const [open, setOpen] = useState<string | null>(null);
    const counted = d.prefixes.reduce((s, p) => s + p.holders.length, 0);
    return (
        <Panel title={t("numbers.prefixes")} footer={t("numbers.prefixNote")}>
            <div className="flex flex-col">
                {shown.map((p) => {
                    const isOpen = open === p.prefix;
                    return (
                        <div key={p.prefix} className="border-border border-b last:border-b-0">
                            <button type="button" aria-expanded={isOpen} onClick={() => setOpen(isOpen ? null : p.prefix)} className="grid pointer-coarse:min-h-11 w-full cursor-pointer grid-cols-[3.5rem_1fr_auto_3.5rem_1rem] items-center gap-2 py-1.5 text-left text-[12.5px] hover:bg-accent/40">
                                <span className="font-bold font-mono text-primary">{p.prefix}</span>
                                <span className="truncate text-foreground">{p.faction?.name ?? ""}</span>
                                <span className="text-right text-foreground tabular-nums">{f.number(p.holders.length)}</span>
                                <span className="text-right text-muted-foreground tabular-nums">{f.percent(p.holders.length / counted)}</span>
                                <ChevronDownIcon aria-hidden className={cn("size-3.5 text-muted-foreground transition-transform", isOpen && "rotate-180")} />
                            </button>
                            {isOpen ? (
                                <div className="flex flex-col gap-2 pb-3">
                                    <div className="flex flex-wrap gap-1.5">
                                        {p.holders.map((h) => (
                                            <span key={h.charId} className="inline-flex items-center gap-1">
                                                <span className="font-mono text-[10.5px] text-muted-foreground">{h.number}</span>
                                                <PersonChip person={people.get(h.charId)} onOpen={onOpenFile} />
                                            </span>
                                        ))}
                                    </div>
                                    <p className="m-0 text-[12px] text-muted-foreground">{p.faction ? t("numbers.prefixShare", { shared: p.factionHolders, holders: p.holders.length, prefix: p.prefix, faction: p.faction.name }) : t("numbers.prefixNoFaction", { prefix: p.prefix })}</p>
                                </div>
                            ) : null}
                        </div>
                    );
                })}
            </div>
            <ShowAllButton count={d.prefixes.length} all={all} onToggle={toggle} t={t} />
        </Panel>
    );
}
