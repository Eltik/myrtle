import { describe, expect, it } from "vitest";
import type { Resolution } from "#/types/generated/Resolution";
import { EMPTY_STATE, enEnded, type IPlanRow, type IPlanState, rowDeviates, rowIncome, stageKey, stageOn } from "./plan";

/** 奇象巡展's shape: two different stages share the code EE-01. */
const row = {
    key: "event:act1arkhub",
    rerun: false,
    ended: false,
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

describe("an event EN has closed", () => {
    const ended = { ...row, ended: true } as IPlanRow;
    const SEP_30_END = 1790765999; // 2026-09-30 10:59:59 UTC, an EN close

    it("counts none of its stages, ticked or not", () => {
        const state: IPlanState = { ...EMPTY_STATE, stages: { [row.key]: { [stageKey(first)]: true } } };
        expect(rowIncome(row, EMPTY_STATE, null)).toBe(2);
        expect(rowIncome(ended, EMPTY_STATE, null)).toBe(0);
        expect(rowIncome(ended, state, null)).toBe(0);
        expect(stageOn(ended, first, state, null)).toBe(false);
        expect(rowDeviates(ended, state, null)).toBe(false);
    });

    it("closes on a known EN end only", () => {
        const confirmed: Resolution = { status: "confirmed", enId: "x", enStart: SEP_30_END - 14 * 86_400, enEnd: SEP_30_END };
        expect(enEnded(confirmed, SEP_30_END - 1)).toBe(false);
        expect(enEnded(confirmed, SEP_30_END)).toBe(true);
        expect(enEnded({ status: "override", enId: null, enStart: 0, enEnd: null, source: "", note: "" }, SEP_30_END)).toBe(false);
        expect(enEnded({ status: "estimated", enStart: 0, lo: 0, hi: 0 }, SEP_30_END)).toBe(false);
    });
});
