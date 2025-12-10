"use client";

import { ArrowDown, ArrowUp, X } from "lucide-react";
import Image from "next/image";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/shadcn/tooltip";
import { capitalize, cn, formatNationId, formatSubProfession } from "~/lib/utils";
import { CLASS_DISPLAY, CLASS_ICON, NATION_DISPLAY } from "./constants";
import { FilterDropdown } from "./ui/impl/filter-dropdown";

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
    sortBy: "name" | "rarity";
    sortOrder: "asc" | "desc";
    onClassChange: (classes: string[]) => void;
    onSubclassChange: (subclasses: string[]) => void;
    onRarityChange: (rarities: number[]) => void;
    onGenderChange: (genders: string[]) => void;
    onBirthPlaceChange: (birthPlaces: string[]) => void;
    onNationChange: (nations: string[]) => void;
    onFactionChange: (factions: string[]) => void;
    onRaceChange: (races: string[]) => void;
    onArtistChange: (artists: string[]) => void;
    onSortByChange: (sortBy: "name" | "rarity") => void;
    onSortOrderChange: (order: "asc" | "desc") => void;
    onClearFilters: () => void;
}

// Helper to create toggle functions for array state
function createToggle<T>(selected: T[], onChange: (items: T[]) => void) {
    return (item: T) => {
        if (selected.includes(item)) {
            onChange(selected.filter((i) => i !== item));
        } else {
            onChange([...selected, item]);
        }
    };
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
    sortBy,
    sortOrder,
    onClassChange,
    onSubclassChange,
    onRarityChange,
    onGenderChange,
    onBirthPlaceChange,
    onNationChange,
    onFactionChange,
    onRaceChange,
    onArtistChange,
    onSortByChange,
    onSortOrderChange,
    onClearFilters,
}: OperatorFiltersProps) {
    const toggleClass = createToggle(selectedClasses, onClassChange);
    const toggleRarity = createToggle(selectedRarities, onRarityChange);
    const toggleGender = createToggle(selectedGenders, onGenderChange);

    const hasFilters = selectedClasses.length > 0 || selectedSubclasses.length > 0 || selectedRarities.length > 0 || selectedGenders.length > 0 || selectedBirthPlaces.length > 0 || selectedNations.length > 0 || selectedFactions.length > 0 || selectedRaces.length > 0 || selectedArtists.length > 0;

    return (
        <div className="z-99 min-w-0 overflow-hidden rounded-lg text-foreground">
            <div className="p-3 sm:p-4">
                {/* Header */}
                <div className="mb-4 flex items-center justify-between">
                    <h3 className="font-semibold text-foreground">Filters & Sorting</h3>
                    {hasFilters && (
                        <button className="flex items-center gap-1 text-muted-foreground text-sm transition-colors hover:text-foreground" onClick={onClearFilters} type="button">
                            <X className="h-3 w-3" />
                            Clear all
                        </button>
                    )}
                </div>

                <div className="space-y-6">
                    {/* Row 1: Class, Subclass, Rarity, Sorting */}
                    <div className="grid gap-4 sm:gap-6 md:grid-cols-2 lg:grid-cols-4">
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

                        {/* Subclass Filter - Dropdown */}
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

                        {/* Sorting */}
                        <div className="space-y-3">
                            <span className="font-medium text-muted-foreground text-sm">Sort By</span>
                            <div className="flex gap-2">
                                <button
                                    className={cn("flex-1 rounded-lg border px-3 py-1.5 text-sm transition-all", sortBy === "rarity" ? "border-primary bg-primary/20 text-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")}
                                    onClick={() => onSortByChange("rarity")}
                                    type="button"
                                >
                                    Rarity
                                </button>
                                <button className={cn("flex-1 rounded-lg border px-3 py-1.5 text-sm transition-all", sortBy === "name" ? "border-primary bg-primary/20 text-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")} onClick={() => onSortByChange("name")} type="button">
                                    Name
                                </button>
                                <button className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:text-foreground" onClick={() => onSortOrderChange(sortOrder === "asc" ? "desc" : "asc")} type="button">
                                    {sortOrder === "asc" ? <ArrowUp className="h-4 w-4" /> : <ArrowDown className="h-4 w-4" />}
                                </button>
                            </div>
                        </div>
                    </div>

                    {/* Row 2: Gender, Race, and Dropdown Filters */}
                    <div className="grid gap-4 sm:gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
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

                        {/* Race Filter - Dropdown */}
                        <FilterDropdown label="Race" onRemove={(r) => onRaceChange(selectedRaces.filter((x) => x !== r))} onToggle={createToggle(selectedRaces, onRaceChange)} options={races} placeholder="Select race" selectedOptions={selectedRaces} />

                        {/* Birth Place Filter - Dropdown */}
                        <FilterDropdown label="Place of Birth" onRemove={(p) => onBirthPlaceChange(selectedBirthPlaces.filter((x) => x !== p))} onToggle={createToggle(selectedBirthPlaces, onBirthPlaceChange)} options={birthPlaces} placeholder="Select birth place" selectedOptions={selectedBirthPlaces} />

                        {/* Nation Filter - Dropdown */}
                        <FilterDropdown formatOption={(n) => NATION_DISPLAY[n] ?? n} label="Nation" onRemove={(n) => onNationChange(selectedNations.filter((x) => x !== n))} onToggle={createToggle(selectedNations, onNationChange)} options={nations} placeholder="Select nation" selectedOptions={selectedNations} />

                        {/* Faction Filter - Dropdown */}
                        <FilterDropdown formatOption={formatNationId} label="Faction" onRemove={(f) => onFactionChange(selectedFactions.filter((x) => x !== f))} onToggle={createToggle(selectedFactions, onFactionChange)} options={factions} placeholder="Select faction" selectedOptions={selectedFactions} />

                        {/* Artist Filter - Dropdown */}
                        <FilterDropdown label="Artist" onRemove={(a) => onArtistChange(selectedArtists.filter((x) => x !== a))} onToggle={createToggle(selectedArtists, onArtistChange)} options={artists} placeholder="Select artist" selectedOptions={selectedArtists} />
                    </div>
                </div>
            </div>
        </div>
    );
}
