import { describe, expect, it } from "vitest";
import { editReducer, type IEditState, type IEditTier, isLadderTierColor, nextFallbackTierColor } from "./state";

function tier(id: string, color: string, operatorIds: string[] = []): IEditTier {
    return { id, name: id.toUpperCase(), color, description: "", operatorIds };
}

function stateOf(tiers: IEditTier[]): IEditState {
    return { title: "t", description: "", tiers, operatorById: {}, descriptionByOperatorId: {} };
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
        expect(next.tiers.map((t) => t.operatorIds)).toEqual([["y"], ["x"]]);
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
