import { useQuery } from "@tanstack/react-query";
import { ArrowDownIcon, ArrowUpIcon } from "lucide-react";
import type * as React from "react";
import { useCallback, useDeferredValue, useMemo, useState } from "react";
import { ActiveFilterChips, type IActiveChip } from "#/components/operators/list/impl/components/ActiveFilterChips";
import { FilterPanel } from "#/components/operators/list/impl/components/FilterPanel";
import { FilterToggleButton } from "#/components/operators/list/impl/components/FilterToggleButton";
import { SearchField } from "#/components/SearchField";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "#/components/ui/table";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { archiveDossiersQueryOptions } from "#/lib/api/archives";
import { useFormatters, useGamedataServer } from "#/lib/i18n";
import { searchAndRank } from "#/lib/search/fuzzy";
import { cn, professionLabel } from "#/lib/utils";
import type { ArchiveDossier } from "#/types/generated/ArchiveDossier";
import { DossierFilterFields } from "./DossierFilters";
import { buildDossierOptions, countDossierFilters, type DossierListKey, EMPTY_DOSSIER_FILTERS, type IDossierFilters, INFECTION_ORDER, matchesDossier } from "./dossierMatch";
import { type ArchivesT, INFECTION_KEY, rarityColor, useArchivesT } from "./shared";

type SortKey = "name" | "race" | "birthplace" | "height" | "experience" | "infection" | "assimilation" | "stories" | "records" | "links";

/** The pinned first column: opaque so the scrolled cells pass under it, with a hairline on its right edge. */
const STICKY_COL = "sticky left-0 z-1 bg-card shadow-[inset_-1px_0_0_var(--border)]";

const FILTERS_KEY = "archives:dossiers:filters";
const FILTERS_VISIBLE_KEY = "archives:dossiers:filtersVisible";

/** A stored filter set from an older page shape gets the missing fields filled in. */
function parseFilters(raw: string): IDossierFilters | undefined {
    const v = JSON.parse(raw) as Partial<IDossierFilters> | null;
    return v && typeof v === "object" ? { ...EMPTY_DOSSIER_FILTERS, ...v } : undefined;
}

/** Missing values sort last in both directions. */
function compare(a: ArchiveDossier, b: ArchiveDossier, key: SortKey, desc: boolean): number {
    const pick = (d: ArchiveDossier): string | number | undefined => {
        switch (key) {
            case "name":
                return d.name;
            case "race":
                return d.race;
            case "birthplace":
                return d.birthplace;
            case "height":
                return d.heightCm;
            case "experience":
                return d.combatYears;
            case "infection":
                return INFECTION_ORDER.indexOf(d.infectionState);
            case "assimilation":
                return d.assimilationPct;
            case "stories":
                return d.storyCount;
            case "records":
                return d.recordCount;
            case "links":
                return d.linkCount;
        }
    };
    const x = pick(a);
    const y = pick(b);
    if (x === undefined && y === undefined) return 0;
    if (x === undefined) return 1;
    if (y === undefined) return -1;
    const c = typeof x === "number" && typeof y === "number" ? x - y : String(x).localeCompare(String(y));
    return desc ? -c : c;
}

export function DossiersTab({ onOpenFile }: { onOpenFile: (id: string) => void }): React.ReactElement {
    const t = useArchivesT();
    const f = useFormatters();
    const server = useGamedataServer();
    const { data, isLoading, isError } = useQuery(archiveDossiersQueryOptions(server));
    const [query, setQuery] = useState("");
    const deferred = useDeferredValue(query);
    const [filters, setFilters] = useLocalStorageState<IDossierFilters>(FILTERS_KEY, EMPTY_DOSSIER_FILTERS, { parse: parseFilters });
    const [filtersVisible, setFiltersVisible] = useLocalStorageState<boolean>(FILTERS_VISIBLE_KEY, false, {
        parse: (raw) => raw === "1",
        serialize: (v) => (v ? "1" : "0"),
    });
    const [sort, setSort] = useState<{ key: SortKey; desc: boolean }>({ key: "name", desc: false });

    const all = useMemo(() => data?.dossiers ?? [], [data]);
    const options = useMemo(() => buildDossierOptions(all), [all]);
    const infectionCounts = useMemo(() => {
        const m = new Map<string, number>();
        for (const d of all) m.set(d.infectionState, (m.get(d.infectionState) ?? 0) + 1);
        return m;
    }, [all]);
    const setFilter = useCallback(<K extends keyof IDossierFilters>(key: K, value: IDossierFilters[K]) => setFilters((prev) => ({ ...prev, [key]: value })), [setFilters]);
    const clearFilters = useCallback(() => setFilters(EMPTY_DOSSIER_FILTERS), [setFilters]);
    const activeCount = countDossierFilters(filters);
    const chips = useMemo(() => dossierChips(filters, options, setFilter, t), [filters, options, setFilter, t]);

    const rows = useMemo(() => {
        const list = all.filter((d) => matchesDossier(d, filters));
        if (deferred.trim()) {
            return searchAndRank(deferred, list, (d) => ({ name: d.name })).map((s) => s.item);
        }
        return [...list].sort((a, b) => compare(a, b, sort.key, sort.desc) || a.name.localeCompare(b.name));
    }, [all, filters, deferred, sort]);

    if (isError) return <p className="text-muted-foreground text-sm">{t("page.error")}</p>;

    const header = (key: SortKey, label: string, className?: string) => {
        const active = sort.key === key;
        return (
            <TableHead className={className} aria-sort={active ? (sort.desc ? "descending" : "ascending") : "none"}>
                <button
                    type="button"
                    aria-label={t("dossiers.sortBy", { column: label })}
                    className={cn("inline-flex pointer-coarse:min-h-11 cursor-pointer items-center gap-1 font-mono text-[10.5px] uppercase tracking-[0.12em]", active ? "text-primary" : "text-muted-foreground hover:text-foreground")}
                    onClick={() => setSort((s) => ({ key, desc: s.key === key ? !s.desc : false }))}
                >
                    {label}
                    {active ? sort.desc ? <ArrowDownIcon aria-hidden className="size-3" /> : <ArrowUpIcon aria-hidden className="size-3" /> : null}
                </button>
            </TableHead>
        );
    };

    return (
        <div className="relative flex items-start">
            <FilterPanel collapsed={!filtersVisible} onToggle={() => setFiltersVisible((v) => !v)} hasActiveFilters={activeCount > 0} onClearAll={clearFilters} activeFilterCount={activeCount} ariaLabel={t("dossiers.filter.aria")}>
                <DossierFilterFields filters={filters} options={options} infectionCounts={infectionCounts} onChange={setFilter} t={t} />
            </FilterPanel>

            <div className="flex min-w-0 flex-1 flex-col gap-3">
                <div className="flex flex-wrap items-center gap-2.5">
                    <FilterToggleButton visible={filtersVisible} onToggle={() => setFiltersVisible((v) => !v)} activeCount={activeCount} />
                    <SearchField value={query} onChange={setQuery} placeholder={t("dossiers.search")} label={t("dossiers.search")} className="h-9 min-w-56 flex-1 sm:max-w-115" />
                    <span className="ml-auto font-mono text-[11px] text-muted-foreground tabular-nums">{t("dossiers.count", { shown: f.number(rows.length), total: f.number(all.length) })}</span>
                </div>

                <ActiveFilterChips chips={chips} onClearAll={clearFilters} />

                {isLoading ? (
                    <p className="text-muted-foreground text-sm">{t("page.loading")}</p>
                ) : (
                    <div className="overflow-x-auto rounded-[14px] border border-border bg-card">
                        <Table className="min-w-[880px]">
                            <TableHeader>
                                <TableRow>
                                    {/* The name column stays put while the other nine scroll under it: on a phone the table is 880 px in a 343 px box. */}
                                    {header("name", t("dossiers.col.operator"), STICKY_COL)}
                                    {header("race", t("dossiers.col.race"))}
                                    {header("birthplace", t("dossiers.col.birthplace"))}
                                    {header("height", t("dossiers.col.height"))}
                                    {header("experience", t("dossiers.col.experience"))}
                                    {header("infection", t("dossiers.col.infection"))}
                                    {header("assimilation", t("dossiers.col.assimilation"))}
                                    {header("stories", t("dossiers.col.stories"), "text-right")}
                                    {header("records", t("dossiers.col.records"), "text-right")}
                                    {header("links", t("dossiers.col.links"), "text-right")}
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {rows.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={10} className="py-8 text-center text-muted-foreground">
                                            {t("dossiers.empty")}
                                        </TableCell>
                                    </TableRow>
                                ) : (
                                    rows.map((d) => (
                                        <TableRow key={d.id} className="cursor-pointer" onClick={() => onOpenFile(d.id)}>
                                            <TableCell className={STICKY_COL}>
                                                <button type="button" className="flex cursor-pointer items-center gap-2 text-left" onClick={() => onOpenFile(d.id)}>
                                                    <span className="relative size-6 shrink-0 overflow-hidden rounded-sm border-b-2 bg-muted text-[10px] leading-6" style={{ borderBottomColor: rarityColor(d.rarity) }}>
                                                        <OperatorAvatar charId={d.id} name={d.name} server={server} />
                                                    </span>
                                                    <span className="font-medium text-foreground">{d.name}</span>
                                                </button>
                                            </TableCell>
                                            <TableCell className="text-muted-foreground">{d.race ?? ""}</TableCell>
                                            <TableCell className="text-muted-foreground">{d.birthplace ?? ""}</TableCell>
                                            <TableCell className="tabular-nums">{d.height ?? ""}</TableCell>
                                            <TableCell>{d.combatExperience ?? ""}</TableCell>
                                            <TableCell className={cn(d.infectionState === "infected" && "text-destructive-foreground")}>{t(INFECTION_KEY[d.infectionState])}</TableCell>
                                            <TableCell className="tabular-nums">{d.assimilation ?? ""}</TableCell>
                                            <TableCell className="text-right tabular-nums">{d.storyCount > 0 ? f.number(d.storyCount) : ""}</TableCell>
                                            <TableCell className="text-right tabular-nums">{d.recordCount > 0 ? f.number(d.recordCount) : ""}</TableCell>
                                            <TableCell className="text-right tabular-nums">{d.linkCount > 0 ? f.number(d.linkCount) : ""}</TableCell>
                                        </TableRow>
                                    ))
                                )}
                            </TableBody>
                        </Table>
                    </div>
                )}
            </div>
        </div>
    );
}

/** One chip per active pick, named as the filter shows it; its x clears just that pick. */
function dossierChips(f: IDossierFilters, o: ReturnType<typeof buildDossierOptions>, set: <K extends keyof IDossierFilters>(key: K, value: IDossierFilters[K]) => void, t: ArchivesT): IActiveChip[] {
    const chips: IActiveChip[] = [];
    const list = <K extends DossierListKey>(key: K, label: (v: IDossierFilters[K][number]) => string) => {
        const picked = f[key] as IDossierFilters[K][number][];
        for (const v of picked) {
            chips.push({ key: `${key}-${v}`, label: label(v), onRemove: () => set(key, picked.filter((x) => x !== v) as IDossierFilters[K]) });
        }
    };
    const named = (opts: { value: string; label: string }[]) => (v: string) => opts.find((x) => x.value === v)?.label ?? v;
    list("classes", (v) => professionLabel({ profession: v }));
    list("rarities", (v) => `${v}★`);
    list("infection", (v) => t(INFECTION_KEY[v]));
    list("genders", named(o.genders));
    list("races", (v) => v);
    list("birthplaces", (v) => v);
    list("subclasses", named(o.subclasses));
    list("nations", named(o.nations));
    list("factions", named(o.factions));
    if (f.height) chips.push({ key: "height", label: t("dossiers.chip.height", { min: f.height[0], max: f.height[1] }), onRemove: () => set("height", null) });
    if (f.experience) chips.push({ key: "experience", label: t("dossiers.chip.experience", { min: f.experience[0], max: f.experience[1] }), onRemove: () => set("experience", null) });
    return chips;
}
