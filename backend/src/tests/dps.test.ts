import { describe, expect, test, beforeAll } from "bun:test";
import { OperatorData } from "../lib/impl/dps-calculator/impl/classes/impl/operator-data";
import { init as initLocal } from "../lib/impl/local";
import { init } from "../lib/impl/dps-calculator";
import operators from "../lib/impl/local/impl/gamedata/impl/operators";
import TwelveF from "../lib/impl/dps-calculator/impl/operators/impl/12f";
import Aak from "../lib/impl/dps-calculator/impl/operators/impl/aak";
import Absinthe from "../lib/impl/dps-calculator/impl/operators/impl/absinthe";
import Aciddrop from "../lib/impl/dps-calculator/impl/operators/impl/aciddrop";
import Adnachiel from "../lib/impl/dps-calculator/impl/operators/impl/adnachiel";
import Andreana from "../lib/impl/dps-calculator/impl/operators/impl/andreana";
import Angelina from "../lib/impl/dps-calculator/impl/operators/impl/angelina";
import Aosta from "../lib/impl/dps-calculator/impl/operators/impl/aosta";

// Initialize data before running tests
beforeAll(async () => {
    await initLocal();
    await init();
});

describe("Operator DPS Calculations", () => {
    describe("12F", () => {
        const baseParams = {};
        const operatorId = "char_009_12fce";

        test("should calculate DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new TwelveF(new OperatorData(operator), baseParams);
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(173);
            expect(dps).toBeLessThan(175);
        });
        test("should calculate DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new TwelveF(new OperatorData(operator), baseParams);
            const dps = unit.skillDPS({ defense: 100, res: 0 });
            expect(dps).toBeGreaterThan(173);
            expect(dps).toBeLessThan(175);
        });
        test("should calculate DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new TwelveF(new OperatorData(operator), baseParams);
            const dps = unit.skillDPS({ defense: 0, res: 10 });
            expect(dps).toBeGreaterThan(156);
            expect(dps).toBeLessThan(158);
        });
    });

    describe("Aak", () => {
        const baseParams = {
            moduleIndex: 1,
        };
        const operatorId = "char_225_haak";

        test("should calculate s1 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aak(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(1629);
            expect(dps).toBeLessThan(1631);
        });
        test("should calculate s1 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aak(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1168);
            expect(dps).toBeLessThan(1170);
        });
        test("should calculate s1 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aak(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(1629);
            expect(dps).toBeLessThan(1631);
        });

        test("should calculate s3 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aak(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 2,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(1833);
            expect(dps).toBeLessThan(1835);
        });
        test("should calculate s3 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aak(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 2,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1487);
            expect(dps).toBeLessThan(1489);
        });
        test("should calculate s3 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aak(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 2,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(1833);
            expect(dps).toBeLessThan(1835);
        });
    });

    describe("Absinthe", () => {
        const baseParams = {
            moduleIndex: 1,
        };
        const operatorId = "char_405_absin";

        test("should calculate s1 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Absinthe(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(1232);
            expect(dps).toBeLessThan(1234);
        });
        test("should calculate s1 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Absinthe(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1232);
            expect(dps).toBeLessThan(1234);
        });
        test("should calculate s1 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Absinthe(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(1109);
            expect(dps).toBeLessThan(1111);
        });

        test("should calculate s2 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Absinthe(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(2337);
            expect(dps).toBeLessThan(2339);
        });
        test("should calculate s2 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Absinthe(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(2337);
            expect(dps).toBeLessThan(2339);
        });
        test("should calculate s2 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Absinthe(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(2103);
            expect(dps).toBeLessThan(2105);
        });
    });

    describe("Aciddrop", () => {
        const baseParams = {
            moduleIndex: 1,
        };
        const operatorId = "char_366_acdrop";

        test("should calculate s1 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aciddrop(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(923);
            expect(dps).toBeLessThan(925);
        });
        test("should calculate s1 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aciddrop(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(595);
            expect(dps).toBeLessThan(597);
        });
        test("should calculate s1 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aciddrop(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(923);
            expect(dps).toBeLessThan(925);
        });

        test("should calculate s2 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aciddrop(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(1551);
            expect(dps).toBeLessThan(1553);
        });
        test("should calculate s2 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aciddrop(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1157);
            expect(dps).toBeLessThan(1159);
        });
        test("should calculate s2 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aciddrop(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(1551);
            expect(dps).toBeLessThan(1553);
        });
    });

    describe("Adnachiel", () => {
        const baseParams = {
            skillIndex: 0,
        };
        const operatorId = "char_211_adnach";

        test("should calculate s1 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Adnachiel(new OperatorData(operator), baseParams);
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(708);
            expect(dps).toBeLessThan(710);
        });
        test("should calculate s1 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Adnachiel(new OperatorData(operator), baseParams);
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(384);
            expect(dps).toBeLessThan(386);
        });
        test("should calculate s1 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Adnachiel(new OperatorData(operator), baseParams);
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(708);
            expect(dps).toBeLessThan(710);
        });
    });

    describe("Andreana", () => {
        const baseParams = {
            moduleIndex: 1,
        };
        const operatorId = "char_218_cuttle";

        test("should calculate s1 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Andreana(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(1226);
            expect(dps).toBeLessThan(1228);
        });
        test("should calculate s1 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Andreana(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1093);
            expect(dps).toBeLessThan(1095);
        });
        test("should calculate s1 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Andreana(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(1226);
            expect(dps).toBeLessThan(1228);
        });

        test("should calculate s2 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Andreana(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(2086);
            expect(dps).toBeLessThan(2088);
        });
        test("should calculate s2 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Andreana(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1952);
            expect(dps).toBeLessThan(1954);
        });
        test("should calculate s2 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Andreana(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(2086);
            expect(dps).toBeLessThan(2088);
        });
    });

    describe("Angelina", () => {
        const baseParams = {
            moduleIndex: 2,
        };
        const operatorId = "char_291_aglina";

        test("should calculate s1 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(829);
            expect(dps).toBeLessThan(831);
        });
        test("should calculate s1 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(829);
            expect(dps).toBeLessThan(831);
        });
        test("should calculate s1 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(663);
            expect(dps).toBeLessThan(665);
        });

        test("should calculate s2 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(1185);
            expect(dps).toBeLessThan(1187);
        });
        test("should calculate s2 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1185);
            expect(dps).toBeLessThan(1187);
        });
        test("should calculate s2 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(948);
            expect(dps).toBeLessThan(950);
        });

        test("should calculate s3 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 2,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(988);
            expect(dps).toBeLessThan(990);
        });
        test("should calculate s3 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 2,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(988);
            expect(dps).toBeLessThan(990);
        });
        test("should calculate s3 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Angelina(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 2,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(790);
            expect(dps).toBeLessThan(792);
        });
    });

    describe("Aosta", () => {
        const baseParams = {
            moduleIndex: 1,
        };
        const operatorId = "char_346_aosta";

        test("should calculate s1 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aosta(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(1470);
            expect(dps).toBeLessThan(1472);
        });
        test("should calculate s1 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aosta(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1280);
            expect(dps).toBeLessThan(1282);
        });
        test("should calculate s1 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aosta(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 0,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(1411);
            expect(dps).toBeLessThan(1413);
        });

        test("should calculate s2 DPS correctly", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aosta(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(1319);
            expect(dps).toBeLessThan(1321);
        });
        test("should calculate s2 DPS correctly with def", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aosta(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 300, res: 0 });
            expect(dps).toBeGreaterThan(1232);
            expect(dps).toBeLessThan(1234);
        });
        test("should calculate s2 DPS correctly with res", () => {
            const operator = operators(operatorId);
            if (!operator) throw new Error("Operator not found");
            const unit = new Aosta(new OperatorData(operator), {
                ...baseParams,
                skillIndex: 1,
            });
            const dps = unit.skillDPS({ defense: 0, res: 20 });
            expect(dps).toBeGreaterThan(1182);
            expect(dps).toBeLessThan(1184);
        });
    });
});
