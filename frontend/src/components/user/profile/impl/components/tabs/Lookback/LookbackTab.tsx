import { useQuery } from "@tanstack/react-query";
import { GitCompareArrows, History } from "lucide-react";
import { type ReactNode, useMemo, useState } from "react";
import { CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { type LookbackBuildRef, lookbackBuildQueryOptions, lookbackQueryOptions } from "#/lib/api/lookback";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, RARITY_HEX, rarityToNumber } from "#/lib/utils";
import type { LookbackSnapshot } from "#/types/generated/LookbackSnapshot";
import type { IOperatorIndexEntry } from "#/types/operators";
import { CARD_PADDING, KICKER_TEXT, Kicker, StatCard } from "../Stats/primitives";
import { diffBuilds, type IBuildDiff, type ISeriesPoint, METRICS, type Metric, metricSeries, ownershipSeries, summaryDelta } from "./helpers";
import type { messages } from "./LookbackTab.messages";

interface IProps {
    uid: string;
    server: string;
    operatorsIndex: IOperatorIndexEntry[];
    /** The Roster tab is hidden from this reader, which refuses the snapshots too. */
    rosterPrivate: boolean;
}

const ACCENT = "var(--primary)";
const RECONSTRUCTED = "var(--muted-foreground)";
const TICK_DATE: Intl.DateTimeFormatOptions = { month: "short", year: "2-digit" };
const LONG_DATE: Intl.DateTimeFormatOptions = { dateStyle: "medium" };

type T = TypedT<typeof messages>;

function useMetricLabels(t: T): Record<Metric, string> {
    // Literal keys, one per metric, so the extractor sees every one.
    return useMemo(
        () => ({
            operators: t("lookback.metric.operators"),
            elite2: t("lookback.metric.elite2"),
            max_level: t("lookback.metric.max_level"),
            mastery3: t("lookback.metric.mastery3"),
            modules: t("lookback.metric.modules"),
            skins: t("lookback.metric.skins"),
        }),
        [t],
    );
}

/**
 * The account's build over time: a chart of the snapshot counts (with the
 * owned-operator line rebuilt from obtain dates behind it), and a comparison
 * of any two snapshots, or a snapshot against the live roster.
 */
export function LookbackTab({ uid, server, operatorsIndex, rosterPrivate }: IProps) {
    const t: T = useT("user");
    const { data: overview, isLoading } = useQuery({ ...lookbackQueryOptions(uid), enabled: !rosterPrivate });
    const { data: live } = useQuery({ ...lookbackBuildQueryOptions(uid, "current"), enabled: !rosterPrivate && Boolean(overview) });

    if (rosterPrivate || (!isLoading && overview === null)) return <Notice>{t("lookback.private")}</Notice>;
    if (!overview) return <Notice>{t("lookback.loading")}</Notice>;

    const liveSummary = live ? { ts: new Date(live.taken_at).getTime(), summary: live.summary } : null;
    return (
        <div className="grid gap-3.5">
            <TimelineCard snapshots={overview.snapshots} ownership={ownershipSeries(overview.ownership)} live={liveSummary} />
            <CompareCard uid={uid} server={server} snapshots={overview.snapshots} operatorsIndex={operatorsIndex} />
        </div>
    );
}

function Notice({ children }: { children: ReactNode }) {
    return <p className="rounded-2xl border border-border/60 border-dashed bg-muted/15 px-4 py-6 text-center text-muted-foreground text-sm">{children}</p>;
}

interface ITimelineProps {
    snapshots: LookbackSnapshot[];
    ownership: ISeriesPoint[];
    live: Parameters<typeof metricSeries>[2];
}

function TimelineCard({ snapshots, ownership, live }: ITimelineProps) {
    const t: T = useT("user");
    const f = useFormatters();
    const labels = useMetricLabels(t);
    const [metric, setMetric] = useState<Metric>("operators");
    const recorded = useMemo(() => metricSeries(snapshots, metric, live), [snapshots, metric, live]);
    // The rebuilt line is ownership only, so it backs the Operators metric alone.
    const rebuilt = metric === "operators" ? ownership : [];

    return (
        <StatCard color={ACCENT}>
            <div className={`${CARD_PADDING} flex flex-col gap-4`}>
                <div className="flex flex-wrap items-center justify-between gap-2">
                    <Kicker icon={History} label={t("lookback.timeline.title")} />
                    <fieldset className="flex flex-wrap gap-1" aria-label={t("lookback.timeline.title")}>
                        {METRICS.map((m) => (
                            <button
                                key={m}
                                type="button"
                                aria-pressed={m === metric}
                                onClick={() => setMetric(m)}
                                className={cn("rounded-md border px-2 py-0.5 text-[11px] transition-colors", m === metric ? "border-primary/50 bg-primary/10 text-foreground" : "border-border/60 text-muted-foreground hover:text-foreground")}
                            >
                                {labels[m]}
                            </button>
                        ))}
                    </fieldset>
                </div>
                {recorded.length === 0 && rebuilt.length === 0 ? (
                    <p className="rounded-md border border-border/40 border-dashed bg-muted/15 px-3 py-2 text-[11px] text-muted-foreground">{t("lookback.empty")}</p>
                ) : (
                    <div className="h-52">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart margin={{ top: 4, right: 8, bottom: 0, left: -18 }}>
                                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" strokeOpacity={0.4} vertical={false} />
                                <XAxis dataKey="ts" type="number" scale="time" domain={["dataMin", "dataMax"]} allowDuplicatedCategory={false} tickFormatter={(ts: number) => f.date(ts, TICK_DATE)} tick={{ fontSize: 10, fill: "var(--muted-foreground)" }} tickLine={false} axisLine={false} minTickGap={40} />
                                <YAxis domain={["auto", "auto"]} allowDecimals={false} tickFormatter={(v: number) => f.number(v)} tick={{ fontSize: 10, fill: "var(--muted-foreground)" }} tickLine={false} axisLine={false} width={46} />
                                <Tooltip
                                    content={({ active, payload }) => {
                                        const p = active && payload?.[0]?.payload ? (payload[0].payload as ISeriesPoint) : null;
                                        if (!p) return null;
                                        return (
                                            <div className="rounded-md border border-border/60 bg-popover px-2.5 py-1.5 text-[11px] shadow-md">
                                                <div className="font-mono text-muted-foreground tabular-nums">{f.date(p.ts, LONG_DATE)}</div>
                                                {p.label && <div className="text-muted-foreground">{p.label}</div>}
                                                <div className="font-semibold tabular-nums">
                                                    {labels[metric]}: {f.number(p.value)}
                                                </div>
                                            </div>
                                        );
                                    }}
                                />
                                {rebuilt.length > 0 && <Line data={rebuilt} name={t("lookback.series.reconstructed")} type="stepAfter" dataKey="value" stroke={RECONSTRUCTED} strokeWidth={1.5} strokeDasharray="4 3" dot={false} activeDot={false} isAnimationActive={false} />}
                                <Line data={recorded} name={t("lookback.series.recorded")} type="monotone" dataKey="value" stroke={ACCENT} strokeWidth={2} dot={{ r: 2.5, fill: ACCENT, strokeWidth: 0 }} activeDot={{ r: 4 }} isAnimationActive={false} />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                )}
                <div className="flex flex-col gap-1.5 text-[11px] text-muted-foreground">
                    <div className="flex flex-wrap gap-x-4 gap-y-1">
                        <LegendSwatch color={ACCENT} label={t("lookback.series.recorded")} />
                        {rebuilt.length > 0 && <LegendSwatch color={RECONSTRUCTED} dashed label={t("lookback.series.reconstructed")} />}
                    </div>
                    {rebuilt.length > 0 && <p>{t("lookback.reconstructed.note")}</p>}
                    {snapshots.length === 0 && recorded.length + rebuilt.length > 0 && <p>{t("lookback.empty")}</p>}
                </div>
            </div>
        </StatCard>
    );
}

function LegendSwatch({ color, label, dashed }: { color: string; label: string; dashed?: boolean }) {
    return (
        <span className="inline-flex items-center gap-1.5">
            <span aria-hidden className="inline-block w-4 border-t-2" style={{ borderColor: color, borderStyle: dashed ? "dashed" : "solid" }} />
            {label}
        </span>
    );
}

interface ICompareProps {
    uid: string;
    server: string;
    snapshots: LookbackSnapshot[];
    operatorsIndex: IOperatorIndexEntry[];
}

/** A picker value: a snapshot id as a string, or "current". */
function refOf(value: string): LookbackBuildRef {
    return value === "current" ? "current" : Number(value);
}

function CompareCard({ uid, server, snapshots, operatorsIndex }: ICompareProps) {
    const t: T = useT("user");
    const f = useFormatters();
    const labels = useMetricLabels(t);
    const [fromPick, setFromPick] = useState<string | null>(null);
    const [toPick, setToPick] = useState<string>("current");
    // The earliest snapshot against now, until the reader picks.
    const from = fromPick ?? (snapshots.length > 0 ? String(snapshots[0].id) : null);
    const same = from === toPick;

    const before = useQuery({ ...lookbackBuildQueryOptions(uid, refOf(from ?? "current")), enabled: from !== null && !same });
    const after = useQuery({ ...lookbackBuildQueryOptions(uid, refOf(toPick)), enabled: from !== null && !same });

    const options = useMemo(() => {
        const name = (s: LookbackSnapshot) => {
            const date = f.date(s.period_start, LONG_DATE);
            return s.activity_name ? t("lookback.compare.event", { event: s.activity_name, date }) : t("lookback.compare.week", { date });
        };
        return [...snapshots.map((s) => ({ value: String(s.id), label: name(s) })), { value: "current", label: t("lookback.compare.now") }];
    }, [snapshots, f, t]);
    const labelOf = (value: string | null) => options.find((o) => o.value === value)?.label ?? "";

    const diff = useMemo(() => (before.data && after.data ? diffBuilds(before.data.operators, after.data.operators) : null), [before.data, after.data]);
    const delta = before.data && after.data ? summaryDelta(before.data.summary, after.data.summary) : null;
    const grades = before.data?.summary.grade && after.data?.summary.grade ? { from: before.data.summary.grade, to: after.data.summary.grade } : null;
    const byId = useMemo(() => new Map(operatorsIndex.map((op) => [op.id, op])), [operatorsIndex]);

    return (
        <StatCard color={ACCENT}>
            <div className={`${CARD_PADDING} flex flex-col gap-4`}>
                <Kicker icon={GitCompareArrows} label={t("lookback.compare.title")} />
                {from === null ? (
                    <p className="rounded-md border border-border/40 border-dashed bg-muted/15 px-3 py-2 text-[11px] text-muted-foreground">{t("lookback.empty")}</p>
                ) : (
                    <>
                        <div className="grid gap-2 sm:grid-cols-2">
                            <Picker label={t("lookback.compare.from")} value={from} options={options} current={labelOf(from)} onChange={setFromPick} />
                            <Picker label={t("lookback.compare.to")} value={toPick} options={options} current={labelOf(toPick)} onChange={setToPick} />
                        </div>
                        {same ? (
                            <p className="text-[11px] text-muted-foreground">{t("lookback.diff.same")}</p>
                        ) : (
                            delta &&
                            diff && (
                                <>
                                    <div className="grid grid-cols-3 gap-2 sm:grid-cols-6">
                                        {METRICS.map((m) => (
                                            <div key={m} className="rounded-lg border border-border/50 bg-muted/10 px-2.5 py-2">
                                                <div className={KICKER_TEXT}>{labels[m]}</div>
                                                <div className={cn("font-mono font-semibold text-lg tabular-nums", delta[m] > 0 ? "text-emerald-500" : "text-muted-foreground")}>
                                                    {delta[m] > 0 ? "+" : ""}
                                                    {f.number(delta[m])}
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                    {grades && grades.from !== grades.to && <span className="w-fit rounded-md border border-border/60 px-2 py-0.5 font-mono text-[11px]">{t("lookback.compare.grade", grades)}</span>}
                                    <DiffLists diff={diff} byId={byId} server={server} />
                                </>
                            )
                        )}
                    </>
                )}
            </div>
        </StatCard>
    );
}

function Picker({ label, value, options, current, onChange }: { label: string; value: string; options: { value: string; label: string }[]; current: string; onChange: (next: string) => void }) {
    return (
        <div className="flex flex-col gap-1">
            <span className={KICKER_TEXT} aria-hidden>
                {label}
            </span>
            <Select
                value={value}
                onValueChange={(next: string | null) => {
                    if (next !== null) onChange(next);
                }}
            >
                <SelectTrigger size="sm" aria-label={label}>
                    <SelectValue>{() => current}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                    {options.map((o) => (
                        <SelectItem key={o.value} value={o.value}>
                            {o.label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    );
}

function DiffLists({ diff, byId, server }: { diff: IBuildDiff; byId: Map<string, IOperatorIndexEntry>; server: string }) {
    const t: T = useT("user");
    return (
        <div className="grid gap-4 md:grid-cols-2">
            <DiffSection title={t("lookback.diff.gained")} count={diff.gained.length}>
                {diff.gained.map((id) => (
                    <OperatorRow key={id} id={id} byId={byId} server={server} />
                ))}
            </DiffSection>
            <DiffSection title={t("lookback.diff.elite2")} count={diff.elite2.length}>
                {diff.elite2.map((id) => (
                    <OperatorRow key={id} id={id} byId={byId} server={server} />
                ))}
            </DiffSection>
            <DiffSection title={t("lookback.diff.masteries")} count={diff.masteries.length}>
                {diff.masteries.map((m) => (
                    <OperatorRow key={`${m.operatorId}:${m.skill}`} id={m.operatorId} byId={byId} server={server} detail={t("lookback.diff.mastery", { skill: m.skill + 1, from: m.from, to: m.to })} />
                ))}
            </DiffSection>
            <DiffSection title={t("lookback.diff.modules")} count={diff.modules.length}>
                {diff.modules.map((m) => (
                    <OperatorRow key={`${m.operatorId}:${m.moduleId}`} id={m.operatorId} byId={byId} server={server} detail={m.from === 0 ? t("lookback.diff.moduleNew", { to: m.to }) : t("lookback.diff.module", { from: m.from, to: m.to })} />
                ))}
            </DiffSection>
        </div>
    );
}

function DiffSection({ title, count, children }: { title: string; count: number; children: ReactNode }) {
    const t: T = useT("user");
    const f = useFormatters();
    return (
        <section className="flex flex-col gap-2">
            <h3 className="flex items-baseline gap-2 font-semibold text-sm">
                {title}
                <span className="font-mono text-muted-foreground text-xs tabular-nums">{f.number(count)}</span>
            </h3>
            {count === 0 ? <p className="text-[11px] text-muted-foreground">{t("lookback.diff.none")}</p> : <ul className="flex max-h-72 flex-col gap-1 overflow-y-auto pr-1">{children}</ul>}
        </section>
    );
}

function OperatorRow({ id, byId, server, detail }: { id: string; byId: Map<string, IOperatorIndexEntry>; server: string; detail?: string }) {
    const op = byId.get(id);
    const name = op?.name ?? id;
    const color = RARITY_HEX[rarityToNumber(op?.rarity)];
    return (
        <li className="flex items-center gap-2 text-sm">
            <span className="relative size-7 shrink-0 overflow-hidden rounded-md bg-muted" style={{ boxShadow: `inset 0 -2px 0 ${color}` }}>
                <OperatorAvatar charId={id} name={name} server={server} />
            </span>
            <span className="min-w-0 flex-1 truncate">{name}</span>
            {detail && <span className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{detail}</span>}
        </li>
    );
}
