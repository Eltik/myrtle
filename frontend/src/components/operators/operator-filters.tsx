"use client";

import { ArrowDown, ArrowUp, ChevronDown, X } from "lucide-react";
import Image from "next/image";
import { cn, formatSubProfession, capitalize, formatNationId } from "~/lib/utils";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "~/components/ui/dropdown-menu";
import { Button } from "~/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/ui/tooltip";

interface OperatorFiltersProps {
    isOpen: boolean;
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

// Maps internal profession names to display names
const CLASS_DISPLAY: Record<string, string> = {
    WARRIOR: "Guard",
    SNIPER: "Sniper",
    TANK: "Defender",
    MEDIC: "Medic",
    SUPPORT: "Supporter",
    CASTER: "Caster",
    SPECIAL: "Specialist",
    PIONEER: "Vanguard",
};

// Maps internal profession names to icon file names
const CLASS_ICON: Record<string, string> = {
    WARRIOR: "warrior",
    SNIPER: "sniper",
    TANK: "tank",
    MEDIC: "medic",
    SUPPORT: "support",
    CASTER: "caster",
    SPECIAL: "special",
    PIONEER: "pioneer",
};

const NATION_DISPLAY: Record<string, string> = {
    rhodes: "Rhodes Island",
    kazimierz: "Kazimierz",
    columbia: "Columbia",
    laterano: "Laterano",
    victoria: "Victoria",
    sami: "Sami",
    bolivar: "Bolivar",
    iberia: "Iberia",
    siracusa: "Siracusa",
    higashi: "Higashi",
    sargon: "Sargon",
    kjerag: "Kjerag",
    minos: "Minos",
    yan: "Yan",
    lungmen: "Lungmen",
    ursus: "Ursus",
    egir: "Ægir",
    leithanien: "Leithanien",
    rim: "Rim Billiton",
};

export function OperatorFilters({
    isOpen,
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
    const toggleClass = (cls: string) => {
        if (selectedClasses.includes(cls)) {
            onClassChange(selectedClasses.filter((c) => c !== cls));
        } else {
            onClassChange([...selectedClasses, cls]);
        }
    };

    const toggleSubclass = (subclass: string) => {
        if (selectedSubclasses.includes(subclass)) {
            onSubclassChange(selectedSubclasses.filter((s) => s !== subclass));
        } else {
            onSubclassChange([...selectedSubclasses, subclass]);
        }
    };

    const toggleRarity = (rarity: number) => {
        if (selectedRarities.includes(rarity)) {
            onRarityChange(selectedRarities.filter((r) => r !== rarity));
        } else {
            onRarityChange([...selectedRarities, rarity]);
        }
    };

    const toggleGender = (gender: string) => {
        if (selectedGenders.includes(gender)) {
            onGenderChange(selectedGenders.filter((g) => g !== gender));
        } else {
            onGenderChange([...selectedGenders, gender]);
        }
    };

    const toggleBirthPlace = (place: string) => {
        if (selectedBirthPlaces.includes(place)) {
            onBirthPlaceChange(selectedBirthPlaces.filter((p) => p !== place));
        } else {
            onBirthPlaceChange([...selectedBirthPlaces, place]);
        }
    };

    const toggleNation = (nation: string) => {
        if (selectedNations.includes(nation)) {
            onNationChange(selectedNations.filter((n) => n !== nation));
        } else {
            onNationChange([...selectedNations, nation]);
        }
    };

    const toggleFaction = (faction: string) => {
        if (selectedFactions.includes(faction)) {
            onFactionChange(selectedFactions.filter((f) => f !== faction));
        } else {
            onFactionChange([...selectedFactions, faction]);
        }
    };

    const toggleRace = (race: string) => {
        if (selectedRaces.includes(race)) {
            onRaceChange(selectedRaces.filter((r) => r !== race));
        } else {
            onRaceChange([...selectedRaces, race]);
        }
    };

    const toggleArtist = (artist: string) => {
        if (selectedArtists.includes(artist)) {
            onArtistChange(selectedArtists.filter((a) => a !== artist));
        } else {
            onArtistChange([...selectedArtists, artist]);
        }
    };

    const hasFilters = selectedClasses.length > 0 || selectedSubclasses.length > 0 || selectedRarities.length > 0 || selectedGenders.length > 0 || selectedBirthPlaces.length > 0 || selectedNations.length > 0 || selectedFactions.length > 0 || selectedRaces.length > 0 || selectedArtists.length > 0;

    if (!isOpen) return null;

    return (
        <div className="min-w-0 overflow-hidden rounded-lg border border-border bg-card/50 backdrop-blur-sm">
            <div className="p-3 sm:p-4">
                <div className="mb-4 flex items-center justify-between">
                    <h3 className="font-semibold text-foreground">Filters & Sorting</h3>
                    {hasFilters && (
                        <button className="flex items-center gap-1 text-muted-foreground text-sm transition-colors hover:text-foreground" onClick={onClearFilters}>
                            <X className="h-3 w-3" />
                            Clear all
                        </button>
                    )}
                </div>

                <div className="space-y-6">
                    {/* Row 1: Class, Subclass and Rarity */}
                    <div className="grid gap-4 sm:gap-6 md:grid-cols-2 lg:grid-cols-4">
                        {/* Class Filter */}
                        <div className="space-y-3">
                            <label className="font-medium text-muted-foreground text-sm">Class</label>
                            <div className="flex flex-wrap gap-1.5">
                                {classes.map((cls) => (
                                    <Tooltip key={cls}>
                                        <TooltipTrigger asChild>
                                            <button className={cn("flex h-10 w-10 items-center justify-center rounded-lg border transition-all", selectedClasses.includes(cls) ? "border-primary bg-primary/20" : "border-border bg-secondary/50 hover:border-primary/50")} onClick={() => toggleClass(cls)}>
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

                        {/* Subclass Filter */}
                        <div className="space-y-3">
                            <label className="font-medium text-muted-foreground text-sm">Archetype</label>
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button variant="outline" className="w-full justify-between">
                                        <span className="truncate">{selectedSubclasses.length > 0 ? selectedSubclasses.map((s) => capitalize(formatSubProfession(s))).join(", ") : "Select archetype"}</span>
                                        <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent className="max-h-64 w-56 overflow-y-auto">
                                    <DropdownMenuLabel>Archetypes</DropdownMenuLabel>
                                    <DropdownMenuSeparator />
                                    {subclasses.map((subclass) => (
                                        <DropdownMenuCheckboxItem key={subclass} checked={selectedSubclasses.includes(subclass)} onCheckedChange={() => toggleSubclass(subclass)} onSelect={(e) => e.preventDefault()}>
                                            {formatSubProfession(subclass)}
                                        </DropdownMenuCheckboxItem>
                                    ))}
                                </DropdownMenuContent>
                            </DropdownMenu>
                            {selectedSubclasses.length > 0 && (
                                <div className="flex flex-wrap gap-1">
                                    {selectedSubclasses.map((subclass) => (
                                        <span key={subclass} className="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-foreground text-xs">
                                            {capitalize(formatSubProfession(subclass))}
                                            <button onClick={() => onSubclassChange(selectedSubclasses.filter((s) => s !== subclass))} className="hover:text-destructive">
                                                <X className="h-3 w-3" />
                                            </button>
                                        </span>
                                    ))}
                                </div>
                            )}
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

                    {/* Row 2: Gender, Race and Dropdown Filters */}
                    <div className="grid gap-4 sm:gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                        {/* Gender Filter */}
                        <div className="space-y-3">
                            <label className="font-medium text-muted-foreground text-sm">Gender</label>
                            <div className="flex flex-wrap gap-2">
                                {genders.map((gender) => (
                                    <button
                                        className={cn("rounded-lg border px-3 py-1.5 text-sm transition-all", selectedGenders.includes(gender) ? "border-primary bg-primary/20 text-foreground" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")}
                                        key={gender}
                                        onClick={() => toggleGender(gender)}
                                    >
                                        {gender}
                                    </button>
                                ))}
                            </div>
                        </div>

                        {/* Race Filter */}
                        <div className="space-y-3">
                            <label className="font-medium text-muted-foreground text-sm">Race</label>
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button variant="outline" className="w-full justify-between">
                                        <span className="truncate">{selectedRaces.length > 0 ? selectedRaces.join(", ") : "Select race"}</span>
                                        <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent className="max-h-64 w-56 overflow-y-auto">
                                    <DropdownMenuLabel>Races</DropdownMenuLabel>
                                    <DropdownMenuSeparator />
                                    {races.map((race) => (
                                        <DropdownMenuCheckboxItem key={race} checked={selectedRaces.includes(race)} onCheckedChange={() => toggleRace(race)} onSelect={(e) => e.preventDefault()}>
                                            {race}
                                        </DropdownMenuCheckboxItem>
                                    ))}
                                </DropdownMenuContent>
                            </DropdownMenu>
                            {selectedRaces.length > 0 && (
                                <div className="flex flex-wrap gap-1">
                                    {selectedRaces.map((race) => (
                                        <span key={race} className="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-foreground text-xs">
                                            {race}
                                            <button onClick={() => onRaceChange(selectedRaces.filter((r) => r !== race))} className="hover:text-destructive">
                                                <X className="h-3 w-3" />
                                            </button>
                                        </span>
                                    ))}
                                </div>
                            )}
                        </div>

                        {/* Birth Place Filter */}
                        <div className="space-y-3">
                            <label className="font-medium text-muted-foreground text-sm">Place of Birth</label>
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button variant="outline" className="w-full justify-between">
                                        <span className="truncate">{selectedBirthPlaces.length > 0 ? selectedBirthPlaces.join(", ") : "Select birth place"}</span>
                                        <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent className="max-h-64 w-56 overflow-y-auto">
                                    <DropdownMenuLabel>Places of Birth</DropdownMenuLabel>
                                    <DropdownMenuSeparator />
                                    {birthPlaces.map((place) => (
                                        <DropdownMenuCheckboxItem key={place} checked={selectedBirthPlaces.includes(place)} onCheckedChange={() => toggleBirthPlace(place)} onSelect={(e) => e.preventDefault()}>
                                            {place}
                                        </DropdownMenuCheckboxItem>
                                    ))}
                                </DropdownMenuContent>
                            </DropdownMenu>
                            {selectedBirthPlaces.length > 0 && (
                                <div className="flex flex-wrap gap-1">
                                    {selectedBirthPlaces.map((place) => (
                                        <span key={place} className="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-foreground text-xs">
                                            {place}
                                            <button onClick={() => onBirthPlaceChange(selectedBirthPlaces.filter((b) => b !== place))} className="hover:text-destructive">
                                                <X className="h-3 w-3" />
                                            </button>
                                        </span>
                                    ))}
                                </div>
                            )}
                        </div>

                        {/* Nation Filter */}
                        <div className="space-y-3">
                            <label className="font-medium text-muted-foreground text-sm">Nation</label>
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button variant="outline" className="w-full justify-between">
                                        <span className="truncate">{selectedNations.length > 0 ? selectedNations.map((n) => NATION_DISPLAY[n] ?? n).join(", ") : "Select nation"}</span>
                                        <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent className="max-h-64 w-56 overflow-y-auto">
                                    <DropdownMenuLabel>Nations</DropdownMenuLabel>
                                    <DropdownMenuSeparator />
                                    {nations.map((nation) => (
                                        <DropdownMenuCheckboxItem key={nation} checked={selectedNations.includes(nation)} onCheckedChange={() => toggleNation(nation)} onSelect={(e) => e.preventDefault()}>
                                            {NATION_DISPLAY[nation] ?? nation}
                                        </DropdownMenuCheckboxItem>
                                    ))}
                                </DropdownMenuContent>
                            </DropdownMenu>
                            {selectedNations.length > 0 && (
                                <div className="flex flex-wrap gap-1">
                                    {selectedNations.map((nation) => (
                                        <span key={nation} className="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-foreground text-xs">
                                            {NATION_DISPLAY[nation] ?? nation}
                                            <button onClick={() => onNationChange(selectedNations.filter((n) => n !== nation))} className="hover:text-destructive">
                                                <X className="h-3 w-3" />
                                            </button>
                                        </span>
                                    ))}
                                </div>
                            )}
                        </div>

                        {/* Faction Filter */}
                        <div className="space-y-3">
                            <label className="font-medium text-muted-foreground text-sm">Faction</label>
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button variant="outline" className="w-full justify-between">
                                        <span className="truncate">{selectedFactions.length > 0 ? selectedFactions.map((faction) => formatNationId(faction)).join(", ") : "Select faction"}</span>
                                        <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent className="max-h-64 w-56 overflow-y-auto">
                                    <DropdownMenuLabel>Factions</DropdownMenuLabel>
                                    <DropdownMenuSeparator />
                                    {factions.map((faction) => (
                                        <DropdownMenuCheckboxItem key={faction} checked={selectedFactions.includes(faction)} onCheckedChange={() => toggleFaction(faction)} onSelect={(e) => e.preventDefault()}>
                                            {formatNationId(faction)}
                                        </DropdownMenuCheckboxItem>
                                    ))}
                                </DropdownMenuContent>
                            </DropdownMenu>
                            {selectedFactions.length > 0 && (
                                <div className="flex flex-wrap gap-1">
                                    {selectedFactions.map((faction) => (
                                        <span key={faction} className="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-foreground text-xs">
                                            {formatNationId(faction)}
                                            <button onClick={() => onFactionChange(selectedFactions.filter((f) => f !== faction))} className="hover:text-destructive">
                                                <X className="h-3 w-3" />
                                            </button>
                                        </span>
                                    ))}
                                </div>
                            )}
                        </div>

                        {/* Artist Filter */}
                        <div className="space-y-3">
                            <label className="font-medium text-muted-foreground text-sm">Artist</label>
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                    <Button variant="outline" className="w-full justify-between">
                                        <span className="truncate">{selectedArtists.length > 0 ? selectedArtists.join(", ") : "Select artist"}</span>
                                        <ChevronDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                                    </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent className="max-h-64 w-56 overflow-y-auto">
                                    <DropdownMenuLabel>Artists</DropdownMenuLabel>
                                    <DropdownMenuSeparator />
                                    {artists.map((artist) => (
                                        <DropdownMenuCheckboxItem key={artist} checked={selectedArtists.includes(artist)} onCheckedChange={() => toggleArtist(artist)} onSelect={(e) => e.preventDefault()}>
                                            {artist}
                                        </DropdownMenuCheckboxItem>
                                    ))}
                                </DropdownMenuContent>
                            </DropdownMenu>
                            {selectedArtists.length > 0 && (
                                <div className="flex flex-wrap gap-1">
                                    {selectedArtists.map((artist) => (
                                        <span key={artist} className="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-foreground text-xs">
                                            {artist}
                                            <button onClick={() => onArtistChange(selectedArtists.filter((a) => a !== artist))} className="hover:text-destructive">
                                                <X className="h-3 w-3" />
                                            </button>
                                        </span>
                                    ))}
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
