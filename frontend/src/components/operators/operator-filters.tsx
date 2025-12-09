"use client";

import { ArrowDown, ArrowUp, X } from "lucide-react";
import { cn } from "~/lib/utils";

interface OperatorFiltersProps {
    isOpen: boolean;
    classes: string[];
    rarities: number[];
    elements: string[];
    selectedClasses: string[];
    selectedRarities: number[];
    selectedElements: string[];
    sortBy: "name" | "rarity";
    sortOrder: "asc" | "desc";
    onClassChange: (classes: string[]) => void;
    onRarityChange: (rarities: number[]) => void;
    onElementChange: (elements: string[]) => void;
    onSortByChange: (sortBy: "name" | "rarity") => void;
    onSortOrderChange: (order: "asc" | "desc") => void;
    onClearFilters: () => void;
}

const CLASS_ICONS: Record<string, string> = {
    Guard: "⚔️",
    Sniper: "🎯",
    Defender: "🛡️",
    Medic: "💚",
    Supporter: "🔮",
    Caster: "✨",
    Specialist: "🗡️",
    Vanguard: "⚡",
};

export function OperatorFilters({ isOpen, classes, rarities, elements, selectedClasses, selectedRarities, selectedElements, sortBy, sortOrder, onClassChange, onRarityChange, onElementChange, onSortByChange, onSortOrderChange, onClearFilters }: OperatorFiltersProps) {
    const toggleClass = (cls: string) => {
        if (selectedClasses.includes(cls)) {
            onClassChange(selectedClasses.filter((c) => c !== cls));
        } else {
            onClassChange([...selectedClasses, cls]);
        }
    };

    const toggleRarity = (rarity: number) => {
        if (selectedRarities.includes(rarity)) {
            onRarityChange(selectedRarities.filter((r) => r !== rarity));
        } else {
            onRarityChange([...selectedRarities, rarity]);
        }
    };

    const toggleElement = (element: string) => {
        if (selectedElements.includes(element)) {
            onElementChange(selectedElements.filter((e) => e !== element));
        } else {
            onElementChange([...selectedElements, element]);
        }
    };

    const hasFilters = selectedClasses.length > 0 || selectedRarities.length > 0 || selectedElements.length > 0;

    if (!isOpen) return null;

    return (
        <div className="overflow-hidden rounded-lg border border-border bg-card/50 backdrop-blur-sm">
            <div className="p-4">
                <div className="mb-4 flex items-center justify-between">
                    <h3 className="font-semibold text-foreground">Filters & Sorting</h3>
                    {hasFilters && (
                        <button className="flex items-center gap-1 text-muted-foreground text-sm transition-colors hover:text-foreground" onClick={onClearFilters}>
                            <X className="h-3 w-3" />
                            Clear all
                        </button>
                    )}
                </div>

                <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
                    {/* Class Filter */}
                    <div className="space-y-3">
                        <label className="font-medium text-muted-foreground text-sm">Class</label>
                        <div className="flex flex-wrap gap-2">
                            {classes.map((cls) => (
                                <button
                                    className={cn("flex items-center gap-1.5 rounded-lg border px-2.5 py-1.5 text-sm transition-all", selectedClasses.includes(cls) ? "border-primary bg-primary/20 text-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:border-primary/50 hover:text-foreground")}
                                    key={cls}
                                    onClick={() => toggleClass(cls)}
                                >
                                    <span>{CLASS_ICONS[cls]}</span>
                                    <span>{cls}</span>
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Rarity Filter */}
                    <div className="space-y-3">
                        <label className="font-medium text-muted-foreground text-sm">Rarity</label>
                        <div className="flex flex-wrap gap-2">
                            {rarities.map((rarity) => (
                                <button
                                    className={cn(
                                        "flex items-center justify-center rounded-lg border px-3 py-1.5 font-medium text-sm transition-all",
                                        selectedRarities.includes(rarity) ? "border-amber-500 bg-amber-500/20 text-amber-400" : "border-border bg-secondary/50 text-muted-foreground hover:border-amber-500/50 hover:text-amber-400",
                                    )}
                                    key={rarity}
                                    onClick={() => toggleRarity(rarity)}
                                >
                                    {rarity}★
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Element Filter */}
                    <div className="space-y-3">
                        <label className="font-medium text-muted-foreground text-sm">Damage Type</label>
                        <div className="flex flex-wrap gap-2">
                            {elements.map((element) => (
                                <button
                                    className={cn(
                                        "rounded-lg border px-3 py-1.5 text-sm transition-all",
                                        selectedElements.includes(element)
                                            ? element === "Arts"
                                                ? "border-purple-500 bg-purple-500/20 text-purple-400"
                                                : element === "Physical"
                                                  ? "border-orange-500 bg-orange-500/20 text-orange-400"
                                                  : "border-green-500 bg-green-500/20 text-green-400"
                                            : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground",
                                    )}
                                    key={element}
                                    onClick={() => toggleElement(element)}
                                >
                                    {element}
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Sorting */}
                    <div className="space-y-3">
                        <label className="font-medium text-muted-foreground text-sm">Sort By</label>
                        <div className="flex gap-2">
                            <button className={cn("flex-1 rounded-lg border px-3 py-1.5 text-sm transition-all", sortBy === "rarity" ? "border-primary bg-primary/20 text-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")} onClick={() => onSortByChange("rarity")}>
                                Rarity
                            </button>
                            <button className={cn("flex-1 rounded-lg border px-3 py-1.5 text-sm transition-all", sortBy === "name" ? "border-primary bg-primary/20 text-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")} onClick={() => onSortByChange("name")}>
                                Name
                            </button>
                            <button className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:text-foreground" onClick={() => onSortOrderChange(sortOrder === "asc" ? "desc" : "asc")}>
                                {sortOrder === "asc" ? <ArrowUp className="h-4 w-4" /> : <ArrowDown className="h-4 w-4" />}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
