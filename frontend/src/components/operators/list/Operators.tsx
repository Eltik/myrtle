import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ArrowDown, ArrowUp, Download, LayoutGrid, LayoutList, Rows3 } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { DynamicExportDialog } from "#/components/export/ExportDialog.lazy";
import { useOperatorFactLabel } from "#/components/operators/OperatorFacts";
import { SearchField } from "#/components/SearchField";
import { Button } from "#/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader } from "#/components/ui/empty";
import { LoadFailed } from "#/components/ui/load-failed";
import { PageHeader } from "#/components/ui/page-header";
import { useCnUpdates } from "#/hooks/use-cn-updates";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useMediaQuery } from "#/hooks/use-media-query";
import { useOperatorName } from "#/hooks/use-operator-name";
import { noteHasContent, operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { cnChangesQueryOptions, type IOperatorExportFields, operatorOwnershipQueryOptions, operatorsExportFieldsQueryOptions, operatorsIndexQueryOptions } from "#/lib/api/operators";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { operatorsExportSchema } from "#/lib/export";
import { type TypedRichT, useGamedataServer, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cnChangesById } from "#/lib/operators/cn-changes";
import { Route } from "#/routes/operators";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../ui/select";
import { Skeleton } from "../../ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "../../ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "../../ui/tooltip";
import { ActiveFilterChips } from "./impl/components/ActiveFilterChips";
import { FilterToggleButton } from "./impl/components/FilterToggleButton";
import { OperatorCardCompact } from "./impl/components/OperatorCardCompact";
import { OperatorCardGrid } from "./impl/components/OperatorCardGrid";
import { OperatorCardList } from "./impl/components/OperatorCardList";
import { OperatorCardUpcoming } from "./impl/components/OperatorCardUpcoming";
import { OperatorFilters } from "./impl/components/OperatorFilters";
import { Pagination, PaginationCompact } from "./impl/components/Pagination";
import { FILTERS_VISIBLE_KEY, HAS_NOTES_LABEL_KEY, ITEMS_PER_PAGE, ITEMS_PER_PAGE_KEY, ITEMS_PER_PAGE_OPTIONS, type ItemsPerPage, LIST_GRID_COLS, MIN_RARITY_FOR_E2, PAGE_KEY, SORT_OPTIONS, STAT_METRIC_KEY, STAT_METRICS, VIEW_MODE_KEY, VIEW_MODES } from "./impl/constants";
import type { messages as listConstantsMessages } from "./impl/constants.messages";
import { enrichOperators } from "./impl/enrich";
import { operatorFilterPredicate } from "./impl/matchFilters";
import { buildSharedChips } from "./impl/shared-filters";
import type { IOperatorExportRow, IOperatorOwnershipInfo, IOperatorView, SortOption, SortOrder, StatMetric, ViewMode } from "./impl/types";
import { useOperatorFilters } from "./impl/useOperatorFilters";
import type { messages } from "./Operators.messages";

const VIEW_MODE_ICONS = [
    { mode: "grid", Icon: LayoutGrid },
    { mode: "compact", Icon: Rows3 },
    { mode: "list", Icon: LayoutList },
] as const satisfies readonly { mode: ViewMode; Icon: typeof LayoutGrid }[];

const UPCOMING_SKELETON_KEYS = Array.from({ length: 18 }, (_, i) => `upcoming-skeleton-${i}`);

// Shared empty array so the export-row memos keep a stable reference while the
// export dialog is closed, avoiding needless ExportDialog re-renders.
const EMPTY_EXPORT_ROWS: IOperatorExportRow[] = [];

/** A card in the Upcoming view: a CN-only operator, or a Global one CN has `updated`. */
interface IUpcomingEntry {
    operator: IOperatorView;
    updated: boolean;
}

export function OperatorsList() {
    const t: TypedT<typeof messages & typeof listConstantsMessages> = useT("operators");
    const fact = useOperatorFactLabel();
    const rt: TypedRichT<typeof messages> = useRichT("operators");
    const server = useGamedataServer();
    const { data: operators = [] } = useQuery(operatorsIndexQueryOptions(server));
    const { data: notes } = useQuery(operatorNotesListQueryOptions());
    const { data: ownership } = useQuery(operatorOwnershipQueryOptions(server));
    const notedIds = useMemo(() => {
        if (!notes) return undefined;
        return new Set(notes.filter(noteHasContent).map((n) => n.operator_id));
    }, [notes]);
    const ownershipMap = useMemo(() => {
        if (!ownership || ownership.totalUsers <= 0) return undefined;
        const total = ownership.totalUsers;
        const rarityById = new Map(operators.map((op) => [op.id, op.rarity]));
        const map = new Map<string, IOperatorOwnershipInfo>();
        for (const [id, counts] of Object.entries(ownership.counts)) {
            if (!counts) continue;
            const { owners, e2Owners } = counts;
            // An operator that cannot reach E2 has no conversion rate to report,
            // and neither does one nobody owns. Both stay `null` rather than
            // collapsing to 0, which would read as "nobody bothered".
            const canE2 = (rarityById.get(id) ?? 0) >= MIN_RARITY_FOR_E2;
            map.set(id, {
                owners,
                pct: owners / total,
                e2Owners,
                e2Pct: canE2 && owners > 0 ? e2Owners / owners : null,
                canE2,
            });
        }
        return map;
    }, [ownership, operators]);
    // The "Updated" pill and the updated operators under Upcoming follow the CN updates setting;
    // off, the change list is neither fetched nor read from an earlier fetch's cache.
    const showCnUpdates = useCnUpdates(server);
    const { data: cnChanges } = useQuery({ ...cnChangesQueryOptions(server), enabled: showCnUpdates });
    const cnChangesMap = useMemo(() => cnChangesById(showCnUpdates ? cnChanges : undefined), [showCnUpdates, cnChanges]);
    const enriched = useMemo(() => enrichOperators(operators, notedIds, ownershipMap, cnChangesMap), [operators, notedIds, ownershipMap, cnChangesMap]);

    const {
        filters,
        filterOptions,
        filteredOperators,
        setSearchQuery,
        setClasses,
        setSubclasses,
        setRarities,
        setGenders,
        setNations,
        setFactions,
        setRaces,
        setBirthPlaces,
        setArtists,
        setVoiceActors,
        setModules,
        setHasNotes,
        setAvailability,
        setSortBy,
        setSortOrder,
        removeFrom,
        clearFilters,
        hasActiveFilters,
        activeFilterCount,
    } = useOperatorFilters(enriched);

    const { data: upcoming = [], isLoading: upcomingLoading, isError: upcomingFailed, refetch: refetchUpcoming } = useQuery(upcomingQueryOptions(server));
    const operatorName = useOperatorName();
    const isUpcoming = filters.availability === "upcoming";

    // The CN-only list runs the same predicate as the Global one. It used to apply only search, class,
    // subclass, rarity and nation, so a race, faction, gender, birthplace, artist, CV or notes filter
    // showed every upcoming operator under a "Showing N of N" that read as a filtered result.
    const enrichedUpcoming = useMemo(() => enrichOperators(upcoming, notedIds, undefined), [upcoming, notedIds]);
    const upcomingOnly = useMemo(() => enrichedUpcoming.filter(operatorFilterPredicate(filters)).sort((a, b) => b.rarity - a.rarity || operatorName(a).localeCompare(operatorName(b))), [enrichedUpcoming, filters, operatorName]);
    // Upcoming also lists the Global operators CN has updated, after the CN-only ones and in the
    // Global list's own order, so both answer "what is coming from CN".
    const upcomingFiltered = useMemo<IUpcomingEntry[]>(() => [...upcomingOnly.map((operator) => ({ operator, updated: false })), ...filteredOperators.filter((op) => op.cnChanges !== null).map((operator) => ({ operator, updated: true }))], [upcomingOnly, filteredOperators]);

    const [viewMode, setViewMode] = useLocalStorageState<ViewMode>(VIEW_MODE_KEY, "grid", {
        parse: (raw) => (VIEW_MODES.has(raw as ViewMode) ? (raw as ViewMode) : undefined),
        serialize: (v) => v,
    });
    const [filtersVisible, setFiltersVisible] = useLocalStorageState<boolean>(FILTERS_VISIBLE_KEY, false, {
        parse: (raw) => raw === "1",
        serialize: (v) => (v ? "1" : "0"),
    });
    // The stored flag is the desktop sidebar's. Below md the panel is a full-height sheet, and a
    // "1" saved on desktop opened it over the list on every phone load, so phones start closed.
    const isMobile = useMediaQuery("max-md");
    const [mobileFiltersOpen, setMobileFiltersOpen] = useState(false);
    const filtersOpen = isMobile ? mobileFiltersOpen : filtersVisible;
    const toggleFilters = () => (isMobile ? setMobileFiltersOpen((v) => !v) : setFiltersVisible((v) => !v));

    const [statMetric, setStatMetric] = useLocalStorageState<StatMetric>(STAT_METRIC_KEY, "owned", {
        parse: (raw) => (STAT_METRICS.has(raw as StatMetric) ? (raw as StatMetric) : undefined),
        serialize: (v) => v,
    });

    const [itemsPerPage, setItemsPerPage] = useLocalStorageState<ItemsPerPage>(ITEMS_PER_PAGE_KEY, ITEMS_PER_PAGE, {
        parse: (raw) => {
            if (raw === "all") return "all";
            const num = Number(raw);
            return (ITEMS_PER_PAGE_OPTIONS as readonly ItemsPerPage[]).includes(num as ItemsPerPage) ? (num as ItemsPerPage) : undefined;
        },
        serialize: (v) => String(v),
    });

    // The page number round-trips through the URL, so browser back from an
    // operator lands on the page you left rather than restarting at one.
    const { page: pageFromUrl } = Route.useSearch();
    const navigate = useNavigate({ from: "/operators" });
    const [currentPage, setCurrentPage] = useState(pageFromUrl ?? 1);

    const goToPage = useCallback(
        (next: number, scrollToTop: boolean) => {
            setCurrentPage(next);
            navigate({ search: (prev) => ({ ...prev, page: next }), replace: true, resetScroll: false });
            if (typeof window === "undefined") return;
            window.localStorage.setItem(PAGE_KEY, String(next));
            if (scrollToTop) {
                window.scrollTo({ top: 0 });
            }
        },
        [navigate],
    );

    // No page in the URL (the breadcrumb out of an operator, the nav link)
    // resumes the last page, like the filters and view mode do. A page in the
    // URL always wins, which keeps browser back landing where it left.
    const pageRestoredRef = useRef(false);
    useEffect(() => {
        if (pageRestoredRef.current) return;
        pageRestoredRef.current = true;
        if (pageFromUrl != null || typeof window === "undefined") return;
        const stored = Number(window.localStorage.getItem(PAGE_KEY));
        if (Number.isFinite(stored) && stored > 1) goToPage(stored, false);
    }, [pageFromUrl, goToPage]);

    // Paging is a deliberate jump, so it puts the top of the new page in view;
    // a filter change only rewinds the counter and leaves the scroll alone.
    const handlePageChange = useCallback((next: number) => goToPage(next, true), [goToPage]);
    const resetPage = useCallback(() => goToPage(1, false), [goToPage]);
    const [exportOpen, setExportOpen] = useState(false);

    // The export dialog offers full-table-only fields (descriptions, item usage,
    // skin art) the slim index lacks. Fetched only once the dialog opens; the
    // whole table is too large to return to the browser.
    const { data: fullOperators } = useQuery({ ...operatorsExportFieldsQueryOptions(server), enabled: exportOpen });
    const fullById = useMemo(() => {
        const map = new Map<string, IOperatorExportFields>();
        for (const op of fullOperators ?? []) if (op.id) map.set(op.id, op);
        return map;
    }, [fullOperators]);
    const toExportRow = useCallback(
        (v: IOperatorView): IOperatorExportRow => {
            const full = v.id ? fullById.get(v.id) : undefined;
            return {
                ...v,
                displayNumber: full?.displayNumber ?? null,
                description: full?.description ?? null,
                itemUsage: full?.itemUsage ?? null,
                itemDesc: full?.itemDesc ?? null,
                itemObtainApproach: full?.itemObtainApproach ?? null,
                isSpChar: full?.isSpChar ?? null,
                maxPotentialLevel: full?.maxPotentialLevel ?? null,
                skin: full?.skin ?? null,
            };
        },
        [fullById],
    );

    const totalCount = isUpcoming ? upcomingFiltered.length : filteredOperators.length;
    const pageSize = itemsPerPage === "all" ? Math.max(totalCount, 1) : itemsPerPage;
    const totalPages = Math.max(1, Math.ceil(totalCount / pageSize));
    // A narrowed filter can leave the counter past the end of the shorter list;
    // every read below uses the clamped value so the pager and the grid agree.
    const page = Math.min(currentPage, totalPages);

    const { paginated, upcomingPaginated, fromIndex, toIndex } = useMemo(() => {
        const start = (page - 1) * pageSize;
        const end = page * pageSize;
        return {
            paginated: filteredOperators.slice(start, end),
            upcomingPaginated: upcomingFiltered.slice(start, end),
            fromIndex: totalCount === 0 ? 0 : start + 1,
            toIndex: Math.min(end, totalCount),
        };
    }, [filteredOperators, upcomingFiltered, page, pageSize, totalCount]);

    const activeChips = useMemo(() => {
        const chips = buildSharedChips(filters, removeFrom, filterOptions.labels, fact, (type) => t("filters.chip.module", { type }));
        if (filters.hasNotes !== "any") {
            chips.push({
                key: `notes-${filters.hasNotes}`,
                label: t(HAS_NOTES_LABEL_KEY[filters.hasNotes]),
                onRemove: () => setHasNotes("any"),
            });
        }
        return chips;
    }, [filters, filterOptions.labels, removeFrom, setHasNotes, t, fact]);

    // Only the export dialog reads these, and `toExportRow`'s merge is empty until
    // it opens, so skip the rebuild while it's closed.
    const exportAllRows = useMemo(() => (exportOpen ? enriched.map(toExportRow) : EMPTY_EXPORT_ROWS), [exportOpen, enriched, toExportRow]);
    const exportFilteredRows = useMemo(() => (exportOpen ? filteredOperators.map(toExportRow) : EMPTY_EXPORT_ROWS), [exportOpen, filteredOperators, toExportRow]);
    const exportPageRows = useMemo(() => (exportOpen ? paginated.map(toExportRow) : EMPTY_EXPORT_ROWS), [exportOpen, paginated, toExportRow]);

    return (
        <div className="page-shell [--page-max:1400px]">
            <PageHeader className="pb-1.5" breadcrumbLabel={t("list.breadcrumb.aria")} breadcrumb={[t("list.breadcrumb.collection"), t("list.breadcrumb.operators")]} title={t("list.title")} description={rt("list.viewAll", { count: <strong className="text-foreground">{operators.length}</strong> })} />
            <div className="relative flex items-start pt-5">
                <OperatorFilters
                    selectedClasses={filters.classes}
                    selectedSubclasses={filters.subclasses}
                    selectedRarities={filters.rarities}
                    selectedGenders={filters.genders}
                    selectedNations={filters.nations}
                    selectedFactions={filters.factions}
                    selectedRaces={filters.races}
                    selectedBirthPlaces={filters.birthPlaces}
                    selectedArtists={filters.artists}
                    selectedVoiceActors={filters.voiceActors}
                    selectedModules={filters.modules}
                    selectedHasNotes={filters.hasNotes}
                    selectedAvailability={filters.availability}
                    options={filterOptions}
                    onClassesChange={setClasses}
                    onSubclassesChange={setSubclasses}
                    onRaritiesChange={setRarities}
                    onGendersChange={setGenders}
                    onNationsChange={setNations}
                    onFactionsChange={setFactions}
                    onRacesChange={setRaces}
                    onBirthPlacesChange={setBirthPlaces}
                    onArtistsChange={setArtists}
                    onVoiceActorsChange={setVoiceActors}
                    onModulesChange={setModules}
                    onHasNotesChange={setHasNotes}
                    onAvailabilityChange={(v) => {
                        setAvailability(v);
                        resetPage();
                    }}
                    onClearAll={clearFilters}
                    hasActiveFilters={hasActiveFilters}
                    collapsed={!filtersOpen}
                    onToggle={toggleFilters}
                    activeFilterCount={activeFilterCount}
                />

                <section className="flex min-w-0 flex-1 flex-col gap-3.5" aria-label={t("list.results.aria")}>
                    <div className="flex flex-wrap items-center gap-2.5">
                        <FilterToggleButton visible={filtersOpen} onToggle={toggleFilters} activeCount={activeFilterCount} />

                        <SearchField className="h-10 min-w-60 flex-1 sm:h-9 sm:max-w-115" value={filters.searchQuery} onChange={setSearchQuery} placeholder={t("list.search.placeholder")} label={t("list.search.aria")} />

                        <ToggleGroup aria-label={t("list.viewMode.aria")} value={[viewMode]} onValueChange={(next) => next[0] && setViewMode(next[0] as ViewMode)} variant="outline" size="lg">
                            {VIEW_MODE_ICONS.map(({ mode, Icon }) => (
                                <Tooltip key={mode}>
                                    <TooltipTrigger
                                        render={
                                            <ToggleGroupItem
                                                value={mode}
                                                aria-label={t(`list.viewMode.${mode}`)}
                                                className="text-muted-foreground hover:text-foreground data-pressed:inset-ring-0 data-pressed:bg-primary data-pressed:text-primary-foreground data-pressed:hover:bg-primary-hover dark:data-pressed:bg-primary dark:data-pressed:hover:bg-primary-hover"
                                            />
                                        }
                                    >
                                        <Icon aria-hidden="true" />
                                    </TooltipTrigger>
                                    <TooltipPopup side="top" sideOffset={8}>
                                        {t(`list.viewMode.${mode}`)}
                                    </TooltipPopup>
                                </Tooltip>
                            ))}
                        </ToggleGroup>

                        <ToggleGroup aria-label={t("list.stat.aria")} value={[statMetric]} onValueChange={(next) => next[0] && setStatMetric(next[0] as StatMetric)} variant="outline" size="lg">
                            {(["owned", "e2"] as const).map((metric) => (
                                <Tooltip key={metric}>
                                    <TooltipTrigger
                                        render={
                                            <ToggleGroupItem
                                                value={metric}
                                                className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em] hover:text-foreground data-pressed:inset-ring-0 data-pressed:bg-primary data-pressed:text-primary-foreground data-pressed:hover:bg-primary-hover sm:text-[10px] dark:data-pressed:bg-primary dark:data-pressed:hover:bg-primary-hover"
                                            />
                                        }
                                    >
                                        {t(`list.stat.${metric}`)}
                                    </TooltipTrigger>
                                    <TooltipPopup side="top" sideOffset={8}>
                                        {t(`list.stat.${metric}.tip`)}
                                    </TooltipPopup>
                                </Tooltip>
                            ))}
                        </ToggleGroup>

                        <div className="inline-flex h-10 items-center gap-1 rounded-lg border border-input bg-background p-1 sm:h-9 dark:bg-input/32">
                            <Select value={filters.sortBy} onValueChange={(v) => setSortBy(v as SortOption)} aria-label={t("list.sort.aria")}>
                                <SelectTrigger size="sm" className="h-8 min-h-8 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[13px] text-foreground shadow-none before:shadow-none hover:bg-accent">
                                    <span className="mr-1 border-border border-r pr-1 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{t("list.sort.caption")}</span>
                                    <SelectValue>
                                        {(value) => {
                                            const opt = SORT_OPTIONS.find((o) => o.value === value);
                                            return opt ? t(opt.labelKey) : value;
                                        }}
                                    </SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {SORT_OPTIONS.map((opt) => (
                                        <SelectItem key={opt.value} value={opt.value}>
                                            {t(opt.labelKey)}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                            <Tooltip>
                                <TooltipTrigger render={<Button variant="ghost" size="icon-sm" className="text-muted-foreground hover:text-foreground" onClick={() => setSortOrder((filters.sortOrder === "asc" ? "desc" : "asc") as SortOrder)} aria-label={t("list.sort.toggleAria")} />}>
                                    {filters.sortOrder === "asc" ? <ArrowUp className="h-3.5 w-3.5" aria-hidden="true" /> : <ArrowDown className="h-3.5 w-3.5" aria-hidden="true" />}
                                </TooltipTrigger>
                                <TooltipPopup side="top" sideOffset={8}>
                                    {filters.sortOrder === "asc" ? t("list.sort.asc") : t("list.sort.desc")}
                                </TooltipPopup>
                            </Tooltip>
                        </div>

                        <div className="inline-flex h-10 items-center rounded-lg border border-input bg-background p-1 sm:h-9 dark:bg-input/32">
                            <Select
                                value={String(itemsPerPage)}
                                onValueChange={(v) => {
                                    setItemsPerPage(v === "all" ? "all" : (Number(v) as ItemsPerPage));
                                    resetPage();
                                }}
                                aria-label={t("list.perPage.aria")}
                            >
                                <SelectTrigger size="sm" className="h-8 min-h-8 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[13px] text-foreground shadow-none before:shadow-none hover:bg-accent">
                                    <span className="mr-1 border-border border-r pr-1 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{t("list.perPage.caption")}</span>
                                    <SelectValue>{(value) => (value === "all" ? t("list.perPage.all") : value)}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {ITEMS_PER_PAGE_OPTIONS.map((opt) => (
                                        <SelectItem key={opt} value={String(opt)}>
                                            {opt === "all" ? t("list.perPage.all") : opt}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <Button variant="outline" size="lg" className="gap-1.5" onClick={() => setExportOpen(true)} aria-label={t("list.export.tip")}>
                                        <Download className="h-3.5 w-3.5" aria-hidden="true" />
                                        <span className="hidden sm:inline">{t("list.export")}</span>
                                    </Button>
                                }
                            />
                            <TooltipPopup side="top" sideOffset={8}>
                                {t("list.export.tip")}
                            </TooltipPopup>
                        </Tooltip>
                    </div>

                    <ActiveFilterChips chips={activeChips} onClearAll={clearFilters} />

                    <div className="flex flex-wrap items-center justify-between gap-3 font-medium font-sans text-[12.5px] text-muted-foreground leading-none">
                        <span>
                            {rt("list.showing", {
                                from: <strong className="text-foreground">{fromIndex}</strong>,
                                to: <strong className="text-foreground">{toIndex}</strong>,
                                total: <strong className="text-foreground">{totalCount}</strong>,
                            })}
                        </span>
                        <div className="ml-auto flex items-center gap-3">
                            <span className="hidden font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.08em] [@media(hover:hover)]:md:inline">{t("list.hint")}</span>
                            <PaginationCompact currentPage={page} totalPages={totalPages} onPageChange={handlePageChange} />
                        </div>
                    </div>

                    {isUpcoming && upcomingFailed && <LoadFailed message={t("list.error.upcoming")} onRetry={() => void refetchUpcoming()} />}

                    {isUpcoming && upcomingLoading ? (
                        <div className="grid grid-cols-3 gap-2.5 min-[1080px]:grid-cols-6 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5 min-[1280px]:gap-4 min-[780px]:gap-3">
                            {UPCOMING_SKELETON_KEYS.map((k) => (
                                <Skeleton key={k} className="aspect-2/3 w-full rounded-md" />
                            ))}
                        </div>
                    ) : totalCount === 0 ? (
                        // A failed upcoming fetch is said by the alert above; an empty state here would
                        // claim the filters removed operators that never arrived.
                        isUpcoming && upcomingFailed ? null : (
                            <Empty className="rounded-xl border border-border border-dashed bg-card/50 md:py-16">
                                <EmptyHeader>
                                    <EmptyDescription>{isUpcoming ? t("list.empty.upcoming") : t("list.empty")}</EmptyDescription>
                                </EmptyHeader>
                                {hasActiveFilters && (
                                    <EmptyContent>
                                        <Button onClick={clearFilters} size="sm" variant="outline">
                                            {t("list.clearFilters")}
                                        </Button>
                                    </EmptyContent>
                                )}
                            </Empty>
                        )
                    ) : isUpcoming ? (
                        <div className="grid grid-cols-3 gap-2.5 min-[1080px]:grid-cols-6 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5 min-[1280px]:gap-4 min-[780px]:gap-3">
                            {upcomingPaginated.map(({ operator: op, updated }) => (updated ? <OperatorCardGrid key={op.id} operator={op} statMetric={statMetric} /> : <OperatorCardUpcoming key={op.id} operator={op} />))}
                        </div>
                    ) : viewMode === "grid" ? (
                        <div className="grid grid-cols-3 gap-2.5 min-[1080px]:grid-cols-6 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5 min-[1280px]:gap-4 min-[780px]:gap-3">
                            {paginated.map((op) => (
                                <OperatorCardGrid key={op.id} operator={op} statMetric={statMetric} />
                            ))}
                        </div>
                    ) : viewMode === "compact" ? (
                        <div className="grid grid-cols-3 gap-1 min-[1080px]:grid-cols-6 min-[1280px]:grid-cols-7 min-[1536px]:grid-cols-8 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5">
                            {paginated.map((op) => (
                                <OperatorCardCompact key={op.id} operator={op} statMetric={statMetric} />
                            ))}
                        </div>
                    ) : (
                        <div className="flex flex-col gap-1 pt-1">
                            <div className="kicker grid items-center gap-3 rounded-lg border border-transparent border-b-border/60 px-3 pb-2 leading-none max-[900px]:hidden" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
                                <span />
                                <span>{t("list.column.name")}</span>
                                <span className="text-center">{t("list.column.rarity")}</span>
                                <span className="text-center">{t("list.column.class")}</span>
                                <span className="text-center">{t("list.column.archetype")}</span>
                                <span className="text-center">{t("list.column.owned")}</span>
                                <span />
                            </div>
                            <div className="flex flex-col gap-1">
                                {paginated.map((op) => (
                                    <OperatorCardList key={op.id} operator={op} statMetric={statMetric} />
                                ))}
                            </div>
                        </div>
                    )}

                    <Pagination currentPage={page} totalPages={totalPages} onPageChange={handlePageChange} />
                </section>
            </div>
            <DynamicExportDialog open={exportOpen} onOpenChange={setExportOpen} schema={operatorsExportSchema} allRows={exportAllRows} filteredRows={exportFilteredRows} pageRows={exportPageRows} title={t("list.export.dialogTitle")} />
        </div>
    );
}
