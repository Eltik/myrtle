import { describe, expect, test, beforeAll } from "bun:test";
import { OperatorData } from "../lib/impl/dps-calculator/impl/classes/impl/operator-data";
import TwelveF from "../lib/impl/dps-calculator/impl/operators/impl/12f";
import Aak from "../lib/impl/dps-calculator/impl/operators/impl/aak";
import Absinthe from "../lib/impl/dps-calculator/impl/operators/impl/absinthe";
import { init as initLocal } from "../lib/impl/local";
import { init } from "../lib/impl/dps-calculator";
import operators from "../lib/impl/local/impl/gamedata/impl/operators";

// Initialize data before running tests
beforeAll(async () => {
    await initLocal();
    await init();
});

describe("Operator DPS Calculations", () => {
    describe("12F", () => {
        const baseParams = {};

        test("should calculate DPS correctly", () => {
            const operator = operators("char_009_12fce");
            if (!operator) throw new Error("Operator not found");
            const unit = new TwelveF(new OperatorData(operator), baseParams);
            const dps = unit.skillDPS({ defense: 0, res: 0 });
            expect(dps).toBeGreaterThan(173);
            expect(dps).toBeLessThan(175);
        });
        test("should calculate DPS correctly with def", () => {
            const operator = operators("char_009_12fce");
            if (!operator) throw new Error("Operator not found");
            const unit = new TwelveF(new OperatorData(operator), baseParams);
            const dps = unit.skillDPS({ defense: 100, res: 0 });
            expect(dps).toBeGreaterThan(173);
            expect(dps).toBeLessThan(175);
        });
        test("should calculate DPS correctly with res", () => {
            const operator = operators("char_009_12fce");
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

        test("should calculate s1 DPS correctly", () => {
            const operator = operators("char_225_haak");
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
            const operator = operators("char_225_haak");
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
            const operator = operators("char_225_haak");
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
            const operator = operators("char_225_haak");
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
            const operator = operators("char_225_haak");
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
            const operator = operators("char_225_haak");
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

        test("should calculate s1 DPS correctly", () => {
            const operator = operators("char_405_absin");
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
            const operator = operators("char_405_absin");
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
            const operator = operators("char_405_absin");
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
            const operator = operators("char_405_absin");
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
            const operator = operators("char_405_absin");
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
            const operator = operators("char_405_absin");
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
});
