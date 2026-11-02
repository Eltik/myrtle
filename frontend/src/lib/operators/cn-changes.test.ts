import { describe, expect, it } from "vitest";
import type { IOperatorListItem } from "#/types/operators";
import { blackboardDiff, cnChangesById, summarizeChanges, withCnAdditions } from "./cn-changes";

const bb = (rows: [string, number][]) => rows.map(([key, value]) => ({ key, value, valueStr: null }));

describe("cnChangesById", () => {
    it("indexes by operator id and drops rows with no changes", () => {
        const map = cnChangesById([
            { id: "char_2025_shu", changes: [{ kind: "module_added", moduleId: "uniequip_003_shu", translations: [] }] },
            { id: "char_empty", changes: [] },
        ]);
        expect([...map.keys()]).toEqual(["char_2025_shu"]);
        expect(cnChangesById(undefined).size).toBe(0);
    });
});

describe("summarizeChanges", () => {
    it("counts each kind once, in the fixed kind order", () => {
        expect(summarizeChanges([{ kind: "stats_changed" }, { kind: "module_added", moduleId: "a", translations: [] }, { kind: "skill_values_changed", skillId: "s1" }, { kind: "module_added", moduleId: "b", translations: [] }])).toEqual([
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

describe("withCnAdditions", () => {
    const op = (modules: string[], skills: string[]) => ({ modules: modules.map((uniEquipId) => ({ uniEquipId })), skills: skills.map((skillId) => ({ skillId })) }) as unknown as IOperatorListItem;
    const en = op(["uniequip_001_shu", "uniequip_002_shu"], ["skchr_shu_1"]);
    const cn = op(["uniequip_001_shu", "uniequip_002_shu", "uniequip_003_shu"], ["skchr_shu_1", "skchr_shu_2"]);

    it("appends CN's added modules and skills after this server's own, from CN's record", () => {
        const merged = withCnAdditions(en, cn, [
            { kind: "module_added", moduleId: "uniequip_003_shu", translations: [] },
            { kind: "skill_added", skillId: "skchr_shu_2" },
        ]);
        expect(merged.operator.modules.map((m) => m.uniEquipId)).toEqual(["uniequip_001_shu", "uniequip_002_shu", "uniequip_003_shu"]);
        expect(merged.operator.skills.map((s) => s.skillId)).toEqual(["skchr_shu_1", "skchr_shu_2"]);
        expect([...merged.moduleIds]).toEqual(["uniequip_003_shu"]);
        expect([...merged.skillIds]).toEqual(["skchr_shu_2"]);
    });

    it("returns this server's record itself when nothing was added, or the addition is already here", () => {
        expect(withCnAdditions(en, cn, [{ kind: "stats_changed" }]).operator).toBe(en);
        const merged = withCnAdditions(en, cn, [{ kind: "module_added", moduleId: "uniequip_002_shu", translations: [] }]);
        expect(merged.operator).toBe(en);
        expect(merged.moduleIds.size).toBe(0);
    });
});
