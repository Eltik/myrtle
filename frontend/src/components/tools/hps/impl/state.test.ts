import { describe, expect, it } from "vitest";
import { createInstance } from "#/components/tools/shared/instance";
import { INITIAL_STATE, instanceToRequest } from "./state";

const op = { id: "char_m", name: "M", availableSkills: [1, 2], availableModules: [0], availableModuleIds: [""], defaultSkill: 2, defaultModule: 0, conditionals: [] };

describe("HPS request", () => {
    it("sends ASPD and flat ATK as integers (i32 on the backend), whatever the sweep lands on", () => {
        const inst = createInstance(op, "x");
        const swept = instanceToRequest(inst, INITIAL_STATE.buffs, { aspd: 3.25 });
        expect(swept.buffs?.aspd).toBe(3);
        const typed = instanceToRequest(inst, { ...INITIAL_STATE.buffs, buffs: { aspd: 12.6, flatAtk: 99.5 } });
        expect(typed.buffs).toEqual({ aspd: 13, flatAtk: 100 });
        expect(Number.isInteger(typed.targets)).toBe(true);
    });

    it("keeps the ATK percent fractional (f32 on the backend)", () => {
        expect(instanceToRequest(createInstance(op, "x"), INITIAL_STATE.buffs, { atkPercent: 12.5 }).buffs?.atk).toBe(0.125);
    });
});
