import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ListChecks, RotateCcw, X } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { useAuth } from "#/hooks/use-auth";
import { refreshRosterFn } from "#/lib/api/auth";
import { userSkinsQueryOptions } from "#/lib/api/skins";
import { userStageClearsQueryOptions } from "#/lib/api/stages";
import { userQueryOptions } from "#/lib/api/user";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById } from "#/lib/utils";
import { useAutoTranslate } from "../autoTranslate";
import { formatDate } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { balances, EMPTY_STATE, type IPlanRow, type IPlanSkin, type IPlanState, type IRowBalance, rowDeviates, rowExpense, rowIncome, rowPotential, type StageClears, type StageStatus, stageKey, stageOn, stageStatus, usePlanData } from "../plan";
import type { messages as planMessages } from "../plan.messages";
import { useStoredState } from "../planStore";
import type { messages } from "./PlannerTab.messages";
import { Calcs, Op, OpIcon } from "./Primes";
import { ResolutionBadge } from "./ResolutionBadge";
import { FALLBACK_COLOR, SkinCard } from "./SkinCard";
import { CnName, type OperatorLookup, operatorLabel, ReleaseEmpty, ReleaseError, ReleaseLoading, resolveName, Tag, ToggleField, useArt } from "./shared";

/** This tab renders its own chrome plus the shop labels and date wording the impl modules carry. */
type PlannerT = TypedT<typeof messages & typeof planMessages & typeof helperMessages>;

interface IPlannerTabProps {
    today: Date;
}

/** Shared empty set so a signed-out or toggled-off planner never re-renders on a fresh `new Set()`. */
const NO_HIDDEN: ReadonlySet<string> = new Set<string>();

export function PlannerTab({ today }: IPlannerTabProps): React.ReactElement {
    const t: PlannerT = useT("tools");
    const locale = useLocale();
    const [showPast, setShowPast] = React.useState(false);
    // On by default: a signed-in planner opens with the outfits you already own filtered out.
    const [hideOwned, setHideOwned] = React.useState(true);
    const { user } = useAuth();
    const uid = user?.uid ?? null;
    const [state, setState, sync] = useStoredState(uid);
    const [selected, setSelected] = React.useState<string | null>(null);
    const [summary, setSummary] = React.useState(false);
    const [pane, setPane] = React.useState<"events" | "detail">("events");
    const data = usePlanData(today, showPast);
    const profile = useQuery({ ...userQueryOptions(uid ?? ""), enabled: !!uid });
    const clearsQuery = useQuery(userStageClearsQueryOptions(uid));
    const clears: StageClears = uid ? (clearsQuery.data ?? null) : null;
    const ownedQuery = useQuery({ ...userSkinsQueryOptions(uid ?? ""), enabled: !!uid && hideOwned });
    const hidden = React.useMemo<ReadonlySet<string>>(() => (hideOwned && ownedQuery.data ? new Set(ownedQuery.data.map((s) => s.skin_id)) : NO_HIDDEN), [hideOwned, ownedQuery.data]);
    const accountPrimes = profile.data?.originite ?? null;
    const queryClient = useQueryClient();
    const resync = useMutation({
        mutationFn: () => refreshRosterFn(),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ["user"] }),
    });
    React.useEffect(() => {
        if (accountPrimes !== null && !state.initialManual && state.initial !== accountPrimes) setState((s) => ({ ...s, initial: accountPrimes }));
    }, [accountPrimes, state.initialManual, state.initial, setState]);
    const totals = React.useMemo(() => balances(data.rows, state, clears), [data.rows, state, clears]);
    const current = data.rows.find((r) => r.key === selected) ?? data.rows[0];

    const pickSkin = (skin: IPlanSkin, on: boolean) =>
        setState((s) => {
            const picks = { ...s.picks };
            if (on) picks[skin.skinId] = true;
            else delete picks[skin.skinId];
            return { ...s, picks };
        });
    const setInitial = (value: number, manual: boolean) => setState((s) => ({ ...s, initial: Math.max(0, value), initialManual: manual }));
    const setStage = (row: IPlanRow, key: string, on: boolean) => setState((s) => ({ ...s, stages: { ...s.stages, [row.key]: { ...s.stages[row.key], [key]: on } } }));
    const setAllStages = (row: IPlanRow, on: boolean) => setState((s) => ({ ...s, stages: { ...s.stages, [row.key]: Object.fromEntries(row.opStages.map((st) => [stageKey(st), on])) } }));
    const resetStages = (row: IPlanRow) =>
        setState((s) => {
            const stages = { ...s.stages };
            delete stages[row.key];
            return { ...s, stages };
        });
    const detailRef = React.useRef<HTMLDivElement>(null);
    const plannerRef = React.useRef<HTMLDivElement>(null);
    const showDetail = () => {
        setPane("detail");
        if (window.matchMedia("(max-width: 47.999rem)").matches) {
            const rect = plannerRef.current?.getBoundingClientRect();
            if (rect) window.scrollTo({ top: window.scrollY + rect.top });
        } else {
            const rect = detailRef.current?.getBoundingClientRect();
            if (rect && rect.top < 80) window.scrollTo({ top: window.scrollY + rect.top - 80 });
        }
    };
    const showEvents = () => {
        setPane("events");
        if (window.matchMedia("(max-width: 47.999rem)").matches) {
            const rect = plannerRef.current?.getBoundingClientRect();
            if (rect) window.scrollTo({ top: window.scrollY + rect.top });
        }
    };
    const open = (key: string) => {
        setSelected(key);
        setSummary(false);
        showDetail();
    };

    if (data.isPending) return <ReleaseLoading />;
    if (data.error) return <ReleaseError error={data.error} onRetry={() => window.location.reload()} />;

    return (
        <div ref={plannerRef} className="flex flex-col gap-3">
            <div className="flex flex-wrap items-center gap-x-5 gap-y-2">
                <ToggleField id="planner-show-past" label={t("release.planner.showPast")} checked={showPast} onChange={setShowPast} />
                {uid && <ToggleField id="planner-hide-owned" label={t("release.planner.hideOwned")} checked={hideOwned} onChange={setHideOwned} />}
                <span className="font-sans text-[11px] text-muted-foreground">
                    {t("release.planner.blurb")} {uid ? (sync.saving ? t("release.planner.saving") : sync.savedAt ? t("release.planner.saved", { date: formatDate(sync.savedAt, locale) }) : t("release.planner.autosave")) : t("release.planner.signedOut")}
                </span>
            </div>
            {data.rows.length === 0 ? (
                <ReleaseEmpty title={t("release.planner.empty.title")} description={t("release.planner.empty.desc")} />
            ) : (
                /* The page scrolls normally; the left column sticks under the header and
                   its event list scrolls so it remains reachable while reading detail. */
                <div className="grid grid-cols-1 gap-4 md:grid-cols-[340px_minmax(0,1fr)]">
                    <div className={cn("flex flex-col gap-2 md:sticky md:top-20 md:max-h-[calc(100dvh-6rem)] md:self-start", pane !== "events" && "max-md:hidden")}>
                        <div className="flex flex-col gap-2 rounded-xl border border-border bg-card p-3">
                            <div className="flex items-center gap-2">
                                <Button
                                    size="sm"
                                    variant={summary ? "default" : "outline"}
                                    onClick={() => {
                                        setSummary(true);
                                        showDetail();
                                    }}
                                >
                                    <ListChecks className="mr-1.5 size-4" />
                                    {t("release.planner.summaryButton")}
                                </Button>
                                <Button size="sm" variant="ghost" onClick={() => setState(EMPTY_STATE)} aria-label={t("release.planner.reset")}>
                                    <RotateCcw className="size-4" />
                                </Button>
                            </div>
                            <div className="flex flex-wrap items-center gap-2">
                                <label htmlFor="planner-initial" className="flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground">
                                    <OpIcon />
                                    {t("release.planner.initial")}
                                </label>
                                <Input id="planner-initial" type="number" min={0} inputMode="numeric" value={state.initial} onChange={(e) => setInitial(Number(e.target.value) || 0, true)} className="h-8 w-28 text-right font-mono" />
                                {uid && (
                                    <span className="font-sans text-[11px] text-muted-foreground">
                                        {accountPrimes === null ? (
                                            <>
                                                {t("release.planner.notSynced")}
                                                <button type="button" disabled={resync.isPending} onClick={() => resync.mutate()} className="cursor-pointer text-primary hover:underline disabled:opacity-60">
                                                    {resync.isPending ? t("release.planner.syncing") : t("release.planner.syncNow")}
                                                </button>
                                                {resync.isError ? t("release.planner.syncFailed") : ""}
                                            </>
                                        ) : state.initialManual ? (
                                            <button type="button" onClick={() => setInitial(accountPrimes, false)} className="cursor-pointer text-primary hover:underline">
                                                {t("release.planner.useAccount", { count: accountPrimes })}
                                            </button>
                                        ) : (
                                            `${t("release.planner.fromAccount")}${profile.data?.updated_at ? t("release.planner.fromAccountSynced", { date: formatDate(Date.parse(profile.data.updated_at) / 1000, locale) }) : ""}`
                                        )}
                                    </span>
                                )}
                            </div>
                        </div>
                        <div className="flex flex-col gap-2 md:min-h-0 md:flex-1 md:overflow-y-auto md:overscroll-contain md:pr-1">
                            <Button size="sm" variant="outline" className="w-full md:hidden" onClick={showDetail}>
                                {summary ? t("release.planner.showSummary") : t("release.planner.showSelected")}
                            </Button>
                            {data.rows.map((row) => (
                                <EventCard key={row.key} row={row} total={totals.get(row.key)} state={state} hidden={hidden} active={!summary && current?.key === row.key} onOpen={() => open(row.key)} t={t} locale={locale} />
                            ))}
                        </div>
                    </div>
                    <div ref={detailRef} className={cn("min-w-0", pane !== "detail" && "max-md:hidden")}>
                        <Button size="sm" variant="outline" className="mb-3 w-full md:hidden" onClick={showEvents}>
                            {t("release.planner.showEvents")}
                        </Button>
                        {summary ? (
                            <Summary rows={data.rows} state={state} clears={clears} totals={totals} lookup={data.lookup} onRemove={(skin) => pickSkin(skin, false)} t={t} locale={locale} />
                        ) : current ? (
                            <EventDetail row={current} state={state} clears={clears} hidden={hidden} total={totals.get(current.key)} today={today} lookup={data.lookup} onPick={pickSkin} onStage={setStage} onAllStages={setAllStages} onResetStages={resetStages} t={t} />
                        ) : null}
                    </div>
                </div>
            )}
        </div>
    );
}

function rowTag(row: IPlanRow, t: PlannerT): string {
    if (row.kind === "review") return t("release.planner.tag.review");
    if (row.kind === "listing") return t("release.planner.tag.listing");
    return row.rerun ? t("release.planner.tag.rerun") : t("release.planner.tag.event");
}

function EventCard({ row, total, state, hidden, active, onOpen, t, locale }: { row: IPlanRow; total: IRowBalance | undefined; state: IPlanState; hidden: ReadonlySet<string>; active: boolean; onOpen: () => void; t: PlannerT; locale: string }): React.ReactElement {
    const art = useArt(row.imagePath);
    const picked = row.skins.filter((s) => state.picks[s.skinId]).length;
    const outfits = row.skins.filter((s) => !hidden.has(s.skinId)).length;
    return (
        <button type="button" onClick={onOpen} className={cn("grid w-full cursor-pointer gap-x-3 gap-y-2 rounded-xl border p-2.5 text-left transition-all hover:border-primary/50 sm:grid-cols-[minmax(0,1fr)_auto]", active ? "border-primary bg-primary/5 ring-2 ring-primary/20" : "border-border/60 bg-card")}>
            <div className="flex min-w-0 flex-col gap-1">
                <CnName cn={row.nameCn} en={row.nameEn} auto={row.nameAuto} compact primaryClassName="font-sans font-semibold text-[13px] text-foreground">
                    <Tag className={row.kind === "listing" ? "text-fuchsia-400" : row.kind === "review" ? "text-teal-400" : undefined}>{rowTag(row, t)}</Tag>
                </CnName>
                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                    {formatDate(row.enStart, locale)}
                    {row.resolution.status === "estimated" ? t("release.planner.card.estimated") : ""}
                    {outfits > 0 ? t("release.planner.card.outfits", { count: outfits }) : ""}
                    {picked > 0 ? t("release.planner.card.picked", { count: picked }) : ""}
                </span>
                {art.src && <img src={art.src} alt="" loading="lazy" onError={art.onError} className="mt-1 aspect-video w-full rounded-md bg-muted object-cover" />}
            </div>
            <Calcs total={total} inline className="self-center max-sm:justify-self-start" />
        </button>
    );
}

interface IEventDetailProps {
    row: IPlanRow;
    state: IPlanState;
    clears: StageClears;
    hidden: ReadonlySet<string>;
    total: IRowBalance | undefined;
    today: Date;
    lookup: OperatorLookup;
    onPick: (skin: IPlanSkin, on: boolean) => void;
    onStage: (row: IPlanRow, key: string, on: boolean) => void;
    onAllStages: (row: IPlanRow, on: boolean) => void;
    onResetStages: (row: IPlanRow) => void;
    t: PlannerT;
}

const STATUS_TITLE_KEYS: Record<StageStatus, (keyof typeof messages & string) | undefined> = {
    claimed: "release.planner.status.claimed",
    open: "release.planner.status.open",
    unrated: "release.planner.status.unrated",
    unknown: undefined,
};

function EventDetail({ row, state, clears, hidden, total, today, lookup, onPick, onStage, onAllStages, onResetStages, t }: IEventDetailProps): React.ReactElement {
    const art = useArt(row.imagePath);
    const income = rowIncome(row, state, clears);
    const potential = rowPotential(row);
    const allOn = row.opStages.length > 0 && row.opStages.every((st) => stageOn(row, st, state, clears));
    const deviates = rowDeviates(row, state, clears);
    const tally = React.useMemo(() => {
        const t: Record<StageStatus, number> = { claimed: 0, open: 0, unrated: 0, unknown: 0 };
        for (const st of row.opStages) t[stageStatus(st, clears)] += 1;
        return t;
    }, [row.opStages, clears]);
    const onRecord = tally.claimed + tally.open + tally.unrated;
    const groups = React.useMemo(() => {
        const map = new Map<string, { name: string; rerun: boolean; skins: IPlanSkin[] }>();
        for (const s of row.skins) {
            if (hidden.has(s.skinId)) continue;
            const key = `${s.groupName}|${s.rerun ? "r" : "n"}`;
            const g = map.get(key) ?? { name: s.groupName, rerun: s.rerun, skins: [] };
            g.skins.push(s);
            map.set(key, g);
        }
        return [...map.values()];
    }, [row.skins, hidden]);
    const ownedHidden = row.skins.reduce((n, s) => n + (hidden.has(s.skinId) ? 1 : 0), 0);
    return (
        <div className="flex flex-col gap-4 rounded-xl border border-border bg-card p-4">
            <div className={cn("grid gap-x-4 gap-y-2 sm:items-start", art.src ? "grid-cols-[112px_minmax(0,1fr)] sm:grid-cols-[200px_minmax(0,1fr)_auto]" : "grid-cols-1 sm:grid-cols-[minmax(0,1fr)_auto]")}>
                {art.src && <img src={art.src} alt="" loading="lazy" onError={art.onError} className="aspect-video w-full rounded-md bg-muted object-cover sm:w-50" />}
                <div className="flex min-w-0 flex-col gap-1">
                    <CnName cn={row.nameCn} en={row.nameEn} auto={row.nameAuto} primaryClassName="font-sans font-bold text-[16px] text-foreground">
                        <Tag>{rowTag(row, t)}</Tag>
                    </CnName>
                    <Calcs total={total} className="w-max" />
                </div>
                <div className="col-span-full sm:col-span-1">
                    <ResolutionBadge resolution={row.resolution} today={today} />
                </div>
            </div>
            {row.opStages.length > 0 && (
                <div className="flex flex-col gap-2 border-border/40 border-t pt-3">
                    <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
                        <span className="font-sans font-semibold text-[12.5px] text-foreground">{t("release.planner.stages")}</span>
                        <span className="font-mono text-[11px] text-muted-foreground tabular-nums">
                            {t("release.planner.stages.income", { income, potential })} <OpIcon className="size-3.5" />
                        </span>
                        {onRecord > 0 && (
                            <span className="font-sans text-[11px] text-muted-foreground" title={tally.unknown > 0 ? t("release.planner.stages.unknownTitle", { count: tally.unknown }) : undefined}>
                                {[
                                    tally.claimed > 0 && t("release.planner.stages.claimed", { count: tally.claimed }),
                                    tally.open > 0 && t("release.planner.stages.open", { count: tally.open }),
                                    tally.unrated > 0 && t("release.planner.stages.unrated", { count: tally.unrated }),
                                    tally.unknown > 0 && t("release.planner.stages.unknown", { count: tally.unknown }),
                                ]
                                    .filter(Boolean)
                                    .join(t("release.planner.stages.tallyJoin"))}
                            </span>
                        )}
                        {deviates ? (
                            <button type="button" onClick={() => onResetStages(row)} title={clears ? t("release.planner.stages.resetAccount") : t("release.planner.stages.resetDefaults")} className="cursor-pointer font-sans text-[11.5px] text-primary hover:underline">
                                {t("release.planner.stages.resetAll")}
                            </button>
                        ) : (
                            <button type="button" onClick={() => onAllStages(row, !allOn)} className="cursor-pointer font-sans text-[11.5px] text-primary hover:underline">
                                {allOn ? t("release.planner.stages.clearAll") : t("release.planner.stages.selectAll")}
                            </button>
                        )}
                    </div>
                    <div className="flex flex-wrap gap-1.5">
                        {row.opStages.map((st) => {
                            const on = stageOn(row, st, state, clears);
                            const status = stageStatus(st, clears);
                            return (
                                <button
                                    key={stageKey(st)}
                                    type="button"
                                    aria-pressed={on}
                                    onClick={() => onStage(row, stageKey(st), !on)}
                                    title={STATUS_TITLE_KEYS[status] ? t(STATUS_TITLE_KEYS[status]) : undefined}
                                    className={cn(
                                        "inline-flex cursor-pointer items-center gap-1.5 rounded-md border px-2 py-1 font-mono text-[11px] transition-colors",
                                        on ? "border-primary/50 bg-primary/10 text-foreground" : "border-border/60 bg-muted/30 text-muted-foreground",
                                        st.challenge && "border-dashed",
                                        status === "claimed" && !on && "line-through opacity-70",
                                    )}
                                >
                                    <span aria-hidden="true" className={cn("size-2 rounded-full", on ? "bg-primary" : status === "unknown" && clears ? "border border-muted-foreground/60 bg-transparent" : "bg-muted-foreground/40")} />
                                    {st.code}
                                    {st.challenge ? t("release.planner.stages.cm") : ""}
                                    {st.op !== 1 ? t("release.planner.stages.multiplier", { count: st.op }) : ""}
                                </button>
                            );
                        })}
                    </div>
                </div>
            )}
            {groups.length === 0 ? (
                <p className="m-0 border-border/40 border-t pt-3 font-sans text-[12.5px] text-muted-foreground">{ownedHidden > 0 ? t("release.planner.ownedHidden", { count: ownedHidden }) : t("release.planner.noOutfit")}</p>
            ) : (
                groups.map((g) => (
                    <div key={`${g.name}|${g.rerun}`} className="flex flex-col gap-2 border-border/40 border-t pt-3">
                        <div className="flex items-baseline gap-2">
                            <span className="font-sans font-semibold text-[13px] text-foreground">{g.name}</span>
                            <Tag className={g.rerun ? "text-violet-400" : "text-fuchsia-400"}>{g.rerun ? t("release.planner.group.rerun") : t("release.planner.group.new")}</Tag>
                        </div>
                        <div className="flex flex-wrap gap-3">
                            {g.skins.map((s) => (
                                <SkinCard key={s.skinId} skin={s} on={!!state.picks[s.skinId]} lookup={lookup} onPick={onPick} />
                            ))}
                        </div>
                    </div>
                ))
            )}
            {groups.length > 0 && ownedHidden > 0 && <p className="m-0 font-sans text-[11.5px] text-muted-foreground">{t("release.planner.ownedHidden", { count: ownedHidden })}</p>}
        </div>
    );
}

interface ISummaryProps {
    rows: IPlanRow[];
    state: IPlanState;
    clears: StageClears;
    totals: Map<string, IRowBalance>;
    lookup: OperatorLookup;
    onRemove: (skin: IPlanSkin) => void;
    t: PlannerT;
    locale: string;
}

function Summary({ rows, state, clears, totals, lookup, onRemove, t, locale }: ISummaryProps): React.ReactElement {
    const autoOn = useAutoTranslate();
    const income = rows.reduce((sum, row) => sum + rowIncome(row, state, clears), 0);
    const expense = rows.reduce((sum, row) => sum + rowExpense(row, state), 0);
    const tail = rows.at(-1);
    const end = tail ? (totals.get(tail.key)?.balance ?? state.initial) : state.initial;
    const short = rows.find((row) => (totals.get(row.key)?.balance ?? 0) < 0);
    const picked = rows.flatMap((row) => {
        const skins = row.skins.filter((s) => state.picks[s.skinId]);
        return skins.length > 0 ? [{ row, skins }] : [];
    });
    return (
        <div className="flex flex-col gap-4 rounded-xl border border-border bg-card p-4">
            <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                <span className="font-bold font-sans text-[16px] text-foreground">{t("release.planner.summary.title")}</span>
                <span className="inline-flex flex-wrap items-center gap-x-2 font-mono text-[12px] tabular-nums">
                    <span className="text-muted-foreground">{t("release.planner.summary.start")}</span>
                    <Op value={state.initial} />
                    <Op value={income} sign="+" className="text-emerald-500" />
                    <Op value={expense} sign="-" className="text-rose-400" />
                    <span className="text-muted-foreground">{t("release.planner.summary.end")}</span>
                    <Op value={end} className={cn("font-semibold", end < 0 ? "text-destructive-foreground" : "text-foreground")} />
                </span>
            </div>
            {short && (
                <p className="m-0 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 font-sans text-[12px] text-destructive-foreground">
                    {t("release.planner.summary.short", { count: -(totals.get(short.key)?.balance ?? 0), event: resolveName(short.nameCn, short.nameEn, short.nameAuto, autoOn).text, date: formatDate(short.enStart, locale) })}
                </p>
            )}
            {picked.length === 0 ? (
                <p className="m-0 font-sans text-[12.5px] text-muted-foreground">{t("release.planner.summary.empty")}</p>
            ) : (
                picked.map(({ row, skins }) => (
                    <section key={row.key} className="grid grid-cols-[minmax(0,1fr)_auto] gap-x-4 gap-y-2 border-border/60 border-t pt-3">
                        <div className="flex min-w-0 flex-col gap-2">
                            <CnName cn={row.nameCn} en={row.nameEn} auto={row.nameAuto} compact primaryClassName="font-sans font-semibold text-[13px] text-foreground">
                                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{formatDate(row.enStart, locale)}</span>
                            </CnName>
                            <ul className="m-0 grid list-none grid-cols-[repeat(auto-fill,minmax(15rem,1fr))] gap-1.5 p-0">
                                {skins.map((s) => (
                                    <SummarySkin key={s.skinId} skin={s} lookup={lookup} autoOn={autoOn} onRemove={onRemove} removeLabel={t("release.planner.summary.remove")} />
                                ))}
                            </ul>
                        </div>
                        <Calcs total={totals.get(row.key)} className="self-start" />
                    </section>
                ))
            )}
        </div>
    );
}

/** One picked outfit: operator first, outfit second, an accent in the outfit's brand colour, then its price and a remove cross. */
function SummarySkin({ skin, lookup, autoOn, onRemove, removeLabel }: { skin: IPlanSkin; lookup: OperatorLookup; autoOn: boolean; onRemove: (skin: IPlanSkin) => void; removeLabel: string }): React.ReactElement {
    const entry = lookup.get(skin.charId);
    const opName = operatorLabel(skin.charId, entry, skin.charName, autoOn).text;
    const skinName = resolveName(skin.skinName, skin.skinNameEn, skin.skinNameAuto, autoOn).text;
    return (
        <li className="flex min-w-0 items-center gap-2.5 rounded-md border border-border border-l-[3px] bg-muted/40 py-1 pr-1 pl-1.5" style={{ borderLeftColor: skin.colors[0] ?? FALLBACK_COLOR }}>
            <span className="size-8 shrink-0 overflow-hidden rounded-sm">
                <SkinAvatar skinId={skin.skinId} charId={skin.charId} name={opName} server={entry ? undefined : "cn"} />
            </span>
            <span className="flex min-w-0 flex-1 flex-col leading-tight" title={`${opName} · ${skinName}`}>
                <span className="truncate font-sans font-semibold text-[12.5px] text-foreground">{opName}</span>
                <span className="truncate font-sans text-[11.5px] text-muted-foreground">{skinName}</span>
            </span>
            <Op value={skin.price.price} className="shrink-0 text-[11.5px] text-foreground" />
            <button type="button" onClick={() => onRemove(skin)} aria-label={removeLabel} title={removeLabel} className="flex size-7 shrink-0 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground">
                <X className="size-3.5" aria-hidden="true" />
            </button>
        </li>
    );
}

function SkinAvatar({ skinId, charId, name, server }: { skinId: string; charId: string; name: string; server?: string }): React.ReactElement {
    const [fallback, setFallback] = React.useState(0);
    // Of 262 planner outfits, CN has 258 skin avatars, EN has 220, and 4 have neither.
    if (fallback === 2) return <>{name.charAt(0).toUpperCase()}</>;
    return (
        <img
            src={fallback === 0 ? getAvatarById(skinId, "cn") : getAvatarById(charId, server)}
            alt=""
            aria-hidden="true"
            loading="lazy"
            decoding="async"
            draggable={false}
            onDragStart={(e) => e.preventDefault()}
            onError={() => setFallback((value) => value + 1)}
            className="block h-full w-full rounded-[inherit] object-cover"
        />
    );
}
