"use client";

import { ChevronDown, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { cn } from "~/lib/utils";
import type { ApplyWay, DamageType, EnemyLevel } from "~/types/api";
import { APPLY_WAY_COLORS, APPLY_WAY_DISPLAY, DAMAGE_TYPE_COLORS, DAMAGE_TYPE_DISPLAY, ENEMY_LEVEL_COLORS, ENEMY_LEVEL_DISPLAY } from "../constants";
import { createToggle } from "./impl/helpers";

interface EnemyFiltersProps {
    // Available filter options
    enemyLevels: EnemyLevel[];
    damageTypes: DamageType[];
    enemyTypes: ApplyWay[];
    // Selected values
    selectedEnemyLevels: EnemyLevel[];
    selectedDamageTypes: DamageType[];
    selectedEnemyTypes: ApplyWay[];
    // Change handlers
    onEnemyLevelChange: (levels: EnemyLevel[]) => void;
    onDamageTypeChange: (types: DamageType[]) => void;
    onEnemyTypeChange: (types: ApplyWay[]) => void;
    onClearFilters: () => void;
    hideHeader?: boolean;
}

export function EnemyFilters({ enemyLevels, damageTypes, enemyTypes, selectedEnemyLevels, selectedDamageTypes, selectedEnemyTypes, onEnemyLevelChange, onDamageTypeChange, onEnemyTypeChange, onClearFilters, hideHeader = false }: EnemyFiltersProps) {
    const [isAdvancedOpen, setIsAdvancedOpen] = useState(false);

    const toggleEnemyLevel = createToggle(selectedEnemyLevels, onEnemyLevelChange);
    const toggleDamageType = createToggle(selectedDamageTypes, onDamageTypeChange);
    const toggleEnemyType = createToggle(selectedEnemyTypes, onEnemyTypeChange);

    const hasFilters = selectedEnemyLevels.length > 0 || selectedDamageTypes.length > 0 || selectedEnemyTypes.length > 0;

    const hasAdvancedFilters = selectedEnemyTypes.length > 0;
    const advancedFilterCount = selectedEnemyTypes.length;

    return (
        <div className="z-99 min-w-0 overflow-hidden rounded-lg text-foreground">
            <div className="p-3 sm:p-4">
                {/* Header - hidden on mobile when using drawer-style header */}
                {!hideHeader && (
                    <div className="mb-4 flex items-center justify-between">
                        <h3 className="font-semibold text-foreground">Filters</h3>
                        {hasFilters && (
                            <button className="flex items-center gap-1 text-muted-foreground text-sm transition-colors hover:text-foreground" onClick={onClearFilters} type="button">
                                <X className="h-3 w-3" />
                                Clear all
                            </button>
                        )}
                    </div>
                )}

                <div className="space-y-5">
                    {/* Basic Filters: Enemy Level and Damage Type */}
                    <div className="space-y-4">
                        <div className="flex items-center gap-2">
                            <span className="font-medium text-muted-foreground text-xs uppercase tracking-wider">Basic</span>
                            <div className="h-px flex-1 bg-border" />
                        </div>

                        <div className="grid gap-4 sm:gap-6 md:grid-cols-2">
                            {/* Enemy Level Filter */}
                            <div className="space-y-3">
                                <span className="font-medium text-muted-foreground text-sm">Enemy Level</span>
                                <div className="flex flex-wrap gap-2">
                                    {enemyLevels
                                        .filter((level) => level in ENEMY_LEVEL_COLORS)
                                        .map((level) => {
                                            const isSelected = selectedEnemyLevels.includes(level);
                                            const colors = ENEMY_LEVEL_COLORS[level];
                                            return (
                                                <button className={cn("flex items-center justify-center rounded-lg border px-3 py-1.5 font-medium text-sm transition-all", isSelected ? colors.selected : colors.unselected)} key={level} onClick={() => toggleEnemyLevel(level)} type="button">
                                                    {ENEMY_LEVEL_DISPLAY[level] ?? level}
                                                </button>
                                            );
                                        })}
                                </div>
                            </div>

                            {/* Damage Type Filter */}
                            <div className="space-y-3">
                                <span className="font-medium text-muted-foreground text-sm">Damage Type</span>
                                <div className="flex flex-wrap gap-2">
                                    {damageTypes
                                        .filter((type) => type in DAMAGE_TYPE_COLORS)
                                        .map((type) => {
                                            const isSelected = selectedDamageTypes.includes(type);
                                            const colors = DAMAGE_TYPE_COLORS[type];
                                            return (
                                                <button className={cn("flex items-center justify-center rounded-lg border px-3 py-1.5 font-medium text-sm transition-all", isSelected ? colors.selected : colors.unselected)} key={type} onClick={() => toggleDamageType(type)} type="button">
                                                    {DAMAGE_TYPE_DISPLAY[type] ?? type}
                                                </button>
                                            );
                                        })}
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Advanced Filters */}
                    <div className="space-y-4">
                        <button className="flex w-full items-center gap-2 transition-colors hover:opacity-80" onClick={() => setIsAdvancedOpen(!isAdvancedOpen)} type="button">
                            <span className="font-medium text-muted-foreground text-xs uppercase tracking-wider">Advanced</span>
                            {hasAdvancedFilters && <span className="flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 text-2.5 text-primary-foreground">{advancedFilterCount}</span>}
                            <div className="h-px flex-1 bg-border" />
                            <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-200", isAdvancedOpen && "rotate-180")} />
                        </button>

                        <AnimatePresence initial={false}>
                            {isAdvancedOpen && (
                                <motion.div animate={{ height: "auto", opacity: 1 }} className="overflow-hidden" exit={{ height: 0, opacity: 0 }} initial={{ height: 0, opacity: 0 }} transition={{ duration: 0.2, ease: [0.4, 0, 0.2, 1] }}>
                                    <div className="grid gap-4 sm:gap-6 md:grid-cols-2">
                                        {/* Attack Type (ApplyWay) Filter */}
                                        <div className="space-y-3">
                                            <span className="font-medium text-muted-foreground text-sm">Attack Type</span>
                                            <div className="flex flex-wrap gap-2">
                                                {enemyTypes
                                                    .filter((type): type is NonNullable<ApplyWay> => type !== null && type in APPLY_WAY_COLORS)
                                                    .map((type) => {
                                                        const isSelected = selectedEnemyTypes.includes(type);
                                                        const colors = APPLY_WAY_COLORS[type];
                                                        return (
                                                            <button className={cn("flex items-center justify-center rounded-lg border px-3 py-1.5 font-medium text-sm transition-all", isSelected ? colors.selected : colors.unselected)} key={type} onClick={() => toggleEnemyType(type)} type="button">
                                                                {APPLY_WAY_DISPLAY[type] ?? type}
                                                            </button>
                                                        );
                                                    })}
                                            </div>
                                        </div>
                                    </div>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </div>
                </div>
            </div>
        </div>
    );
}
