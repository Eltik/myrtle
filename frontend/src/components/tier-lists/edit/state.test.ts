import { describe, expect, it } from "vitest";
import { toTierEntity } from "#/lib/api/tier-entities";
import { editReducer, type IEditState, type IEditTier, isLadderTierColor, kindsChanged, nextFallbackTierColor, offeredKinds, placedEntity, planPlacementChanges } from "./state";

function tier(id: string, color: string, entityKeys: string[] = []): IEditTier {
    return { id, name: id.toUpperCase(), color, description: "", entityKeys };
}

function stateOf(tiers: IEditTier[]): IEditState {
    return { title: "t", description: "", tiers, entityByKey: {}, descriptionByKey: {} };
}

const idsOf = (state: IEditState) => state.tiers.map((t) => t.id);
const colorsOf = (state: IEditState) => state.tiers.map((t) => t.color);

const RED = nextFallbackTierColor(0);
const ORANGE = nextFallbackTierColor(1);
const YELLOW = nextFallbackTierColor(2);
const CUSTOM = "#123456";

describe("MOVE_TIER keeps ladder colours on the row", () => {
    it("swaps two ladder colours so the moved tier takes the colour of its new row", () => {
        const next = editReducer(stateOf([tier("s", RED), tier("a", ORANGE), tier("b", YELLOW)]), { type: "MOVE_TIER", tierId: "b", direction: "up" });
        expect(idsOf(next)).toEqual(["s", "b", "a"]);
        expect(colorsOf(next)).toEqual([RED, ORANGE, YELLOW]);
    });

    it("moving a tier down is the mirror of moving its neighbour up", () => {
        const next = editReducer(stateOf([tier("s", RED), tier("a", ORANGE)]), { type: "MOVE_TIER", tierId: "s", direction: "down" });
        expect(idsOf(next)).toEqual(["a", "s"]);
        expect(colorsOf(next)).toEqual([RED, ORANGE]);
    });

    it("a custom colour travels with its tier and leaves the neighbour alone", () => {
        const next = editReducer(stateOf([tier("s", RED), tier("a", CUSTOM)]), { type: "MOVE_TIER", tierId: "a", direction: "up" });
        expect(idsOf(next)).toEqual(["a", "s"]);
        expect(colorsOf(next)).toEqual([CUSTOM, RED]);
    });

    it("two custom colours both travel", () => {
        const next = editReducer(stateOf([tier("s", "#abcdef"), tier("a", CUSTOM)]), { type: "MOVE_TIER", tierId: "s", direction: "down" });
        expect(colorsOf(next)).toEqual([CUSTOM, "#abcdef"]);
    });

    it("operators stay with their tier through the colour swap", () => {
        const next = editReducer(stateOf([tier("s", RED, ["x"]), tier("a", ORANGE, ["y"])]), { type: "MOVE_TIER", tierId: "a", direction: "up" });
        expect(next.tiers.map((t) => t.entityKeys)).toEqual([["y"], ["x"]]);
    });

    it("a move off either end is a no-op", () => {
        const state = stateOf([tier("s", RED), tier("a", ORANGE)]);
        expect(editReducer(state, { type: "MOVE_TIER", tierId: "s", direction: "up" })).toBe(state);
        expect(editReducer(state, { type: "MOVE_TIER", tierId: "a", direction: "down" })).toBe(state);
    });

    it("matches ladder colours case-insensitively and nothing else", () => {
        expect(isLadderTierColor(RED)).toBe(true);
        expect(isLadderTierColor(RED.toUpperCase())).toBe(true);
        expect(isLadderTierColor(CUSTOM)).toBe(false);
    });
});

describe("planPlacementChanges diffs on (kind, id)", () => {
    const summarize = (original: IEditState, current: IEditState) =>
        planPlacementChanges(original, current).map((c) => {
            switch (c.op) {
                case "remove":
                    return `remove ${c.kind}/${c.id}`;
                case "move":
                    return `move ${c.kind}/${c.id} -> ${c.tier.id}#${c.subOrder}`;
                case "add":
                    return `add ${c.kind}/${c.id} -> ${c.tier.id}#${c.subOrder} "${c.description}"`;
                default:
                    return `describe ${c.kind}/${c.id} "${c.description}"`;
            }
        });

    it("nothing changed is nothing to save", () => {
        const state = stateOf([tier("s", RED, ["operator:a", "operator:b"])]);
        expect(planPlacementChanges(state, state)).toEqual([]);
    });

    it("splits each key back into its kind and id, an id holding a colon included", () => {
        const original = stateOf([tier("s", RED, ["operator:char_002_amiya"])]);
        const current = stateOf([tier("s", RED, ["operator:x:y"])]);
        expect(summarize(original, current)).toEqual(["remove operator/char_002_amiya", 'add operator/x:y -> s#0 ""']);
    });

    it("removes, then moves and reorders, then adds, then re-describes", () => {
        const original: IEditState = { ...stateOf([tier("s", RED, ["operator:a", "operator:b", "operator:c"]), tier("a", ORANGE, ["operator:d"])]), descriptionByKey: { "operator:d": "old" } };
        const current: IEditState = {
            ...stateOf([tier("s", RED, ["operator:b", "operator:a"]), tier("a", ORANGE, ["operator:d", "operator:e", "operator:c"])]),
            descriptionByKey: { "operator:d": "new", "operator:e": "  fresh  " },
        };
        expect(summarize(original, current)).toEqual(["move operator/a -> s#1", "move operator/b -> s#0", "move operator/c -> a#2", 'add operator/e -> a#1 "fresh"', 'describe operator/d "new"']);
    });

    it("a placement whose original tier is deleted is re-added with its note, not moved", () => {
        const original: IEditState = { ...stateOf([tier("b", RED, ["operator:a", "operator:z"]), tier("c", ORANGE)]), descriptionByKey: { "operator:a": "kept" } };
        const current: IEditState = { ...stateOf([tier("c", ORANGE, ["operator:a"])]), descriptionByKey: { "operator:a": "kept" } };
        expect(summarize(original, current)).toEqual(["remove operator/z", 'add operator/a -> c#0 "kept"']);
    });

    it("a placement in a draft tier always moves, even at the same index", () => {
        const original = stateOf([tier("s", RED, ["operator:a"])]);
        const current = stateOf([tier("draft_x_1", RED, ["operator:a"]), tier("s", ORANGE)]);
        expect(summarize(original, current)).toEqual(["move operator/a -> draft_x_1#0"]);
        // With its old tier deleted, the same placement is re-added instead.
        expect(summarize(original, stateOf([tier("draft_x_1", RED, ["operator:a"])]))).toEqual(['add operator/a -> draft_x_1#0 ""']);
    });
});

describe("SET_ENTITY_KINDS", () => {
    it("a state from before kinds were editable offers operators", () => {
        expect(offeredKinds(stateOf([]))).toEqual(["operator"]);
    });

    it("sets the offered kinds, deduplicated, in the order given", () => {
        const next = editReducer(stateOf([]), { type: "SET_ENTITY_KINDS", kinds: ["operator", "enemy", "operator"] });
        expect(offeredKinds(next)).toEqual(["operator", "enemy"]);
        expect(kindsChanged(stateOf([]), next)).toBe(true);
    });

    it("refuses an empty set, which the backend would refuse too", () => {
        const before = stateOf([]);
        expect(editReducer(before, { type: "SET_ENTITY_KINDS", kinds: [] })).toBe(before);
    });

    it("dropping a kind leaves its placements on the board", () => {
        const before = { ...stateOf([tier("s", RED, ["operator:char_002_amiya", "enemy:enemy_1007_slime"])]), entityKinds: ["operator" as const, "enemy" as const] };
        const next = editReducer(before, { type: "SET_ENTITY_KINDS", kinds: ["operator"] });
        expect(next.tiers[0]?.entityKeys).toEqual(["operator:char_002_amiya", "enemy:enemy_1007_slime"]);
        expect(planPlacementChanges(before, next)).toEqual([]);
    });

    it("an entity placed this session still resolves after its kind is unticked", () => {
        const slime = toTierEntity("enemy", "enemy_1007_slime", { kind: "enemy", id: "enemy_1007_slime", name: "Originium Slug", icon: null, href: null, facets: {} }, { subOrder: 0, description: null, updatedAt: new Date(0).toISOString() });
        const before = { ...stateOf([tier("s", RED)]), entityKinds: ["operator" as const, "enemy" as const] };
        const placed = editReducer(before, { type: "PLACE_ENTITY", key: slime.key, tierId: "s", entity: slime });
        const next = editReducer(placed, { type: "SET_ENTITY_KINDS", kinds: ["operator"] });
        expect(next.tiers[0]?.entityKeys).toEqual([slime.key]);
        expect(placedEntity(next.entityByKey, slime.key)).toBe(slime);
    });

    it("a key nothing resolves renders as the unresolved placeholder, never as no tile", () => {
        const entity = placedEntity({}, "enemy:enemy_9999_gone");
        expect(entity).toMatchObject({ key: "enemy:enemy_9999_gone", kind: "enemy", id: "enemy_9999_gone", resolved: false });
    });

    it("a reorder of the same kinds is a change: it is the tab order", () => {
        const a = { ...stateOf([]), entityKinds: ["operator" as const, "enemy" as const] };
        const b = { ...stateOf([]), entityKinds: ["enemy" as const, "operator" as const] };
        expect(kindsChanged(a, b)).toBe(true);
        expect(kindsChanged(a, { ...a })).toBe(false);
    });
});
