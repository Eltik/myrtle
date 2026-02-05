"use client";

import { ArrowDownAZ, ArrowUpAZ, ChevronDown, Search, SortAsc, SortDesc, X } from "lucide-react";
import { useTheme } from "next-themes";
import { useMemo, useState } from "react";
import { CLASS_DISPLAY, CLASS_SORT_ORDER, CLASSES, RARITIES, RARITY_COLORS, RARITY_COLORS_LIGHT } from "~/components/collection/operators/list/constants";
import { Button } from "~/components/ui/shadcn/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import { Input } from "~/components/ui/shadcn/input";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { cn } from "~/lib/utils";
import type { DpsOperatorListEntry } from "~/types/api/impl/dps-calculator";

const DISPLAY_TO_CLASS: Record<string, string> = Object.fromEntries(Object.entries(CLASS_DISPLAY).map(([internal, display]) => [display.toUpperCase(), internal]));

function normalizeClass(profession: string): string {
    const upper = profession.toUpperCase();
    if (CLASSES.includes(upper as (typeof CLASSES)[number])) {
        return upper;
    }
    return DISPLAY_TO_CLASS[upper] ?? upper;
}

type SortOption = "name-asc" | "name-desc" | "rarity-desc" | "rarity-asc";

const SORT_OPTIONS: { value: SortOption; label: string; icon: React.ReactNode }[] = [
    { value: "rarity-desc", label: "Rarity (High to Low)", icon: <SortDesc className="h-4 w-4" /> },
    { value: "rarity-asc", label: "Rarity (Low to High)", icon: <SortAsc className="h-4 w-4" /> },
    { value: "name-asc", label: "Name (A-Z)", icon: <ArrowDownAZ className="h-4 w-4" /> },
    { value: "name-desc", label: "Name (Z-A)", icon: <ArrowUpAZ className="h-4 w-4" /> },
];

interface OperatorSelectorProps {
    operators: DpsOperatorListEntry[];
    onSelectOperator: (operator: DpsOperatorListEntry) => void;
}

export function OperatorSelector({ operators, onSelectOperator }: OperatorSelectorProps) {
    const [searchQuery, setSearchQuery] = useState("");
    const [selectedRarities, setSelectedRarities] = useState<Set<number>>(new Set());
    const [selectedClasses, setSelectedClasses] = useState<Set<string>>(new Set());
    const [sortOption, setSortOption] = useState<SortOption>("rarity-desc");
    const { resolvedTheme } = useTheme();

    const rarityColors = resolvedTheme === "light" ? RARITY_COLORS_LIGHT : RARITY_COLORS;

    const toggleRarity = (rarity: number) => {
        setSelectedRarities((prev) => {
            const next = new Set(prev);
            if (next.has(rarity)) {
                next.delete(rarity);
            } else {
                next.add(rarity);
            }
            return next;
        });
    };

    const toggleClass = (cls: string) => {
        setSelectedClasses((prev) => {
            const next = new Set(prev);
            if (next.has(cls)) {
                next.delete(cls);
            } else {
                next.add(cls);
            }
            return next;
        });
    };

    const clearFilters = () => {
        setSelectedRarities(new Set());
        setSelectedClasses(new Set());
        setSearchQuery("");
    };

    const hasActiveFilters = selectedRarities.size > 0 || selectedClasses.size > 0 || searchQuery.trim() !== "";

    const filteredAndSortedOperators = useMemo(() => {
        let result = [...operators];

        if (searchQuery.trim()) {
            const query = searchQuery.toLowerCase();
            result = result.filter((op) => op.name.toLowerCase().includes(query) || op.id.toLowerCase().includes(query) || op.profession.toLowerCase().includes(query) || (CLASS_DISPLAY[op.profession]?.toLowerCase().includes(query) ?? false));
        }

        if (selectedRarities.size > 0) {
            result = result.filter((op) => selectedRarities.has(op.rarity));
        }

        if (selectedClasses.size > 0) {
            result = result.filter((op) => selectedClasses.has(normalizeClass(op.profession)));
        }

        result.sort((a, b) => {
            switch (sortOption) {
                case "name-asc":
                    return a.name.localeCompare(b.name);
                case "name-desc":
                    return b.name.localeCompare(a.name);
                case "rarity-desc": {
                    if (b.rarity !== a.rarity) return b.rarity - a.rarity;
                    const classOrderA = CLASS_SORT_ORDER[a.profession] ?? 99;
                    const classOrderB = CLASS_SORT_ORDER[b.profession] ?? 99;
                    if (classOrderA !== classOrderB) return classOrderA - classOrderB;
                    return a.name.localeCompare(b.name);
                }
                case "rarity-asc": {
                    if (a.rarity !== b.rarity) return a.rarity - b.rarity;
                    const classOrderA = CLASS_SORT_ORDER[a.profession] ?? 99;
                    const classOrderB = CLASS_SORT_ORDER[b.profession] ?? 99;
                    if (classOrderA !== classOrderB) return classOrderA - classOrderB;
                    return a.name.localeCompare(b.name);
                }
                default:
                    return 0;
            }
        });

        return result;
    }, [operators, searchQuery, selectedRarities, selectedClasses, sortOption]);

    const currentSortOption = SORT_OPTIONS.find((opt) => opt.value === sortOption);

    return (
        <div className="space-y-3">
            {/* Search and Sort Row */}
            <div className="flex gap-2">
                <div className="relative flex-1">
                    <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input className="pl-9" onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search operators..." value={searchQuery} />
                </div>
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button className="shrink-0 gap-2" size="default" variant="outline">
                            {currentSortOption?.icon}
                            <span className="hidden sm:inline">{currentSortOption?.label}</span>
                            <ChevronDown className="h-4 w-4 opacity-50" />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                        {SORT_OPTIONS.map((option) => (
                            <DropdownMenuItem className={cn("gap-2", sortOption === option.value && "bg-accent")} key={option.value} onClick={() => setSortOption(option.value)}>
                                {option.icon}
                                {option.label}
                            </DropdownMenuItem>
                        ))}
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>

            {/* Rarity Filters */}
            <div className="flex flex-wrap items-center gap-1.5">
                <span className="mr-1 text-muted-foreground text-xs">Rarity:</span>
                {RARITIES.map((rarity) => {
                    const isSelected = selectedRarities.has(rarity);
                    const color = rarityColors[rarity] ?? "#ffffff";
                    return (
                        <Button
                            className={cn("h-7 min-w-10.5 px-2 font-medium text-xs transition-all duration-150", isSelected ? "ring-2 ring-offset-1 ring-offset-background" : "opacity-60 hover:opacity-100")}
                            key={rarity}
                            onClick={() => toggleRarity(rarity)}
                            size="sm"
                            style={
                                {
                                    borderColor: color,
                                    color: isSelected ? (resolvedTheme === "light" ? "#fff" : "#000") : color,
                                    backgroundColor: isSelected ? color : "transparent",
                                    "--tw-ring-color": color,
                                } as React.CSSProperties
                            }
                            variant="outline"
                        >
                            {rarity}★
                        </Button>
                    );
                })}
            </div>

            {/* Class Filters */}
            <div className="flex flex-wrap items-center gap-1.5">
                <span className="mr-1 text-muted-foreground text-xs">Class:</span>
                {CLASSES.map((cls) => {
                    const isSelected = selectedClasses.has(cls);
                    return (
                        <Button className={cn("h-7 px-2 font-medium text-xs transition-all duration-150", isSelected ? "bg-primary" : "opacity-60 hover:opacity-100")} key={cls} onClick={() => toggleClass(cls)} size="sm" variant="outline">
                            {CLASS_DISPLAY[cls] ?? cls}
                        </Button>
                    );
                })}
            </div>

            {/* Active Filters Summary */}
            {hasActiveFilters && (
                <div className="flex items-center justify-between rounded-md bg-muted/50 px-3 py-1.5">
                    <span className="text-muted-foreground text-xs">
                        Showing {filteredAndSortedOperators.length} of {operators.length} operators
                    </span>
                    <Button className="h-6 gap-1 px-2 text-xs" onClick={clearFilters} size="sm" variant="ghost">
                        <X className="h-3 w-3" />
                        Clear filters
                    </Button>
                </div>
            )}

            {/* Operator List */}
            <ScrollArea className="h-70 rounded-md border border-border">
                <div className="grid grid-cols-1 gap-2 p-3 sm:grid-cols-2 lg:grid-cols-3">
                    {filteredAndSortedOperators.map((operator) => {
                        const rarityColor = rarityColors[operator.rarity] ?? "#ffffff";

                        return (
                            <Button className="h-auto justify-start gap-3 px-3 py-2 text-left transition-all duration-150 hover:scale-[1.02]" key={operator.id} onClick={() => onSelectOperator(operator)} variant="outline">
                                <div className="h-8 w-1 shrink-0 rounded-full" style={{ backgroundColor: rarityColor }} />
                                <div className="flex-1 overflow-hidden">
                                    <div className="truncate font-semibold text-sm">{operator.name}</div>
                                    <div className="flex items-center gap-1.5 text-muted-foreground text-xs">
                                        <span style={{ color: rarityColor }}>{"★".repeat(operator.rarity)}</span>
                                        <span>•</span>
                                        <span>{CLASS_DISPLAY[operator.profession] ?? operator.profession}</span>
                                    </div>
                                </div>
                            </Button>
                        );
                    })}
                </div>

                {filteredAndSortedOperators.length === 0 && <div className="flex h-50 items-center justify-center text-muted-foreground text-sm">No operators found</div>}
            </ScrollArea>
        </div>
    );
}
