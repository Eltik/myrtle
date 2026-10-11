import { describe, expect, it } from "vitest";
import type { ArchiveDossier } from "#/types/generated/ArchiveDossier";
import { buildDossierOptions, countDossierFilters, EMPTY_DOSSIER_FILTERS, matchesDossier } from "./dossierMatch";

const row = (over: Partial<ArchiveDossier>): ArchiveDossier => ({
    id: "char_x",
    name: "X",
    rarity: 6,
    profession: "SNIPER",
    subProfessionId: "fastshot",
    affiliations: [],
    infectionState: "clear",
    storyCount: 0,
    recordCount: 0,
    linkCount: 0,
    ...over,
});

const amiya = row({ id: "a", profession: "CASTER", rarity: 5, gender: "Female", genderKey: "female", race: "Cautus", heightCm: 142, combatYears: 3, infectionState: "infected", nation: { id: "rhodes", name: "Rhodes Island" }, affiliations: [{ id: "elite", name: "Elite Operators" }] });
const lancet = row({ id: "l", rarity: 1, gender: "Male", genderKey: "male", race: undefined, heightCm: undefined, infectionState: "unstated" });
const odd = row({ id: "o", gender: "Conviction", heightCm: 170 });

describe("dossier filters", () => {
    it("lets every row through with no filter set", () => {
        expect([amiya, lancet, odd].every((d) => matchesDossier(d, EMPTY_DOSSIER_FILTERS))).toBe(true);
        expect(countDossierFilters(EMPTY_DOSSIER_FILTERS)).toBe(0);
    });

    it("ORs within a field and ANDs across fields", () => {
        const f = { ...EMPTY_DOSSIER_FILTERS, rarities: [5, 1], infection: ["infected" as const] };
        expect([amiya, lancet, odd].filter((d) => matchesDossier(d, f)).map((d) => d.id)).toEqual(["a"]);
        expect(countDossierFilters(f)).toBe(3);
    });

    it("reads gender by key, and an unread word as written", () => {
        expect(matchesDossier(amiya, { ...EMPTY_DOSSIER_FILTERS, genders: ["female"] })).toBe(true);
        expect(matchesDossier(odd, { ...EMPTY_DOSSIER_FILTERS, genders: ["Conviction"] })).toBe(true);
        expect(matchesDossier(lancet, { ...EMPTY_DOSSIER_FILTERS, genders: ["female"] })).toBe(false);
    });

    it("drops rows with no stated value from a range", () => {
        const f = { ...EMPTY_DOSSIER_FILTERS, height: [140, 175] as [number, number] };
        expect([amiya, lancet, odd].filter((d) => matchesDossier(d, f)).map((d) => d.id)).toEqual(["a", "o"]);
    });

    it("matches a group or team pick against any affiliation", () => {
        expect(matchesDossier(amiya, { ...EMPTY_DOSSIER_FILTERS, factions: ["elite"] })).toBe(true);
        expect(matchesDossier(odd, { ...EMPTY_DOSSIER_FILTERS, factions: ["elite"] })).toBe(false);
    });

    it("builds options from the rows, with counts and the rows' own extremes", () => {
        const o = buildDossierOptions([amiya, lancet, odd]);
        expect(o.genders.map((g) => [g.value, g.label, g.count])).toEqual([
            ["Conviction", "Conviction", 1],
            ["female", "Female", 1],
            ["male", "Male", 1],
        ]);
        expect(o.nations).toEqual([{ value: "rhodes", label: "Rhodes Island", count: 1 }]);
        expect(o.height).toEqual([142, 170]);
        expect(o.races.map((r) => r.value)).toEqual(["Cautus"]);
    });
});
