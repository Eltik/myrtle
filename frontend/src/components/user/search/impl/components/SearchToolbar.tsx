import { useQuery } from "@tanstack/react-query";
import { ArrowDownWideNarrow, ArrowUpNarrowWide, ChevronDown, X } from "lucide-react";
import { useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { FilterChip } from "#/components/ui/filter-chip";
import { LoadFailed } from "#/components/ui/load-failed";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatArchetype, professionLabel, serverNames } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { METRIC_SORT_LABEL_KEYS, METRIC_SORTS, type MetricSort, parseScope, type Scope, scopeToken } from "../searchControls";
import type { messages as sortMessages } from "../searchControls.messages";
import type { ISearchControls } from "../useSearchControls";
import { OperatorMultiCombobox, OperatorSingleCombobox } from "./OperatorCombobox";
import { ScopePicker } from "./ScopePicker";
import type { messages } from "./SearchToolbar.messages";

/** The sort names in `searchControls.messages.ts` are rendered here too. */
type ToolbarT = TypedT<typeof messages & typeof sortMessages>;

/** The chip box sized to the single picker and the scope trigger: 36px narrow, 32px from sm (chips 24px there). */
const CHIPS_SIZE = "sm:min-h-8 sm:*:min-h-6";

/** Filter trigger: the kit input's surface and type, 36px on narrow screens, the site's 32px on desktop. */
const FILTER = "inline-flex h-9 cursor-pointer items-center gap-2 rounded-lg border border-input bg-background px-2.5 font-sans text-base text-foreground leading-none transition-colors hover:border-foreground/20 sm:h-8 sm:text-sm dark:bg-input/32";
/** 44px under a coarse pointer for the toolbar row's own buttons. */
const COARSE = "pointer-coarse:h-11";
/** A label over a control or a group: the one kicker voice. */
const LABEL = "kicker leading-none";
/** A label inside a trigger, before its value: plain muted sans. */
const INLINE_LABEL = "shrink-0 text-muted-foreground";

/**
 * The game's name for a scope, the class or the archetype, in the server's own wording read off the operator index
 * (cached, so every caller shares one fetch); the English formatters answer while it loads.
 */
export function useScopeLabel(): (scope: Scope) => string {
    const { data } = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    return useMemo(() => {
        const classes = serverNames(data ?? [], "profession");
        const archetypes = serverNames(data ?? [], "subProfession");
        return (scope: Scope) => (scope.kind === "class" ? professionLabel({ profession: scope.profession, professionName: classes.get(scope.profession) }) : (archetypes.get(scope.subProfessionId) ?? formatArchetype(scope.subProfessionId)));
    }, [data]);
}

/** Name of a sort token for the trigger and the status line. */
export function useSortLabel(sort: string): string {
    const t: ToolbarT = useT("user");
    const scopeLabel = useScopeLabel();
    const scope = parseScope(sort);
    if (scope) return t("search.sort.scoped", { label: scopeLabel(scope) });
    return t(METRIC_SORT_LABEL_KEYS[(sort in METRIC_SORT_LABEL_KEYS ? sort : "score") as MetricSort]);
}

/** Obtainable operators, rarity first then by name: the order both pickers list. `failed` = the index request failed with nothing cached. */
function useObtainableOperators(): { operators: IOperatorIndexEntry[] | undefined; failed: boolean; retry: () => void } {
    const { data, isError, refetch } = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    const operators = useMemo(() => {
        if (!data) return undefined;
        return data.filter((op) => !op.isNotObtainable).sort((a, b) => (b.rarity !== a.rarity ? b.rarity - a.rarity : a.name.localeCompare(b.name)));
    }, [data]);
    return { operators, failed: isError && !data, retry: () => void refetch() };
}

export function SearchToolbar({ controls }: { controls: ISearchControls }) {
    const t: TypedT<typeof messages> = useT("user");
    const { operators, failed: operatorsFailed, retry: retryOperators } = useObtainableOperators();

    return (
        <div className="flex flex-col gap-2.5">
            <div className="flex flex-wrap items-center gap-2">
                <SortPicker sort={controls.sort} onSort={controls.setSort} operators={operators} />
                <DirToggle sort={controls.sort} dir={controls.dir} onToggle={controls.toggleDir} />
            </div>

            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 pb-1.5 font-sans text-[12.5px] text-muted-foreground leading-none">
                <span className="font-medium text-foreground">{t("search.toolbar.filters")}</span>
                <span>{t("search.toolbar.filters.count", { count: controls.activeFilters })}</span>
                {controls.activeFilters > 0 ? (
                    <Button variant="ghost" size="xs" onClick={controls.clearFilters} className="h-6 gap-1 px-1.5 text-[12px]">
                        <X className="size-3" aria-hidden="true" />
                        {t("search.toolbar.clearFilters")}
                    </Button>
                ) : null}
            </div>

            <div className="grid gap-2 sm:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)_auto] sm:items-start">
                <div className="flex flex-col gap-1">
                    <label htmlFor="search-has" className={LABEL}>
                        {t("search.toolbar.has.label")}
                    </label>
                    <OperatorMultiCombobox className={CHIPS_SIZE} id="search-has" operators={operators} value={controls.has} onChange={controls.setHas} label={t("search.toolbar.has.label")} placeholder={t("search.toolbar.has.placeholder")} />
                </div>
                <div className="flex flex-col gap-1">
                    <label htmlFor="search-support" className={LABEL}>
                        {t("search.toolbar.support.label")}
                    </label>
                    <OperatorSingleCombobox id="search-support" operators={operators} value={controls.support} onChange={controls.setSupport} label={t("search.toolbar.support.label")} placeholder={t("search.toolbar.support.placeholder")} />
                </div>
                <div className="flex flex-col gap-1">
                    {/* Height matches the two picker labels so the three controls sit on one baseline. */}
                    <span className={LABEL} aria-hidden="true">
                        &nbsp;
                    </span>
                    <AllPicker scope={controls.all} onChange={controls.setAll} operators={operators} />
                </div>
            </div>
            {/* Without this the pickers sat on their loading state for good. */}
            {operatorsFailed && <LoadFailed layout="inline" message={t("search.toolbar.operatorsFailed")} onRetry={retryOperators} />}
        </div>
    );
}

function SortPicker({ sort, onSort, operators }: { sort: string; onSort: (next: string) => void; operators: IOperatorIndexEntry[] | undefined }) {
    const t: ToolbarT = useT("user");
    const [open, setOpen] = useState(false);
    const label = useSortLabel(sort);
    const scope = parseScope(sort);

    const chooseMetric = (next: MetricSort) => {
        onSort(next);
        setOpen(false);
    };
    // A class press keeps the popup open so the archetype row it reveals can
    // be used; an archetype press is the end of that path and closes it.
    const chooseScope = (next: Scope | null) => {
        onSort(next ? scopeToken(next) : "score");
        if (!next || next.kind === "sub") setOpen(false);
    };

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger aria-label={t("search.toolbar.rankBy.aria")} className={cn(FILTER, COARSE, "min-w-0 max-w-full")}>
                <span className={INLINE_LABEL}>{t("search.toolbar.rankBy")}</span>
                <span className="min-w-0 truncate font-semibold text-foreground">{label}</span>
                <ChevronDown className="size-3 shrink-0 opacity-70" aria-hidden />
            </PopoverTrigger>
            <PopoverPopup align="start" className="w-[min(420px,calc(100vw-2rem))] p-0">
                <section aria-label={t("search.toolbar.rankBy.group.metric")} className="flex flex-col gap-2 p-3">
                    <span className={LABEL}>{t("search.toolbar.rankBy.group.metric")}</span>
                    <ul className="m-0 flex list-none flex-wrap gap-1.5 p-0">
                        {METRIC_SORTS.map((key) => {
                            const active = sort === key;
                            return (
                                <li key={key}>
                                    <FilterChip label={t(METRIC_SORT_LABEL_KEYS[key])} active={active} onSelect={() => chooseMetric(key)} />
                                </li>
                            );
                        })}
                    </ul>
                </section>
                <section aria-label={t("search.toolbar.rankBy.group.scope")} className="flex flex-col gap-2 border-border/70 border-t p-3">
                    <span className={LABEL}>{t("search.toolbar.rankBy.group.scope")}</span>
                    <ScopePicker scope={scope} onChange={chooseScope} operators={operators} />
                </section>
            </PopoverPopup>
        </Popover>
    );
}

function DirToggle({ sort, dir, onToggle }: { sort: string; dir: "asc" | "desc"; onToggle: () => void }) {
    const t: TypedT<typeof messages> = useT("user");
    const label = sort === "joined" ? (dir === "asc" ? t("search.toolbar.dir.joined.asc") : t("search.toolbar.dir.joined.desc")) : dir === "asc" ? t("search.toolbar.dir.asc") : t("search.toolbar.dir.desc");
    const Icon = dir === "asc" ? ArrowUpNarrowWide : ArrowDownWideNarrow;
    return (
        <button type="button" onClick={onToggle} aria-label={t("search.toolbar.dir.aria")} title={label} className={cn(FILTER, COARSE)}>
            <Icon className="size-3.5 opacity-80" aria-hidden />
            <span>{label}</span>
        </button>
    );
}

function AllPicker({ scope, onChange, operators }: { scope: Scope | null; onChange: (next: Scope | null) => void; operators: IOperatorIndexEntry[] | undefined }) {
    const t: TypedT<typeof messages> = useT("user");
    const scopeLabel = useScopeLabel();
    const [open, setOpen] = useState(false);

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger
                aria-label={t("search.toolbar.all.aria")}
                className={cn(
                    FILTER,
                    "relative w-full min-w-0 pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2 sm:w-auto",
                    scope && "inset-ring inset-ring-ring bg-primary/12 hover:bg-primary/16 dark:bg-primary/16 dark:hover:bg-primary/22",
                )}
            >
                <span className={INLINE_LABEL}>{t("search.toolbar.all.label")}</span>
                <span className="min-w-0 truncate font-semibold text-foreground">{scope ? scopeLabel(scope) : t("search.toolbar.all.none")}</span>
                <ChevronDown className="size-3 shrink-0 opacity-70" aria-hidden />
            </PopoverTrigger>
            <PopoverPopup align="end" className="flex w-[min(420px,calc(100vw-2rem))] flex-col gap-3 p-3">
                <p className="m-0 font-sans text-[11.5px] text-muted-foreground leading-snug">{t("search.toolbar.all.hint")}</p>
                <ScopePicker scope={scope} onChange={onChange} operators={operators} />
                {scope ? (
                    <div className="flex justify-end">
                        <Button
                            variant="outline"
                            size="xs"
                            onClick={() => {
                                onChange(null);
                                setOpen(false);
                            }}
                        >
                            {t("search.toolbar.all.clear")}
                        </Button>
                    </div>
                ) : null}
            </PopoverPopup>
        </Popover>
    );
}
