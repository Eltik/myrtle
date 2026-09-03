/** This module exists so the two surfaces cannot drift; a filter added here appears on both pages. */
import type { IActiveChip } from "./components/ActiveFilterChips";
import { CHIP_CONFIG } from "./constants";
import type { ArrayFilterKey, IFilterOptions, IFilterSubject, ISharedFilters } from "./types";

export const EMPTY_SHARED_FILTERS: ISharedFilters = {
    classes: [],
    subclasses: [],
    rarities: [],
    genders: [],
    nations: [],
    factions: [],
    races: [],
    birthPlaces: [],
    artists: [],
    voiceActors: [],
};

export function buildFilterOptions(subjects: readonly IFilterSubject[]): IFilterOptions {
    const subclasses = new Set<string>();
    const nations = new Set<string>();
    const factions = new Set<string>();
    const races = new Set<string>();
    const birthPlaces = new Set<string>();
    const artists = new Set<string>();
    const voiceActors = new Set<string>();

    for (const op of subjects) {
        const { race, placeOfBirth } = op;
        if (op.subProfessionId) subclasses.add(op.subProfessionId);
        if (op.nationId) nations.add(op.nationId);
        if (op.groupId) factions.add(op.groupId);
        if (op.teamId) factions.add(op.teamId);
        if (race && race !== "Unknown") races.add(race);
        if (placeOfBirth) birthPlaces.add(placeOfBirth);
        for (const a of op.artists) artists.add(a);
        for (const v of op.voiceActors) voiceActors.add(v);
    }

    return {
        subclasses: [...subclasses].sort(),
        nations: [...nations].sort(),
        factions: [...factions].sort(),
        races: [...races].sort(),
        birthPlaces: [...birthPlaces].sort(),
        artists: [...artists].sort(),
        voiceActors: [...voiceActors].sort(),
    };
}

export type FilterSets = Readonly<Record<ArrayFilterKey, ReadonlySet<string>>>;

export function toFilterSets(filters: ISharedFilters): FilterSets {
    return {
        classes: new Set(filters.classes),
        subclasses: new Set(filters.subclasses),
        rarities: new Set(filters.rarities),
        genders: new Set(filters.genders),
        nations: new Set(filters.nations),
        factions: new Set(filters.factions),
        races: new Set(filters.races),
        birthPlaces: new Set(filters.birthPlaces),
        artists: new Set(filters.artists),
        voiceActors: new Set(filters.voiceActors),
    };
}

export function hasAnySharedFilter(sets: FilterSets): boolean {
    return Object.values(sets).some((set) => set.size > 0);
}

export function matchesSharedFilters(subject: IFilterSubject, sets: FilterSets): boolean {
    if (sets.classes.size && !sets.classes.has(subject.profession)) return false;
    if (sets.subclasses.size && !sets.subclasses.has(subject.subProfessionId)) return false;
    if (sets.rarities.size && !sets.rarities.has(`TIER_${subject.rarity}`)) return false;
    if (sets.genders.size && (!subject.gender || !sets.genders.has(subject.gender))) return false;
    if (sets.nations.size && (!subject.nationId || !sets.nations.has(subject.nationId))) return false;
    if (sets.factions.size) {
        const hit = (subject.groupId && sets.factions.has(subject.groupId)) || (subject.teamId && sets.factions.has(subject.teamId));
        if (!hit) return false;
    }
    if (sets.races.size && (!subject.race || !sets.races.has(subject.race))) return false;
    if (sets.birthPlaces.size && (!subject.placeOfBirth || !sets.birthPlaces.has(subject.placeOfBirth))) return false;
    if (sets.artists.size && !subject.artists.some((a) => sets.artists.has(a))) return false;
    if (sets.voiceActors.size && !subject.voiceActors.some((v) => sets.voiceActors.has(v))) return false;
    return true;
}

export function countSharedFilters(filters: ISharedFilters): number {
    return filters.classes.length + filters.subclasses.length + filters.rarities.length + filters.genders.length + filters.nations.length + filters.factions.length + filters.races.length + filters.birthPlaces.length + filters.artists.length + filters.voiceActors.length;
}

export function buildSharedChips(filters: ISharedFilters, removeFrom: (key: ArrayFilterKey, value: string) => void): IActiveChip[] {
    return CHIP_CONFIG.flatMap(({ key, prefix, label }) =>
        (filters[key] as string[]).map((v) => ({
            key: `${prefix}-${v}`,
            label: label(v),
            onRemove: () => removeFrom(key, v),
        })),
    );
}
