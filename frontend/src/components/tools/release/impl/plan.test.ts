import { describe, expect, it } from "vitest";
import { EMPTY_STATE, type IPlanRow, type IPlanState, rowIncome, stageKey, stageOn } from "./plan";

/** 奇象巡展's shape: two different stages share the code EE-01. */
const row = {
    key: "event:act1arkhub",
    rerun: false,
    opStages: [
        { stageId: "act1arkhub_01", code: "EE-01", op: 1, challenge: false },
        { stageId: "act1arkhub_15", code: "EE-01", op: 1, challenge: false },
    ],
} as IPlanRow;
const [first, second] = row.opStages;

describe("stage picks", () => {
    it("keys stages that share a code apart", () => {
        expect(stageKey(first)).not.toBe(stageKey(second));
        const state: IPlanState = { ...EMPTY_STATE, stages: { [row.key]: { [stageKey(first)]: false } } };
        expect(stageOn(row, first, state, null)).toBe(false);
        expect(stageOn(row, second, state, null)).toBe(true);
        expect(rowIncome(row, state, null)).toBe(1);
    });

    it("still reads a plan saved under the old code keys", () => {
        const state: IPlanState = { ...EMPTY_STATE, stages: { [row.key]: { "EE-01": false } } };
        expect(rowIncome(row, state, null)).toBe(0);
    });

    it("lets a new pick override an old code-keyed one", () => {
        const state: IPlanState = { ...EMPTY_STATE, stages: { [row.key]: { "EE-01": false, [stageKey(second)]: true } } };
        expect(stageOn(row, first, state, null)).toBe(false);
        expect(stageOn(row, second, state, null)).toBe(true);
    });
});
