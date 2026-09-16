import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ChevronDown, ListChecks, RotateCcw } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useAuth } from "#/hooks/use-auth";
import { refreshRosterFn } from "#/lib/api/auth";
import { userStageClearsQueryOptions } from "#/lib/api/stages";
import { userQueryOptions } from "#/lib/api/user";
import { useFormatters, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ShopGood } from "#/types/generated/ShopGood";
import { useAutoTranslate } from "../autoTranslate";
import { formatDate, formatDateRange } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { balances, EMPTY_STATE, type IPlanRow, type IPlanSkin, type IPlanState, type IRowBalance, rowDeviates, rowExpense, rowIncome, rowPotential, SANITY_PER_TOKEN, SHOP_KIND_LABEL_KEYS, type StageClears, type StageStatus, shopBuyout, shopGroups, stageKey, stageOn, stageStatus, usePlanData } from "../plan";
import type { messages as planMessages } from "../plan.messages";
import { useStoredState } from "../planStore";
import type { messages } from "./PlannerTab.messages";
import { Calcs, Op, OpIcon } from "./Primes";
import { ResolutionBadge } from "./ResolutionBadge";
import { FarmStages } from "./ScheduleShared";
import { cardVars, FALLBACK_COLOR, SkinCard, stopsOf } from "./SkinCard";
import { CnName, type OperatorLookup, ReleaseEmpty, ReleaseError, ReleaseLoading, resolveName, Tag, ToggleField, useArt } from "./shared";

/** This tab renders its own chrome plus the shop labels and date wording the impl modules carry. */
type PlannerT = TypedT<typeof messages & typeof planMessages & typeof helperMessages>;

interface IPlannerTabProps {
    today: Date;
}

export function PlannerTab({ today }: IPlannerTabProps): React.ReactElement {
    const t: PlannerT = useT("tools");
    const locale = useLocale();
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
                <ToggleField id="planner-show-past" label={t("release.planner.showPast")} checked={showPast} onChange={setShowPast} />
                <span className="font-sans text-[11px] text-muted-foreground">
                    {t("release.planner.blurb")} {uid ? (sync.saving ? t("release.planner.saving") : sync.savedAt ? t("release.planner.saved", { date: formatDate(sync.savedAt, locale) }) : t("release.planner.autosave")) : t("release.planner.signedOut")}
                </span>
            </div>
            {data.rows.length === 0 ? (
                <ReleaseEmpty title={t("release.planner.empty.title")} description={t("release.planner.empty.desc")} />
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
                        <div className="flex flex-col gap-2 md:max-h-[calc(100vh-14rem)] md:overflow-y-auto md:pr-1">
                            <Button size="sm" variant="outline" className="sticky top-16 z-10 w-full bg-background/90 backdrop-blur md:hidden" onClick={() => setPane("detail")}>
                                {summary ? t("release.planner.showSummary") : t("release.planner.showSelected")}
                            </Button>
                            {data.rows.map((row) => (
                                <EventCard key={row.key} row={row} total={totals.get(row.key)} state={state} active={!summary && current?.key === row.key} onOpen={() => open(row.key)} t={t} locale={locale} />
                            ))}
                        </div>
                    </div>
                    <div className={cn("min-w-0", pane !== "detail" && "max-md:hidden")}>
                        <Button size="sm" variant="outline" className="sticky top-16 z-10 mb-3 w-full bg-background/90 backdrop-blur md:hidden" onClick={() => setPane("events")}>
                            {t("release.planner.showEvents")}
                        </Button>
                        {summary ? (
                            <Summary rows={data.rows} state={state} clears={clears} totals={totals} lookup={data.lookup} onRemove={(skin) => pickSkin(skin, false)} t={t} locale={locale} />
                        ) : current ? (
                            <EventDetail row={current} state={state} clears={clears} total={totals.get(current.key)} today={today} lookup={data.lookup} onPick={pickSkin} onStage={setStage} onAllStages={setAllStages} onResetStages={resetStages} t={t} locale={locale} />
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

function EventCard({ row, total, state, active, onOpen, t, locale }: { row: IPlanRow; total: IRowBalance | undefined; state: IPlanState; active: boolean; onOpen: () => void; t: PlannerT; locale: string }): React.ReactElement {
    const art = useArt(row.imagePath);
    const picked = row.skins.filter((s) => state.picks[s.skinId]).length;
    return (
        <button type="button" onClick={onOpen} className={cn("grid w-full cursor-pointer gap-x-3 gap-y-2 rounded-xl border p-2.5 text-left transition-all hover:border-primary/50 sm:grid-cols-[minmax(0,1fr)_auto]", active ? "border-primary bg-primary/5 ring-2 ring-primary/20" : "border-border/60 bg-card")}>
            <div className="flex min-w-0 flex-col gap-1">
                <CnName cn={row.nameCn} en={row.nameEn} auto={row.nameAuto} compact primaryClassName="font-sans font-semibold text-[13px] text-foreground">
                    <Tag className={row.kind === "listing" ? "text-fuchsia-400" : row.kind === "review" ? "text-teal-400" : undefined}>{rowTag(row, t)}</Tag>
                </CnName>
                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">
                    {formatDate(row.enStart, locale)}
                    {row.resolution.status === "estimated" ? t("release.planner.card.estimated") : ""}
                    {row.skins.length > 0 ? t("release.planner.card.outfits", { count: row.skins.length }) : ""}
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
    total: IRowBalance | undefined;
    today: Date;
    lookup: OperatorLookup;
    onPick: (skin: IPlanSkin, on: boolean) => void;
    onStage: (row: IPlanRow, key: string, on: boolean) => void;
    onAllStages: (row: IPlanRow, on: boolean) => void;
    onResetStages: (row: IPlanRow) => void;
    t: PlannerT;
    locale: string;
}

const STATUS_TITLE_KEYS: Record<StageStatus, (keyof typeof messages & string) | undefined> = {
    claimed: "release.planner.status.claimed",
    open: "release.planner.status.open",
    unrated: "release.planner.status.unrated",
    unknown: undefined,
};

function EventDetail({ row, state, clears, total, today, lookup, onPick, onStage, onAllStages, onResetStages, t, locale }: IEventDetailProps): React.ReactElement {
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
            {row.kind === "event" && (row.missionTokens > 0 || row.shop) && <ShopBuyout row={row} t={t} locale={locale} />}
            {row.farmStages.length > 0 && (
                <div className="flex flex-col gap-2 border-border/40 border-t pt-3">
                    <span className="font-sans font-semibold text-[12.5px] text-foreground">
                        {t("release.planner.farming")}
                        <span className="font-medium font-mono text-[10.5px] text-muted-foreground">{t("release.planner.farming.cnDrops")}</span>
                    </span>
                    <FarmStages stages={row.farmStages} />
                </div>
            )}
            {row.farmStages.length === 0 && row.kind === "event" && row.opStages.length > 0 && <p className="m-0 border-border/40 border-t pt-3 font-sans text-[11.5px] text-muted-foreground">{t("release.planner.farming.none")}</p>}
            {groups.length === 0 ? (
                <p className="m-0 border-border/40 border-t pt-3 font-sans text-[12.5px] text-muted-foreground">{t("release.planner.noOutfit")}</p>
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
        </div>
    );
}

function ShopBuyout({ row, t, locale }: { row: IPlanRow; t: PlannerT; locale: string }): React.ReactElement {
    const f = useFormatters();
    const buyout = shopBuyout(row);
    const [open, setOpen] = React.useState(false);
    const shop = row.shop;
    return (
        <div className="flex flex-col gap-1.5 border-border/40 border-t pt-3">
            <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
                <span className="font-sans font-semibold text-[12.5px] text-foreground">
                    {t("release.planner.shop")}
                    {shop && <span className="font-medium font-mono text-[10.5px] text-muted-foreground">{t("release.planner.shop.meta", { name: shop.shopName, server: shop.server.toUpperCase(), dates: formatDateRange(shop.startTime, shop.endTime, locale, t) })}</span>}
                </span>
                {shop && (
                    <button type="button" onClick={() => setOpen((v) => !v)} className="flex cursor-pointer items-center gap-1 font-sans text-[11.5px] text-muted-foreground hover:text-foreground">
                        {open ? t("release.planner.shop.hide") : t("release.planner.shop.show", { count: shop.goods.length })}
                        <ChevronDown className={cn("size-3.5 transition-transform", open && "rotate-180")} />
                    </button>
                )}
            </div>
            {buyout && shop ? (
                <>
                    <dl className="m-0 grid grid-cols-[auto_auto] gap-x-4 gap-y-0.5 font-mono text-[11.5px] tabular-nums">
                        <dt className="flex items-center gap-1.5 font-sans text-muted-foreground">
                            <ShopIcon item={shop.token} size="size-4" />
                            {t("release.planner.shop.buyEverything")}
                            <span className="text-[10px]">{t("release.planner.shop.limitedGoods", { count: buyout.limitedGoods })}</span>
                        </dt>
                        <dd className="m-0 text-right text-foreground">{f.number(buyout.total)}</dd>
                        <dt className="font-sans text-muted-foreground">{t("release.planner.shop.missions")}</dt>
                        <dd className="m-0 text-right text-emerald-500">{t("release.planner.shop.missionsValue", { count: f.number(buyout.missions) })}</dd>
                        <dt className="font-sans font-semibold text-foreground">{t("release.planner.shop.toFarm")}</dt>
                        <dd className="m-0 text-right font-semibold text-foreground">{t("release.planner.shop.toFarmValue", { count: f.number(buyout.remaining), token: shop.token.nameEn ?? shop.token.name, sanity: f.number(buyout.sanity) })}</dd>
                    </dl>
                    <p className="m-0 font-sans text-[10.5px] text-muted-foreground">{t("release.planner.shop.sanityNote", { perToken: SANITY_PER_TOKEN })}</p>
                    {open && <ShopGoods row={row} t={t} />}
                </>
            ) : (
                <p className="m-0 font-sans text-[11.5px] text-muted-foreground">{t("release.planner.shop.noShop", { count: row.missionTokens })}</p>
            )}
        </div>
    );
}

function ShopGoods({ row, t }: { row: IPlanRow; t: PlannerT }): React.ReactElement | null {
    const f = useFormatters();
    if (!row.shop) return null;
    const { limited, unlimited } = shopGroups(row.shop);
    const token = row.shop.token.nameEn ?? row.shop.token.name;
    return (
        <div className="flex flex-col gap-3 pt-1">
            {limited.map((g) => (
                <div key={g.kind} className="flex flex-col gap-1">
                    <div className="flex items-baseline justify-between gap-2">
                        <span className="font-sans font-semibold text-[11.5px] text-foreground">
                            {t(SHOP_KIND_LABEL_KEYS[g.kind])} <span className="font-medium font-mono text-[10px] text-muted-foreground">{t("release.planner.shop.groupCount", { count: g.goods.length })}</span>
                        </span>
                        <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{f.number(g.tokens)}</span>
                    </div>
                    <ul className="m-0 grid list-none grid-cols-1 gap-x-4 gap-y-0.5 p-0 sm:grid-cols-2">
                        {g.goods.map((good) => (
                            <ShopGoodRow key={good.goodId} good={good} t={t} />
                        ))}
                    </ul>
                </div>
            ))}
            {unlimited.length > 0 && (
                <div className="flex flex-col gap-1">
                    <span className="font-sans font-semibold text-[11.5px] text-foreground">
                        {t("release.planner.shop.unlimited")}
                        <span className="font-medium font-mono text-[10px] text-muted-foreground">{t("release.planner.shop.unlimitedNote", { token })}</span>
                    </span>
                    <ul className="m-0 grid list-none grid-cols-1 gap-x-4 gap-y-0.5 p-0 sm:grid-cols-2">
                        {unlimited.map((good) => (
                            <ShopGoodRow key={good.goodId} good={good} t={t} />
                        ))}
                    </ul>
                </div>
            )}
        </div>
    );
}

function ShopGoodRow({ good, t }: { good: ShopGood; t: PlannerT }): React.ReactElement {
    const f = useFormatters();
    const name = good.item.nameEn ?? good.item.name;
    const cnOnly = good.item.nameEn === null;
    return (
        <li className="flex min-w-0 items-center gap-2 font-mono text-[11px] tabular-nums">
            <ShopIcon item={good.item} size="size-6" />
            <span className="min-w-0 flex-1 truncate font-sans text-[11.5px] text-foreground" lang={cnOnly ? "zh-CN" : undefined} translate={cnOnly ? "yes" : undefined} title={name}>
                {name}
                {good.count > 1 && <span className="text-muted-foreground">{t("release.planner.shop.goodCount", { count: f.number(good.count) })}</span>}
            </span>
            <span className="shrink-0 text-muted-foreground">
                {f.number(good.price)}
                {good.availCount > 0 ? t("release.planner.shop.stock", { count: good.availCount }) : ""}
            </span>
            {good.availCount > 0 && <span className="w-12 shrink-0 text-right text-foreground">{f.number(good.price * good.availCount)}</span>}
        </li>
    );
}

function ShopIcon({ item, size }: { item: ShopGood["item"]; size: string }): React.ReactElement {
    const art = useArt(item.iconPath);
    return art.src ? <img src={art.src} alt="" loading="lazy" onError={art.onError} className={cn("shrink-0 rounded-sm bg-black/30 object-contain", size)} /> : <span aria-hidden="true" className={cn("shrink-0 rounded-sm bg-muted", size)} />;
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
    const touched = rows.filter((row) => rowIncome(row, state, clears) > 0 || row.skins.some((s) => state.picks[s.skinId]));
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
            {touched.length === 0 ? (
                <p className="m-0 font-sans text-[12.5px] text-muted-foreground">{t("release.planner.summary.empty")}</p>
            ) : (
                touched.map((row) => {
                    const picked = row.skins.filter((s) => state.picks[s.skinId]);
                    return (
                        <div key={row.key} className="grid grid-cols-[minmax(0,1fr)_auto] gap-x-3 gap-y-2 border-border/40 border-t pt-3">
                            <div className="flex min-w-0 flex-col gap-2">
                                <CnName cn={row.nameCn} en={row.nameEn} auto={row.nameAuto} compact primaryClassName="font-sans font-semibold text-[13px] text-foreground">
                                    <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{formatDate(row.enStart, locale)}</span>
                                </CnName>
                                {picked.length > 0 && (
                                    <ul className="m-0 flex list-none flex-wrap gap-2 p-0">
                                        {picked.map((s) => {
                                            const op = lookup.get(s.charId);
                                            return (
                                                <li key={s.skinId} className="flex items-center gap-2 border-2 py-1 pr-1 pl-1" style={{ ...cardVars(s.colors, 220), borderImage: `linear-gradient(90deg, ${stopsOf(s.colors.length > 0 ? s.colors : [FALLBACK_COLOR])}) 1`, background: "rgba(0,0,0,0.5)" }}>
                                                    <span className="size-7 shrink-0 overflow-hidden rounded-sm">
                                                        <OperatorAvatar charId={s.charId} name={op?.name ?? s.charId} server={op ? undefined : "cn"} />
                                                    </span>
                                                    <span className="max-w-40 truncate font-sans text-[12px] text-foreground">{resolveName(s.skinName, s.skinNameEn, s.skinNameAuto, autoOn).text}</span>
                                                    <Op value={s.price.price} className="text-[11px]" />
                                                    <button type="button" onClick={() => onRemove(s)} aria-label={t("release.planner.summary.remove")} className="flex size-6 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground">
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
