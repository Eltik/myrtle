import { describe, expect, it } from "vitest";
import { type OperatorFilterInput, operatorFilterPredicate } from "./matchFilters";
import type { IOperatorView } from "./types";

const NONE: OperatorFilterInput = {
    searchQuery: "",
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
    hasNotes: "any",
};

function op(id: string, fields: Partial<IOperatorView>): IOperatorView {
    return {
        id,
        name: id,
        appellation: "",
        rarity: 6,
        profession: "WARRIOR",
        subProfessionId: "lord",
        nationId: "",
        groupId: null,
        teamId: null,
        gender: "Female",
        race: "Feline",
        placeOfBirth: "",
        artists: [],
        voiceActors: [],
        hasNotes: false,
        ownership: null,
        ...fields,
    } as IOperatorView;
}

// Shaped like the CN-only "upcoming" list: one Cautus among operators that are not.
const upcoming = [op("cautus_one", { race: "Cautus", groupId: "rhodes", gender: "Male", placeOfBirth: "Victoria", artists: ["A"], voiceActors: ["V"], hasNotes: true }), op("feline_one", { race: "Feline", teamId: "rainbow" }), op("unknown_race", { race: "" })];

const ids = (input: Partial<OperatorFilterInput>) => upcoming.filter(operatorFilterPredicate({ ...NONE, ...input })).map((o) => o.id);

describe("operatorFilterPredicate", () => {
    it("passes everything with no filter set", () => {
        expect(ids({})).toEqual(["cautus_one", "feline_one", "unknown_race"]);
    });

    // Each of these was ignored on the CN-only list before the shared predicate.
    it.each([
        ["races", { races: ["Cautus"] }],
        ["factions", { factions: ["rhodes"] }],
        ["genders", { genders: ["Male"] }],
        ["birthPlaces", { birthPlaces: ["Victoria"] }],
        ["artists", { artists: ["A"] }],
        ["voiceActors", { voiceActors: ["V"] }],
        ["hasNotes", { hasNotes: "yes" as const }],
    ])("applies the %s filter", (_, input) => {
        expect(ids(input)).toEqual(["cautus_one"]);
    });

    it("matches a faction by team id as well as group id", () => {
        expect(ids({ factions: ["rainbow"] })).toEqual(["feline_one"]);
    });

    it("still applies search, class, rarity and nation", () => {
        expect(ids({ searchQuery: "feline" })).toEqual(["feline_one"]);
        expect(ids({ classes: ["SNIPER"] })).toEqual([]);
        expect(ids({ rarities: ["TIER_5"] })).toEqual([]);
        expect(ids({ nations: ["kazimierz"] })).toEqual([]);
    });
});
