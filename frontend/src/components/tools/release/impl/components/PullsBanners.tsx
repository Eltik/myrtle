import { useQuery } from "@tanstack/react-query";
import { ChevronDown, Minus, Plus, RotateCcw } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Collapsible, CollapsiblePanel, CollapsibleTrigger } from "#/components/ui/collapsible";
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
    /** Commits a preset count AND drops the banner's potential picks, so the panel that set the count is the one describing it. */
    onPreset: (key: string, pulls: number) => void;
    onSetTarget: (key: string, charId: string, copies: number) => void;
    /** Clears one banner: its committed pulls and its potential picks. */
    onClearRow: (key: string) => void;
    onReset: () => void;
    onExport: () => void;
}

export function PullsBanners({ rows, totals, lookup, charNames, today, onAllocate, onPreset, onSetTarget, onClearRow, onReset, onExport }: IPullsBannersProps): React.ReactElement {
    const t: PullsT = useT("tools");
    const f = useFormatters();
    const events = useQuery(releaseEventsQueryOptions());
    const planned = totals.allocated > 0;
    // Said once for the whole list, not per row.
    const inferred = rows.filter((r) => r.model.inferred).length;

    // A banner often ships no art of its own and borrows the event it is anchored to,
    // which is how the Banners and Planner tabs get a picture on every row.
    const eventArt = React.useMemo(() => {
        const map: EventArt = new Map();
        for (const e of events.data?.events ?? []) map.set(e.cnId, { cn: e.nameCn, en: e.nameEn, auto: e.nameEnAuto, imagePath: e.imagePath });
        return map;
    }, [events.data]);

    return (
        <Card className="flex flex-col gap-3 p-4">
            <div className="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
                <div className="flex min-w-0 flex-col gap-1">
                    <SectionTitle count={rows.length}>{t("release.pulls.plan.title")}</SectionTitle>
                    <p className="m-0 max-w-2xl font-sans text-[12px] text-muted-foreground leading-normal">{t("release.pulls.plan.intro")}</p>
                    <p className="m-0 max-w-2xl font-sans text-[12px] text-muted-foreground leading-normal">{t("release.pulls.plan.pickHint")}</p>
                    {inferred > 0 && <p className="m-0 max-w-2xl font-sans text-[12px] text-amber-500/90 leading-normal">{t("release.pulls.plan.inferredCount", { count: inferred })}</p>}
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
                {!planned && <p className="m-0 self-center font-sans text-[12px] text-muted-foreground">{t("release.pulls.plan.untouched")}</p>}
            </div>

            {rows.length === 0 ? (
                <ReleaseEmpty title={t("release.pulls.plan.title")} description={t("release.pulls.banners.empty")} />
            ) : (
                <div className="flex flex-col">
                    {rows.map((row) => (
                        <PlanRow key={row.key} row={row} lookup={lookup} charNames={charNames} today={today} t={t} onAllocate={onAllocate} onPreset={onPreset} onSetTarget={onSetTarget} onClearRow={onClearRow} eventArt={eventArt} />
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
function Estimated({ row, t, onPreset, onAllocate }: { row: IPlanRow; t: PullsT; onPreset: (key: string, pulls: number) => void; onAllocate: (key: string, pulls: number) => void }): React.ReactElement {
    const f = useFormatters();
    const pct = usePct();
    const { estimate, maxPot, goalEstimate } = row;
    const worstTail = Math.max(estimate.specific.unresolved, estimate.both?.unresolved ?? 0, maxPot.unresolved, goalEstimate?.unresolved ?? 0);

    /**
     * What each row measures, in the row's own words. `odds.ts` tracks the NAMED operator
     * against the rest of the rate-up pool as a group, so `both` is "the named one and at
     * least one other" on a pool of more than two, and `maxPot` is six copies of the named
     * one rather than of every one. (A fixed "Either one / Both" read as nonsense on an
     * Orienteering pool of six: "Both" sat 9 rolls above "This operator".)
     */
    const goals: { key: string; label: string; value: number }[] = [];
    if (goalEstimate) goals.push({ key: "yours", label: t("release.pulls.plan.yourGoal"), value: goalEstimate.p50 });
    // On a single-rate-up pool `any` IS `specific`, so only the one row.
    if (estimate.both !== null) goals.push({ key: "any", label: t("release.pulls.plan.goal.any"), value: estimate.any.p50 });
    goals.push({ key: "specific", label: t("release.pulls.plan.goal.specific"), value: estimate.specific.p50 });
    if (estimate.both !== null) {
        goals.push({ key: "both", label: row.model.featuredCount > 2 ? t("release.pulls.plan.goal.bothMulti") : t("release.pulls.plan.goal.both"), value: estimate.both.p50 });
    }
    goals.push({ key: "maxPot", label: t("release.pulls.plan.goal.maxPot"), value: maxPot.p50 });

    /**
     * Which goal the panel describes, derived from the committed count rather than stored.
     * Clicking a row sets the count to that row's figure, so the highlight, the average and the
     * unlucky figure follow it. Without this, Max pot set 397 rolls and the panel went on
     * describing "this operator" at 57: the wrong goal, answered confidently. A count that
     * matches no goal leaves none active.
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
    /** `active` is an ESTIMATE; the collapsed heading needs the goal row's own wording. */
    const activeLabel = goals.find((g) => g.key === activeKey)?.label ?? null;
    /**
     * Measured against what will actually be thrown at the banner, not what was typed: 397
     * typed into a row the bank can only pay 339 of printed "Your 397 covers it" in green beside
     * "58 pulls short" in amber. Free rolls count because they are spent on the banner too.
     */
    const gap = active.p50 - row.totalPulls;

    /* Goal list folds away; open by default since it is this panel's own control. The heading
       carries the active goal while closed. */
    return (
        <Collapsible defaultOpen className="flex w-full min-w-0 flex-col gap-1 rounded-lg border border-border bg-muted/20 px-2.5 py-2">
            <CollapsibleTrigger className="group flex w-full items-baseline justify-between gap-3 text-left">
                <span className="font-sans text-[11.5px] text-muted-foreground uppercase tracking-[0.06em]">{t("release.pulls.plan.estimated")}</span>
                <span className="flex min-w-0 items-baseline gap-1.5">
                    {activeLabel ? <span className="min-w-0 truncate font-sans text-[11.5px] text-muted-foreground group-data-[panel-open]:hidden">{activeLabel}</span> : null}
                    <ChevronDown className="size-3.5 shrink-0 text-muted-foreground transition-transform group-data-[panel-open]:rotate-180" aria-hidden="true" />
                </span>
            </CollapsibleTrigger>

            <CollapsiblePanel>
                <ul className="m-0 flex list-none flex-col gap-0.5 p-0">
                    {goals.map((g) => (
                        <li key={g.key}>
                            <button
                                type="button"
                                aria-pressed={g.key === activeKey}
                                aria-label={t("release.pulls.plan.goalPick", { count: f.number(g.value), goal: g.label })}
                                /* A preset DROPS the potential picks, so the panel that sets the
                                   count is the one describing it. "Your goal" is the picks' own
                                   row and the one preset that keeps them. */
                                onClick={() => (g.key === "yours" ? onAllocate(row.key, g.value) : onPreset(row.key, g.value))}
                                className={cn(
                                    "flex w-full cursor-pointer items-baseline justify-between gap-3 rounded border px-2 py-1 text-left transition-colors hover:bg-accent/50",
                                    // A border on every row, not just on hover: these ARE the
                                    // panel's control, and on touch there is no hover to
                                    // reveal that. Without it they read as a definition list.
                                    g.key === activeKey ? "border-primary/50 bg-primary/10" : "border-border/50",
                                )}
                            >
                                <span className={cn("min-w-0 truncate font-sans text-[12px]", g.key === activeKey ? "text-foreground" : "text-muted-foreground")}>{g.label}</span>
                                <span className={cn("shrink-0 font-mono text-[12px] tabular-nums", g.key === activeKey ? "font-bold text-foreground" : "text-muted-foreground")}>{t("release.pulls.plan.goalValue", { count: f.number(g.value) })}</span>
                            </button>
                        </li>
                    ))}
                </ul>
            </CollapsiblePanel>

            {/* A p90 of zero is a SENTINEL, not a pull count: the curve never reached nine runs
                in ten inside the horizon. Printed raw it read "358 on average, 0 if unlucky".
                The mean is truncated at the same horizon, so it is a floor (the plus). */}
            <span className="font-mono text-[12px] text-muted-foreground tabular-nums">
                {active.p90 > 0 ? t("release.pulls.plan.estimatedDetail", { mean: f.number(Math.round(active.mean)), p90: f.number(active.p90) }) : t("release.pulls.plan.estimatedDetailOpen", { mean: f.number(Math.round(active.mean)), horizon: f.number(estimate.horizon) })}
            </span>
            {worstTail >= 0.005 && <span className="font-sans text-[12px] text-muted-foreground">{t("release.pulls.plan.estimatedTail", { percent: pct(worstTail, 1), horizon: f.number(estimate.horizon) })}</span>}
            {/* The arithmetic is shown, not performed silently. This line counts the banner's own
                free rolls and the budget line above cannot (free rolls never enter the pool), so
                they printed "40 short" vs "16 under". "30 + 24 free = 54" makes them two facts. */}
            {row.totalPulls > 0 && (
                <span className={cn("font-mono text-[12px] tabular-nums", gap > 0 ? "text-amber-500" : "text-emerald-500")}>
                    {row.freePulls > 0
                        ? gap > 0
                            ? t("release.pulls.plan.sumFreeGap", { spent: f.number(row.spent), free: f.number(row.freePulls), total: f.number(row.totalPulls), gap: f.number(gap), goal: f.number(active.p50) })
                            : t("release.pulls.plan.sumFreeCovers", { spent: f.number(row.spent), free: f.number(row.freePulls), total: f.number(row.totalPulls), goal: f.number(active.p50) })
                        : gap > 0
                          ? t("release.pulls.plan.sumGap", { total: f.number(row.totalPulls), gap: f.number(gap), goal: f.number(active.p50) })
                          : t("release.pulls.plan.sumCovers", { total: f.number(row.totalPulls), goal: f.number(active.p50) })}
                </span>
            )}
        </Collapsible>
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
            <span className={cn("inline-flex size-5 shrink-0 items-center justify-center overflow-hidden rounded font-bold font-sans text-[11px]", rarity === null && "bg-muted text-muted-foreground")} style={rarity === null ? undefined : { backgroundColor: `var(--rarity-${rarity})` }}>
                <OperatorAvatar charId={id} name={label.text} server={entry ? undefined : "cn"} />
            </span>
            <span className="min-w-0 truncate" title={entry?.name ?? id}>
                {label.text}
            </span>
            {rarity !== null && <span className="hidden shrink-0 font-mono text-[12px] text-muted-foreground xl:inline">{rarity}★</span>}
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
                <span className={cn("min-w-8 text-center font-mono text-[12px] tabular-nums", active ? "font-bold text-foreground" : "text-muted-foreground")}>{active ? t("release.pulls.plan.potValue", { count: copies }) : t("release.pulls.plan.potNone")}</span>
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
    onPreset: (key: string, pulls: number) => void;
    onSetTarget: (key: string, charId: string, copies: number) => void;
    onClearRow: (key: string) => void;
    eventArt: EventArt;
}

/**
 * One banner.
 *
 * ONE number a planner wants per banner: the chance the current commitment buys, so that is
 * the only thing set large. Everything else is a control or a muted supporting line, and the
 * caveats that would repeat per row appear once above the list.
 */
function PlanRow({ row, lookup, charNames, today, t, onAllocate, onPreset, onSetTarget, onClearRow, eventArt }: IPlanRowProps): React.ReactElement {
    const locale = useLocale();
    const f = useFormatters();
    const autoOn = useAutoTranslate();
    const { banner, model } = row;

    /**
     * The banner's own art, or the art of the event it is ANCHORED to. Nothing else: an event
     * merely opening the same CN day is a calendar coincidence and put the wrong key art on
     * banners. With neither, the row shows its operators.
     */
    const anchor = banner.anchorActivity ? eventArt.get(banner.anchorActivity) : undefined;
    const art = useArt(banner.imagePath ?? anchor?.imagePath ?? null);
    const alt = resolveName(banner.nameCn, null, banner.nameEnAuto, autoOn).text;
    // Every rate-up operator, not the first three: an Orienteering pool features six.
    const faces = art.src ? [] : row.featured;
    // On a phone the art is a wide thin strip, not a square that eats half the row.
    const visual = art.src ? <img src={art.src} alt={alt} loading="lazy" onError={art.onError} className="aspect-[5/2] w-full rounded-md bg-muted object-cover" /> : faces.length > 0 ? <FaceStrip ids={faces} lookup={lookup} /> : null;

    const tagLabel = useReleaseTagLabel();
    const targets = planTargets(row);
    // The threshold itself, for the tooltip: `targets.guarantee` is what the player
    // must commit to reach it, which is lower whenever the banner gives free pulls.
    const g = model.guarantee;
    const rawGuarantee = g.kind === "linkage" ? (g.at ?? null) : g.kind === "selection" ? (g.first ?? null) : null;
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
                {/* Baseline, not centred: `ResolutionBadge` is two lines tall with a range or a
                    source, and centring a one-line name against it dropped name and date nine
                    pixels, between the badge's lines. On a baseline they share its first line. */}
                <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
                    <CnName cn={banner.nameCn} auto={banner.nameEnAuto} primaryClassName="font-sans font-semibold text-[13px] text-foreground" compact>
                        <Tag>{tagLabel(banner.ruleType)}</Tag>
                    </CnName>
                    {/* The badge prints the start date for every status that has one, so this
                        printed "Dec 16, 2026" twice. It stays only for the two statuses whose
                        badge is a bare chip. */}
                    {banner.resolution.status === "unmodelled" || banner.resolution.status === "independent" ? <span className="font-mono text-[12px] text-muted-foreground tabular-nums">{formatDate(row.enStart, locale)}</span> : null}
                    <ResolutionBadge resolution={banner.resolution} today={today} />
                </div>

                {/* The row's anchor is its BUDGET, not its odds: a percentage answered what the
                    Estimated panel beside it answers better, in rolls.

                    The leading figure switches with state. Nothing committed: what the banner
                    has. Count typed: THAT COUNT, the same number as the input below it. A
                    derived figure failed twice: leftover pins to zero on every overcommitted
                    row (five identical zeroes), and a clause quoting capped `spent` announced
                    "spending 339" where 397 was entered. A headline that echoes the control
                    cannot drift from it. */}
                <div className="flex flex-wrap items-baseline gap-x-2.5 gap-y-0.5">
                    {row.allocated > 0 ? (
                        <>
                            <span className={cn("font-bold font-mono text-[28px] tabular-nums leading-none", row.shortfall > 0 ? "text-amber-500" : "text-foreground")}>{f.number(row.allocated)}</span>
                            <span className="font-sans text-[12px] text-muted-foreground">{t("release.pulls.plan.plannedHere")}</span>
                            {row.shortfall > 0 ? (
                                <span className="font-sans text-[12px] text-amber-500">{t("release.pulls.plan.planShort", { short: f.number(row.shortfall), available: f.number(row.available) })}</span>
                            ) : (
                                <span className="font-sans text-[12px] text-muted-foreground">{t("release.pulls.plan.planFits", { available: f.number(row.available) })}</span>
                            )}
                        </>
                    ) : (
                        <>
                            <span className="font-bold font-mono text-[28px] text-foreground tabular-nums leading-none">{f.number(row.available)}</span>
                            <span className="font-sans text-[12px] text-muted-foreground">{t("release.pulls.plan.availableHere")}</span>
                        </>
                    )}
                </div>

                {/* Only rendered when non-empty: an always-present empty line still
                costs the column's `gap-2` and pushed every row apart. */}
                {(row.freePulls > 0 || (row.sparkMet && model.spark !== null)) && (
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-0.5 font-mono text-[12px] text-muted-foreground tabular-nums">
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
                        {/* Presets carry a label: a bare number beside them reads as another
                            statistic, not a button. */}
                        {/* Label and first button travel together: as wrap siblings they separate
                            at narrow widths, leaving a bare number. */}
                        {/* Each preset says what it IS (the bank, the exchange, the
                            guarantee), not just what it equals: three bare numbers under a
                            "SET TO" kicker read as unexplained ("the numbers seem random"). */}
                        <span className="inline-flex items-center gap-1">
                            <span className="font-sans text-[11.5px] text-muted-foreground uppercase tracking-[0.06em]">{t("release.pulls.plan.setTo")}</span>
                            <Button size="sm" variant="outline" className="h-7 px-2 font-mono text-[12px]" onClick={() => onPreset(row.key, targets.max)} disabled={targets.max === 0} aria-label={t("release.pulls.plan.max")} title={t("release.pulls.plan.maxTitle", { count: f.number(targets.max) })}>
                                {t("release.pulls.plan.maxLabel", { count: f.number(targets.max) })}
                            </Button>
                        </span>
                        {targets.spark !== null && model.spark !== null && (
                            <Button
                                size="sm"
                                variant="outline"
                                className="h-7 px-2 font-mono text-[12px]"
                                onClick={() => onPreset(row.key, targets.spark ?? 0)}
                                title={row.freePulls > 0 ? t("release.pulls.plan.sparkTitle", { spark: f.number(model.spark), free: f.number(row.freePulls), count: f.number(targets.spark) }) : t("release.pulls.plan.sparkTitlePlain", { spark: f.number(model.spark) })}
                            >
                                {t("release.pulls.plan.spark", { count: targets.spark })}
                            </Button>
                        )}
                        {targets.guarantee !== null && rawGuarantee !== null && (
                            <Button
                                size="sm"
                                variant="outline"
                                className="h-7 px-2 font-mono text-[12px]"
                                onClick={() => onPreset(row.key, targets.guarantee ?? 0)}
                                title={row.freePulls > 0 ? t("release.pulls.plan.guaranteeTitle", { at: f.number(rawGuarantee), free: f.number(row.freePulls), count: f.number(targets.guarantee) }) : t("release.pulls.plan.guaranteeTitlePlain", { at: f.number(rawGuarantee) })}
                            >
                                {t("release.pulls.plan.guarantee", { count: targets.guarantee })}
                            </Button>
                        )}
                        {/* Clears the potential picks too, else operators stay marked P6 with
                            nothing committed and the next rebuild prices that goal and restores
                            the count. */}
                        {(row.allocated > 0 || row.totalCopies > 0) && (
                            <Button size="sm" variant="ghost" className="h-7 px-2 font-mono text-[12px]" onClick={() => onClearRow(row.key)} title={t("release.pulls.plan.clearTitle")}>
                                {t("release.pulls.plan.clear")}
                            </Button>
                        )}
                    </div>
                </div>
            </div>
            <div className="min-w-0 md:col-span-2 xl:col-span-1">
                <Estimated row={row} t={t} onPreset={onPreset} onAllocate={onAllocate} />
            </div>
        </div>
    );
}

/**
 * The banner's rate-up operators, for a banner with no art to show.
 *
 * Four and up letterbox rather than crop. An Orienteering pool features six; at six a cropping
 * square shows a sliver of each face, so past three the faces keep their whole width and share
 * the strip instead.
 */
function FaceStrip({ ids, lookup }: { ids: string[]; lookup: OperatorLookup }): React.ReactElement {
    const many = ids.length > 3;
    return (
        <div className="flex aspect-[5/2] w-full items-center justify-center gap-1 overflow-hidden rounded-md bg-linear-to-br from-zinc-800 to-zinc-950 p-1 sm:p-1.5">
            {ids.map((id) => (
                <Face key={id} id={id} onEn={lookup.has(id)} many={many} />
            ))}
        </div>
    );
}

function Face({ id, onEn, many }: { id: string; onEn: boolean; many: boolean }): React.ReactElement | null {
    const [failed, setFailed] = React.useState(false);
    if (failed) return null;
    return <img src={getAvatarById(id, onEn ? undefined : "cn")} alt="" loading="lazy" onError={() => setFailed(true)} className={cn("h-full rounded-sm", many ? "min-w-0 flex-1 object-contain" : "aspect-square flex-none object-cover")} />;
}
