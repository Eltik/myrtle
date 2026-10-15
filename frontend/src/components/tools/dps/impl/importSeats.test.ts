import { describe, expect, it } from "vitest";
import type { IDpsImportSeat } from "#/components/tools/squad-builder/model";
import { importSeatToConfig, seatModuleIndex, seatSkillRank } from "./importSeats";

const op = {
    availableSkills: [1, 2, 3],
    availableModules: [0, 1, 2],
    availableModuleIds: ["uniequip_001_x", "uniequip_002_x", "uniequip_003_x"],
    defaultSkill: 3,
};

function seat(patch: Partial<IDpsImportSeat> = {}): IDpsImportSeat {
    return { charId: "char_x", phase: 2, level: 90, potential: 0, skillIndex: 1, skillLevel: 7, mastery: 3, moduleId: "uniequip_003_x", moduleLevel: 2, buffs: { atk: 0, flatAtk: 0, aspd: 0 }, ...patch };
}

describe("importSeatToConfig", () => {
    it("folds skill level and mastery into one rank", () => {
        expect(seatSkillRank(7, 0)).toBe(7);
        expect(seatSkillRank(7, 1)).toBe(8);
        expect(seatSkillRank(7, 3)).toBe(10);
        expect(seatSkillRank(4, 0)).toBe(4);
    });

    it("shifts potential and skill to 1-based", () => {
        const cfg = importSeatToConfig(seat({ potential: 5, skillIndex: 0 }), op);
        expect(cfg.potential).toBe(6);
        expect(cfg.skillIndex).toBe(1);
        expect(importSeatToConfig(seat({ potential: 0 }), op).potential).toBe(1);
    });

    it("falls back to the default skill when the engine lacks the seat's", () => {
        expect(importSeatToConfig(seat({ skillIndex: 2 }), { ...op, availableSkills: [1, 2] }).skillIndex).toBe(3);
    });

    it("resolves the module id to the engine's module number", () => {
        expect(seatModuleIndex(op, "uniequip_003_x")).toBe(2);
        expect(seatModuleIndex(op, "uniequip_009_x")).toBe(0);
        expect(seatModuleIndex(op, null)).toBe(0);
        const cfg = importSeatToConfig(seat(), op);
        expect(cfg).toMatchObject({ promotion: 2, level: 90, skillRank: 10, moduleIndex: 2, moduleLevel: 2 });
        const noModule = importSeatToConfig(seat({ moduleId: null }), op);
        expect(noModule).toMatchObject({ moduleIndex: 0 });
        expect(noModule.moduleLevel).toBeUndefined();
    });

    it("carries only the buffs that are set", () => {
        expect(importSeatToConfig(seat({ buffs: { atk: 0.16, flatAtk: 0, aspd: 12 } }), op).buffs).toEqual({ atk: 0.16, aspd: 12 });
        expect(importSeatToConfig(seat(), op).buffs).toEqual({});
    });
});
