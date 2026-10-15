import type { ArchiveDossier } from "#/types/generated/ArchiveDossier";
import type { InfectionState } from "#/types/generated/InfectionState";

/** A closed numeric range; `null` is no bound set. */
export type Range = [number, number] | null;

/**
 * The Dossiers filters. Within one field the picks are alternatives (any of
 * them), across fields they all must hold, the way /operators combines its
 * filters. Values are the rows' own: class codes, star counts, faction ids,
 * and the file's words for race and birthplace as the server writes them.
 */
export interface IDossierFilters {
    classes: string[];
    subclasses: string[];
    rarities: number[];
    genders: string[];
    nations: string[];
    factions: string[];
    races: string[];
    birthplaces: string[];
    infection: InfectionState[];
    height: Range;
    experience: Range;
}

export type DossierListKey = Exclude<keyof IDossierFilters, "height" | "experience">;

export const EMPTY_DOSSIER_FILTERS: IDossierFilters = {
    classes: [],
    subclasses: [],
    rarities: [],
    genders: [],
    nations: [],
    factions: [],
    races: [],
    birthplaces: [],
    infection: [],
    height: null,
    experience: null,
};

export const INFECTION_ORDER: InfectionState[] = ["infected", "clear", "unclear", "unstated"];

/**
 * A row's gender as a filter value: the canonical key when the file's word
 * reads as one (`female`, `male`), else the word as written (`Conviction`).
 */
export function genderValue(d: ArchiveDossier): string | undefined {
    return d.genderKey ?? d.gender;
}

export function matchesDossier(d: ArchiveDossier, f: IDossierFilters): boolean {
    const any = <T>(picked: readonly T[], value: T | undefined) => picked.length === 0 || (value !== undefined && picked.includes(value));
    const within = (r: Range, v: number | undefined) => r === null || (v !== undefined && v >= r[0] && v <= r[1]);
    return (
        any(f.classes, d.profession) &&
        any(f.subclasses, d.subProfessionId) &&
        any(f.rarities, d.rarity) &&
        any(f.genders, genderValue(d)) &&
        any(f.nations, d.nation?.id) &&
        (f.factions.length === 0 || (d.affiliations ?? []).some((a) => f.factions.includes(a.id))) &&
        any(f.races, d.race) &&
        any(f.birthplaces, d.birthplace) &&
        any(f.infection, d.infectionState) &&
        within(f.height, d.heightCm) &&
        within(f.experience, d.combatYears)
    );
}

export function countDossierFilters(f: IDossierFilters): number {
    return f.classes.length + f.subclasses.length + f.rarities.length + f.genders.length + f.nations.length + f.factions.length + f.races.length + f.birthplaces.length + f.infection.length + (f.height ? 1 : 0) + (f.experience ? 1 : 0);
}

/** One choice in a filter, with the rows that carry it. */
export interface IOption {
    value: string;
    label: string;
    count: number;
}

export interface IDossierOptions {
    subclasses: IOption[];
    genders: IOption[];
    nations: IOption[];
    factions: IOption[];
    races: IOption[];
    birthplaces: IOption[];
    /** The rows' own extremes, rounded out to whole units; `null` when no row states the value. */
    height: Range;
    experience: Range;
}

/** An option's label by its value, the value itself when no option carries it. */
export function optionLabel(options: readonly IOption[]): (value: string) => string {
    const labels = new Map(options.map((o) => [o.value, o.label]));
    return (value) => labels.get(value) ?? value;
}

function tally(rows: readonly ArchiveDossier[], pick: (d: ArchiveDossier) => readonly { value: string; label: string }[]): IOption[] {
    const seen = new Map<string, { labels: Map<string, number>; count: number }>();
    for (const d of rows) {
        for (const { value, label } of pick(d)) {
            const entry = seen.get(value) ?? { labels: new Map(), count: 0 };
            entry.count += 1;
            entry.labels.set(label, (entry.labels.get(label) ?? 0) + 1);
            seen.set(value, entry);
        }
    }
    // A value spelled several ways (JP's 女 and 女性 both read Female) shows its most common spelling.
    return [...seen.entries()].map(([value, e]) => ({ value, count: e.count, label: [...e.labels.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))[0][0] })).sort((a, b) => b.count - a.count || a.label.localeCompare(b.label));
}

function extent(values: (number | undefined)[]): Range {
    const xs = values.filter((v): v is number => v !== undefined && Number.isFinite(v));
    if (xs.length === 0) return null;
    return [Math.floor(Math.min(...xs)), Math.ceil(Math.max(...xs))];
}

const one = (value: string | undefined, label = value) => (value && label ? [{ value, label }] : []);

export function buildDossierOptions(rows: readonly ArchiveDossier[]): IDossierOptions {
    return {
        subclasses: tally(rows, (d) => one(d.subProfessionId || undefined, d.subProfessionName ?? d.subProfessionId)),
        genders: tally(rows, (d) => one(genderValue(d), d.gender)),
        nations: tally(rows, (d) => one(d.nation?.id, d.nation?.name)),
        factions: tally(rows, (d) => (d.affiliations ?? []).map((a) => ({ value: a.id, label: a.name }))),
        races: tally(rows, (d) => one(d.race)),
        birthplaces: tally(rows, (d) => one(d.birthplace)),
        height: extent(rows.map((d) => d.heightCm)),
        experience: extent(rows.map((d) => d.combatYears)),
    };
}
