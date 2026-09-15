import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { OperatorRarityTier } from "#/types/operators";
import { HAS_NOTES_OPTIONS } from "../constants";
import type { messages as listConstantsMessages } from "../constants.messages";
import type { ArrayFilterKey, AvailabilityFilter, HasNotesFilter, IFilterOptions, ISharedFilters } from "../types";
import { FilterPanel } from "./FilterPanel";
import { OperatorFilterFields, TagRow } from "./OperatorFilterFields";
import type { messages } from "./OperatorFilters.messages";

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

const AVAILABILITY_OPTIONS: { value: AvailabilityFilter; labelKey: keyof typeof messages & string }[] = [
    { value: "global", labelKey: "filters.availability.global" },
    { value: "upcoming", labelKey: "filters.availability.upcoming" },
];

function FiltersContent(props: Omit<IOperatorFiltersProps, "collapsed" | "onToggle" | "activeFilterCount">) {
    const t: TypedT<typeof messages & typeof listConstantsMessages> = useT("operators");
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
            basicLeading={<TagRow label={t("filters.availability")} options={AVAILABILITY_OPTIONS.map((o) => ({ value: o.value, label: t(o.labelKey) }))} value={props.selectedAvailability} onChange={props.onAvailabilityChange} />}
            basicTrailing={<TagRow label={t("filters.notes")} options={HAS_NOTES_OPTIONS.map((o) => ({ value: o.value, label: t(o.labelKey) }))} value={props.selectedHasNotes} onChange={props.onHasNotesChange} />}
        />
    );
}

export function OperatorFilters(props: IOperatorFiltersProps) {
    const t: TypedT<typeof messages> = useT("operators");
    const { collapsed, onToggle, activeFilterCount, ...content } = props;
    return (
        <FilterPanel collapsed={collapsed} onToggle={onToggle} activeFilterCount={activeFilterCount} hasActiveFilters={props.hasActiveFilters} onClearAll={props.onClearAll} ariaLabel={t("filters.aria")}>
            <FiltersContent {...content} />
        </FilterPanel>
    );
}
