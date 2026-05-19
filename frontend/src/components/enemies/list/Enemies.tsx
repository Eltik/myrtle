import { useQuery } from "@tanstack/react-query";
import { ArrowDown, ArrowUp, ChevronRight, Download, LayoutGrid, LayoutList, Search } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { ExportDialog } from "#/components/export/ExportDialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { enemiesQueryOptions } from "#/lib/api/enemies";
import { enemiesExportSchema } from "#/lib/export";
import { Pagination } from "../../operators/list/impl/components/Pagination";
import { EnemyCardGrid } from "./impl/components/EnemyCardGrid";
import { EnemyCardList } from "./impl/components/EnemyCardList";
import { EnemyDialog } from "./impl/components/EnemyDialog";
import { EnemyFilterChips } from "./impl/components/EnemyFilterChips";
import { ITEMS_PER_PAGE, ITEMS_PER_PAGE_KEY, ITEMS_PER_PAGE_OPTIONS, type ItemsPerPage, LIST_GRID_COLS, SORT_OPTIONS, VIEW_MODE_KEY, VIEW_MODES } from "./impl/constants";
import { computeStatMaxByLevel, enrichEnemies } from "./impl/enrich";
import type { IEnemyView, SortOption, SortOrder, ViewMode } from "./impl/types";
import { useEnemyFilters } from "./impl/useEnemyFilters";

export function EnemiesList() {
    const { data: handbook } = useQuery(enemiesQueryOptions());

    const enriched = useMemo(() => {
        if (!handbook) return [] as IEnemyView[];
        const list = Object.values(handbook.enemyData).sort((a, b) => a.sortId - b.sortId);
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
        return Object.values(handbook.raceData)
            .filter((r) => present.has(r.id))
            .sort((a, b) => a.raceName.localeCompare(b.raceName))
            .map((r) => ({ id: r.id, label: r.raceName }));
    }, [handbook, enriched]);

    const { filters, filteredEnemies, setSearchQuery, setLevels, setDamageTypes, setAttackTypes, setRaces, setSortBy, setSortOrder, clearFilters, activeFilterCount } = useEnemyFilters(enriched);

    const [viewMode, setViewMode] = useLocalStorageState<ViewMode>(VIEW_MODE_KEY, "grid", {
        parse: (raw) => (VIEW_MODES.has(raw as ViewMode) ? (raw as ViewMode) : undefined),
        serialize: (v) => v,
    });

    const [itemsPerPage, setItemsPerPage] = useLocalStorageState<ItemsPerPage>(ITEMS_PER_PAGE_KEY, ITEMS_PER_PAGE, {
        parse: (raw) => {
            const num = Number(raw);
            return (ITEMS_PER_PAGE_OPTIONS as readonly number[]).includes(num) ? (num as ItemsPerPage) : undefined;
        },
        serialize: (v) => String(v),
    });

    const [currentPage, setCurrentPage] = useState(1);
    useEffect(() => {
        setCurrentPage(1);
    }, []);

    const filtersKey = `${filters.q}|${filters.levels.join(",")}|${filters.damageTypes.join(",")}|${filters.attackTypes.join(",")}|${filters.races.join(",")}`;
    // biome-ignore lint/correctness/useExhaustiveDependencies: intentionally watch the joined key as a stable signal
    useEffect(() => {
        setCurrentPage(1);
    }, [filtersKey]);

    const totalPages = Math.max(1, Math.ceil(filteredEnemies.length / itemsPerPage));
    const page = Math.min(currentPage, totalPages);
    const { paginated, fromIndex, toIndex } = useMemo(() => {
        const start = (page - 1) * itemsPerPage;
        const end = page * itemsPerPage;
        return {
            paginated: filteredEnemies.slice(start, end),
            fromIndex: filteredEnemies.length === 0 ? 0 : start + 1,
            toIndex: Math.min(end, filteredEnemies.length),
        };
    }, [filteredEnemies, page, itemsPerPage]);

    const [openEnemy, setOpenEnemy] = useState<IEnemyView | null>(null);
    const [exportOpen, setExportOpen] = useState(false);

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-2rem))] pb-20">
            <div className="pt-7 pb-1.5">
                <nav className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none" aria-label="Breadcrumb">
                    <span>Collection</span>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <span className="text-foreground">Enemies</span>
                </nav>
                <h1 className="m-0 font-bold font-sans text-[30px] text-foreground leading-[1.1] tracking-tight">Enemy Database</h1>
                <p className="mt-1.5 font-sans text-[13.5px] text-muted-foreground leading-normal">
                    View all <strong className="text-foreground">{visibleCount}</strong> enemies catalogued in Arknights, sorted by Hypergryph's internal sort index by default.
                </p>
            </div>

            <main className="flex min-w-0 flex-col gap-5.5 pt-5" aria-label="Enemy results">
                <EnemyFilterChips filters={filters} setLevels={setLevels} setDamageTypes={setDamageTypes} setAttackTypes={setAttackTypes} setRaces={setRaces} races={availableRaces} />

                <div className="flex flex-wrap items-center justify-between gap-3">
                    <div className="relative flex h-9.5 min-w-55 max-w-100 flex-1 items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] px-3.5 transition-[border-color,box-shadow] duration-150 focus-within:border-primary focus-within:shadow-[0_0_0_1px_var(--primary)] [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
                        <Search className="h-3.5 w-3.5" aria-hidden="true" />
                        <input
                            type="text"
                            value={filters.q}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            placeholder="Search by name, callsign, or race…"
                            aria-label="Search enemies"
                            className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-[13px] text-foreground leading-none outline-none placeholder:text-muted-foreground"
                        />
                    </div>

                    <div className="inline-flex flex-wrap items-center gap-2">
                        {/* biome-ignore lint/a11y/useSemanticElements: role="group" is appropriate for this toggle button group */}
                        <div
                            role="group"
                            aria-label="View mode"
                            className="inline-flex h-9.5 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] p-0.75 [&>button:not([data-on]):hover]:text-foreground [&>button[data-on]]:bg-primary [&>button[data-on]]:text-primary-foreground [&>button]:inline-flex [&>button]:h-7.5 [&>button]:w-8 [&>button]:cursor-pointer [&>button]:items-center [&>button]:justify-center [&>button]:rounded-md [&>button]:border-0 [&>button]:bg-transparent [&>button]:p-0 [&>button]:text-muted-foreground [&>button]:transition-[background-color,color] [&>button]:duration-150"
                        >
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title="Grid" data-on={viewMode === "grid" || undefined} onClick={() => setViewMode("grid")} aria-pressed={viewMode === "grid"}>
                                            <LayoutGrid className="h-3.5 w-3.5" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    Grid
                                </TooltipPopup>
                            </Tooltip>
                            <Tooltip>
                                <TooltipTrigger
                                    render={
                                        <button type="button" title="List" data-on={viewMode === "list" || undefined} onClick={() => setViewMode("list")} aria-pressed={viewMode === "list"}>
                                            <LayoutList className="h-3.5 w-3.5" aria-hidden="true" />
                                        </button>
                                    }
                                />
                                <TooltipPopup side="top" sideOffset={8}>
                                    List
                                </TooltipPopup>
                            </Tooltip>
                        </div>

                        <div className="inline-flex h-9.5 items-center gap-1 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] p-0.75">
                            <Select value={filters.sortBy} onValueChange={(v) => setSortBy(v as SortOption)} aria-label="Sort enemies">
                                <SelectTrigger size="sm" className="h-7.5 min-h-7.5 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[12.5px] text-foreground shadow-none before:shadow-none hover:bg-[color-mix(in_oklch,var(--secondary)_80%,transparent)]">
                                    <span className="mr-1 border-border border-r pr-1 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">Sort</span>
                                    <SelectValue>{(value) => SORT_OPTIONS.find((o) => o.value === value)?.label ?? value}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {SORT_OPTIONS.map((opt) => (
                                        <SelectItem key={opt.value} value={opt.value}>
                                            {opt.label}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                            <button
                                type="button"
                                className="inline-flex h-7.5 w-7.5 cursor-pointer appearance-none items-center justify-center rounded-md border-0 bg-transparent p-0 text-muted-foreground transition-[background-color,color] duration-150 hover:bg-secondary hover:text-foreground"
                                title={filters.sortOrder === "asc" ? "Ascending" : "Descending"}
                                onClick={() => setSortOrder((filters.sortOrder === "asc" ? "desc" : "asc") as SortOrder)}
                                aria-label="Toggle sort direction"
                            >
                                {filters.sortOrder === "asc" ? <ArrowUp className="h-3 w-3" aria-hidden="true" /> : <ArrowDown className="h-3 w-3" aria-hidden="true" />}
                            </button>
                        </div>

                        <div className="inline-flex h-9.5 items-center rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_50%,transparent)] p-0.75">
                            <Select
                                value={String(itemsPerPage)}
                                onValueChange={(v) => {
                                    setItemsPerPage(Number(v) as ItemsPerPage);
                                    setCurrentPage(1);
                                }}
                                aria-label="Items per page"
                            >
                                <SelectTrigger size="sm" className="h-7.5 min-h-7.5 min-w-0 gap-1.5 border-0 bg-transparent px-2 font-medium font-sans text-[12.5px] text-foreground shadow-none before:shadow-none hover:bg-[color-mix(in_oklch,var(--secondary)_80%,transparent)]">
                                    <span className="mr-1 border-border border-r pr-1 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">Show</span>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {ITEMS_PER_PAGE_OPTIONS.map((opt) => (
                                        <SelectItem key={opt} value={String(opt)}>
                                            {opt}
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
                                        aria-label="Export enemies"
                                    >
                                        <Download className="h-3.5 w-3.5" aria-hidden="true" />
                                        <span className="hidden sm:inline">Export</span>
                                    </button>
                                }
                            />
                            <TooltipPopup side="top" sideOffset={8}>
                                Export enemies
                            </TooltipPopup>
                        </Tooltip>
                    </div>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-3 font-medium font-sans text-[12.5px] text-muted-foreground leading-none">
                    <span>
                        Showing <strong className="text-foreground">{fromIndex}</strong>–<strong className="text-foreground">{toIndex}</strong> of <strong className="text-foreground">{filteredEnemies.length}</strong> enemies
                        {activeFilterCount > 0 && (
                            <>
                                {" · "}
                                <button type="button" onClick={clearFilters} className="cursor-pointer appearance-none border-0 bg-transparent p-0 font-medium text-primary hover:underline">
                                    Clear {activeFilterCount} filter{activeFilterCount > 1 ? "s" : ""}
                                </button>
                            </>
                        )}
                    </span>
                    <span className="hidden font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em] sm:inline">Click a card to inspect</span>
                </div>

                {filteredEnemies.length === 0 ? (
                    <EmptyState onClear={clearFilters} />
                ) : viewMode === "grid" ? (
                    <div className="grid grid-cols-2 gap-2.5 min-[1100px]:grid-cols-6 min-[420px]:grid-cols-3 min-[640px]:grid-cols-4 min-[860px]:grid-cols-5 min-[1280px]:gap-4">
                        {paginated.map((e) => (
                            <EnemyCardGrid key={e.enemyId} enemy={e} statMax={statMax} onOpen={setOpenEnemy} />
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col gap-1">
                        <div className="grid items-center gap-3.5 border-b border-b-[color-mix(in_oklch,var(--border)_60%,transparent)] px-3.5 pb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em] max-[900px]:hidden" style={{ gridTemplateColumns: LIST_GRID_COLS }}>
                            <span />
                            <span>Name</span>
                            <span>Threat</span>
                            <span>Damage</span>
                            <span className="text-right">HP</span>
                        </div>
                        {paginated.map((e) => (
                            <EnemyCardList key={e.enemyId} enemy={e} statMax={statMax} onOpen={setOpenEnemy} />
                        ))}
                    </div>
                )}

                <Pagination currentPage={page} totalPages={totalPages} onPageChange={setCurrentPage} />
            </main>

            <EnemyDialog enemy={openEnemy} onClose={() => setOpenEnemy(null)} />

            <ExportDialog open={exportOpen} onOpenChange={setExportOpen} schema={enemiesExportSchema} allRows={enriched} filteredRows={filteredEnemies} pageRows={paginated} title="Enemies" />
        </div>
    );
}

function EmptyState({ onClear }: { onClear: () => void }) {
    return (
        <div className="flex flex-col items-center justify-center rounded-[14px] border border-border border-dashed bg-card/50 px-6 py-16 text-center">
            <div className="mb-3.5 grid h-14 w-14 place-items-center rounded-full bg-secondary">
                <Search className="h-6.5 w-6.5 text-muted-foreground" strokeWidth={2} aria-hidden="true" />
            </div>
            <h3 className="m-0 font-sans font-semibold text-[16px] text-foreground leading-none">No enemies match your filters</h3>
            <p className="m-0 mt-2 mb-3.5 max-w-80 font-sans text-[13px] text-muted-foreground leading-normal">Adjust the chip rail or your search above, or clear everything and start fresh.</p>
            <button type="button" onClick={onClear} className="cursor-pointer appearance-none border-0 bg-transparent p-0 font-medium font-sans text-[13px] text-primary hover:underline">
                Clear all filters
            </button>
        </div>
    );
}
