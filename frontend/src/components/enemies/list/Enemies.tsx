import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ArrowDown, ArrowUp, ChevronRight, Download, LayoutGrid, LayoutList, Search } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ExportDialog } from "#/components/export/ExportDialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { enemiesQueryOptions, enemyStagesQueryOptions } from "#/lib/api/enemies";
import { enemiesExportSchema } from "#/lib/export";
import { type TypedRichT, useGamedataServer, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { values } from "#/lib/records";
import type { StageGroupKey } from "#/lib/registry/stage-groups";
import { Route } from "#/routes/enemies";
import { Pagination, PaginationCompact } from "../../operators/list/impl/components/Pagination";
import type { messages } from "./Enemies.messages";
import { EnemyCardGrid } from "./impl/components/EnemyCardGrid";
import { EnemyCardList } from "./impl/components/EnemyCardList";
import { EnemyFilterChips } from "./impl/components/EnemyFilterChips";
import { buildLocationTree, EnemyLocationFilter, type IRawStage, type IRawZone } from "./impl/components/EnemyLocationFilter";
import { ITEMS_PER_PAGE, ITEMS_PER_PAGE_KEY, ITEMS_PER_PAGE_OPTIONS, type ItemsPerPage, LIST_GRID_COLS, PAGE_KEY, SORT_OPTIONS, VIEW_MODE_KEY, VIEW_MODES } from "./impl/constants";
import type { messages as listConstantsMessages } from "./impl/constants.messages";
import { computeStatMaxByLevel, enrichEnemies } from "./impl/enrich";
import type { IEnemyLocationIndex, IEnemyView, SortOption, SortOrder, ViewMode } from "./impl/types";
import { useEnemyFilters } from "./impl/useEnemyFilters";

export function EnemiesList() {
    const t: TypedT<typeof messages> = useT("enemies");
    const rt: TypedRichT<typeof messages> = useRichT("enemies");
    const tConst: TypedT<typeof listConstantsMessages> = useT("enemies");
    const server = useGamedataServer();
    const { data: handbook } = useQuery(enemiesQueryOptions(server));

    const enriched = useMemo(() => {
        if (!handbook) return [] as IEnemyView[];
        const list = values(handbook.enemyData).sort((a, b) => a.sortId - b.sortId);
        return enrichEnemies(list, handbook.raceData);
    }, [handbook]);

    const visibleCount = useMemo(() => enriched.filter((e) => !e.hideInHandbook).length, [enriched]);
    const statMax = useMemo(() => computeStatMaxByLevel(enriched), [enriched]);

    const availableRaces = useMemo(() => {
        if (!handbook) return [];
        const present = new Set<string>();
        for (const e of enriched) {
            if (e.hideInHandbook) continue;
            for (const tag of e.enemyTags ?? []) present.add(tag);
        }
        return values(handbook.raceData)
            .filter((r) => present.has(r.id))
            .sort((a, b) => a.raceName.localeCompare(b.raceName))
            .map((r) => ({ id: r.id, label: r.raceName }));
    }, [handbook, enriched]);

    // Location index ("Appears In" filter), derived from the enemy-stage data.
    const { data: stageIndex } = useQuery(enemyStagesQueryOptions(server));
    const { locationIndex, locationTree } = useMemo(() => {
        const zonesByEnemy = new Map<string, Set<string>>();
        const stagesByEnemy = new Map<string, Set<string>>();
        const zoneTmp = new Map<string, { name: string; group: StageGroupKey; stages: Map<string, IRawStage> }>();
        for (const [enemyId, refs] of Object.entries(stageIndex ?? {})) {
            const zones = new Set<string>();
            const stages = new Set<string>();
            for (const r of refs) {
                zones.add(r.zoneId);
                stages.add(r.stageId);
                let z = zoneTmp.get(r.zoneId);
                if (!z) {
                    z = { name: r.zoneName ?? r.zoneId, group: r.group, stages: new Map() };
                    zoneTmp.set(r.zoneId, z);
                }
                if (!z.stages.has(r.stageId)) z.stages.set(r.stageId, { stageId: r.stageId, code: r.code, stageName: r.stageName, isHard: r.isHard });
            }
            zonesByEnemy.set(enemyId, zones);
            stagesByEnemy.set(enemyId, stages);
        }
        const rawZones: IRawZone[] = [...zoneTmp].map(([zoneId, z]) => ({ zoneId, name: z.name, group: z.group, stages: [...z.stages.values()] }));
        return { locationIndex: { zonesByEnemy, stagesByEnemy } satisfies IEnemyLocationIndex, locationTree: buildLocationTree(rawZones) };
    }, [stageIndex]);

    // The page number round-trips through the URL, so browser back from an enemy
    // lands on the page you left rather than restarting at one.
    const { page: pageFromUrl } = Route.useSearch();
    const navigate = useNavigate({ from: "/enemies" });
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

    // Arriving with no page in the URL - the breadcrumb out of an enemy, the nav
    // link - resumes the page you were last on, the way the filters and the view
    // mode already resume. A page in the URL is explicit and always wins, which
    // is what keeps browser back landing exactly where it left.
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

    const { filters, filteredEnemies, setSearchQuery, setLevels, setDamageTypes, setAttackTypes, setRaces, setAppearsIn, setSortBy, setSortOrder, clearFilters, activeFilterCount } = useEnemyFilters(enriched, locationIndex, resetPage);

    const [viewMode, setViewMode] = useLocalStorageState<ViewMode>(VIEW_MODE_KEY, "grid", {
        parse: (raw) => (VIEW_MODES.has(raw as ViewMode) ? (raw as ViewMode) : undefined),
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

    const pageSize = itemsPerPage === "all" ? Math.max(filteredEnemies.length, 1) : itemsPerPage;
    const totalPages = Math.max(1, Math.ceil(filteredEnemies.length / pageSize));
    // A narrowed filter can leave the counter past the end of the shorter list;
    // every read below uses the clamped value so the pager and the grid agree.
    const page = Math.min(currentPage, totalPages);
    const { paginated, fromIndex, toIndex } = useMemo(() => {
        const start = (page - 1) * pageSize;
        const end = page * pageSize;
        return {
            paginated: filteredEnemies.slice(start, end),
            fromIndex: filteredEnemies.length === 0 ? 0 : start + 1,
            toIndex: Math.min(end, filteredEnemies.length),
        };
    }, [filteredEnemies, page, pageSize]);

    const [exportOpen, setExportOpen] = useState(false);

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-2rem))] pb-20">
            <div className="pt-7 pb-1.5">
                <nav className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none" aria-label={t("list.breadcrumb")}>
                    <span>{t("list.breadcrumb.collection")}</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <span className="text-foreground">{t("list.breadcrumb.enemies")}</span>
                </nav>
                <h1 className="m-0 font-bold font-sans text-[30px] text-foreground leading-[1.1] tracking-tight">{t("list.title")}</h1>
                <p className="mt-1.5 font-sans text-[13.5px] text-muted-foreground leading-normal">{rt("list.blurb", { count: <strong className="text-foreground">{visibleCount}</strong> })}</p>
            </div>

            <main className="flex min-w-0 flex-col gap-5.5 pt-5" aria-label={t("list.results.aria")}>
                <EnemyFilterChips filters={filters} setLevels={setLevels} setDamageTypes={setDamageTypes} setAttackTypes={setAttackTypes} setRaces={setRaces} races={availableRaces} />

                {locationTree.length > 0 && <EnemyLocationFilter tree={locationTree} selected={filters.appearsIn} onChange={setAppearsIn} />}

                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="relative flex h-9.5 min-w-55 max-w-100 flex-1 items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] px-3.5 transition-[border-color,box-shadow] duration-150 focus-within:border-primary focus-within:shadow-[0_0_0_1px_var(--primary)] [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
                        <Search className="h-3.5 w-3.5" aria-hidden="true" />
                        <input
                            type="text"
                            value={filters.q}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            placeholder={t("list.search.placeholder")}
                            aria-label={t("list.search.aria")}
                            className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-[13px] text-foreground leading-none outline-none placeholder:text-muted-foreground"
                        />
                    </div>

                    <div className="inline-flex flex-wrap items-center gap-2">
                        {/* biome-ignore lint/a11y/useSemanticElements: role="group" is appropriate for this toggle button group */}
                        <div
                            role="group"
                            aria-label={t("list.viewMode.aria")}
                            className="inline-flex h-9.5 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] p-0.75 [&>button:not([data-on]):hover]:text-foreground [&>button[data-on]]:bg-primary [&>button[data-on]]:text-primary-foreground [&>button]:inline-flex [&>button]:h-7.5 [&>button]:w-8 [&>button]:cursor-pointer [&>button]:items-center [&>button]:justify-center [&>button]:rounded-md [&>button]:border-0 [&>button]:bg-transparent [&>button]:p-0 [&>button]:text-muted-foreground [&>button]:transition-[background-color,color] [&>button]:duration-150"
                        >
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title={t("list.viewMode.grid")} data-on={viewMode === "grid" || undefined} onClick={() => setViewMode("grid")} aria-pressed={viewMode === "grid"}>
                                            <LayoutGrid className="h-3.5 w-3.5" aria-hidden="true" />
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
                                        <button type="button" title={t("list.viewMode.list")} data-on={viewMode === "list" || undefined} onClick={() => setViewMode("list")} aria-pressed={viewMode === "list"}>
                                            <LayoutList className="h-3.5 w-3.5" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    {t("list.viewMode.list")}
                                </TooltipPopup>
                            </Tooltip>
                        </div>

                        <div className="inline-flex h-9.5 items-center gap-1 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] p-0.75">
                            <Select value={filters.sortBy} onValueChange={(v) => setSortBy(v as SortOption)} aria-label={t("list.sort.aria")}>
                                <SelectTrigger size="sm" className="h-7.5 min-h-7.5 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[12.5px] text-foreground shadow-none before:shadow-none hover:bg-[color-mix(in_oklch,var(--secondary)_80%,transparent)]">
                                    <span className="mr-1 border-border border-r pr-1 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{t("list.sort.label")}</span>
                                    <SelectValue>
                                        {(value) => {
                                            const opt = SORT_OPTIONS.find((o) => o.value === value);
                                            return opt ? tConst(opt.labelKey) : value;
                                        }}
                                    </SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {SORT_OPTIONS.map((opt) => (
                                        <SelectItem key={opt.value} value={opt.value}>
                                            {tConst(opt.labelKey)}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                            <button
                                type="button"
                                className="inline-flex h-7.5 w-7.5 cursor-pointer appearance-none items-center justify-center rounded-md border-0 bg-transparent p-0 text-muted-foreground transition-[background-color,color] duration-150 hover:bg-secondary hover:text-foreground"
                                title={filters.sortOrder === "asc" ? t("list.sort.asc") : t("list.sort.desc")}
                                onClick={() => setSortOrder((filters.sortOrder === "asc" ? "desc" : "asc") as SortOrder)}
                                aria-label={t("list.sort.toggle")}
                            >
                                {filters.sortOrder === "asc" ? <ArrowUp className="h-3 w-3" aria-hidden="true" /> : <ArrowDown className="h-3 w-3" aria-hidden="true" />}
                            </button>
                        </div>

                        <div className="inline-flex h-9.5 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] p-0.75">
                            <Select
                                value={String(itemsPerPage)}
                                onValueChange={(v) => {
                                    setItemsPerPage(v === "all" ? "all" : (Number(v) as ItemsPerPage));
                                    resetPage();
                                }}
                                aria-label={t("list.perPage.aria")}
                            >
                                <SelectTrigger size="sm" className="h-7.5 min-h-7.5 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[12.5px] text-foreground shadow-none before:shadow-none hover:bg-[color-mix(in_oklch,var(--secondary)_80%,transparent)]">
                                    <span className="mr-1 border-border border-r pr-1 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{t("list.perPage.label")}</span>
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
                                        className="inline-flex h-9.5 cursor-pointer items-center gap-1.5 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] px-3 font-medium font-sans text-[12.5px] text-foreground transition-colors hover:border-[color-mix(in_oklch,var(--primary)_55%,var(--border))] hover:bg-card"
                                        onClick={() => setExportOpen(true)}
                                        aria-label={t("list.export.aria")}
                                    >
                                        <Download className="h-3.5 w-3.5" aria-hidden="true" />
                                        <span className="hidden sm:inline">{t("list.export")}</span>
                                    </button>
                                }
                            />
                            <TooltipPopup side="top" sideOffset={8}>
                                {t("list.export.aria")}
                            </TooltipPopup>
                        </Tooltip>
                    </div>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-3 font-medium font-sans text-[12.5px] text-muted-foreground leading-none">
                    <span>
                        {rt("list.showing", {
                            from: <strong className="text-foreground">{fromIndex}</strong>,
                            to: <strong className="text-foreground">{toIndex}</strong>,
                            total: <strong className="text-foreground">{filteredEnemies.length}</strong>,
                        })}
                        {activeFilterCount > 0 && (
                            <>
                                {" · "}
                                <button type="button" onClick={clearFilters} className="cursor-pointer appearance-none border-0 bg-transparent p-0 font-medium text-primary hover:underline">
                                    {t("list.clearFilters", { count: activeFilterCount })}
                                </button>
                            </>
                        )}
                    </span>
                    <div className="ml-auto flex items-center gap-3">
                        <span className="hidden font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em] sm:inline">{t("list.hint")}</span>
                        <PaginationCompact currentPage={page} totalPages={totalPages} onPageChange={handlePageChange} />
                    </div>
                </div>

                {filteredEnemies.length === 0 ? (
                    <EmptyState onClear={clearFilters} />
                ) : viewMode === "grid" ? (
                    <div className="grid grid-cols-2 gap-2.5 min-[1100px]:grid-cols-6 min-[420px]:grid-cols-3 min-[640px]:grid-cols-4 min-[860px]:grid-cols-5 min-[1280px]:gap-4">
                        {paginated.map((e) => (
                            <EnemyCardGrid key={e.enemyId} enemy={e} statMax={statMax} />
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col gap-1">
                        <div className="grid items-center gap-3.5 border-b border-b-[color-mix(in_oklch,var(--border)_60%,transparent)] px-3.5 pb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em] max-[900px]:hidden" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
                            <span />
                            <span>{t("list.col.name")}</span>
                            <span>{t("list.col.threat")}</span>
                            <span>{t("list.col.damage")}</span>
                            <span className="text-right">{t("list.col.hp")}</span>
                        </div>
                        {paginated.map((e) => (
                            <EnemyCardList key={e.enemyId} enemy={e} statMax={statMax} />
                        ))}
                    </div>
                )}

                <Pagination currentPage={page} totalPages={totalPages} onPageChange={handlePageChange} />
            </main>

            <ExportDialog open={exportOpen} onOpenChange={setExportOpen} schema={enemiesExportSchema} allRows={enriched} filteredRows={filteredEnemies} pageRows={paginated} title={t("list.export.dialogTitle")} />
        </div>
    );
}

function EmptyState({ onClear }: { onClear: () => void }) {
    const t: TypedT<typeof messages> = useT("enemies");
    return (
        <div className="flex flex-col items-center justify-center rounded-[14px] border border-border border-dashed bg-card/50 px-6 py-16 text-center">
            <div className="mb-3.5 grid h-14 w-14 place-items-center rounded-full bg-secondary">
                <Search className="h-6.5 w-6.5 text-muted-foreground" strokeWidth={2} aria-hidden="true" />
            </div>
            <h3 className="m-0 font-sans font-semibold text-[16px] text-foreground leading-none">{t("list.empty.title")}</h3>
            <p className="m-0 mt-2 mb-3.5 max-w-80 font-sans text-[13px] text-muted-foreground leading-normal">{t("list.empty.body")}</p>
            <button type="button" onClick={onClear} className="cursor-pointer appearance-none border-0 bg-transparent p-0 font-medium font-sans text-[13px] text-primary hover:underline">
                {t("list.empty.clear")}
            </button>
        </div>
    );
}
