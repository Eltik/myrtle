import { describe, expect, it } from "vitest";
import type { Cut } from "#/types/generated/Cut";
import type { EconomySeat } from "#/types/generated/EconomySeat";
import { baseDpFromLine, clampStartDp, countOpeners, cutAmountText, cutLabelInput, dpFromPointer, fractionOf, ladderScale, noteChangeText, respawnRange, squadRespawnCut, startBonusTotal } from "./economyMath";

const potentialCut: Cut = { kind: "potential", name: null, from_seat: null, amount: -3, ratio: null };
const talentCut: Cut = { kind: "talent", name: "Lightweight", from_seat: null, amount: -1, ratio: null };
const moduleCut: Cut = { kind: "module", name: "Ever-Vigilant", from_seat: null, amount: -1, ratio: null };
const squadCostCut: Cut = { kind: "squad", name: "Spearheader", from_seat: 3, amount: -1, ratio: null };
const silverAsh: Cut = { kind: "squad", name: "Tactical Coordination", from_seat: 5, amount: 0, ratio: -0.12 };

function seat(seatIndex: number, cost: number, respawn: number, respawn_cuts: Cut[] = []): EconomySeat {
    return { seat: seatIndex, char_id: `char_${seatIndex}`, support: false, cost, base_cost: cost + 2, cuts: [], respawn, base_respawn: respawn, respawn_cuts, first_respawn: null, first_respawn_cuts: [] };
}

// Texas, Myrtle, Saria, Exusiai, SilverAsh, Eyjafjalla at E2 after potential.
const seats: EconomySeat[] = [seat(1, 9, 18), seat(0, 10, 70), seat(2, 18, 70), seat(3, 12, 70), seat(5, 19, 70, [silverAsh]), seat(4, 21, 70)];

describe("ladder scale", () => {
    it("is the dearest seat, never below 1", () => {
        expect(ladderScale(seats.map((s) => s.cost))).toBe(21);
        expect(ladderScale([])).toBe(1);
        expect(ladderScale([0, 0])).toBe(1);
    });
    it("clamps bar and line fractions to the track", () => {
        expect(fractionOf(21, 21)).toBe(1);
        expect(fractionOf(7, 14)).toBe(0.5);
        expect(fractionOf(30, 21)).toBe(1);
        expect(fractionOf(-4, 21)).toBe(0);
        expect(fractionOf(5, 0)).toBe(0);
    });
});

describe("line position and DP", () => {
    it("maps a pointer x to an integer DP over the track", () => {
        expect(dpFromPointer(0, 300, 21)).toBe(0);
        expect(dpFromPointer(300, 300, 21)).toBe(21);
        expect(dpFromPointer(150, 300, 21)).toBe(11);
        expect(dpFromPointer(-40, 300, 21)).toBe(0);
        expect(dpFromPointer(900, 300, 21)).toBe(21);
        expect(dpFromPointer(10, 0, 21)).toBe(0);
    });
    it("round-trips a DP through its line position", () => {
        const width = 420;
        const scale = 21;
        for (let dp = 0; dp <= scale; dp++) expect(dpFromPointer(fractionOf(dp, scale) * width, width, scale)).toBe(dp);
    });
    it("takes the squad bonus back off a dragged line and clamps to the stepper", () => {
        expect(baseDpFromLine(14, 4)).toBe(10);
        expect(baseDpFromLine(2, 4)).toBe(0);
        expect(baseDpFromLine(140, 0)).toBe(99);
        expect(clampStartDp(12.6)).toBe(13);
        expect(clampStartDp(Number.NaN)).toBe(0);
    });
    it("sums start bonuses", () => {
        expect(startBonusTotal([])).toBe(0);
        expect(
            startBonusTotal([
                { seat: 2, char_id: "char_2", name: "Logistics", amount: 2 },
                { seat: 7, char_id: "char_7", name: "Prep", amount: 3 },
            ]),
        ).toBe(5);
    });
});

describe("openers", () => {
    it("counts seats at or under the starting DP", () => {
        const costs = seats.map((s) => s.cost);
        expect(countOpeners(costs, 10)).toBe(2);
        expect(countOpeners(costs, 9)).toBe(1);
        expect(countOpeners(costs, 8)).toBe(0);
        expect(countOpeners(costs, 99)).toBe(6);
    });
    it("lets a DP floor below zero open dearer seats", () => {
        const costs = seats.map((s) => s.cost);
        // Closure's "DP lower limit reduced by 4": a 10-cost seat deploys from 6.
        expect(countOpeners(costs, 6, -4)).toBe(2);
        expect(countOpeners([3], 0, -4)).toBe(1);
        expect(countOpeners([3], 0, 0)).toBe(0);
    });
});

describe("redeploy", () => {
    it("gives the shortest and longest time", () => {
        expect(respawnRange(seats.map((s) => s.respawn))).toEqual({ min: 18, max: 70 });
        expect(respawnRange([70, 70])).toEqual({ min: 70, max: 70 });
        expect(respawnRange([])).toBeNull();
    });
    it("finds the first squad cut that scales", () => {
        expect(squadRespawnCut(seats)).toBe(silverAsh);
        expect(squadRespawnCut([seat(0, 10, 70, [{ ...squadCostCut, amount: -4 }])])).toBeNull();
        expect(squadRespawnCut([])).toBeNull();
    });
});

describe("cut labels", () => {
    it("signs flat and scaled amounts", () => {
        expect(cutAmountText(potentialCut)).toBe("-3");
        expect(cutAmountText({ amount: 2, ratio: null })).toBe("+2");
        expect(cutAmountText(silverAsh)).toBe("-12%");
        expect(cutAmountText({ amount: 0, ratio: 0.1 })).toBe("+10%");
    });
    it("carries the sentence, name and giver", () => {
        expect(cutLabelInput(potentialCut, null)).toEqual({ key: "potential", name: "", giver: "", amount: "-3" });
        expect(cutLabelInput(talentCut, null)).toEqual({ key: "talent", name: "Lightweight", giver: "", amount: "-1" });
        expect(cutLabelInput(moduleCut, null).key).toBe("module");
        expect(cutLabelInput(squadCostCut, "Zima")).toEqual({ key: "squad", name: "Spearheader", giver: "Zima", amount: "-1" });
    });
});

describe("note changes", () => {
    it("signs the cost or redeploy change a note carries", () => {
        expect(noteChangeText({ stat: "cost", amount: -1, ratio: null })).toEqual({ stat: "cost", amount: "-1" });
        expect(noteChangeText({ stat: "respawn", amount: 0, ratio: -0.1 })).toEqual({ stat: "respawn", amount: "-10%" });
        expect(noteChangeText({ stat: null, amount: 2, ratio: null })).toBeNull();
    });
});
