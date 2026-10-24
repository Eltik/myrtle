import { useQuery } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Check, Plus } from "lucide-react";
import * as React from "react";
import { formatLargeNumber } from "#/components/tools/shared/constants";
import { Button } from "#/components/ui/button";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { FilterChip } from "#/components/ui/filter-chip";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Spinner } from "#/components/ui/spinner";
import { toastManager } from "#/components/ui/toast";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { dpsOperatorsQueryOptions, dpsRankQueryOptions, type IDpsRankRow } from "#/lib/api/dps";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatProfession } from "#/lib/utils";
import type { DamageKind } from "#/types/generated/DamageKind";
import type { RankPreset } from "#/types/generated/RankPreset";
import type { messages as dpsConstantsMessages } from "../constants.messages";
import { hasRetention, RANK_CLASSES, rankMetric, rankRequest, rankRowKey, ratioToTop, rowToBuild, selectRankRows } from "../rank";
import type { DpsAction, IDpsState, RankSortKey } from "../types";
import type { ITeamBuffsContext } from "../useTeamBuffs";
import type { messages } from "./RankingView.messages";
import { formatSeconds, useCategoryLabel } from "./ResultDetail";
import type { messages as panelMessages } from "./ZonePanel.messages";

type RankT = TypedT<typeof messages & typeof dpsConstantsMessages & typeof panelMessages>;

const RARITIES = [6, 5, 4, 3, 2, 1] as const;
const DAMAGE_TYPES: readonly DamageKind[] = ["physical", "arts"];
const PRESETS: readonly RankPreset[] = ["p1Max", "p6Max", "default"];
/** The share link carries at most 32 cards (`decodeInstances`). */
const MAX_CARDS = 32;
const ROW_HEIGHT = 44;

interface IRankingViewProps {
    state: IDpsState;
    dispatch: React.Dispatch<DpsAction>;
    ctx: ITeamBuffsContext;
}

/**
 * Every (operator, skill, module) under the page's enemy and team buffs, from `POST
 * /dps/rank`. Filters, search and sort run on the returned rows, so changing them sends
 * nothing; the list is virtualised (1,343 rows at p1Max on EN data).
 */
export function RankingView({ state, dispatch, ctx }: IRankingViewProps): React.ReactElement {
    const t: RankT = useT("tools");
    const locale = useLocale();
    const { rank } = state;
    const [query, setQuery] = React.useState("");
    const body = React.useMemo(() => rankRequest(state.enemy, state.team, ctx.sources, rank.preset), [state.enemy, state.team, ctx.sources, rank.preset]);
    const { data, isPending, error, isFetching } = useQuery({ ...dpsRankQueryOptions(body), enabled: !ctx.waiting && !ctx.error });
    const allRows = data?.rows ?? [];
    const withRetention = hasRetention(allRows);
    const sortBy: RankSortKey = rank.sortBy === "retention" && !withRetention ? "skillDps" : rank.sortBy;
    const rows = React.useMemo(() => selectRankRows(allRows, { ...rank, sortBy }, query), [allRows, rank, sortBy, query]);
    const top = rows[0] ? rankMetric(rows[0], sortBy) : undefined;
    const withSplit = state.enemy.secondary.enabled;
    const withTimings = state.enemy.cycleModel === "v2";
    const scoped = ctx.sources.some((s) => s.professions.length > 0);

    const { data: operators } = useQuery(dpsOperatorsQueryOptions());
    const inComparison = React.useMemo(() => new Set(state.instances.map((i) => `${i.op.id}|${i.config.skillIndex}|${i.config.moduleIndex}`)), [state.instances]);
    const onAdd = React.useCallback(
        (row: IDpsRankRow) => {
            const op = operators?.find((o) => o.id === row.operatorId);
            if (!op) return;
            if (state.instances.length >= MAX_CARDS) {
                toastManager.add({ id: `dps-rank-full-${Date.now()}`, title: t("dps.rank.full", { count: MAX_CARDS }), type: "warning" });
                return;
            }
            dispatch({ type: "ADD_INSTANCE", op, config: rowToBuild(row, rank.preset) });
            toastManager.add({ id: `dps-rank-add-${Date.now()}`, title: t("dps.rank.added", { name: row.name }), type: "success" });
        },
        [operators, state.instances.length, dispatch, rank.preset, t],
    );

    const sortOptions: { value: RankSortKey; label: string }[] = [
        { value: "skillDps", label: t("dps.metric.skillDps") },
        { value: "averageDps", label: t("dps.metric.averageDps") },
        { value: "totalDamage", label: t("dps.metric.totalDamage") },
        ...(withRetention ? [{ value: "retention" as const, label: t("dps.rank.col.retention") }] : []),
    ];
    const presetLabel = (p: RankPreset) => (p === "p1Max" ? t("dps.rank.preset.p1Max") : p === "p6Max" ? t("dps.rank.preset.p6Max") : t("dps.rank.preset.default"));
    const toggle = <T,>(list: readonly T[], v: T): T[] => (list.includes(v) ? list.filter((x) => x !== v) : [...list, v]);
    const anyFilter = rank.professions.length + rank.rarities.length + rank.damageTypes.length > 0 || query.length > 0;

    const columns = ["2.5rem", "minmax(10rem,1.6fr)", "minmax(8rem,1.2fr)", "minmax(5.5rem,0.8fr)", "4.5rem", "5rem", "5rem", "5.5rem", ...(withRetention ? ["5rem"] : []), "4.5rem", "4.5rem", "4.5rem", ...(withSplit ? ["7.5rem"] : []), "4rem", "2.5rem"].join(" ");

    return (
        <Card>
            <CardHeader className="space-y-3">
                <div>
                    <CardTitle className="text-[15px]">{t("dps.rank.title")}</CardTitle>
                    <p className="text-[12px] text-muted-foreground">{t("dps.rank.desc")}</p>
                </div>
                <div className="flex flex-wrap items-end gap-3">
                    <div className="space-y-1">
                        <Tooltip>
                            <TooltipTrigger render={<Label className="block cursor-help font-medium text-[11px] text-muted-foreground leading-none" />}>{t("dps.rank.preset")}</TooltipTrigger>
                            <TooltipPopup className="max-w-64">{t("dps.rank.preset.hint")}</TooltipPopup>
                        </Tooltip>
                        <ToggleGroup aria-label={t("dps.rank.preset")} variant="outline" size="sm" value={[rank.preset]} onValueChange={(v) => v[0] && dispatch({ type: "SET_RANK", patch: { preset: v[0] as RankPreset } })}>
                            {PRESETS.map((p) => (
                                <ToggleGroupItem key={p} value={p} className="text-[12px]">
                                    {presetLabel(p)}
                                </ToggleGroupItem>
                            ))}
                        </ToggleGroup>
                    </div>
                    <div className="space-y-1">
                        <Label className="block font-medium text-[11px] text-muted-foreground leading-none">{t("dps.rank.sort")}</Label>
                        <Select value={sortBy} onValueChange={(v) => v !== null && dispatch({ type: "SET_RANK", patch: { sortBy: v as RankSortKey } })}>
                            <SelectTrigger size="sm" className="w-40" aria-label={t("dps.rank.sort")}>
                                <SelectValue>{() => sortOptions.find((o) => o.value === sortBy)?.label}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {sortOptions.map((o) => (
                                    <SelectItem key={o.value} value={o.value}>
                                        {o.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <Input type="search" size="sm" className="min-w-48 flex-1" value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("dps.rank.search")} aria-label={t("dps.rank.search")} />
                </div>
                <div className="flex flex-wrap items-center gap-x-3 gap-y-1.5">
                    <ChipGroup legend={t("dps.rank.class")}>
                        {RANK_CLASSES.map((c) => (
                            <FilterChip key={c} label={formatProfession(c)} active={rank.professions.includes(c)} onSelect={() => dispatch({ type: "SET_RANK", patch: { professions: toggle(rank.professions, c) } })} className="h-7 px-2.5 text-[12px]" />
                        ))}
                    </ChipGroup>
                    <ChipGroup legend={t("dps.rank.rarity")}>
                        {RARITIES.map((r) => (
                            <FilterChip key={r} label={t("dps.rank.rarityValue", { stars: r })} active={rank.rarities.includes(r)} onSelect={() => dispatch({ type: "SET_RANK", patch: { rarities: toggle(rank.rarities, r) } })} className="h-7 px-2.5 text-[12px]" />
                        ))}
                    </ChipGroup>
                    <ChipGroup legend={t("dps.rank.damageType")}>
                        {DAMAGE_TYPES.map((d) => (
                            <FilterChip key={d} label={d === "physical" ? t("dps.rank.physical") : t("dps.rank.arts")} active={rank.damageTypes.includes(d)} onSelect={() => dispatch({ type: "SET_RANK", patch: { damageTypes: toggle(rank.damageTypes, d) } })} className="h-7 px-2.5 text-[12px]" />
                        ))}
                    </ChipGroup>
                    {anyFilter && (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                                setQuery("");
                                dispatch({ type: "SET_RANK", patch: { professions: [], rarities: [], damageTypes: [] } });
                            }}
                        >
                            {t("dps.rank.clearFilters")}
                        </Button>
                    )}
                </div>
                <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-muted-foreground">
                    {data && <span className="tabular-nums">{t("dps.rank.count", { shown: rows.length.toLocaleString(locale), total: data.computed.toLocaleString(locale) })}</span>}
                    {data && data.skipped > 0 && <span>{t("dps.rank.skipped", { count: data.skipped })}</span>}
                    {!withTimings && <span>{t("dps.rank.timingsHint")}</span>}
                    {data?.retentionStatus === "pending" && <span>{t("dps.rank.retentionPending")}</span>}
                    {scoped && <span>{t("dps.supporters.rankNote")}</span>}
                    {isFetching && data && <Spinner className="size-3" />}
                </div>
            </CardHeader>
            <CardPanel className="pt-0">
                {ctx.error ? (
                    <p className="py-10 text-center text-[12.5px] text-destructive">{t("dps.supporters.blocked", { error: ctx.error.message })}</p>
                ) : ctx.waiting ? (
                    <Status>{t("dps.supporters.waiting")}</Status>
                ) : isPending ? (
                    <Status>
                        <Spinner className="size-3.5" />
                        {t("dps.rank.loading")}
                    </Status>
                ) : error ? (
                    <p className="py-10 text-center text-[12.5px] text-destructive">{t("dps.rank.failed", { error: error.message })}</p>
                ) : rows.length === 0 ? (
                    <Status>{t("dps.rank.empty")}</Status>
                ) : (
                    <RankTable rows={rows} columns={columns} sortBy={sortBy} top={top} withRetention={withRetention} withSplit={withSplit} inComparison={inComparison} onAdd={onAdd} />
                )}
            </CardPanel>
        </Card>
    );
}

/** A labelled run of filter chips; the legend is for assistive tech, the chips speak for themselves visually. */
function ChipGroup({ legend, children }: { legend: string; children: React.ReactNode }): React.ReactElement {
    return (
        <fieldset className="flex flex-wrap items-center gap-1.5">
            <legend className="sr-only">{legend}</legend>
            {children}
        </fieldset>
    );
}

function Status({ children }: { children: React.ReactNode }): React.ReactElement {
    return <div className="flex items-center justify-center gap-2 py-10 text-[12.5px] text-muted-foreground">{children}</div>;
}

interface IRankTableProps {
    rows: IDpsRankRow[];
    columns: string;
    sortBy: RankSortKey;
    top: number | undefined;
    withRetention: boolean;
    withSplit: boolean;
    inComparison: ReadonlySet<string>;
    onAdd: (row: IDpsRankRow) => void;
}

function RankTable({ rows, columns, sortBy, top, withRetention, withSplit, inComparison, onAdd }: IRankTableProps): React.ReactElement {
    const t: RankT = useT("tools");
    const scrollRef = React.useRef<HTMLDivElement>(null);
    const virtualizer = useVirtualizer({ count: rows.length, getScrollElement: () => scrollRef.current, estimateSize: () => ROW_HEIGHT, overscan: 12 });
    const heads: { id: string; label: string; hint?: string; numeric?: boolean; active?: boolean }[] = [
        { id: "rank", label: t("dps.rank.col.rank"), numeric: true },
        { id: "operator", label: t("dps.rank.col.operator") },
        { id: "skill", label: t("dps.rank.col.skill") },
        { id: "module", label: t("dps.rank.col.module") },
        { id: "category", label: t("dps.rank.col.category") },
        { id: "skillDps", label: t("dps.metric.skillDps.short"), numeric: true, active: sortBy === "skillDps" },
        { id: "averageDps", label: t("dps.metric.averageDps.short"), numeric: true, active: sortBy === "averageDps" },
        { id: "totalDamage", label: t("dps.metric.totalDamage.short"), numeric: true, active: sortBy === "totalDamage" },
        ...(withRetention ? [{ id: "retention", label: t("dps.rank.col.retention"), hint: t("dps.rank.col.retention.hint"), numeric: true, active: sortBy === "retention" }] : []),
        { id: "startup", label: t("dps.rank.col.startup"), numeric: true },
        { id: "duration", label: t("dps.rank.col.duration"), numeric: true },
        { id: "cycle", label: t("dps.rank.col.cycle"), numeric: true },
        ...(withSplit ? [{ id: "split", label: t("dps.rank.col.split"), numeric: true }] : []),
        { id: "ratio", label: t("dps.rank.col.ratio"), numeric: true },
        { id: "add", label: "" },
    ];
    return (
        <div ref={scrollRef} className="max-h-[70vh] overflow-auto rounded-lg border border-border">
            <div className="min-w-[68rem]">
                <div className="sticky top-0 z-1 grid items-center gap-x-2 border-border border-b bg-card px-2 py-2 font-medium text-[10.5px] text-muted-foreground" style={{ gridTemplateColumns: columns }}>
                    {heads.map((h) =>
                        h.hint ? (
                            <Tooltip key={h.id}>
                                <TooltipTrigger render={<span className={cn("cursor-help truncate underline decoration-dotted underline-offset-2", h.numeric && "text-right", h.active && "text-foreground")} />}>{h.label}</TooltipTrigger>
                                <TooltipPopup className="max-w-72">{h.hint}</TooltipPopup>
                            </Tooltip>
                        ) : (
                            <span key={h.id} className={cn("truncate", h.numeric && "text-right", h.active && "text-foreground")}>
                                {h.label}
                            </span>
                        ),
                    )}
                </div>
                <div className="relative" style={{ height: virtualizer.getTotalSize() }}>
                    {virtualizer.getVirtualItems().map((v) => {
                        const row = rows[v.index];
                        return (
                            <div key={rankRowKey(row)} className="absolute inset-x-0" style={{ top: v.start, height: ROW_HEIGHT }}>
                                <RankRowView row={row} index={v.index} columns={columns} sortBy={sortBy} top={top} withRetention={withRetention} withSplit={withSplit} added={inComparison.has(rankRowKey(row))} onAdd={onAdd} />
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
}

interface IRankRowViewProps {
    row: IDpsRankRow;
    index: number;
    columns: string;
    sortBy: RankSortKey;
    top: number | undefined;
    withRetention: boolean;
    withSplit: boolean;
    added: boolean;
    onAdd: (row: IDpsRankRow) => void;
}

export function RankRowView({ row, index, columns, sortBy, top, withRetention, withSplit, added, onAdd }: IRankRowViewProps): React.ReactElement {
    const t: RankT = useT("tools");
    const locale = useLocale();
    const categoryLabel = useCategoryLabel();
    const ratio = ratioToTop(rankMetric(row, sortBy), top);
    const splitKey = sortBy === "averageDps" ? "average_dps" : sortBy === "totalDamage" ? "total_damage" : "skill_dps";
    const num = (v: number | undefined) => <span className="text-right font-mono tabular-nums">{v === undefined ? "-" : formatLargeNumber(v, locale)}</span>;
    const secs = (v: number | undefined) => <span className="text-right font-mono text-muted-foreground tabular-nums">{v === undefined ? "-" : formatSeconds(v)}</span>;
    return (
        <div className={cn("grid h-full items-center gap-x-2 border-border/60 border-b px-2 text-[12px]", added && "bg-primary/6")} style={{ gridTemplateColumns: columns }}>
            <span className="text-right font-mono text-[11px] text-muted-foreground tabular-nums">{index + 1}</span>
            <span className="flex min-w-0 items-center gap-2">
                <span aria-hidden="true" className="inline-flex size-7 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted text-[9px]">
                    <OperatorAvatar charId={row.operatorId} name={row.name} />
                </span>
                <span className="min-w-0">
                    <span className="block truncate font-medium">{row.name}</span>
                    <span className="block truncate text-[10.5px] text-muted-foreground">
                        {row.rarity}★ {formatProfession(row.profession)}
                    </span>
                </span>
            </span>
            <span className="min-w-0 truncate" title={row.skillName}>
                {row.skillIndex === 0 ? t("dps.rank.basicAttack") : `S${row.skillIndex} ${row.skillName ?? ""}`}
            </span>
            <span className="min-w-0 truncate text-muted-foreground" title={row.moduleName}>
                {row.moduleIndex === 0 ? t("dps.rank.noModule") : (row.moduleName ?? row.moduleId ?? `#${row.moduleIndex}`)}
            </span>
            <span className="truncate text-[11px] text-muted-foreground">{row.category ? categoryLabel(row.category) : "-"}</span>
            {num(row.skillDps)}
            {num(row.averageDps)}
            {num(row.totalDamage)}
            {withRetention && <span className="text-right font-mono tabular-nums">{typeof row.retention === "number" ? `${(row.retention * 100).toFixed(1)}%` : "-"}</span>}
            {secs(row.startupTime)}
            {secs(row.duration)}
            {secs(row.cycleTime)}
            {withSplit && <span className="truncate text-right font-mono text-[11px] tabular-nums">{row.primary && row.secondary ? `${formatLargeNumber(row.primary[splitKey], locale)} / ${formatLargeNumber(row.secondary[splitKey], locale)}` : "-"}</span>}
            <span className="text-right font-mono text-[11px] text-muted-foreground tabular-nums">{ratio === null ? "-" : `${(ratio * 100).toFixed(1)}%`}</span>
            <span className="flex justify-end">
                {added ? (
                    <Tooltip>
                        <TooltipTrigger render={<span className="inline-flex size-6 items-center justify-center text-primary" />}>
                            <Check className="size-3.5" aria-hidden="true" />
                            <span className="sr-only">{t("dps.rank.inComparison")}</span>
                        </TooltipTrigger>
                        <TooltipPopup>{t("dps.rank.inComparison")}</TooltipPopup>
                    </Tooltip>
                ) : (
                    <Tooltip>
                        <TooltipTrigger
                            render={(p) => (
                                <Button {...p} aria-label={t("dps.rank.add", { name: row.name })} variant="ghost" size="icon-xs" onClick={() => onAdd(row)}>
                                    <Plus />
                                </Button>
                            )}
                        />
                        <TooltipPopup>{t("dps.rank.add", { name: row.name })}</TooltipPopup>
                    </Tooltip>
                )}
            </span>
        </div>
    );
}
