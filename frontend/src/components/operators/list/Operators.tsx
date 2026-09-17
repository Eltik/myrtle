import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ArrowDown, ArrowUp, ChevronRight, Download, LayoutGrid, LayoutList, Rows3, Search } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ExportDialog } from "#/components/export/ExportDialog";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useOperatorName } from "#/hooks/use-operator-name";
import { noteHasContent, operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorOwnershipQueryOptions, operatorsIndexQueryOptions, operatorsListQueryOptions } from "#/lib/api/operators";
import { upcomingQueryOptions } from "#/lib/api/upcoming";
import { voicesQueryOptions } from "#/lib/api/voices";
import { operatorsExportSchema } from "#/lib/export";
import { type TypedRichT, useGamedataServer, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { compactForSearch } from "#/lib/search/fuzzy";
import { Route } from "#/routes/operators";
import type { IOperatorListItem, OperatorRarityTier } from "#/types/operators";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../../ui/select";
import { Skeleton } from "../../ui/skeleton";
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
import { buildSharedChips } from "./impl/shared-filters";
import type { IOperatorExportRow, IOperatorOwnershipInfo, IOperatorView, SortOption, SortOrder, StatMetric, ViewMode } from "./impl/types";
import { useOperatorFilters } from "./impl/useOperatorFilters";
import type { messages } from "./Operators.messages";

const UPCOMING_SKELETON_KEYS = Array.from({ length: 18 }, (_, i) => `upcoming-skeleton-${i}`);

// Shared empty array so the export-row memos keep a stable reference while the
// export dialog is closed, avoiding needless ExportDialog re-renders.
const EMPTY_EXPORT_ROWS: IOperatorExportRow[] = [];

export function OperatorsList() {
    const t: TypedT<typeof messages & typeof listConstantsMessages> = useT("operators");
    const rt: TypedRichT<typeof messages> = useRichT("operators");
    const server = useGamedataServer();
    const { data: operators = [] } = useQuery(operatorsIndexQueryOptions(server));
    const { data: voices } = useQuery(voicesQueryOptions(server));
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
    const enriched = useMemo(() => enrichOperators(operators, voices, notedIds, ownershipMap), [operators, voices, notedIds, ownershipMap]);

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
        setHasNotes,
        setAvailability,
        setSortBy,
        setSortOrder,
        removeFrom,
        clearFilters,
        hasActiveFilters,
        activeFilterCount,
    } = useOperatorFilters(enriched);

    const { data: upcoming = [], isLoading: upcomingLoading } = useQuery(upcomingQueryOptions(server));
    const operatorName = useOperatorName();
    const isUpcoming = filters.availability === "upcoming";

    const upcomingFiltered = useMemo(() => {
        const query = compactForSearch(filters.searchQuery.trim());
        const classes = new Set(filters.classes);
        const subclasses = new Set(filters.subclasses);
        const rarities = new Set(filters.rarities);
        const nations = new Set(filters.nations);
        return upcoming
            .filter((op) => {
                if (query && !compactForSearch(`${op.name} ${op.appellation ?? ""} ${op.subProfessionId}`).includes(query)) return false;
                if (classes.size && !classes.has(op.profession)) return false;
                if (subclasses.size && !subclasses.has(op.subProfessionId)) return false;
                if (rarities.size && !rarities.has(`TIER_${op.rarity}` as OperatorRarityTier)) return false;
                if (nations.size && !nations.has(op.nationId)) return false;
                return true;
            })
            .sort((a, b) => b.rarity - a.rarity || operatorName(a).localeCompare(operatorName(b)));
    }, [upcoming, filters.searchQuery, filters.classes, filters.subclasses, filters.rarities, filters.nations, operatorName]);

    const [viewMode, setViewMode] = useLocalStorageState<ViewMode>(VIEW_MODE_KEY, "grid", {
        parse: (raw) => (VIEW_MODES.has(raw as ViewMode) ? (raw as ViewMode) : undefined),
        serialize: (v) => v,
    });
    const [filtersVisible, setFiltersVisible] = useLocalStorageState<boolean>(FILTERS_VISIBLE_KEY, false, {
        parse: (raw) => raw === "1",
        serialize: (v) => (v ? "1" : "0"),
    });
    const toggleFilters = () => setFiltersVisible((v) => !v);

    // Defaults to "owned", which reproduces the badge exactly as it was before
    // the E2 metric existed.
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

    // Arriving with no page in the URL - the breadcrumb out of an operator, the
    // nav link - resumes the page you were last on, the way the filters and the
    // view mode already resume. A page in the URL is explicit and always wins,
    // which is what keeps browser back landing exactly where it left.
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
    // skin art, etc.) that the slim operators index doesn't carry. Fetch the full
    // operator table lazily - only once the dialog is opened - and merge those
    // fields onto the rows we hand the dialog.
    const { data: fullOperators } = useQuery({ ...operatorsListQueryOptions(server), enabled: exportOpen });
    const fullById = useMemo(() => {
        const map = new Map<string, IOperatorListItem>();
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
        const chips = buildSharedChips(filters, removeFrom);
        if (filters.hasNotes !== "any") {
            chips.push({
                key: `notes-${filters.hasNotes}`,
                label: t(HAS_NOTES_LABEL_KEY[filters.hasNotes]),
                onRemove: () => setHasNotes("any"),
            });
        }
        return chips;
    }, [filters, removeFrom, setHasNotes, t]);

    // Only the export dialog consumes these, and the full-table merge (toExportRow)
    // is empty until it opens - so skip the row rebuild entirely while it's closed.
    const exportAllRows = useMemo(() => (exportOpen ? enriched.map(toExportRow) : EMPTY_EXPORT_ROWS), [exportOpen, enriched, toExportRow]);
    const exportFilteredRows = useMemo(() => (exportOpen ? filteredOperators.map(toExportRow) : EMPTY_EXPORT_ROWS), [exportOpen, filteredOperators, toExportRow]);
    const exportPageRows = useMemo(() => (exportOpen ? paginated.map(toExportRow) : EMPTY_EXPORT_ROWS), [exportOpen, paginated, toExportRow]);

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-2rem))] pb-20">
            <div className="pt-7 pb-1.5">
                <nav className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none" aria-label={t("list.breadcrumb.aria")}>
                    <span>{t("list.breadcrumb.collection")}</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <span className="text-foreground">{t("list.breadcrumb.operators")}</span>
                </nav>
                <h1 className="m-0 font-bold font-sans text-[30px] text-foreground leading-[1.1] tracking-tight">{t("list.title")}</h1>
                <p className="mt-1.5 font-sans text-[13.5px] text-muted-foreground leading-normal">{rt("list.viewAll", { count: <strong className="text-foreground">{operators.length}</strong> })}</p>
            </div>
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
                    onHasNotesChange={setHasNotes}
                    onAvailabilityChange={(v) => {
                        setAvailability(v);
                        resetPage();
                    }}
                    onClearAll={clearFilters}
                    hasActiveFilters={hasActiveFilters}
                    collapsed={!filtersVisible}
                    onToggle={toggleFilters}
                    activeFilterCount={activeFilterCount}
                />

                <main className="flex min-w-0 flex-1 flex-col gap-3.5" aria-label={t("list.results.aria")}>
                    <div className="flex flex-wrap items-center gap-2.5">
                        <FilterToggleButton visible={filtersVisible} onToggle={toggleFilters} activeCount={activeFilterCount} />

                        <div className="flex h-10 min-w-60 flex-1 items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] px-3 transition-[border-color,box-shadow] duration-150 focus-within:border-primary focus-within:shadow-[0_0_0_1px_var(--primary)] sm:max-w-115 [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
                            <Search className="h-3.75 w-3.75" aria-hidden="true" />
                            <input
                                type="text"
                                value={filters.searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                                placeholder={t("list.search.placeholder")}
                                aria-label={t("list.search.aria")}
                                className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-foreground text-sm leading-none outline-none placeholder:text-muted-foreground"
                            />
                        </div>

                        {/* biome-ignore lint/a11y/useSemanticElements: role="group" is appropriate for this toggle button group */}
                        <div
                            className="inline-flex h-10 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] p-1 [&>button:not([data-on]):hover]:text-foreground [&>button[data-on]]:bg-primary [&>button[data-on]]:text-primary-foreground [&>button]:inline-flex [&>button]:h-8 [&>button]:w-8 [&>button]:cursor-pointer [&>button]:items-center [&>button]:justify-center [&>button]:rounded-md [&>button]:border-0 [&>button]:bg-transparent [&>button]:p-0 [&>button]:text-muted-foreground [&>button]:transition-[background-color,color] [&>button]:duration-150"
                            role="group"
                            aria-label={t("list.viewMode.aria")}
                        >
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title={t("list.viewMode.grid")} data-on={viewMode === "grid" || undefined} onClick={() => setViewMode("grid")} aria-pressed={viewMode === "grid"}>
                                            <LayoutGrid className="h-4 w-4" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    {t("list.viewMode.grid")}
                                </TooltipPopup>
                            </Tooltip>
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title={t("list.viewMode.compact")} data-on={viewMode === "compact" || undefined} onClick={() => setViewMode("compact")} aria-pressed={viewMode === "compact"}>
                                            <Rows3 className="h-4 w-4" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    {t("list.viewMode.compact")}
                                </TooltipPopup>
                            </Tooltip>
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title={t("list.viewMode.list")} data-on={viewMode === "list" || undefined} onClick={() => setViewMode("list")} aria-pressed={viewMode === "list"}>
                                            <LayoutList className="h-4 w-4" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    {t("list.viewMode.list")}
                                </TooltipPopup>
                            </Tooltip>
                        </div>

                        {/* biome-ignore lint/a11y/useSemanticElements: role="group" is appropriate for this toggle button group */}
                        <div
                            className="inline-flex h-10 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] p-1 [&>button:not([data-on]):hover]:text-foreground [&>button[data-on]]:bg-primary [&>button[data-on]]:text-primary-foreground [&>button]:inline-flex [&>button]:h-8 [&>button]:cursor-pointer [&>button]:items-center [&>button]:justify-center [&>button]:rounded-md [&>button]:border-0 [&>button]:bg-transparent [&>button]:px-2 [&>button]:font-medium [&>button]:font-mono [&>button]:text-[10px] [&>button]:text-muted-foreground [&>button]:uppercase [&>button]:tracking-[0.12em] [&>button]:transition-[background-color,color] [&>button]:duration-150"
                            role="group"
                            aria-label={t("list.stat.aria")}
                        >
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" data-on={statMetric === "owned" || undefined} onClick={() => setStatMetric("owned")} aria-pressed={statMetric === "owned"}>
                                            {t("list.stat.owned")}
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    {t("list.stat.owned.tip")}
                                </TooltipPopup>
                            </Tooltip>
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" data-on={statMetric === "e2" || undefined} onClick={() => setStatMetric("e2")} aria-pressed={statMetric === "e2"}>
                                            {t("list.stat.e2")}
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    {t("list.stat.e2.tip")}
                                </TooltipPopup>
                            </Tooltip>
                        </div>

                        <div className="inline-flex h-10 items-center gap-1 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] p-1">
                            <Select value={filters.sortBy} onValueChange={(v) => setSortBy(v as SortOption)} aria-label={t("list.sort.aria")}>
                                <SelectTrigger size="sm" className="h-8 min-h-8 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[13px] text-foreground shadow-none before:shadow-none hover:bg-[color-mix(in_oklch,var(--secondary)_80%,transparent)]">
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
                            <button
                                type="button"
                                className="inline-flex h-8 w-8 cursor-pointer appearance-none items-center justify-center rounded-md border-0 bg-transparent p-0 text-muted-foreground transition-[background-color,color] duration-150 hover:bg-secondary hover:text-foreground"
                                title={filters.sortOrder === "asc" ? t("list.sort.asc") : t("list.sort.desc")}
                                onClick={() => setSortOrder((filters.sortOrder === "asc" ? "desc" : "asc") as SortOrder)}
                                aria-label={t("list.sort.toggleAria")}
                            >
                                {filters.sortOrder === "asc" ? <ArrowUp className="h-3.5 w-3.5" aria-hidden="true" /> : <ArrowDown className="h-3.5 w-3.5" aria-hidden="true" />}
                            </button>
                        </div>

                        <div className="inline-flex h-10 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] p-1">
                            <Select
                                value={String(itemsPerPage)}
                                onValueChange={(v) => {
                                    setItemsPerPage(v === "all" ? "all" : (Number(v) as ItemsPerPage));
                                    resetPage();
                                }}
                                aria-label={t("list.perPage.aria")}
                            >
                                <SelectTrigger size="sm" className="h-8 min-h-8 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[13px] text-foreground shadow-none before:shadow-none hover:bg-[color-mix(in_oklch,var(--secondary)_80%,transparent)]">
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
                                    <button
                                        type="button"
                                        className="inline-flex h-10 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] px-3 font-medium font-sans text-[13px] text-foreground transition-colors hover:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] hover:bg-card"
                                        onClick={() => setExportOpen(true)}
                                        aria-label={t("list.export.tip")}
                                    >
                                        <Download className="h-3.5 w-3.5" aria-hidden="true" />
                                        <span className="hidden sm:inline">{t("list.export")}</span>
                                    </button>
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
                            <span className="hidden font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.08em] md:inline">{t("list.hint")}</span>
                            <PaginationCompact currentPage={page} totalPages={totalPages} onPageChange={handlePageChange} />
                        </div>
                    </div>

                    {isUpcoming && upcomingLoading ? (
                        <div className="grid grid-cols-3 gap-2.5 min-[1080px]:grid-cols-6 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5 min-[1280px]:gap-4 min-[780px]:gap-3">
                            {UPCOMING_SKELETON_KEYS.map((k) => (
                                <Skeleton key={k} className="aspect-2/3 w-full rounded-md" />
                            ))}
                        </div>
                    ) : totalCount === 0 ? (
                        <div className="rounded-xl border border-border border-dashed bg-card/50 py-16 text-center">
                            <p className="font-sans text-muted-foreground text-sm">{isUpcoming ? t("list.empty.upcoming") : t("list.empty")}</p>
                            <button type="button" onClick={clearFilters} className="mt-3 inline-flex items-center gap-1 font-medium text-[12px] text-primary hover:underline">
                                {t("list.clearFilters")}
                            </button>
                        </div>
                    ) : isUpcoming ? (
                        <div className="grid grid-cols-3 gap-2.5 min-[1080px]:grid-cols-6 min-[520px]:grid-cols-4 min-[780px]:grid-cols-5 min-[1280px]:gap-4 min-[780px]:gap-3">
                            {upcomingPaginated.map((op) => (
                                <OperatorCardUpcoming key={op.id} operator={op} />
                            ))}
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
                            <div className="grid items-center gap-3 rounded-lg border border-transparent border-b-border/60 px-3 pb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em] max-[900px]:hidden" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
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
                </main>
            </div>
            <ExportDialog open={exportOpen} onOpenChange={setExportOpen} schema={operatorsExportSchema} allRows={exportAllRows} filteredRows={exportFilteredRows} pageRows={exportPageRows} title={t("list.export.dialogTitle")} />
        </div>
    );
}
