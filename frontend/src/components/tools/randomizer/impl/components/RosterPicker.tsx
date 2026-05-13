import { Check, ChevronDown, FilterX, Info, RotateCcw, Search, X } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { cn, getAvatarById, RARITY_LABELS } from "#/lib/utils";
import type { OperatorRarity } from "#/types/operators";
import { ALL_CLASSES, ALL_RARITIES, CLASS_LABEL } from "../constants";
import type { IRandomizerOperator, IRandomizerSettings, IRosterIndex } from "../types";

interface IRosterPickerProps {
    /** Complete operator list - needed so "select all visible" doesn't drop hidden operators from the explicit set. */
    allOperators: IRandomizerOperator[];
    /** Operators that pass the operator-tab filters (class / rarity / hide unplayable). */
    visibleOperators: IRandomizerOperator[];
    selected: Set<string>;
    /** True once the user has explicitly modified the roster. False = the default "all" state. */
    isExplicit: boolean;
    onChange: (next: Set<string>) => void;
    onReset: () => void;
    rosterIndex: IRosterIndex;
    hasProfile: boolean;
    settings: IRandomizerSettings;
}

const RARITY_SORT: Record<OperatorRarity, number> = { 6: 0, 5: 1, 4: 2, 3: 3, 2: 4, 1: 5 };

export function RosterPicker({ allOperators, visibleOperators, selected, isExplicit, onChange, onReset, rosterIndex, hasProfile, settings }: IRosterPickerProps): React.ReactElement {
    const [query, setQuery] = React.useState("");

    const trimmedQuery = query.trim().toLowerCase();
    const searchMatches = React.useCallback(
        (op: IRandomizerOperator) => {
            if (!trimmedQuery) return true;
            return op.name.toLowerCase().includes(trimmedQuery);
        },
        [trimmedQuery],
    );

    const visibleSorted = React.useMemo(() => {
        const list = visibleOperators.filter(searchMatches);
        list.sort((a, b) => {
            const r = RARITY_SORT[a.rarity] - RARITY_SORT[b.rarity];
            if (r !== 0) return r;
            return a.name.localeCompare(b.name);
        });
        return list;
    }, [visibleOperators, searchMatches]);

    const groupedByRarity = React.useMemo(() => {
        const groups = new Map<OperatorRarity, IRandomizerOperator[]>();
        for (const op of visibleSorted) {
            const arr = groups.get(op.rarity) ?? [];
            arr.push(op);
            groups.set(op.rarity, arr);
        }
        return Array.from(groups.entries()).sort(([a], [b]) => RARITY_SORT[a] - RARITY_SORT[b]);
    }, [visibleSorted]);

    const visibleIdSet = React.useMemo(() => new Set(visibleOperators.map((op) => op.id)), [visibleOperators]);
    const visibleSelectedCount = React.useMemo(() => {
        let n = 0;
        for (const id of visibleIdSet) if (selected.has(id)) n++;
        return n;
    }, [visibleIdSet, selected]);

    const allVisibleSelected = visibleOperators.length > 0 && visibleSelectedCount === visibleOperators.length;
    const noVisibleSelected = visibleSelectedCount === 0;

    const onToggle = React.useCallback(
        (id: string) => {
            const next = new Set(selected);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            onChange(next);
        },
        [selected, onChange],
    );

    const onSelectAllVisible = React.useCallback(() => {
        const next = new Set(selected);
        for (const op of visibleOperators) next.add(op.id);
        onChange(next);
    }, [selected, visibleOperators, onChange]);

    const onDeselectAllVisible = React.useCallback(() => {
        const next = new Set(selected);
        for (const op of visibleOperators) next.delete(op.id);
        onChange(next);
    }, [selected, visibleOperators, onChange]);

    const onSyncProfile = React.useCallback(() => onChange(new Set(rosterIndex.owned)), [rosterIndex.owned, onChange]);

    const filteredClassCount = ALL_CLASSES.length - settings.allowedClasses.length;
    const filteredRarityCount = ALL_RARITIES.length - settings.allowedRarities.length;
    const hasActiveFilters = filteredClassCount > 0 || filteredRarityCount > 0 || settings.onlyOwnedOperators || settings.onlyE2Operators;

    const hiddenCount = allOperators.length - visibleOperators.length;

    const [collapsed, setCollapsed] = React.useState<Set<OperatorRarity>>(() => new Set<OperatorRarity>([4, 3, 2, 1]));
    const toggleCollapsed = React.useCallback((rarity: OperatorRarity) => {
        setCollapsed((prev) => {
            const next = new Set(prev);
            if (next.has(rarity)) next.delete(rarity);
            else next.add(rarity);
            return next;
        });
    }, []);

    return (
        <div className="flex h-full flex-col gap-3">
            {hasActiveFilters && <ActiveFiltersHint settings={settings} hiddenCount={hiddenCount} />}

            <div className="flex items-center gap-2">
                <div className="relative min-w-0 flex-1">
                    <Search aria-hidden="true" className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
                    <Input value={query} onChange={(e) => setQuery(e.currentTarget.value)} placeholder="Search operators…" size="sm" className="rounded-md border-input pl-7" />
                    {query && (
                        <button type="button" onClick={() => setQuery("")} className="absolute right-1.5 top-1/2 inline-flex size-5 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground" aria-label="Clear search">
                            <X className="size-3" />
                        </button>
                    )}
                </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] uppercase tracking-[0.16em] text-muted-foreground">
                <div className="flex flex-col gap-0.5">
                    <span>
                        <span className="text-foreground">{visibleSelectedCount}</span> / {visibleOperators.length} {hasActiveFilters ? "in view" : "selected"}
                    </span>
                    {hasActiveFilters && <span className="text-[10.5px] normal-case tracking-normal text-muted-foreground/70">{selected.size} total in roster</span>}
                </div>
                <div className="flex flex-wrap gap-1.5">
                    <Button onClick={onSelectAllVisible} size="xs" variant="ghost" disabled={allVisibleSelected || visibleOperators.length === 0}>
                        All
                    </Button>
                    <Button onClick={onDeselectAllVisible} size="xs" variant="ghost" disabled={noVisibleSelected}>
                        None
                    </Button>
                    {isExplicit && (
                        <Button onClick={onReset} size="xs" variant="ghost" title="Restore the default state (every operator selected).">
                            <RotateCcw aria-hidden="true" className="size-3" />
                            Reset
                        </Button>
                    )}
                    {hasProfile && rosterIndex.owned.size > 0 && (
                        <Button onClick={onSyncProfile} size="xs" variant="outline">
                            Sync profile ({rosterIndex.owned.size})
                        </Button>
                    )}
                </div>
            </div>

            <div className="-mx-1 flex flex-1 flex-col gap-3 overflow-y-auto px-1 pb-2">
                {visibleSorted.length === 0 ? (
                    <EmptyRoster query={query} hasActiveFilters={hasActiveFilters} onReset={onReset} isExplicit={isExplicit} />
                ) : (
                    groupedByRarity.map(([rarity, ops]) => {
                        const groupSelected = ops.reduce((n, op) => (selected.has(op.id) ? n + 1 : n), 0);
                        const allGroupSelected = groupSelected === ops.length;
                        const isCollapsed = collapsed.has(rarity);
                        const onToggleGroup = () => {
                            const next = new Set(selected);
                            if (allGroupSelected) {
                                for (const op of ops) next.delete(op.id);
                            } else {
                                for (const op of ops) next.add(op.id);
                            }
                            onChange(next);
                        };
                        return (
                            <div key={rarity} className="flex flex-col gap-1.5">
                                <div className="flex items-stretch gap-1.5">
                                    <button
                                        type="button"
                                        onClick={() => toggleCollapsed(rarity)}
                                        aria-expanded={!isCollapsed}
                                        className="group flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                                    >
                                        <ChevronDown aria-hidden="true" className={cn("size-3.5 shrink-0 text-muted-foreground transition-transform", isCollapsed && "-rotate-90")} />
                                        <span className="font-mono text-[11px] uppercase tracking-[0.18em]" style={{ color: `var(--rarity-${rarity})` }}>
                                            {RARITY_LABELS[rarity]}
                                        </span>
                                        <span className="font-mono text-[10.5px] uppercase tracking-[0.16em] text-muted-foreground/70">
                                            {groupSelected} / {ops.length}
                                        </span>
                                    </button>
                                    <Button type="button" onClick={onToggleGroup} size="xs" variant="outline" className="shrink-0 px-2.5 font-mono text-[10.5px] uppercase tracking-[0.16em]" aria-label={allGroupSelected ? `Clear all ${RARITY_LABELS[rarity]} operators` : `Select all ${RARITY_LABELS[rarity]} operators`}>
                                        {allGroupSelected ? "Clear" : "All"}
                                    </Button>
                                </div>
                                {!isCollapsed && (
                                    <div className="grid grid-cols-4 gap-1.5 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-5 xl:grid-cols-6">
                                        {ops.map((op) => (
                                            <RosterChip key={op.id} op={op} selected={selected.has(op.id)} onToggle={() => onToggle(op.id)} />
                                        ))}
                                    </div>
                                )}
                            </div>
                        );
                    })
                )}
            </div>
        </div>
    );
}

function ActiveFiltersHint({ settings, hiddenCount }: { settings: IRandomizerSettings; hiddenCount: number }) {
    const excludedClasses = ALL_CLASSES.filter((c) => !settings.allowedClasses.includes(c));
    const excludedRarities = ALL_RARITIES.filter((r) => !settings.allowedRarities.includes(r));

    const parts: string[] = [];
    if (excludedRarities.length > 0) parts.push(excludedRarities.map((r) => `${r}★`).join(", "));
    if (excludedClasses.length > 0) parts.push(excludedClasses.map((c) => CLASS_LABEL[c]).join(", "));
    if (settings.onlyE2Operators) parts.push("E2 only");
    else if (settings.onlyOwnedOperators) parts.push("Owned only");

    return (
        <div className="flex items-start gap-2 rounded-md border border-border/60 bg-accent/30 px-3 py-2 text-[11.5px] leading-snug text-muted-foreground">
            <Info aria-hidden="true" className="mt-0.5 size-3.5 shrink-0 text-muted-foreground/80" />
            <p className="min-w-0">
                Showing operators that match your Operators filters. <span className="text-foreground">{hiddenCount}</span> hidden by {parts.length > 0 ? <span className="text-foreground">{parts.join(" · ")}</span> : <span>active filters</span>}.
            </p>
        </div>
    );
}

function EmptyRoster({ query, hasActiveFilters, onReset, isExplicit }: { query: string; hasActiveFilters: boolean; onReset: () => void; isExplicit: boolean }) {
    return (
        <div className="flex flex-1 flex-col items-center justify-center gap-2 py-10 text-center">
            <FilterX aria-hidden="true" className="size-6 text-muted-foreground/70" />
            <p className="text-sm text-foreground">No operators to show</p>
            <p className="max-w-xs text-[12px] text-muted-foreground">{query ? "No operators match your search." : hasActiveFilters ? "All operators are filtered out by your Operators tab settings." : "There are no operators available."}</p>
            {!query && hasActiveFilters && isExplicit && (
                <Button onClick={onReset} size="xs" variant="outline" className="mt-1">
                    <RotateCcw aria-hidden="true" className="size-3" />
                    Reset roster
                </Button>
            )}
        </div>
    );
}

function RosterChip({ op, selected, onToggle }: { op: IRandomizerOperator; selected: boolean; onToggle: () => void }) {
    return (
        <button
            type="button"
            onClick={onToggle}
            aria-pressed={selected}
            className={cn("group relative aspect-square overflow-hidden rounded-md border bg-secondary/40 outline-none transition-all duration-150", "focus-visible:ring-2 focus-visible:ring-ring", selected ? "border-primary/80 shadow-sm shadow-primary/20" : "border-border/50 opacity-60 hover:opacity-95")}
            title={op.name}
        >
            <img
                src={getAvatarById(op.id)}
                alt=""
                aria-hidden="true"
                loading="lazy"
                decoding="async"
                className="absolute inset-0 h-full w-full object-cover"
                onError={(e) => {
                    e.currentTarget.style.display = "none";
                }}
            />
            <div className="absolute inset-x-0 bottom-0 h-1" style={{ background: `var(--rarity-${op.rarity})` }} />
            {selected && (
                <div className="absolute right-0.5 top-0.5 inline-flex size-4 items-center justify-center rounded-full bg-primary text-primary-foreground">
                    <Check className="size-2.5" />
                </div>
            )}
            <span className="sr-only">{op.name}</span>
        </button>
    );
}
