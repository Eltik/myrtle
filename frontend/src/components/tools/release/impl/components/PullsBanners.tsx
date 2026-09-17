import { useQuery } from "@tanstack/react-query";
import { Minus, Plus, RotateCcw } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { releaseEventsQueryOptions } from "#/lib/api/release";
import { useFormatters, useLocale, useT } from "#/lib/i18n";
import { cn, getAvatarById, rarityToNumber } from "#/lib/utils";
import type { AutoName } from "#/types/generated/AutoName";
import { useAutoTranslate } from "../autoTranslate";
import { formatDate } from "../helpers";
import { useReleaseTagLabel } from "../labels";
import type { IGoalEstimate } from "../pulls/odds";
import { MAX_POT_COPIES } from "../pulls/odds";
import type { IPlanRow, IPlanTotals } from "../pulls/plan";
import { planTargets } from "../pulls/plan";
import { cnDay } from "../schedule";
import { type PullsT, Stat, usePct } from "./PullsShared";
import { ResolutionBadge } from "./ResolutionBadge";
import { AutoTag, CnName, type OperatorLookup, operatorLabel, ReleaseEmpty, resolveName, SectionTitle, Tag, useArt } from "./shared";

/** Banner name, art and anchor event, exactly as the Banners and Planner tabs resolve them. */
type EventArt = Map<string, { cn: string; en: string | null; auto: AutoName | null; imagePath: string | null }>;

interface IPullsBannersProps {
    rows: IPlanRow[];
    totals: IPlanTotals;
    lookup: OperatorLookup;
    charNames: { [key in string]?: AutoName };
    today: Date;
    onAllocate: (key: string, pulls: number) => void;
    onSetTarget: (key: string, charId: string, copies: number) => void;
    onReset: () => void;
    onExport: () => void;
}

export function PullsBanners({ rows, totals, lookup, charNames, today, onAllocate, onSetTarget, onReset, onExport }: IPullsBannersProps): React.ReactElement {
    const t: PullsT = useT("tools");
    const f = useFormatters();
    const events = useQuery(releaseEventsQueryOptions());
    const planned = totals.allocated > 0;
    // Said once for the whole list rather than repeated on every row, which is what
    // made the caveat louder than the data it was qualifying.
    const inferred = rows.filter((r) => r.model.inferred).length;

    // A banner often ships no art of its own and borrows the event it is anchored to,
    // which is how the Banners and Planner tabs get a picture on every row.
    const eventArt = React.useMemo(() => {
        const map: EventArt = new Map();
        for (const e of events.data?.events ?? []) map.set(e.cnId, { cn: e.nameCn, en: e.nameEn, auto: e.nameEnAuto, imagePath: e.imagePath });
        return map;
    }, [events.data]);
    const eventsByDay = React.useMemo(() => {
        const map = new Map<string, string>();
        for (const e of events.data?.events ?? []) {
            if (e.hasStage && e.imagePath && !map.has(cnDay(e.cnStart))) map.set(cnDay(e.cnStart), e.cnId);
        }
        return map;
    }, [events.data]);

    return (
        <Card className="flex flex-col gap-3 p-4">
            <div className="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
                <div className="flex min-w-0 flex-col gap-1">
                    <SectionTitle count={rows.length}>{t("release.pulls.plan.title")}</SectionTitle>
                    <p className="m-0 max-w-2xl font-sans text-[12px] text-muted-foreground leading-normal">{t("release.pulls.plan.intro")}</p>
                    <p className="m-0 max-w-2xl font-sans text-[11.5px] text-muted-foreground leading-normal">{t("release.pulls.plan.pickHint")}</p>
                    {inferred > 0 && <p className="m-0 max-w-2xl font-sans text-[11.5px] text-amber-500/90 leading-normal">{t("release.pulls.plan.inferredCount", { count: inferred })}</p>}
                </div>
                <div className="flex flex-none items-center gap-2">
                    <Button size="sm" variant="outline" onClick={onExport} disabled={rows.length === 0}>
                        {t("release.pulls.plan.export")}
                    </Button>
                    <Button size="sm" variant="ghost" onClick={onReset} disabled={!planned} aria-label={t("release.pulls.plan.reset")}>
                        <RotateCcw className="size-4" />
                    </Button>
                </div>
            </div>

            <div className="flex flex-wrap items-start gap-x-8 gap-y-3 rounded-lg border border-border bg-muted/30 p-3">
                <Stat label={t("release.pulls.plan.committed")} value={f.number(totals.spent)} />
                <Stat label={t("release.pulls.plan.remaining")} value={f.number(totals.remaining)} />
                {totals.shortBanners > 0 && <Stat label={t("release.pulls.plan.overrun", { count: totals.shortBanners })} value={f.number(totals.shortfall)} className="text-amber-500" />}
                {!planned && <p className="m-0 self-center font-sans text-[11.5px] text-muted-foreground">{t("release.pulls.plan.untouched")}</p>}
            </div>

            {rows.length === 0 ? (
                <ReleaseEmpty title={t("release.pulls.plan.title")} description={t("release.pulls.banners.empty")} />
            ) : (
                <div className="flex flex-col">
                    {rows.map((row) => (
                        <PlanRow key={row.key} row={row} lookup={lookup} charNames={charNames} today={today} t={t} onAllocate={onAllocate} onSetTarget={onSetTarget} eventArt={eventArt} eventsByDay={eventsByDay} />
                    ))}
                </div>
            )}
        </Card>
    );
}

/**
 * The right rail: what this banner is expected to cost, and whether the plan pays it.
 *
 * A two-rate-up banner is several different questions with different answers. On a
 * DOUBLE pool: 57 rolls for either operator, 102 for a named one, 174 for both, and
 * 783 to take one to maximum potential. Showing only the middle figure would flatter
 * a player chasing either and badly mislead one chasing six copies.
 *
 * Each row is a button that fills the pull count in, so the estimates are the control
 * rather than a readout you have to copy by hand.
 */
function Estimated({ row, t, onAllocate }: { row: IPlanRow; t: PullsT; onAllocate: (key: string, pulls: number) => void }): React.ReactElement {
    const f = useFormatters();
    const pct = usePct();
    const { estimate, maxPot, goalEstimate } = row;
    const worstTail = Math.max(estimate.specific.unresolved, estimate.both?.unresolved ?? 0, maxPot.unresolved, goalEstimate?.unresolved ?? 0);

    const goals: { key: string; label: string; value: number }[] = [];
    if (goalEstimate) goals.push({ key: "yours", label: t("release.pulls.plan.yourGoal"), value: goalEstimate.p50 });
    if (estimate.both !== null) {
        goals.push({ key: "any", label: t("release.pulls.plan.goal.any"), value: estimate.any.p50 });
        goals.push({ key: "specific", label: t("release.pulls.plan.goal.specific"), value: estimate.specific.p50 });
        goals.push({ key: "both", label: t("release.pulls.plan.goal.both"), value: estimate.both.p50 });
    } else {
        goals.push({ key: "specific", label: t("release.pulls.plan.goal.specific"), value: estimate.specific.p50 });
    }
    goals.push({ key: "maxPot", label: t("release.pulls.plan.goal.maxPot"), value: maxPot.p50 });

    /**
     * Which goal the panel is describing, derived from the committed count rather than
     * stored. Clicking a row sets the count to that row's figure, so the match picks it
     * up and the highlight, the average and the unlucky figure all follow the row you
     * pressed. Before this, pressing Max pot set 397 rolls and then went on describing
     * "this operator" at 57, which is the wrong goal answered confidently.
     *
     * Typing a count that matches no goal leaves none active, which is honest: the
     * panel then has no particular question to be answering.
     */
    const byGoal = new Map<string, IGoalEstimate>([
        ["yours", goalEstimate ?? estimate.specific],
        ["any", estimate.any],
        ["specific", estimate.specific],
        ["both", estimate.both ?? estimate.specific],
        ["maxPot", maxPot],
    ]);
    const activeKey = row.allocated > 0 ? (goals.find((g) => g.value === row.allocated)?.key ?? null) : null;
    const active = (activeKey && byGoal.get(activeKey)) || goalEstimate || estimate.specific;
    /**
     * Measured against what will actually be thrown at the banner, not against what
     * was typed. Typing 397 into a row the bank can only pay 339 of used to print
     * "Your 397 covers it" in green directly beside "58 pulls short" in amber, which
     * is the same row contradicting itself. Free rolls count here because they are
     * spent on the banner too.
     */
    const gap = active.p50 - row.totalPulls;

    return (
        <div className="flex w-full min-w-0 flex-col gap-1 rounded-lg border border-border bg-muted/20 px-2.5 py-2">
            <span className="font-sans text-[10.5px] text-muted-foreground uppercase tracking-[0.06em]">{t("release.pulls.plan.estimated")}</span>

            <ul className="m-0 flex list-none flex-col gap-0.5 p-0">
                {goals.map((g) => (
                    <li key={g.key}>
                        <button
                            type="button"
                            aria-pressed={g.key === activeKey}
                            aria-label={t("release.pulls.plan.goalPick", { count: f.number(g.value), goal: g.label })}
                            onClick={() => onAllocate(row.key, g.value)}
                            className={cn(
                                "flex w-full cursor-pointer items-baseline justify-between gap-3 rounded border px-2 py-1 text-left transition-colors hover:bg-accent/50",
                                // A border on every row, not just on hover: these ARE the
                                // panel's control, and on touch there is no hover to
                                // reveal that. Without it they read as a definition list.
                                g.key === activeKey ? "border-primary/50 bg-primary/10" : "border-border/50",
                            )}
                        >
                            <span className={cn("min-w-0 truncate font-sans text-[11.5px]", g.key === activeKey ? "text-foreground" : "text-muted-foreground")}>{g.label}</span>
                            <span className={cn("shrink-0 font-mono text-[11.5px] tabular-nums", g.key === activeKey ? "font-bold text-foreground" : "text-muted-foreground")}>{t("release.pulls.plan.goalValue", { count: f.number(g.value) })}</span>
                        </button>
                    </li>
                ))}
            </ul>

            {/* A p90 of zero is a SENTINEL, not a pull count: it means the curve never
                reached nine runs in ten inside the horizon. Printed raw it came out as
                "358 on average, 0 if unlucky", which reads as the unlucky case being
                free. The mean is truncated at the same horizon and so is a floor, which
                is what the plus marks. */}
            <span className="font-mono text-[11px] text-muted-foreground tabular-nums">
                {active.p90 > 0 ? t("release.pulls.plan.estimatedDetail", { mean: f.number(Math.round(active.mean)), p90: f.number(active.p90) }) : t("release.pulls.plan.estimatedDetailOpen", { mean: f.number(Math.round(active.mean)), horizon: f.number(estimate.horizon) })}
            </span>
            {worstTail >= 0.005 && <span className="font-sans text-[11px] text-muted-foreground">{t("release.pulls.plan.estimatedTail", { percent: pct(worstTail, 1), horizon: f.number(estimate.horizon) })}</span>}
            {/* The arithmetic is shown rather than performed silently. This line counts
                the banner's own free rolls, the budget line above it cannot (free rolls
                never enter the pool), so the two used to print different numbers for
                what looked like the same question: "40 short" above, "16 under" here.
                Spelling out 30 + 24 free = 54 makes them two facts instead of a
                contradiction. */}
            {row.totalPulls > 0 && (
                <span className={cn("font-mono text-[11px] tabular-nums", gap > 0 ? "text-amber-500" : "text-emerald-500")}>
                    {row.freePulls > 0
                        ? gap > 0
                            ? t("release.pulls.plan.sumFreeGap", { spent: f.number(row.spent), free: f.number(row.freePulls), total: f.number(row.totalPulls), gap: f.number(gap), goal: f.number(active.p50) })
                            : t("release.pulls.plan.sumFreeCovers", { spent: f.number(row.spent), free: f.number(row.freePulls), total: f.number(row.totalPulls), goal: f.number(active.p50) })
                        : gap > 0
                          ? t("release.pulls.plan.sumGap", { total: f.number(row.totalPulls), gap: f.number(gap), goal: f.number(active.p50) })
                          : t("release.pulls.plan.sumCovers", { total: f.number(row.totalPulls), goal: f.number(active.p50) })}
                </span>
            )}
        </div>
    );
}

/**
 * A featured operator with the potential the user is pulling for.
 *
 * `OpRef` renders these as links to /operators/{id}, which is right everywhere else
 * in the release planner and wrong here: on this row the operator IS the input, and
 * navigating away mid-plan is the last thing a click should do. This keeps `OpRef`'s
 * look so the two read as the same object, and hangs a stepper off it.
 *
 * Zero is a real value meaning "not pulling for this one", which is why the stepper
 * shows a word there rather than a bare 0 that would read as a potential.
 */
function OperatorPot({ id, lookup, name, copies, onSet, t }: { id: string; lookup: OperatorLookup; name: AutoName | null; copies: number; onSet: (copies: number) => void; t: PullsT }): React.ReactElement {
    const autoOn = useAutoTranslate();
    const entry = lookup.get(id);
    const label = operatorLabel(id, entry, name, autoOn);
    const rarity = entry ? rarityToNumber(entry.rarity) : null;
    const active = copies > 0;
    return (
        <div className={cn("inline-flex min-w-0 max-w-full items-center gap-1.5 rounded-md border py-0.5 pr-1 pl-0.5 font-medium font-sans text-[12px] transition-colors", active ? "border-primary bg-primary/15 text-foreground" : "border-border/60 bg-secondary/40 text-muted-foreground")}>
            <span className={cn("inline-flex size-5 shrink-0 items-center justify-center overflow-hidden rounded font-bold font-sans text-[10px]", rarity === null && "bg-muted text-muted-foreground")} style={rarity === null ? undefined : { backgroundColor: `var(--rarity-${rarity})` }}>
                <OperatorAvatar charId={id} name={label.text} server={entry ? undefined : "cn"} />
            </span>
            <span className="min-w-0 truncate" title={entry?.name ?? id}>
                {label.text}
            </span>
            {rarity !== null && <span className="hidden shrink-0 font-mono text-[11px] text-muted-foreground xl:inline">{rarity}★</span>}
            {/* The provenance tag is the first thing to go when space is tight, and it
                comes back at xl rather than sm: sm is where the row is TIGHTEST, not
                widest, so revealing anything there is backwards. */}
            {label.auto && (
                <span className="hidden shrink-0 xl:inline">
                    <AutoTag source={label.auto.source} />
                </span>
            )}
            <span className="ml-0.5 inline-flex shrink-0 items-center gap-0.5 rounded bg-background/60 p-0.5">
                <button type="button" onClick={() => onSet(copies - 1)} disabled={copies <= 0} aria-label={t("release.pulls.plan.potLess", { operator: label.text })} className="inline-flex size-6 cursor-pointer items-center justify-center rounded hover:bg-accent disabled:cursor-default disabled:opacity-40">
                    <Minus className="size-3" />
                </button>
                <span className={cn("min-w-8 text-center font-mono text-[11.5px] tabular-nums", active ? "font-bold text-foreground" : "text-muted-foreground")}>{active ? t("release.pulls.plan.potValue", { count: copies }) : t("release.pulls.plan.potNone")}</span>
                <button
                    type="button"
                    onClick={() => onSet(copies + 1)}
                    disabled={copies >= MAX_POT_COPIES}
                    aria-label={t("release.pulls.plan.potMore", { operator: label.text })}
                    className="inline-flex size-6 cursor-pointer items-center justify-center rounded hover:bg-accent disabled:cursor-default disabled:opacity-40"
                >
                    <Plus className="size-3" />
                </button>
            </span>
        </div>
    );
}

interface IPlanRowProps {
    row: IPlanRow;
    lookup: OperatorLookup;
    charNames: { [key in string]?: AutoName };
    today: Date;
    t: PullsT;
    onAllocate: (key: string, pulls: number) => void;
    onSetTarget: (key: string, charId: string, copies: number) => void;
    eventArt: EventArt;
    eventsByDay: Map<string, string>;
}

/**
 * One banner.
 *
 * The hierarchy is deliberate and was not here at first: an earlier version gave the
 * budget, the odds and the caveats identical weight, so a row of eight equal figures
 * said nothing at a glance. There is ONE number a planner wants per banner, the chance
 * the current commitment buys, so that is the only thing set large. Everything else is
 * either a control or a muted supporting line, and the two sentences that used to
 * repeat on every row now appear once above the list.
 */
function PlanRow({ row, lookup, charNames, today, t, onAllocate, onSetTarget, eventArt, eventsByDay }: IPlanRowProps): React.ReactElement {
    const locale = useLocale();
    const f = useFormatters();
    const autoOn = useAutoTranslate();
    const { banner, model } = row;

    const anchor = banner.anchorActivity ? eventArt.get(banner.anchorActivity) : undefined;
    const sameDay = !banner.imagePath && !anchor?.imagePath ? eventArt.get(eventsByDay.get(cnDay(banner.cnOpen)) ?? "") : undefined;
    const borrowed = anchor?.imagePath ? anchor : sameDay?.imagePath ? sameDay : undefined;
    const art = useArt(banner.imagePath ?? borrowed?.imagePath ?? null);
    const alt = resolveName(banner.nameCn, null, banner.nameEnAuto, autoOn).text;
    const faces = art.src ? [] : row.featured.slice(0, 3);
    // On a phone the art is a wide thin strip rather than a square that would eat
    // half the row; it carries no information the name does not.
    const visual = art.src ? <img src={art.src} alt={alt} loading="lazy" onError={art.onError} className="aspect-[5/2] w-full rounded-md bg-muted object-cover" /> : faces.length > 0 ? <FaceStrip ids={faces} lookup={lookup} /> : null;

    const tagLabel = useReleaseTagLabel();
    const targets = planTargets(row);
    const inputId = `plan-alloc-${row.key}`;
    const step = (delta: number) => onAllocate(row.key, Math.max(0, row.allocated + delta));

    return (
        /**
         * This row does NOT use `ListRow`, which the sibling tabs use, and that is
         * deliberate. Its grid is `sm:grid-cols-[240px_minmax(0,1fr)_minmax(240px,auto)]`
         * with `gap-x-3`, a rigid 504px floor against a content column whose minimum is
         * zero. At a 639px viewport the content column is 435px; at 640px it is 72px.
         * The `sm` breakpoint makes the layout NARROWER by 363px, and this row carries
         * far more than the name-and-dates those tabs put there, so it collapsed.
         *
         * The breakpoints below are chosen from what the content actually needs: the
         * allocate stepper is an irreducible 152px and the preset group is about 330px
         * unwrapped, so the columns only split once there is room for them. Worst case
         * is 256px at a 320px viewport and 512px at 768px, never 72.
         */
        <div className="grid grid-cols-1 gap-x-4 gap-y-3 border-border border-t py-4 first:border-t-0 md:grid-cols-[180px_minmax(0,1fr)] md:items-start xl:grid-cols-[200px_minmax(0,1fr)_236px]">
            <div className="min-w-0">{visual}</div>
            <div className="flex min-w-0 flex-col gap-2">
                {/* Baseline, not centred. `ResolutionBadge` is two lines tall whenever it
                carries a range or a source, and centring a one-line name against it
                dropped the name and the date nine pixels, so they sat between the
                badge's two lines and lined up with neither. On a baseline the name,
                the tag, the date and the badge's first line share one line and the
                badge's second line hangs below, which is what it is. */}
                <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
                    <CnName cn={banner.nameCn} auto={banner.nameEnAuto} primaryClassName="font-sans font-semibold text-[13px] text-foreground" compact>
                        <Tag>{tagLabel(banner.ruleType)}</Tag>
                    </CnName>
                    <span className="font-mono text-[11.5px] text-muted-foreground tabular-nums">{formatDate(row.enStart, locale)}</span>
                    <ResolutionBadge resolution={banner.resolution} today={today} />
                </div>

                {/* The row's anchor is its BUDGET, not its odds.
                    A percentage sat here first, and it was the wrong thing to set large:
                    it answered a question the Estimated panel beside it already answers
                    better, in rolls rather than in a probability nobody can act on.

                    Which budget figure leads switches with the state, and deliberately.
                    Before anything is committed the only question is what the banner has,
                    so that is the headline. Once a count is typed the question becomes
                    whether it can be paid for, so the headline is THAT COUNT, the same
                    number as the input below it.

                    Two earlier versions led with a derived figure instead and both went
                    wrong the same way. The leftover pins to zero on every overcommitted
                    row, so five overruns meant five identical large zeroes. Then the bank
                    led and the clause quoted `spent`, a capped number the user never
                    typed, so a row where 397 was entered announced "spending 339" and the
                    reader had to hunt for where 397 went. A headline that echoes the
                    control cannot drift from it. */}
                <div className="flex flex-wrap items-baseline gap-x-2.5 gap-y-0.5">
                    {row.allocated > 0 ? (
                        <>
                            <span className={cn("font-bold font-mono text-[28px] tabular-nums leading-none", row.shortfall > 0 ? "text-amber-500" : "text-foreground")}>{f.number(row.allocated)}</span>
                            <span className="font-sans text-[11.5px] text-muted-foreground">{t("release.pulls.plan.plannedHere")}</span>
                            {row.shortfall > 0 ? (
                                <span className="font-sans text-[11.5px] text-amber-500">{t("release.pulls.plan.planShort", { short: f.number(row.shortfall), available: f.number(row.available) })}</span>
                            ) : (
                                <span className="font-sans text-[11.5px] text-muted-foreground">{t("release.pulls.plan.planFits", { available: f.number(row.available) })}</span>
                            )}
                        </>
                    ) : (
                        <>
                            <span className="font-bold font-mono text-[28px] text-foreground tabular-nums leading-none">{f.number(row.available)}</span>
                            <span className="font-sans text-[11.5px] text-muted-foreground">{t("release.pulls.plan.availableHere")}</span>
                        </>
                    )}
                </div>

                {/* Only rendered when it has something to say. An always-present empty
                line still costs the column's `gap-2` and pushed every row apart. */}
                {(row.freePulls > 0 || (row.sparkMet && model.spark !== null)) && (
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-0.5 font-mono text-[11.5px] text-muted-foreground tabular-nums">
                        {row.freePulls > 0 && <span>{t("release.pulls.plan.freeOnBanner", { count: row.freePulls })}</span>}
                        {row.sparkMet && model.spark !== null && <span>{t("release.pulls.banners.sparkMet", { spark: model.spark })}</span>}
                    </div>
                )}

                {row.featured.length > 0 && (
                    <div className="flex flex-wrap gap-1">
                        {row.featured.map((id) => (
                            <OperatorPot key={id} id={id} lookup={lookup} name={charNames[id] ?? null} copies={row.targets[id] ?? 0} onSet={(copies) => onSetTarget(row.key, id, copies)} t={t} />
                        ))}
                    </div>
                )}

                <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
                    <div className="flex items-center gap-1">
                        <label htmlFor={inputId} className="sr-only">
                            {t("release.pulls.plan.allocate")}
                        </label>
                        <Button size="sm" variant="outline" className="size-8 shrink-0 p-0" onClick={() => step(-10)} disabled={row.allocated === 0} aria-label={`${t("release.pulls.plan.allocate")} -10`}>
                            <Minus className="size-3.5" />
                        </Button>
                        <Input
                            id={inputId}
                            type="number"
                            inputMode="numeric"
                            min={0}
                            max={9999}
                            value={row.allocated}
                            onChange={(e) => {
                                const next = Number(e.target.value);
                                if (!Number.isFinite(next)) return;
                                onAllocate(row.key, Math.min(9999, Math.max(0, Math.floor(next))));
                            }}
                            className="h-8 w-20 text-right font-mono tabular-nums"
                        />
                        <Button size="sm" variant="outline" className="size-8 shrink-0 p-0" onClick={() => step(10)} aria-label={`${t("release.pulls.plan.allocate")} +10`}>
                            <Plus className="size-3.5" />
                        </Button>
                    </div>

                    <div className="flex min-w-0 flex-wrap items-center gap-1">
                        {/* The presets carry a label because a bare number beside them reads
                        as another statistic rather than a button. */}
                        {/* Label and first button travel together: as siblings in a wrap
                        container they separate at narrow widths, leaving a bare number
                        that reads as another statistic. */}
                        <span className="inline-flex items-center gap-1">
                            <span className="font-sans text-[10.5px] text-muted-foreground uppercase tracking-[0.06em]">{t("release.pulls.plan.setTo")}</span>
                            <Button size="sm" variant="outline" className="h-7 px-2 font-mono text-[11.5px]" onClick={() => onAllocate(row.key, targets.max)} disabled={targets.max === 0} aria-label={t("release.pulls.plan.max")}>
                                {f.number(targets.max)}
                            </Button>
                        </span>
                        {targets.spark !== null && (
                            <Button size="sm" variant="outline" className="h-7 px-2 font-mono text-[11.5px]" onClick={() => onAllocate(row.key, targets.spark ?? 0)}>
                                {t("release.pulls.plan.spark", { count: targets.spark })}
                            </Button>
                        )}
                        {targets.guarantee !== null && (
                            <Button size="sm" variant="outline" className="h-7 px-2 font-mono text-[11.5px]" onClick={() => onAllocate(row.key, targets.guarantee ?? 0)}>
                                {t("release.pulls.plan.guarantee", { count: targets.guarantee })}
                            </Button>
                        )}
                        {row.allocated > 0 && (
                            <Button size="sm" variant="ghost" className="h-7 px-2 font-mono text-[11.5px]" onClick={() => onAllocate(row.key, 0)}>
                                {t("release.pulls.plan.clear")}
                            </Button>
                        )}
                    </div>
                </div>
            </div>
            <div className="min-w-0 md:col-span-2 xl:col-span-1">
                <Estimated row={row} t={t} onAllocate={onAllocate} />
            </div>
        </div>
    );
}

function FaceStrip({ ids, lookup }: { ids: string[]; lookup: OperatorLookup }): React.ReactElement {
    return (
        <div className="flex aspect-[5/2] w-full items-center justify-center gap-1 overflow-hidden rounded-md bg-linear-to-br from-zinc-800 to-zinc-950 p-1 sm:p-1.5">
            {ids.map((id) => (
                <Face key={id} id={id} onEn={lookup.has(id)} tight={ids.length > 2} />
            ))}
        </div>
    );
}

function Face({ id, onEn, tight }: { id: string; onEn: boolean; tight: boolean }): React.ReactElement | null {
    const [failed, setFailed] = React.useState(false);
    if (failed) return null;
    return <img src={getAvatarById(id, onEn ? undefined : "cn")} alt="" loading="lazy" onError={() => setFailed(true)} className={cn("h-full rounded-sm object-cover", tight ? "min-w-0 flex-1" : "aspect-square flex-none")} />;
}
