"use client";

import { ChevronDown, X } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import Image from "next/image";
import { useState } from "react";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { capitalize, cn, formatNationId, formatSubProfession } from "~/lib/utils";
import { CLASS_DISPLAY, CLASS_ICON, NATION_DISPLAY } from "../constants";
import { FilterDropdown } from "../ui/impl/filter-dropdown";
import { createToggle } from "./impl/helpers";

interface OperatorFiltersProps {
    classes: string[];
    subclasses: string[];
    rarities: number[];
    genders: string[];
    birthPlaces: string[];
    nations: string[];
    factions: string[];
    races: string[];
    artists: string[];
    selectedClasses: string[];
    selectedSubclasses: string[];
    selectedRarities: number[];
    selectedGenders: string[];
    selectedBirthPlaces: string[];
    selectedNations: string[];
    selectedFactions: string[];
    selectedRaces: string[];
    selectedArtists: string[];
    onClassChange: (classes: string[]) => void;
    onSubclassChange: (subclasses: string[]) => void;
    onRarityChange: (rarities: number[]) => void;
    onGenderChange: (genders: string[]) => void;
    onBirthPlaceChange: (birthPlaces: string[]) => void;
    onNationChange: (nations: string[]) => void;
    onFactionChange: (factions: string[]) => void;
    onRaceChange: (races: string[]) => void;
    onArtistChange: (artists: string[]) => void;
    onClearFilters: () => void;
}

export function OperatorFilters({
    classes,
    subclasses,
    rarities,
    genders,
    birthPlaces,
    nations,
    factions,
    races,
    artists,
    selectedClasses,
    selectedSubclasses,
    selectedRarities,
    selectedGenders,
    selectedBirthPlaces,
    selectedNations,
    selectedFactions,
    selectedRaces,
    selectedArtists,
    onClassChange,
    onSubclassChange,
    onRarityChange,
    onGenderChange,
    onBirthPlaceChange,
    onNationChange,
    onFactionChange,
    onRaceChange,
    onArtistChange,
    onClearFilters,
}: OperatorFiltersProps) {
    const [isBasicOpen, setIsBasicOpen] = useState(true);
    const [isAdvancedOpen, setIsAdvancedOpen] = useState(false);

    const toggleClass = createToggle(selectedClasses, onClassChange);
    const toggleRarity = createToggle(selectedRarities, onRarityChange);
    const toggleGender = createToggle(selectedGenders, onGenderChange);

    const hasFilters = selectedClasses.length > 0 || selectedSubclasses.length > 0 || selectedRarities.length > 0 || selectedGenders.length > 0 || selectedBirthPlaces.length > 0 || selectedNations.length > 0 || selectedFactions.length > 0 || selectedRaces.length > 0 || selectedArtists.length > 0;

    const hasBasicFilters = selectedClasses.length > 0 || selectedRarities.length > 0;
    const basicFilterCount = selectedClasses.length + selectedRarities.length;

    const hasAdvancedFilters = selectedSubclasses.length > 0 || selectedGenders.length > 0 || selectedBirthPlaces.length > 0 || selectedNations.length > 0 || selectedFactions.length > 0 || selectedRaces.length > 0 || selectedArtists.length > 0;
    const advancedFilterCount = selectedSubclasses.length + selectedGenders.length + selectedBirthPlaces.length + selectedNations.length + selectedFactions.length + selectedRaces.length + selectedArtists.length;

    return (
        <div className="z-99 min-w-0 overflow-hidden rounded-lg text-foreground">
            <div className="p-3 sm:p-4">
                {/* Header */}
                <div className="mb-4 flex items-center justify-between">
                    <h3 className="font-semibold text-foreground">Filters</h3>
                    {hasFilters && (
                        <button className="flex items-center gap-1 text-muted-foreground text-sm transition-colors hover:text-foreground" onClick={onClearFilters} type="button">
                            <X className="h-3 w-3" />
                            Clear all
                        </button>
                    )}
                </div>

                <div className="space-y-3">
                    {/* Basic Filters: Class and Rarity */}
                    <div className="rounded-lg border border-border bg-secondary/30">
                        <button
                            className="flex w-full items-center justify-between px-3 py-2.5 text-left transition-colors hover:bg-secondary/50"
                            onClick={() => setIsBasicOpen(!isBasicOpen)}
                            type="button"
                        >
                            <div className="flex items-center gap-2">
                                <span className="font-medium text-foreground text-sm">Basic</span>
                                {hasBasicFilters && !isBasicOpen && (
                                    <span className="flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 text-primary-foreground text-xs">{basicFilterCount}</span>
                                )}
                            </div>
                            <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-200", isBasicOpen && "rotate-180")} />
                        </button>

                        <AnimatePresence initial={false}>
                            {isBasicOpen && (
                                <motion.div
                                    animate={{ height: "auto", opacity: 1 }}
                                    exit={{ height: 0, opacity: 0 }}
                                    initial={{ height: 0, opacity: 0 }}
                                    transition={{ duration: 0.2, ease: [0.4, 0, 0.2, 1] }}
                                >
                                    <div className="border-border border-t px-3 py-3">
                                        <div className="grid gap-4 sm:gap-6 md:grid-cols-2">
                                            {/* Class Filter - Icon buttons */}
                                            <div className="space-y-3">
                                                <span className="font-medium text-muted-foreground text-sm">Class</span>
                                                <div className="flex flex-wrap gap-1.5">
                                                    {classes.map((cls) => (
                                                        <Tooltip key={cls}>
                                                            <TooltipTrigger asChild>
                                                                <button className={cn("flex h-10 w-10 items-center justify-center rounded-lg border transition-all", selectedClasses.includes(cls) ? "border-primary bg-primary/20" : "border-border bg-secondary/50 hover:border-primary/50")} onClick={() => toggleClass(cls)} type="button">
                                                                    <Image alt={CLASS_DISPLAY[cls] ?? cls} className={cn("h-6 w-6", selectedClasses.includes(cls) ? "opacity-100" : "opacity-60")} height={24} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${CLASS_ICON[cls] ?? cls.toLowerCase()}.png`} width={24} />
                                                                </button>
                                                            </TooltipTrigger>
                                                            <TooltipContent>
                                                                <p>{CLASS_DISPLAY[cls] ?? cls}</p>
                                                            </TooltipContent>
                                                        </Tooltip>
                                                    ))}
                                                </div>
                                            </div>

                                            {/* Rarity Filter - Buttons */}
                                            <div className="space-y-3">
                                                <span className="font-medium text-muted-foreground text-sm">Rarity</span>
                                                <div className="flex flex-wrap gap-2">
                                                    {rarities.map((rarity) => (
                                                        <button
                                                            className={cn(
                                                                "flex items-center justify-center rounded-lg border px-3 py-1.5 font-medium text-sm transition-all",
                                                                selectedRarities.includes(rarity) ? "border-amber-500 bg-amber-500/20 text-amber-400" : "border-border bg-secondary/50 text-muted-foreground hover:border-amber-500/50 hover:text-amber-400",
                                                            )}
                                                            key={rarity}
                                                            onClick={() => toggleRarity(rarity)}
                                                            type="button"
                                                        >
                                                            {rarity}★
                                                        </button>
                                                    ))}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </motion.div>
                            )}
                        </AnimatePresence>
                    </div>

                    {/* Advanced Filters */}
                    <div className="rounded-lg border border-border bg-secondary/30">
                        <button
                            className="flex w-full items-center justify-between px-3 py-2.5 text-left transition-colors hover:bg-secondary/50"
                            onClick={() => setIsAdvancedOpen(!isAdvancedOpen)}
                            type="button"
                        >
                            <div className="flex items-center gap-2">
                                <span className="font-medium text-foreground text-sm">Advanced</span>
                                {hasAdvancedFilters && !isAdvancedOpen && (
                                    <span className="flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1.5 text-primary-foreground text-xs">{advancedFilterCount}</span>
                                )}
                            </div>
                            <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-200", isAdvancedOpen && "rotate-180")} />
                        </button>

                        <AnimatePresence initial={false}>
                            {isAdvancedOpen && (
                                <motion.div
                                    animate={{ height: "auto", opacity: 1 }}
                                    exit={{ height: 0, opacity: 0 }}
                                    initial={{ height: 0, opacity: 0 }}
                                    transition={{ duration: 0.2, ease: [0.4, 0, 0.2, 1] }}
                                >
                                    <div className="border-border border-t px-3 py-3">
                                        <div className="grid gap-4 sm:gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                                            {/* Archetype/Subclass Filter - Dropdown */}
                                            <FilterDropdown
                                                formatOption={(s) => formatSubProfession(s)}
                                                formatSelected={(s) => capitalize(formatSubProfession(s))}
                                                label="Archetype"
                                                onRemove={(s) => onSubclassChange(selectedSubclasses.filter((x) => x !== s))}
                                                onToggle={createToggle(selectedSubclasses, onSubclassChange)}
                                                options={subclasses}
                                                placeholder="Select archetype"
                                                selectedOptions={selectedSubclasses}
                                            />

                                            {/* Gender Filter - Buttons */}
                                            <div className="space-y-3">
                                                <span className="font-medium text-muted-foreground text-sm">Gender</span>
                                                <div className="flex flex-wrap gap-2">
                                                    {genders.map((gender) => (
                                                        <button
                                                            className={cn("rounded-lg border px-3 py-1.5 text-sm transition-all", selectedGenders.includes(gender) ? "border-primary bg-primary/20 text-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")}
                                                            key={gender}
                                                            onClick={() => toggleGender(gender)}
                                                            type="button"
                                                        >
                                                            {gender}
                                                        </button>
                                                    ))}
                                                </div>
                                            </div>

                                            {/* Nation Filter - Dropdown */}
                                            <FilterDropdown formatOption={(n) => NATION_DISPLAY[n] ?? n} label="Nation" onRemove={(n) => onNationChange(selectedNations.filter((x) => x !== n))} onToggle={createToggle(selectedNations, onNationChange)} options={nations} placeholder="Select nation" selectedOptions={selectedNations} />

                                            {/* Faction Filter - Dropdown */}
                                            <FilterDropdown formatOption={formatNationId} label="Faction" onRemove={(f) => onFactionChange(selectedFactions.filter((x) => x !== f))} onToggle={createToggle(selectedFactions, onFactionChange)} options={factions} placeholder="Select faction" selectedOptions={selectedFactions} />

                                            {/* Race Filter - Dropdown */}
                                            <FilterDropdown label="Race" onRemove={(r) => onRaceChange(selectedRaces.filter((x) => x !== r))} onToggle={createToggle(selectedRaces, onRaceChange)} options={races} placeholder="Select race" selectedOptions={selectedRaces} />

                                            {/* Birth Place Filter - Dropdown */}
                                            <FilterDropdown label="Place of Birth" onRemove={(p) => onBirthPlaceChange(selectedBirthPlaces.filter((x) => x !== p))} onToggle={createToggle(selectedBirthPlaces, onBirthPlaceChange)} options={birthPlaces} placeholder="Select birth place" selectedOptions={selectedBirthPlaces} />

                                            {/* Artist Filter - Dropdown */}
                                            <FilterDropdown label="Artist" onRemove={(a) => onArtistChange(selectedArtists.filter((x) => x !== a))} onToggle={createToggle(selectedArtists, onArtistChange)} options={artists} placeholder="Select artist" selectedOptions={selectedArtists} />
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
