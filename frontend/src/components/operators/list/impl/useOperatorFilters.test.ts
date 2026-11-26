import { describe, expect, it } from "vitest";
import { parseStoredFilters } from "./useOperatorFilters";

describe("parseStoredFilters", () => {
    it("keeps an availability still offered", () => {
        expect(parseStoredFilters(JSON.stringify({ availability: "upcoming" })).availability).toBe("upcoming");
        expect(parseStoredFilters(JSON.stringify({ availability: "global" })).availability).toBe("global");
    });

    it("moves the retired Updated on CN choice to Upcoming, which lists those operators now", () => {
        const parsed = parseStoredFilters(JSON.stringify({ availability: "updated", rarities: [6], searchQuery: "shu" }));
        expect(parsed.availability).toBe("upcoming");
        expect(parsed.rarities).toEqual([6]);
        expect(parsed.searchQuery).toBe("shu");
    });

    it("sends any other unknown availability, or none, to the default", () => {
        expect(parseStoredFilters(JSON.stringify({ availability: "nonsense" })).availability).toBe("global");
        expect(parseStoredFilters(JSON.stringify({ availability: null })).availability).toBe("global");
        expect(parseStoredFilters("{}").availability).toBe("global");
    });
});
