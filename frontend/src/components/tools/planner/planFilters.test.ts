import { describe, expect, it } from "vitest";

import { activePlansForFilter, groupFilterCounts, isPlanActive, matchesGroupFilter, renameGroupFilterKey, toggleGroupFilterKey, UNGROUPED_FILTER_KEY } from "./planFilters";

const plans = [{ operator_id: "a", groups: ["Farm"] }, { operator_id: "b", groups: ["Farm", "IS"] }, { operator_id: "c", groups: ["IS"] }, { operator_id: "d", groups: [] }, { operator_id: "e" }];

describe("matchesGroupFilter", () => {
    it("passes everything when nothing is selected", () => {
        expect(plans.every((p) => matchesGroupFilter(p, new Set()))).toBe(true);
    });

    it("keeps ungrouped plans out of a single-group filter", () => {
        const farm = new Set(["Farm"]);
        expect(plans.filter((p) => matchesGroupFilter(p, farm)).map((p) => p.operator_id)).toEqual(["a", "b"]);
    });

    it("selects plans with no groups only through the Ungrouped key", () => {
        const ungrouped = new Set([UNGROUPED_FILTER_KEY]);
        expect(plans.filter((p) => matchesGroupFilter(p, ungrouped)).map((p) => p.operator_id)).toEqual(["d", "e"]);
        const mixed = new Set(["IS", UNGROUPED_FILTER_KEY]);
        expect(plans.filter((p) => matchesGroupFilter(p, mixed)).map((p) => p.operator_id)).toEqual(["b", "c", "d", "e"]);
    });
});

describe("activePlansForFilter", () => {
    it("writes an entry for every plan", () => {
        expect(activePlansForFilter(plans, new Set(["IS"]))).toEqual({ a: false, b: true, c: true, d: false, e: false });
        expect(activePlansForFilter(plans, new Set())).toEqual({ a: true, b: true, c: true, d: true, e: true });
    });
});

describe("isPlanActive", () => {
    it("prefers an explicit entry and falls back to the filter", () => {
        const filter = new Set(["Farm"]);
        expect(isPlanActive(plans[0], { a: false }, filter)).toBe(false);
        expect(isPlanActive(plans[3], { d: true }, filter)).toBe(true);
        expect(isPlanActive({ operator_id: "new", groups: [] }, {}, filter)).toBe(false);
        expect(isPlanActive({ operator_id: "new", groups: [] }, {}, new Set())).toBe(true);
    });
});

describe("filter key edits", () => {
    it("toggles, renames and drops keys", () => {
        expect([...toggleGroupFilterKey(new Set(["Farm"]), "IS")]).toEqual(["Farm", "IS"]);
        expect([...toggleGroupFilterKey(new Set(["Farm"]), "Farm")]).toEqual([]);
        expect([...renameGroupFilterKey(new Set(["Farm"]), "Farm", "Farming")]).toEqual(["Farming"]);
        expect([...renameGroupFilterKey(new Set(["Farm", "IS"]), "Farm", null)]).toEqual(["IS"]);
        const untouched = new Set(["IS"]);
        expect(renameGroupFilterKey(untouched, "Farm", "Farming")).toBe(untouched);
    });
});

describe("groupFilterCounts", () => {
    it("counts a plan under each of its groups and the groupless under Ungrouped", () => {
        expect(Object.fromEntries(groupFilterCounts(plans))).toEqual({ Farm: 2, IS: 2, [UNGROUPED_FILTER_KEY]: 2 });
    });
});
