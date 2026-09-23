import { useQuery } from "@tanstack/react-query";
import { ArrowDownWideNarrow, ArrowUpNarrowWide, Check, ChevronDown, X } from "lucide-react";
import { useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, formatArchetype, formatProfession } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { METRIC_SORT_LABEL_KEYS, METRIC_SORTS, type MetricSort, parseScope, type Scope, scopeToken } from "../searchControls";
import type { messages as sortMessages } from "../searchControls.messages";
import type { ISearchControls } from "../useSearchControls";
import { OperatorMultiCombobox, OperatorSingleCombobox } from "./OperatorCombobox";
import { ScopePicker } from "./ScopePicker";
import type { messages } from "./SearchToolbar.messages";

/** The sort names in `searchControls.messages.ts` are rendered here too. */
type ToolbarT = TypedT<typeof messages & typeof sortMessages>;

/** Filter trigger: a 36px tap target on touch widths, the site's 32px on desktop. */
const FILTER = "inline-flex h-9 cursor-pointer items-center gap-2 rounded-lg border border-input bg-card px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:border-foreground/20 sm:h-8";
const KICKER = "shrink-0 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]";
const GROUP_HEADING = "font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.16em]";

/** The game's name for a scope: the class, or the archetype. */
function scopeLabel(scope: Scope): string {
    return scope.kind === "class" ? formatProfession(scope.profession) : formatArchetype(scope.subProfessionId);
}

/** Name of a sort token for the trigger and the status line. */
export function useSortLabel(sort: string): string {
    const t: ToolbarT = useT("user");
    const scope = parseScope(sort);
    if (scope) return t("search.sort.scoped", { label: scopeLabel(scope) });
    return t(METRIC_SORT_LABEL_KEYS[(sort in METRIC_SORT_LABEL_KEYS ? sort : "score") as MetricSort]);
}

/** Obtainable operators, rarity first then by name: the order both pickers list. */
function useObtainableOperators(): IOperatorIndexEntry[] | undefined {
    const { data } = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    return useMemo(() => {
        if (!data) return undefined;
        return data.filter((op) => !op.isNotObtainable).sort((a, b) => (b.rarity !== a.rarity ? b.rarity - a.rarity : a.name.localeCompare(b.name)));
    }, [data]);
}

export function SearchToolbar({ controls }: { controls: ISearchControls }) {
    const t: TypedT<typeof messages> = useT("user");
    const operators = useObtainableOperators();

    return (
        <div className="flex flex-col gap-2.5">
            <div className="flex flex-wrap items-center gap-2">
                <SortPicker sort={controls.sort} onSort={controls.setSort} operators={operators} />
                <DirToggle sort={controls.sort} dir={controls.dir} onToggle={controls.toggleDir} />
            </div>

            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-sans text-[12.5px] text-muted-foreground leading-none">
                <span className={KICKER}>{t("search.toolbar.filters")}</span>
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
                    <label htmlFor="search-has" className={KICKER}>
                        {t("search.toolbar.has.label")}
                    </label>
                    <OperatorMultiCombobox id="search-has" operators={operators} value={controls.has} onChange={controls.setHas} label={t("search.toolbar.has.label")} placeholder={t("search.toolbar.has.placeholder")} />
                </div>
                <div className="flex flex-col gap-1">
                    <label htmlFor="search-support" className={KICKER}>
                        {t("search.toolbar.support.label")}
                    </label>
                    <OperatorSingleCombobox id="search-support" operators={operators} value={controls.support} onChange={controls.setSupport} label={t("search.toolbar.support.label")} placeholder={t("search.toolbar.support.placeholder")} />
                </div>
                <div className="flex flex-col gap-1">
                    {/* Height matches the two picker labels so the three controls sit on one baseline. */}
                    <span className={KICKER} aria-hidden="true">
                        &nbsp;
                    </span>
                    <AllPicker scope={controls.all} onChange={controls.setAll} operators={operators} />
                </div>
            </div>
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
            <PopoverTrigger aria-label={t("search.toolbar.rankBy.aria")} className={cn(FILTER, "min-w-0 max-w-full")}>
                <span className={KICKER}>{t("search.toolbar.rankBy")}</span>
                <span className="min-w-0 truncate font-semibold text-foreground">{label}</span>
                <ChevronDown className="size-3 shrink-0 opacity-70" aria-hidden />
            </PopoverTrigger>
            <PopoverPopup align="start" className="w-[min(420px,calc(100vw-2rem))] p-0">
                <section aria-label={t("search.toolbar.rankBy.group.metric")} className="flex flex-col gap-2 p-3">
                    <span className={GROUP_HEADING}>{t("search.toolbar.rankBy.group.metric")}</span>
                    <ul className="m-0 flex list-none flex-wrap gap-1.5 p-0">
                        {METRIC_SORTS.map((key) => {
                            const active = sort === key;
                            return (
                                <li key={key}>
                                    <button
                                        type="button"
                                        aria-pressed={active}
                                        onClick={() => chooseMetric(key)}
                                        className={cn(
                                            "inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border px-2.5 font-medium font-sans text-xs leading-none transition-colors",
                                            active ? "border-primary/40 bg-[color-mix(in_srgb,var(--primary)_10%,transparent)] text-primary" : "border-input bg-card text-foreground hover:border-foreground/20",
                                        )}
                                    >
                                        {active ? <Check className="size-3" aria-hidden /> : null}
                                        {t(METRIC_SORT_LABEL_KEYS[key])}
                                    </button>
                                </li>
                            );
                        })}
                    </ul>
                </section>
                <section aria-label={t("search.toolbar.rankBy.group.scope")} className="flex flex-col gap-2 border-border/70 border-t p-3">
                    <span className={GROUP_HEADING}>{t("search.toolbar.rankBy.group.scope")}</span>
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
        <button type="button" onClick={onToggle} aria-label={t("search.toolbar.dir.aria")} title={label} className={FILTER}>
            <Icon className="size-3.5 opacity-80" aria-hidden />
            <span>{label}</span>
        </button>
    );
}

function AllPicker({ scope, onChange, operators }: { scope: Scope | null; onChange: (next: Scope | null) => void; operators: IOperatorIndexEntry[] | undefined }) {
    const t: TypedT<typeof messages> = useT("user");
    const [open, setOpen] = useState(false);

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger aria-label={t("search.toolbar.all.aria")} className={cn(FILTER, "w-full min-w-0 sm:w-auto", scope && "border-primary/40 bg-[color-mix(in_srgb,var(--primary)_10%,transparent)]")}>
                <span className={KICKER}>{t("search.toolbar.all.label")}</span>
                <span className={cn("min-w-0 truncate font-semibold", scope ? "text-primary" : "text-foreground")}>{scope ? scopeLabel(scope) : t("search.toolbar.all.none")}</span>
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
