import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ListChecks, RotateCcw } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useAuth } from "#/hooks/use-auth";
import { refreshRosterFn } from "#/lib/api/auth";
import { userStageClearsQueryOptions } from "#/lib/api/stages";
import { userQueryOptions } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { useAutoTranslate } from "../autoTranslate";
import { formatDate } from "../helpers";
import { balances, EMPTY_STATE, type IPlanRow, type IPlanSkin, type IPlanState, type IRowBalance, rowDeviates, rowExpense, rowIncome, rowPotential, type StageClears, type StageStatus, stageKey, stageOn, stageStatus, usePlanData } from "../plan";
import { useStoredState } from "../planStore";
import { Calcs, Op, OpIcon } from "./Primes";
import { ResolutionBadge } from "./ResolutionBadge";
import { FarmStages } from "./ScheduleShared";
import { cardVars, FALLBACK_COLOR, SkinCard, stopsOf } from "./SkinCard";
import { CnName, type OperatorLookup, ReleaseEmpty, ReleaseError, ReleaseLoading, resolveName, Tag, ToggleField, useArt } from "./shared";

interface IPlannerTabProps {
    today: Date;
}

export function PlannerTab({ today }: IPlannerTabProps): React.ReactElement {
    const [showPast, setShowPast] = React.useState(false);
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
    const open = (key: string) => {
        setSelected(key);
        setSummary(false);
        setPane("detail");
    };

    if (data.isPending) return <ReleaseLoading />;
    if (data.error) return <ReleaseError error={data.error} onRetry={() => window.location.reload()} />;

    return (
        <div className="flex flex-col gap-3">
            <div className="flex flex-wrap items-center gap-x-5 gap-y-2">
                <ToggleField id="planner-show-past" label="Show past" checked={showPast} onChange={setShowPast} />
                <span className="font-sans text-[11px] text-muted-foreground">
                    Income is each event's first-clear Originite Prime, expenses the outfits you pick. Stage defaults come from your account: three-starred stages start unchecked, the rest checked, with closed events read from your missions, medals and story unlocks. Store outfits cost 18, plus 3 each for dynamic art,
                    own voice lines and a special variant (the old 15 tier shows as 18); outfits handed out elsewhere cost nothing.{" "}
                    {uid ? (sync.saving ? "Saving to your account…" : sync.savedAt ? `Saved to your account ${formatDate(sync.savedAt)}.` : "Saved to your account as you go.") : "Sign in to keep the plan on your account; until then it stays in this browser."}
                </span>
            </div>
            {data.rows.length === 0 ? (
                <ReleaseEmpty title="Nothing ahead" description="No upcoming EN event or store sale has a date. Turn on Show past to see recent ones." />
            ) : (
                <div className="grid grid-cols-1 gap-4 md:grid-cols-[340px_minmax(0,1fr)] md:items-start">
                    <div className={cn("flex flex-col gap-2", pane !== "events" && "max-md:hidden")}>
                        <div className="flex flex-col gap-2 rounded-xl border border-border bg-card p-3">
                            <div className="flex items-center gap-2">
                                <Button
                                    size="sm"
                                    variant={summary ? "default" : "outline"}
                                    onClick={() => {
                                        setSummary(true);
                                        setPane("detail");
                                    }}
                                >
                                    <ListChecks className="mr-1.5 size-4" />
                                    Selection summary
                                </Button>
                                <Button size="sm" variant="ghost" onClick={() => setState(EMPTY_STATE)} aria-label="Reset the plan">
                                    <RotateCcw className="size-4" />
                                </Button>
                            </div>
                            <div className="flex flex-wrap items-center gap-2">
                                <label htmlFor="planner-initial" className="flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground">
                                    <OpIcon />
                                    Initial primes
                                </label>
                                <Input id="planner-initial" type="number" min={0} inputMode="numeric" value={state.initial} onChange={(e) => setInitial(Number(e.target.value) || 0, true)} className="h-8 w-28 text-right font-mono" />
                                {uid && (
                                    <span className="font-sans text-[11px] text-muted-foreground">
                                        {accountPrimes === null ? (
                                            <>
                                                not synced yet:{" "}
                                                <button type="button" disabled={resync.isPending} onClick={() => resync.mutate()} className="cursor-pointer text-primary hover:underline disabled:opacity-60">
                                                    {resync.isPending ? "syncing…" : "sync my account"}
                                                </button>
                                                {resync.isError ? " (failed)" : ""}
                                            </>
                                        ) : state.initialManual ? (
                                            <button type="button" onClick={() => setInitial(accountPrimes, false)} className="cursor-pointer text-primary hover:underline">
                                                use my account's {accountPrimes}
                                            </button>
                                        ) : (
                                            `from your account${profile.data?.updated_at ? `, synced ${formatDate(Date.parse(profile.data.updated_at) / 1000)}` : ""}`
                                        )}
                                    </span>
                                )}
                            </div>
                        </div>
                        <div className="flex flex-col gap-2 md:max-h-[calc(100vh-14rem)] md:overflow-y-auto md:pr-1">
                            <Button size="sm" variant="outline" className="sticky top-16 z-10 w-full bg-background/90 backdrop-blur md:hidden" onClick={() => setPane("detail")}>
                                {summary ? "Show summary" : "Show the selected event"}
                            </Button>
                            {data.rows.map((row) => (
                                <EventCard key={row.key} row={row} total={totals.get(row.key)} state={state} active={!summary && current?.key === row.key} onOpen={() => open(row.key)} />
                            ))}
                        </div>
                    </div>
                    <div className={cn("min-w-0", pane !== "detail" && "max-md:hidden")}>
                        <Button size="sm" variant="outline" className="sticky top-16 z-10 mb-3 w-full bg-background/90 backdrop-blur md:hidden" onClick={() => setPane("events")}>
                            Show events
                        </Button>
                        {summary ? (
                            <Summary rows={data.rows} state={state} clears={clears} totals={totals} lookup={data.lookup} onRemove={(skin) => pickSkin(skin, false)} />
                        ) : current ? (
                            <EventDetail row={current} state={state} clears={clears} total={totals.get(current.key)} today={today} lookup={data.lookup} onPick={pickSkin} onStage={setStage} onAllStages={setAllStages} onResetStages={resetStages} />
                        ) : null}
                    </div>
                </div>
            )}
        </div>
    );
}

function rowTag(row: IPlanRow): string {
    return row.kind === "event" ? (row.rerun ? "Rerun" : "Event") : "Store sale";
}

function EventCard({ row, total, state, active, onOpen }: { row: IPlanRow; total: IRowBalance | undefined; state: IPlanState; active: boolean; onOpen: () => void }): React.ReactElement {
    const art = useArt(row.imagePath);
    const picked = row.skins.filter((s) => state.picks[s.skinId]).length;
    return (
        <button type="button" onClick={onOpen} className={cn("grid w-full cursor-pointer gap-x-3 gap-y-2 rounded-xl border p-2.5 text-left transition-all hover:border-primary/50 sm:grid-cols-[minmax(0,1fr)_auto]", active ? "border-primary bg-primary/5 ring-2 ring-primary/20" : "border-border/60 bg-card")}>
            <div className="flex min-w-0 flex-col gap-1">
                <CnName cn={row.nameCn} en={row.nameEn} auto={row.nameAuto} compact primaryClassName="font-sans font-semibold text-[13px] text-foreground">
                    <Tag className={row.kind === "listing" ? "text-fuchsia-400" : undefined}>{rowTag(row)}</Tag>
                </CnName>
                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                    {formatDate(row.enStart)}
                    {row.resolution.status === "estimated" ? " est." : ""}
                    {row.skins.length > 0 ? ` · ${row.skins.length} outfit${row.skins.length === 1 ? "" : "s"}` : ""}
                    {picked > 0 ? `, ${picked} picked` : ""}
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
    total: IRowBalance | undefined;
    today: Date;
    lookup: OperatorLookup;
    onPick: (skin: IPlanSkin, on: boolean) => void;
    onStage: (row: IPlanRow, key: string, on: boolean) => void;
    onAllStages: (row: IPlanRow, on: boolean) => void;
    onResetStages: (row: IPlanRow) => void;
}

const STATUS_TITLE: Record<StageStatus, string | undefined> = {
    claimed: "Three-starred on your account: its Originite Prime is already claimed",
    open: "Your account shows this short of three stars: its Originite Prime is still available",
    unrated: "Cleared on your account, star rating not on record",
    unknown: undefined,
};

function EventDetail({ row, state, clears, total, today, lookup, onPick, onStage, onAllStages, onResetStages }: IEventDetailProps): React.ReactElement {
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
            const key = `${s.groupName}|${s.rerun ? "r" : "n"}`;
            const g = map.get(key) ?? { name: s.groupName, rerun: s.rerun, skins: [] };
            g.skins.push(s);
            map.set(key, g);
        }
        return [...map.values()];
    }, [row.skins]);
    return (
        <div className="flex flex-col gap-4 rounded-xl border border-border bg-card p-4">
            <div className={cn("grid gap-x-4 gap-y-2 sm:items-start", art.src ? "grid-cols-[112px_minmax(0,1fr)] sm:grid-cols-[200px_minmax(0,1fr)_auto]" : "grid-cols-1 sm:grid-cols-[minmax(0,1fr)_auto]")}>
                {art.src && <img src={art.src} alt="" loading="lazy" onError={art.onError} className="aspect-video w-full rounded-md bg-muted object-cover sm:w-50" />}
                <div className="flex min-w-0 flex-col gap-1">
                    <CnName cn={row.nameCn} en={row.nameEn} auto={row.nameAuto} primaryClassName="font-sans font-bold text-[16px] text-foreground">
                        <Tag>{rowTag(row)}</Tag>
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
                        <span className="font-sans font-semibold text-[12.5px] text-foreground">Event stages</span>
                        <span className="font-mono text-[11px] text-muted-foreground tabular-nums">
                            {income} of {potential} <OpIcon className="size-3.5" />
                        </span>
                        {onRecord > 0 && (
                            <span className="font-sans text-[11px] text-muted-foreground" title={tally.unknown > 0 ? `${tally.unknown} ${tally.unknown === 1 ? "stage has" : "stages have"} no record on your account; tick any you did not clear.` : undefined}>
                                {[tally.claimed > 0 && `${tally.claimed} claimed`, tally.open > 0 && `${tally.open} open`, tally.unrated > 0 && `${tally.unrated} cleared, rating not on record`, tally.unknown > 0 && `${tally.unknown} no record`].filter(Boolean).join(" · ")}
                            </span>
                        )}
                        {deviates ? (
                            <button type="button" onClick={() => onResetStages(row)} title={clears ? "Back to what your account shows" : "Back to the defaults"} className="cursor-pointer font-sans text-[11.5px] text-primary hover:underline">
                                Reset all
                            </button>
                        ) : (
                            <button type="button" onClick={() => onAllStages(row, !allOn)} className="cursor-pointer font-sans text-[11.5px] text-primary hover:underline">
                                {allOn ? "Clear all" : "Select all"}
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
                                    title={STATUS_TITLE[status]}
                                    className={cn(
                                        "inline-flex cursor-pointer items-center gap-1.5 rounded-md border px-2 py-1 font-mono text-[11px] transition-colors",
                                        on ? "border-primary/50 bg-primary/10 text-foreground" : "border-border/60 bg-muted/30 text-muted-foreground",
                                        st.challenge && "border-dashed",
                                        status === "claimed" && !on && "line-through opacity-70",
                                    )}
                                >
                                    <span aria-hidden="true" className={cn("size-2 rounded-full", on ? "bg-primary" : status === "unknown" && clears ? "border border-muted-foreground/60 bg-transparent" : "bg-muted-foreground/40")} />
                                    {st.code}
                                    {st.challenge ? " CM" : ""}
                                    {st.op !== 1 ? ` ×${st.op}` : ""}
                                </button>
                            );
                        })}
                    </div>
                </div>
            )}
            {row.farmStages.length > 0 && (
                <div className="flex flex-col gap-2 border-border/40 border-t pt-3">
                    <span className="font-sans font-semibold text-[12.5px] text-foreground">
                        Farming stages <span className="font-medium font-mono text-[10.5px] text-muted-foreground">· CN drops</span>
                    </span>
                    <FarmStages stages={row.farmStages} />
                </div>
            )}
            {row.farmStages.length === 0 && row.kind === "event" && row.opStages.length > 0 && (
                <p className="m-0 border-border/40 border-t pt-3 font-sans text-[11.5px] text-muted-foreground">No drop table in the client for this event: the game carries drops only while stages are open, and these have not opened on CN (or their last run closed) since the extract.</p>
            )}
            {groups.length === 0 ? (
                <p className="m-0 border-border/40 border-t pt-3 font-sans text-[12.5px] text-muted-foreground">No outfit arrives with this one.</p>
            ) : (
                groups.map((g) => (
                    <div key={`${g.name}|${g.rerun}`} className="flex flex-col gap-2 border-border/40 border-t pt-3">
                        <div className="flex items-baseline gap-2">
                            <span className="font-sans font-semibold text-[13px] text-foreground">{g.name}</span>
                            <Tag className={g.rerun ? "text-violet-400" : "text-fuchsia-400"}>{g.rerun ? "Rerun" : "New"}</Tag>
                        </div>
                        <div className="flex flex-wrap gap-3">
                            {g.skins.map((s) => (
                                <SkinCard key={s.skinId} skin={s} on={!!state.picks[s.skinId]} lookup={lookup} onPick={onPick} />
                            ))}
                        </div>
                    </div>
                ))
            )}
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
}

function Summary({ rows, state, clears, totals, lookup, onRemove }: ISummaryProps): React.ReactElement {
    const autoOn = useAutoTranslate();
    const income = rows.reduce((sum, row) => sum + rowIncome(row, state, clears), 0);
    const expense = rows.reduce((sum, row) => sum + rowExpense(row, state), 0);
    const tail = rows.at(-1);
    const end = tail ? (totals.get(tail.key)?.balance ?? state.initial) : state.initial;
    const short = rows.find((row) => (totals.get(row.key)?.balance ?? 0) < 0);
    const touched = rows.filter((row) => rowIncome(row, state, clears) > 0 || row.skins.some((s) => state.picks[s.skinId]));
    return (
        <div className="flex flex-col gap-4 rounded-xl border border-border bg-card p-4">
            <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                <span className="font-bold font-sans text-[16px] text-foreground">Selection summary</span>
                <span className="inline-flex flex-wrap items-center gap-x-2 font-mono text-[12px] tabular-nums">
                    <span className="text-muted-foreground">start</span>
                    <Op value={state.initial} />
                    <Op value={income} sign="+" className="text-emerald-500" />
                    <Op value={expense} sign="-" className="text-rose-400" />
                    <span className="text-muted-foreground">end</span>
                    <Op value={end} className={cn("font-semibold", end < 0 ? "text-destructive-foreground" : "text-foreground")} />
                </span>
            </div>
            {short && (
                <p className="m-0 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 font-sans text-[12px] text-destructive-foreground">
                    Short by {-(totals.get(short.key)?.balance ?? 0)} at {resolveName(short.nameCn, short.nameEn, short.nameAuto, autoOn).text} ({formatDate(short.enStart)}).
                </p>
            )}
            {touched.length === 0 ? (
                <p className="m-0 font-sans text-[12.5px] text-muted-foreground">Nothing selected yet. Open an event on the left and pick its stages and outfits.</p>
            ) : (
                touched.map((row) => {
                    const picked = row.skins.filter((s) => state.picks[s.skinId]);
                    return (
                        <div key={row.key} className="grid grid-cols-[minmax(0,1fr)_auto] gap-x-3 gap-y-2 border-border/40 border-t pt-3">
                            <div className="flex min-w-0 flex-col gap-2">
                                <CnName cn={row.nameCn} en={row.nameEn} auto={row.nameAuto} compact primaryClassName="font-sans font-semibold text-[13px] text-foreground">
                                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{formatDate(row.enStart)}</span>
                                </CnName>
                                {picked.length > 0 && (
                                    <ul className="m-0 flex list-none flex-wrap gap-2 p-0">
                                        {picked.map((s) => {
                                            const op = lookup.get(s.charId);
                                            return (
                                                <li key={s.skinId} className="flex items-center gap-2 border-[2px] py-1 pr-1 pl-1" style={{ ...cardVars(s.colors, 220), borderImage: `linear-gradient(90deg, ${stopsOf(s.colors.length > 0 ? s.colors : [FALLBACK_COLOR])}) 1`, background: "rgba(0,0,0,0.5)" }}>
                                                    <span className="size-7 shrink-0 overflow-hidden rounded-sm">
                                                        <OperatorAvatar charId={s.charId} name={op?.name ?? s.charId} server={op ? undefined : "cn"} />
                                                    </span>
                                                    <span className="max-w-40 truncate font-sans text-[12px] text-foreground">{resolveName(s.skinName, s.skinNameEn, s.skinNameAuto, autoOn).text}</span>
                                                    <Op value={s.price.price} className="text-[11px]" />
                                                    <button type="button" onClick={() => onRemove(s)} aria-label="Remove" className="flex size-6 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground">
                                                        ×
                                                    </button>
                                                </li>
                                            );
                                        })}
                                    </ul>
                                )}
                            </div>
                            <Calcs total={totals.get(row.key)} className="self-start" />
                        </div>
                    );
                })
            )}
        </div>
    );
}
