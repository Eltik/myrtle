"use client";

import { ChevronDown, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { cn } from "~/lib/utils";
import type { EnemyLevel } from "~/types/api/impl/enemy";
import { DAMAGE_TYPE_COLORS, DAMAGE_TYPE_DISPLAY, DAMAGE_TYPES, IMMUNITY_DISPLAY, IMMUNITY_TYPES, LEVEL_COLORS, LEVEL_DISPLAY } from "./constants";
import { FilterDropdown } from "./filter-dropdown";
import type { ImmunityType, RaceOption, StatRange } from "./types";

interface EnemyFiltersProps {
    levels: readonly EnemyLevel[];
    selectedLevels: string[];
    selectedDamageTypes: string[];
    selectedRaces: string[];
    selectedImmunities: ImmunityType[];
    statFilters: {
        hp: StatRange;
        atk: StatRange;
        def: StatRange;
        res: StatRange;
    };
    raceOptions: RaceOption[];
    onLevelChange: (levels: string[]) => void;
    onDamageTypeChange: (types: string[]) => void;
    onRaceChange: (races: string[]) => void;
    onImmunityChange: (immunities: ImmunityType[]) => void;
    onStatFilterChange: (stat: "hp" | "atk" | "def" | "res", range: StatRange) => void;
    onClearFilters: () => void;
    hideHeader?: boolean;
}

export function EnemyFilters({ levels, selectedLevels, selectedDamageTypes, selectedRaces, selectedImmunities, statFilters, raceOptions, onLevelChange, onDamageTypeChange, onRaceChange, onImmunityChange, onStatFilterChange, onClearFilters, hideHeader = false }: EnemyFiltersProps) {
    const [isAdvancedOpen, setIsAdvancedOpen] = useState(false);

    const toggleLevel = (level: string) => {
        if (selectedLevels.includes(level)) {
            onLevelChange(selectedLevels.filter((l) => l !== level));
        } else {
            onLevelChange([...selectedLevels, level]);
        }
    };

    const toggleDamageType = (type: string) => {
        if (selectedDamageTypes.includes(type)) {
            onDamageTypeChange(selectedDamageTypes.filter((t) => t !== type));
        } else {
            onDamageTypeChange([...selectedDamageTypes, type]);
        }
    };

    const toggleRace = (race: string) => {
        if (selectedRaces.includes(race)) {
            onRaceChange(selectedRaces.filter((r) => r !== race));
        } else {
            onRaceChange([...selectedRaces, race]);
        }
    };

    const toggleImmunity = (immunity: ImmunityType) => {
        if (selectedImmunities.includes(immunity)) {
            onImmunityChange(selectedImmunities.filter((i) => i !== immunity));
        } else {
            onImmunityChange([...selectedImmunities, immunity]);
        }
    };

    const hasFilters = selectedLevels.length > 0 || selectedDamageTypes.length > 0 || selectedRaces.length > 0 || selectedImmunities.length > 0 || hasStatFilters(statFilters);
    const hasAdvancedFilters = selectedImmunities.length > 0 || hasStatFilters(statFilters);
    const advancedFilterCount = selectedImmunities.length + countStatFilters(statFilters);

    return (
        <div className="z-99 min-w-0 overflow-hidden rounded-lg text-foreground">
            <div className="p-3 sm:p-4">
                {/* Header */}
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
                    {/* Basic Filters */}
                    <div className="space-y-4">
                        <div className="flex items-center gap-2">
                            <span className="font-medium text-muted-foreground text-xs uppercase tracking-wider">Basic</span>
                            <div className="h-px flex-1 bg-border" />
                        </div>

                        <div className="grid gap-4 sm:gap-6 md:grid-cols-2">
                            {/* Level Filter */}
                            <div className="space-y-3">
                                <span className="font-medium text-muted-foreground text-sm">Enemy Level</span>
                                <div className="flex flex-wrap gap-2">
                                    {levels.map((level) => {
                                        const colors = LEVEL_COLORS[level];
                                        const isSelected = selectedLevels.includes(level);
                                        return (
                                            <Tooltip key={level}>
                                                <TooltipTrigger asChild>
                                                    <button
                                                        className={cn("flex items-center justify-center rounded-lg border px-3 py-1.5 font-medium text-sm transition-all", isSelected ? cn("border-primary", colors.bg, colors.text) : "border-border bg-secondary/50 text-muted-foreground hover:border-primary/50")}
                                                        onClick={() => toggleLevel(level)}
                                                        type="button"
                                                    >
                                                        {LEVEL_DISPLAY[level]}
                                                    </button>
                                                </TooltipTrigger>
                                                <TooltipContent>
                                                    <p>{level === "NORMAL" ? "Common enemies" : level === "ELITE" ? "Stronger variants" : "Stage bosses"}</p>
                                                </TooltipContent>
                                            </Tooltip>
                                        );
                                    })}
                                </div>
                            </div>

                            {/* Damage Type Filter */}
                            <div className="space-y-3">
                                <span className="font-medium text-muted-foreground text-sm">Damage Type</span>
                                <div className="flex flex-wrap gap-2">
                                    {DAMAGE_TYPES.map((type) => {
                                        const colors = DAMAGE_TYPE_COLORS[type];
                                        const isSelected = selectedDamageTypes.includes(type);
                                        return (
                                            <button
                                                className={cn("flex items-center justify-center rounded-lg border px-3 py-1.5 font-medium text-sm transition-all", isSelected ? cn("border-primary", colors.bg, colors.text) : "border-border bg-secondary/50 text-muted-foreground hover:border-primary/50")}
                                                key={type}
                                                onClick={() => toggleDamageType(type)}
                                                type="button"
                                            >
                                                {DAMAGE_TYPE_DISPLAY[type]}
                                            </button>
                                        );
                                    })}
                                </div>
                            </div>

                            {/* Race Filter - Dropdown (moved from advanced) */}
                            <div className="md:col-span-2">
                                <FilterDropdown
                                    formatOption={(id) => raceOptions.find((r) => r.id === id)?.name ?? id}
                                    label="Race"
                                    onRemove={(r) => onRaceChange(selectedRaces.filter((x) => x !== r))}
                                    onToggle={toggleRace}
                                    options={raceOptions.map((r) => r.id)}
                                    placeholder="Select race"
                                    selectedOptions={selectedRaces}
                                />
                            </div>
                        </div>
                    </div>

                    {/* Advanced Filters */}
                    <div className="space-y-4">
                        <button className="flex w-full items-center gap-2 transition-colors hover:opacity-80" onClick={() => setIsAdvancedOpen(!isAdvancedOpen)} type="button">
                            <span className="font-medium text-muted-foreground text-xs uppercase tracking-wider">Advanced</span>
                            {hasAdvancedFilters && <span className="flex h-4 min-w-4 items-center justify-center rounded-full bg-primary px-1 text-[10px] text-primary-foreground">{advancedFilterCount}</span>}
                            <div className="h-px flex-1 bg-border" />
                            <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-200", isAdvancedOpen && "rotate-180")} />
                        </button>

                        <AnimatePresence initial={false}>
                            {isAdvancedOpen && (
                                <motion.div animate={{ height: "auto", opacity: 1 }} className="overflow-hidden" exit={{ height: 0, opacity: 0 }} initial={{ height: 0, opacity: 0 }} transition={{ duration: 0.2, ease: [0.4, 0, 0.2, 1] }}>
                                    <div className="space-y-6">
                                        {/* Immunity Filter */}
                                        <div className="space-y-3">
                                            <span className="font-medium text-muted-foreground text-sm">Immunities</span>
                                            <p className="text-muted-foreground/70 text-xs">Filter enemies that are immune to these effects</p>
                                            <div className="flex flex-wrap gap-2">
                                                {IMMUNITY_TYPES.map((immunity) => {
                                                    const isSelected = selectedImmunities.includes(immunity);
                                                    return (
                                                        <button
                                                            className={cn(
                                                                "flex items-center gap-1.5 rounded-lg border px-3 py-1.5 font-medium text-sm transition-all",
                                                                isSelected ? "border-emerald-500/50 bg-emerald-500/15 text-emerald-400" : "border-border bg-secondary/50 text-muted-foreground hover:border-primary/50",
                                                            )}
                                                            key={immunity}
                                                            onClick={() => toggleImmunity(immunity)}
                                                            type="button"
                                                        >
                                                            <span className={cn("h-2 w-2 rounded-full", isSelected ? "bg-emerald-400" : "bg-muted-foreground/30")} />
                                                            {IMMUNITY_DISPLAY[immunity]}
                                                        </button>
                                                    );
                                                })}
                                            </div>
                                        </div>

                                        {/* Stat Range Filters */}
                                        <div className="space-y-3">
                                            <span className="font-medium text-muted-foreground text-sm">Combat Stats</span>
                                            <p className="text-muted-foreground/70 text-xs">Filter by stat ranges (uses max level stats)</p>
                                            <div className="grid gap-4 sm:grid-cols-2">
                                                <StatRangeInput label="HP" onChange={(range) => onStatFilterChange("hp", range)} value={statFilters.hp} />
                                                <StatRangeInput label="ATK" onChange={(range) => onStatFilterChange("atk", range)} value={statFilters.atk} />
                                                <StatRangeInput label="DEF" onChange={(range) => onStatFilterChange("def", range)} value={statFilters.def} />
                                                <StatRangeInput label="RES" onChange={(range) => onStatFilterChange("res", range)} suffix="%" />
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

// Helper component for stat range input
function StatRangeInput({ label, value, onChange, suffix }: { label: string; value?: StatRange; onChange: (range: StatRange) => void; suffix?: string }) {
    const handleMinChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.value === "" ? null : Number(e.target.value);
        onChange({ min: val, max: value?.max ?? null });
    };

    const handleMaxChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.value === "" ? null : Number(e.target.value);
        onChange({ min: value?.min ?? null, max: val });
    };

    return (
        <div className="space-y-2">
            <span className="font-medium text-muted-foreground text-xs">{label}</span>
            <div className="flex items-center gap-2">
                <div className="relative flex-1">
                    <input className="h-8 w-full rounded-md border border-border bg-secondary/50 px-2 text-sm placeholder:text-muted-foreground/50 focus:border-primary focus:outline-none" min={0} onChange={handleMinChange} placeholder="Min" type="number" value={value?.min ?? ""} />
                    {suffix && <span className="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground text-xs">{suffix}</span>}
                </div>
                <span className="text-muted-foreground text-xs">to</span>
                <div className="relative flex-1">
                    <input className="h-8 w-full rounded-md border border-border bg-secondary/50 px-2 text-sm placeholder:text-muted-foreground/50 focus:border-primary focus:outline-none" min={0} onChange={handleMaxChange} placeholder="Max" type="number" value={value?.max ?? ""} />
                    {suffix && <span className="absolute top-1/2 right-2 -translate-y-1/2 text-muted-foreground text-xs">{suffix}</span>}
                </div>
            </div>
        </div>
    );
}

// Helper functions
function hasStatFilters(statFilters: { hp: StatRange; atk: StatRange; def: StatRange; res: StatRange }): boolean {
    return statFilters.hp.min !== null || statFilters.hp.max !== null || statFilters.atk.min !== null || statFilters.atk.max !== null || statFilters.def.min !== null || statFilters.def.max !== null || statFilters.res.min !== null || statFilters.res.max !== null;
}

function countStatFilters(statFilters: { hp: StatRange; atk: StatRange; def: StatRange; res: StatRange }): number {
    let count = 0;
    if (statFilters.hp.min !== null || statFilters.hp.max !== null) count++;
    if (statFilters.atk.min !== null || statFilters.atk.max !== null) count++;
    if (statFilters.def.min !== null || statFilters.def.max !== null) count++;
    if (statFilters.res.min !== null || statFilters.res.max !== null) count++;
    return count;
}
