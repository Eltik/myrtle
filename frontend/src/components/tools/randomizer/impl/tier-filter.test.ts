import { describe, expect, it } from "vitest";
import type { ITierListBrowseItem, ITierListDetail } from "#/lib/api/tier-lists";
import { DEFAULT_SETTINGS } from "./constants";
import { buildTierIndex, type ITierIndex, officialTierChoices, passesTierFilter, resolveTierListSlug, tierFilterNarrows, UNRANKED_TIER_ID } from "./tier-filter";
import type { IRandomizerOperator, IRandomizerSettings } from "./types";
import { buildRosterIndex, selectAvailableOperators } from "./utils";

const entity = (kind: string, id: string) => ({ kind, id });
const detail = {
    slug: "general",
    title: "General",
    tiers: [
        { id: "t-s", name: "S", entities: [entity("operator", "op_a"), entity("operator", "op_b")] },
        // op_b also sits lower; the higher placement wins. The enemy is not an operator.
        { id: "t-a", name: "A", entities: [entity("operator", "op_b"), entity("operator", "op_c"), entity("enemy", "op_d")] },
    ],
} as unknown as ITierListDetail;
const index = buildTierIndex(detail);
const ops = ["op_a", "op_b", "op_c", "op_d"].map((id) => ({ id, rarity: 6, profession: "CASTER" })) as IRandomizerOperator[];
const draw = (deselectedTierIds: string[], idx: ITierIndex | null = index) => selectAvailableOperators(ops, { ...DEFAULT_SETTINGS, tierListSlug: "general", deselectedTierIds } as IRandomizerSettings, buildRosterIndex(null), idx).map((o) => o.id);

describe("tier filter", () => {
    it("keeps the list's own tier order and places each operator once, in its highest tier", () => {
        expect(index.tiers.map((t) => t.name)).toEqual(["S", "A"]);
        expect(index.tierByOperator.get("op_b")).toBe("t-s");
        expect(index.tierByOperator.has("op_d")).toBe(false);
    });

    it("filters nothing until a tier is deselected", () => {
        expect(draw([])).toEqual(["op_a", "op_b", "op_c", "op_d"]);
    });

    it("drops deselected tiers, and Unranked covers operators the list does not place", () => {
        expect(draw(["t-s"])).toEqual(["op_c", "op_d"]);
        expect(draw([UNRANKED_TIER_ID])).toEqual(["op_a", "op_b", "op_c"]);
        expect(draw(["t-a", UNRANKED_TIER_ID])).toEqual(["op_a", "op_b"]);
    });

    it("is off while the list is missing (none chosen, loading, failed)", () => {
        expect(draw(["t-s"], null)).toEqual(["op_a", "op_b", "op_c", "op_d"]);
        expect(passesTierFilter("op_a", null, ["t-s"])).toBe(true);
    });

    it("treats a stale deselection (a tier since deleted) as not narrowing", () => {
        expect(tierFilterNarrows(index, ["t-gone"])).toBe(false);
        expect(draw(["t-gone"])).toHaveLength(4);
        expect(tierFilterNarrows(index, [UNRANKED_TIER_ID])).toBe(true);
    });

    it("offers official lists only, and reads a stored slug that is no longer one as none", () => {
        const lists = [
            { slug: "general", title: "General", listType: "official" },
            { slug: "mine", title: "Mine", listType: "community" },
        ] as ITierListBrowseItem[];
        const choices = officialTierChoices(lists);
        expect(choices).toEqual([{ slug: "general", title: "General" }]);
        expect(resolveTierListSlug("general", choices)).toBe("general");
        expect(resolveTierListSlug("mine", choices)).toBeNull();
        expect(resolveTierListSlug("deleted", choices)).toBeNull();
        expect(resolveTierListSlug(null, choices)).toBeNull();
        expect(officialTierChoices(undefined)).toEqual([]);
    });
});
