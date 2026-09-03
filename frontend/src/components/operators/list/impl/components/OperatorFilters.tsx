import type { OperatorRarityTier } from "#/types/operators";
import { HAS_NOTES_OPTIONS } from "../constants";
import type { ArrayFilterKey, AvailabilityFilter, HasNotesFilter, IFilterOptions, ISharedFilters } from "../types";
import { FilterPanel } from "./FilterPanel";
import { OperatorFilterFields, TagRow } from "./OperatorFilterFields";

interface IOperatorFiltersProps {
    selectedClasses: string[];
    selectedSubclasses: string[];
    selectedRarities: OperatorRarityTier[];
    selectedGenders: string[];
    selectedNations: string[];
    selectedFactions: string[];
    selectedRaces: string[];
    selectedBirthPlaces: string[];
    selectedArtists: string[];
    selectedVoiceActors: string[];
    selectedHasNotes: HasNotesFilter;
    selectedAvailability: AvailabilityFilter;
    options: IFilterOptions;
    onClassesChange: (v: string[]) => void;
    onSubclassesChange: (v: string[]) => void;
    onRaritiesChange: (v: OperatorRarityTier[]) => void;
    onGendersChange: (v: string[]) => void;
    onNationsChange: (v: string[]) => void;
    onFactionsChange: (v: string[]) => void;
    onRacesChange: (v: string[]) => void;
    onBirthPlacesChange: (v: string[]) => void;
    onArtistsChange: (v: string[]) => void;
    onVoiceActorsChange: (v: string[]) => void;
    onHasNotesChange: (v: HasNotesFilter) => void;
    onAvailabilityChange: (v: AvailabilityFilter) => void;
    onClearAll: () => void;
    hasActiveFilters: boolean;
    collapsed?: boolean;
    onToggle?: () => void;
    activeFilterCount?: number;
}

const AVAILABILITY_OPTIONS: { value: AvailabilityFilter; label: string }[] = [
    { value: "global", label: "Global" },
    { value: "upcoming", label: "Upcoming (CN)" },
];

function FiltersContent(props: Omit<IOperatorFiltersProps, "collapsed" | "onToggle" | "activeFilterCount">) {
    const setters: { [K in ArrayFilterKey]: (v: ISharedFilters[K]) => void } = {
        classes: props.onClassesChange,
        subclasses: props.onSubclassesChange,
        rarities: props.onRaritiesChange,
        genders: props.onGendersChange,
        nations: props.onNationsChange,
        factions: props.onFactionsChange,
        races: props.onRacesChange,
        birthPlaces: props.onBirthPlacesChange,
        artists: props.onArtistsChange,
        voiceActors: props.onVoiceActorsChange,
    };
    return (
        <OperatorFilterFields
            filters={{
                classes: props.selectedClasses,
                subclasses: props.selectedSubclasses,
                rarities: props.selectedRarities,
                genders: props.selectedGenders,
                nations: props.selectedNations,
                factions: props.selectedFactions,
                races: props.selectedRaces,
                birthPlaces: props.selectedBirthPlaces,
                artists: props.selectedArtists,
                voiceActors: props.selectedVoiceActors,
            }}
            options={props.options}
            onChange={(key, value) => (setters[key] as (v: ISharedFilters[typeof key]) => void)(value)}
            basicLeading={<TagRow label="Availability" options={AVAILABILITY_OPTIONS} value={props.selectedAvailability} onChange={props.onAvailabilityChange} />}
            basicTrailing={<TagRow label="Notes" options={HAS_NOTES_OPTIONS} value={props.selectedHasNotes} onChange={props.onHasNotesChange} />}
        />
    );
}

export function OperatorFilters(props: IOperatorFiltersProps) {
    const { collapsed, onToggle, activeFilterCount, ...content } = props;
    return (
        <FilterPanel collapsed={collapsed} onToggle={onToggle} activeFilterCount={activeFilterCount} hasActiveFilters={props.hasActiveFilters} onClearAll={props.onClearAll} ariaLabel="Operator filters">
            <FiltersContent {...content} />
        </FilterPanel>
    );
}
