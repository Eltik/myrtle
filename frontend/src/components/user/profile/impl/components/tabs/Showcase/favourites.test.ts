import { describe, expect, it } from "vitest";
import { EMPTY_SHARED_FILTERS, matchesSharedFilters, toFilterSets } from "#/components/operators/list/impl/shared-filters";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorIndexEntry } from "#/types/operators";
import { factionMembers, rosterAccess, rosterById } from "./favourites";

function op(id: string, name: string, rarity: number, nationId: string, groupId: string | null = null, teamId: string | null = null): IOperatorIndexEntry {
    return { id, name, rarity, nationId, groupId, teamId, profession: "WARRIOR", subProfessionId: "", artists: [], voiceActors: [], gender: "", race: "", placeOfBirth: "" } as unknown as IOperatorIndexEntry;
}

const INDEX = [op("char_a", "Ash", 6, "rim", null, "rainbow"), op("char_b", "Blitz", 5, "rim", null, "rainbow"), op("char_c", "Courier", 4, "kjerag", "karlan"), op("char_d", "Silverash", 6, "kjerag", "karlan"), op("char_e", "Pramanix", 6, "kjerag", "karlan"), op("char_f", "Amiya", 5, "rhodes")];

describe("factionMembers", () => {
    it("matches a nation on nationId and a group or team on groupId / teamId", () => {
        expect(factionMembers(INDEX, { id: "kjerag", powerLevel: "nation" }).map((o) => o.id)).toEqual(["char_e", "char_d", "char_c"]);
        expect(factionMembers(INDEX, { id: "karlan", powerLevel: "group" }).map((o) => o.id)).toEqual(["char_e", "char_d", "char_c"]);
        expect(factionMembers(INDEX, { id: "rainbow", powerLevel: "team" }).map((o) => o.id)).toEqual(["char_a", "char_b"]);
        // A nation id is not a faction id: the levels do not cross.
        expect(factionMembers(INDEX, { id: "kjerag", powerLevel: "group" })).toEqual([]);
    });

    it("sorts rarest first, then by name", () => {
        expect(factionMembers(INDEX, { id: "kjerag", powerLevel: "nation" }).map((o) => o.name)).toEqual(["Pramanix", "Silverash", "Courier"]);
    });

    it("counts what the /operators Nation and Faction filters count", () => {
        for (const [id, level] of [
            ["kjerag", "nation"],
            ["rhodes", "nation"],
            ["karlan", "group"],
            ["rainbow", "team"],
        ] as const) {
            const sets = toFilterSets(level === "nation" ? { ...EMPTY_SHARED_FILTERS, nations: [id] } : { ...EMPTY_SHARED_FILTERS, factions: [id] });
            expect(factionMembers(INDEX, { id, powerLevel: level })).toHaveLength(INDEX.filter((o) => matchesSharedFilters(o, sets)).length);
        }
    });
});

describe("rosterAccess", () => {
    const roster = [{ operator_id: "char_a" }] as IRosterEntry[];
    it("is private when the profile never asks or the backend refused", () => {
        expect(rosterAccess(false, roster)).toEqual({ state: "private" });
        expect(rosterAccess(true, null)).toEqual({ state: "private" });
    });
    it("is loading until the rows arrive, then ready", () => {
        expect(rosterAccess(true, undefined)).toEqual({ state: "loading" });
        expect(rosterAccess(true, roster)).toEqual({ state: "ready", roster });
        expect(rosterById(roster).has("char_a")).toBe(true);
        expect(rosterById(roster).has("char_b")).toBe(false);
    });
});
