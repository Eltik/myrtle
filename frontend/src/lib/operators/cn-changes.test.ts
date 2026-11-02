import { describe, expect, it } from "vitest";
import type { IEnrichedSkill, IOperatorListItem } from "#/types/operators";
import { blackboardDiff, cnChangesById, phaseStatDiffs, skillLevelDiffs, summarizeChanges, talentDiffs } from "./cn-changes";

const bb = (rows: [string, number][]) => rows.map(([key, value]) => ({ key, value, valueStr: null }));

describe("cnChangesById", () => {
    it("indexes by operator id and drops rows with no changes", () => {
        const map = cnChangesById([
            { id: "char_2025_shu", changes: [{ kind: "module_added", moduleId: "uniequip_003_shu" }] },
            { id: "char_empty", changes: [] },
        ]);
        expect([...map.keys()]).toEqual(["char_2025_shu"]);
        expect(cnChangesById(undefined).size).toBe(0);
    });
});

describe("summarizeChanges", () => {
    it("counts each kind once, in the fixed kind order", () => {
        expect(summarizeChanges([{ kind: "stats_changed" }, { kind: "module_added", moduleId: "a" }, { kind: "skill_values_changed", skillId: "s1" }, { kind: "module_added", moduleId: "b" }])).toEqual([
            { kind: "module_added", count: 2 },
            { kind: "skill_values_changed", count: 1 },
            { kind: "stats_changed", count: 1 },
        ]);
    });
});

describe("blackboardDiff", () => {
    it("ignores order, key case and float widening", () => {
        expect(
            blackboardDiff(
                bb([
                    ["atk", 0.3],
                    ["HEAL", 0.2],
                ]),
                bb([
                    ["heal", 0.200000002980232],
                    ["atk", 0.3],
                ]),
            ),
        ).toEqual([]);
    });

    it("reports changed, added and removed keys", () => {
        expect(
            blackboardDiff(
                bb([
                    ["atk", 0.3],
                    ["old", 1],
                ]),
                bb([
                    ["atk", 0.35],
                    ["new", 2],
                ]),
            ),
        ).toEqual([
            { key: "atk", en: 0.3, cn: 0.35 },
            { key: "new", en: null, cn: 2 },
            { key: "old", en: 1, cn: null },
        ]);
    });
});

describe("skillLevelDiffs", () => {
    const skill = (spCost: number, atk: number) =>
        ({
            skillId: "skchr_x_1",
            static: { skillId: "skchr_x_1", levels: [{ name: "n", description: "d", duration: 10, spData: { spType: "INCREASE_WITH_TIME", maxChargeTime: 1, spCost, initSp: 10, increment: 1 }, blackboard: bb([["atk", atk]]) }] },
        }) as unknown as IEnrichedSkill;

    it("lists only the numbers that moved, per 1-based level", () => {
        expect(skillLevelDiffs(skill(30, 0.5), skill(30, 0.5))).toEqual([]);
        expect(skillLevelDiffs(skill(30, 0.5), skill(25, 0.6))).toEqual([
            {
                level: 1,
                values: [
                    { key: "spCost", en: 30, cn: 25 },
                    { key: "atk", en: 0.5, cn: 0.6 },
                ],
            },
        ]);
    });
});

describe("phaseStatDiffs and talentDiffs", () => {
    const op = (respawnTime: number, talentValue: number) =>
        ({
            phases: [
                {
                    attributesKeyFrames: [
                        { level: 1, data: { atk: 100, respawnTime, stunImmune: false } },
                        { level: 50, data: { atk: 300, respawnTime, stunImmune: false } },
                    ],
                },
            ],
            talents: [{ candidates: [{ blackboard: bb([["hp_ratio", talentValue]]) }] }],
        }) as unknown as IOperatorListItem;

    it("compares the max-level key frame and skips non-numeric fields", () => {
        expect(phaseStatDiffs(op(80, 0.1), op(70, 0.1))).toEqual([{ phase: 0, level: 50, values: [{ key: "respawnTime", en: 80, cn: 70 }] }]);
        expect(phaseStatDiffs(op(80, 0.1), op(80, 0.1))).toEqual([]);
    });

    it("locates talent value changes by talent and candidate", () => {
        expect(talentDiffs(op(80, 0.1), op(80, 0.12))).toEqual([{ talent: 0, candidate: 0, values: [{ key: "hp_ratio", en: 0.1, cn: 0.12 }] }]);
    });
});
