import { describe, expect, it } from "vitest";
import type { IOperatorPlanResponse, IPlanRequirementItem } from "#/lib/api/planner";
import { formatPlanTarget, formatSubtotal, inCategory, inStatus, parseMaxTier, requirementCategory, requirementStatus, subtotal } from "./requirements";

function req(partial: Partial<IPlanRequirementItem>): IPlanRequirementItem {
    return { id: "30012", name: "Orirock Cube", requiredCount: 0, inventoryCount: 0, missingCount: 0, sortGroup: 5, ...partial } as IPlanRequirementItem;
}

function plan(partial: Partial<IOperatorPlanResponse>): IOperatorPlanResponse {
    return { target_elite: 2, target_level: 60, target_skill_level: 7, target_skills: [], target_modules: [], ...partial } as unknown as IOperatorPlanResponse;
}

describe("requirementStatus", () => {
    it("is complete when inventory covers the requirement", () => {
        expect(requirementStatus(req({ requiredCount: 5, inventoryCount: 5 }))).toEqual({ status: "complete", shortfall: 0 });
        expect(requirementStatus(req({ requiredCount: 5, inventoryCount: 9 }))).toEqual({ status: "complete", shortfall: 0 });
    });

    it("is craft when short but nothing is missing", () => {
        expect(requirementStatus(req({ requiredCount: 5, inventoryCount: 2 }))).toEqual({ status: "craft", shortfall: 3 });
    });

    it("is missing whenever anything is unbuildable, whatever the inventory", () => {
        expect(requirementStatus(req({ requiredCount: 5, inventoryCount: 1, missingCount: 2 }))).toEqual({ status: "missing", shortfall: 4 });
        expect(requirementStatus(req({ requiredCount: 5, inventoryCount: 5, missingCount: 1 }))).toEqual({ status: "missing", shortfall: 0 });
    });
});

describe("requirementCategory", () => {
    it("maps sort groups onto the category chips", () => {
        expect([0, 1, 2, 3, 4, 5].map((sortGroup) => requirementCategory(req({ sortGroup })))).toEqual(["expLmd", "expLmd", "skills", "chips", "modules", "materials"]);
    });

    it("files unexpected groups as materials", () => {
        expect(requirementCategory(req({ sortGroup: 42 }))).toBe("materials");
        expect(requirementCategory(req({ sortGroup: -1 }))).toBe("materials");
    });
});

describe("filters", () => {
    const chip = req({ sortGroup: 3, requiredCount: 2, inventoryCount: 0 });

    it("match all or the item's own category and status", () => {
        expect(inCategory(chip, "all")).toBe(true);
        expect(inCategory(chip, "chips")).toBe(true);
        expect(inCategory(chip, "skills")).toBe(false);
        expect(inStatus(chip, "all")).toBe(true);
        expect(inStatus(chip, "craft")).toBe(true);
        expect(inStatus(chip, "missing")).toBe(false);
    });
});

describe("subtotal and formatSubtotal", () => {
    const items = [req({ requiredCount: 3, inventoryCount: 3 }), req({ requiredCount: 3, inventoryCount: 1 }), req({ requiredCount: 3, missingCount: 3 }), req({ requiredCount: 1, missingCount: 1 })];

    it("counts missing and craftable rows", () => {
        expect(subtotal(items)).toEqual({ items: 4, missing: 2, craft: 1 });
        expect(subtotal([])).toEqual({ items: 0, missing: 0, craft: 0 });
    });

    it("leaves out zero parts", () => {
        expect(formatSubtotal({ items: 4, missing: 2, craft: 1 })).toBe("4 items · 2 missing · 1 to craft");
        expect(formatSubtotal({ items: 1, missing: 0, craft: 0 })).toBe("1 item");
        expect(formatSubtotal({ items: 3, missing: 0, craft: 3 })).toBe("3 items · 3 to craft");
    });
});

describe("formatPlanTarget", () => {
    it("lists masteries and modules after the level", () => {
        const p = plan({
            target_skills: [
                { skill_index: 0, mastery_level: 0 },
                { skill_index: 1, mastery_level: 3 },
                { skill_index: 2, mastery_level: 2 },
            ],
            target_modules: [{ module_stage: 3 }, { module_stage: 0 }],
        } as unknown as Partial<IOperatorPlanResponse>);
        expect(formatPlanTarget(p)).toBe("E2 Lv60 · S2 M3 S3 M2 · 1 module");
    });

    it("falls back to the skill level when nothing is mastered", () => {
        expect(formatPlanTarget(plan({ target_skill_level: 7 }))).toBe("E2 Lv60 · SL7");
        expect(formatPlanTarget(plan({ target_elite: 0, target_level: 30, target_skill_level: 1 }))).toBe("E0 Lv30");
    });
});

describe("parseMaxTier", () => {
    it("accepts every tier option and 0 for all", () => {
        expect(["0", "2", "3", "4"].map(parseMaxTier)).toEqual([0, 2, 3, 4]);
    });

    it("rejects anything else", () => {
        for (const raw of ["1", "5", "abc", "2.5", "-4"]) {
            expect(parseMaxTier(raw)).toBeUndefined();
        }
    });

    it("reads an empty or blank string as 0", () => {
        // `Number("")` is 0, so a cleared stored value means every tier.
        expect(parseMaxTier("")).toBe(0);
        expect(parseMaxTier("  ")).toBe(0);
    });
});
