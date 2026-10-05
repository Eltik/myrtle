import { describe, expect, it } from "vitest";
import type { Resolution } from "#/types/generated/Resolution";
import { EMPTY_STATE, enEnded, type IPlanRow, type IPlanSkin, type IPlanState, mergeSameDay, planGroups, rowDeviates, rowIncome, stageKey, stageOn } from "./plan";

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

const skin = (skinId: string, groupName: string, brand: string, rerun: boolean, anchored = false) => ({ skinId, groupName, groupNameAuto: null, brand, rerun, anchored }) as unknown as IPlanSkin;
const card = (key: string, enStart: number, skins: IPlanSkin[]) => ({ key, enStart, skins: [...skins] }) as unknown as IPlanRow;
// 2026-10-05 12:00 local, and the same day's evening.
const NOON = new Date(2026, 9, 5, 12).getTime() / 1000;
const EVENING = NOON + 8 * 3600;

describe("one card per release day", () => {
    it("folds a same-day sale into the event and keeps the next day's apart", () => {
        const event = card("event:act4mainss", NOON, [skin("a", "斗争血脉/XI", "boc", false, true)]);
        const sameDay = card("sale:same", EVENING, [skin("b", "Iteration Provident", "iteration", true), skin("a", "斗争血脉/XI", "boc", false)]);
        const nextDay = card("sale:next", NOON + 86_400, [skin("c", "EPOQUE/VII", "epoque", true)]);
        const left = mergeSameDay([sameDay, nextDay], [event], []);
        expect(left.map((r) => r.key)).toEqual(["sale:next"]);
        expect(event.skins.map((s) => [s.skinId, s.anchored])).toEqual([
            ["a", true],
            ["b", false],
        ]);
    });

    it("folds a sale into a same-day review when no event opens that day", () => {
        const review = card("review:1", NOON, []);
        expect(mergeSameDay([card("sale:x", EVENING, [skin("b", "g", "g", true)])], [], [review])).toEqual([]);
        expect(review.skins).toHaveLength(1);
    });
});

describe("set headings in a card", () => {
    it("puts the event's own sets first and a series' editions together, new before rerun", () => {
        const groups = planGroups([skin("x", "EPOQUE/XXII", "epoque", true), skin("m1", "Monster Hunter", "mh", true), skin("k", "成就之星/X", "game", false, true), skin("m2", "怪物猎人/II", "mh", false), skin("m3", "怪物猎人/II", "mh", false)]);
        expect(groups.map((g) => [g.name, g.skins.length])).toEqual([
            ["成就之星/X", 1],
            ["EPOQUE/XXII", 1],
            ["怪物猎人/II", 2],
            ["Monster Hunter", 1],
        ]);
    });
});
