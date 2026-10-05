import { describe, expect, it } from "vitest";
import { type ITierEntity, toTierEntity, UNPLACED } from "#/lib/api/tier-entities";
import { compactForSearch } from "#/lib/search/fuzzy";
import { anyFacetSelected, matchesFacets, matchesSearch } from "./poolFilters";
import type { IPoolFacet } from "./poolKinds";

const operator = (id: string, rarity: string, profession: string) => toTierEntity("operator", id, { kind: "operator", id, name: id, icon: null, href: null, facets: { rarity, profession } }, UNPLACED);
const facet = (id: string, read: (entity: ITierEntity) => string | null): IPoolFacet => ({ id, label: id, groupLabel: id, variant: "text", options: [], valueOf: read });
const facetOf = (name: string) => (entity: ITierEntity) => {
    const value = entity.facets[name];
    return typeof value === "string" ? value : null;
};

const facets = [facet("rarity", facetOf("rarity")), facet("class", facetOf("profession"))];
const amiya = operator("char_002_amiya", "TIER_5", "CASTER");
// An operator only CN has released is an entity like any other: the same rows filter it.
const cnOnly = operator("char_1015_aglna2", "TIER_6", "SUPPORT");

describe("matchesFacets", () => {
    it("passes everything when nothing is selected", () => {
        expect(matchesFacets(amiya, facets, {})).toBe(true);
        expect(matchesFacets(amiya, facets, { rarity: [], class: [] })).toBe(true);
    });

    it("needs the entity's value among each selected row's options", () => {
        expect(matchesFacets(amiya, facets, { rarity: ["TIER_5", "TIER_6"] })).toBe(true);
        expect(matchesFacets(amiya, facets, { rarity: ["TIER_6"] })).toBe(false);
        expect(matchesFacets(cnOnly, facets, { rarity: ["TIER_6"], class: ["SUPPORT"] })).toBe(true);
        expect(matchesFacets(cnOnly, facets, { rarity: ["TIER_6"], class: ["CASTER"] })).toBe(false);
    });

    it("never passes an entity with no value for a selected row", () => {
        const none = [facet("brand", () => null)];
        expect(matchesFacets(amiya, none, { brand: ["Coral Coast"] })).toBe(false);
        expect(matchesFacets(amiya, none, {})).toBe(true);
    });

    it("ignores a selection for a row the kind does not have", () => {
        expect(matchesFacets(amiya, facets, { slot: ["1"] })).toBe(true);
    });
});

describe("anyFacetSelected", () => {
    it("is true only when some row has a choice", () => {
        expect(anyFacetSelected({})).toBe(false);
        expect(anyFacetSelected({ rarity: [] })).toBe(false);
        expect(anyFacetSelected({ rarity: [], class: ["CASTER"] })).toBe(true);
    });
});

describe("matchesSearch", () => {
    it("passes everything on a blank query", () => {
        expect(matchesSearch([], "")).toBe(true);
        expect(matchesSearch([null], "")).toBe(true);
    });

    it("finds the compacted query in any text, skipping nulls", () => {
        const q = compactForSearch("Mellow Wish");
        expect(matchesSearch([null, "Angelina the Mellow Wish"], q)).toBe(true);
        expect(matchesSearch(["Amiya", null], q)).toBe(false);
        expect(matchesSearch([], q)).toBe(false);
    });
});
