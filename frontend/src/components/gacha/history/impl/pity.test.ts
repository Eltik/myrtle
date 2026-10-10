import { describe, expect, it } from "vitest";
import type { IBanner, IGachaItem } from "#/lib/api/gacha";
import { isIsolatedPull, sharedPity } from "./pity";

function pull(poolId: string, at: number, star: "3" | "4" | "5" | "6" = "3"): IGachaItem {
    return { charId: "char_x", charName: "", star, color: "", poolId, poolName: "", typeName: "normal", at, atStr: "" } as IGachaItem;
}

const BANNERS = new Map<string, IBanner>(
    (
        [
            ["SINGLE_EN_1", "SINGLE", false],
            ["CLASSIC_EN_1", "CLASSIC", false],
            ["RETURN_EN_41_0_1", "BACKFLOW", true],
            ["SPECIAL_EN_40_0_6", "SPECIAL", false],
            ["ATTAIN_EN_40_0_2", "ATTAIN", false],
            ["CLASSIC_ATTAIN_EN_40_0_1", "CLASSIC_ATTAIN", false],
        ] as const
    ).map(([id, ruleType, returning]) => [id, { gachaPoolId: id, gachaRuleType: ruleType, returning } as unknown as IBanner]),
);

/** Three counted pulls after a 6* on `home`, with ten isolated pulls and an isolated 6* in between. */
function interleaved(home: string, isolated: string): IGachaItem[] {
    return [pull(home, 1, "6"), pull(home, 2), ...Array.from({ length: 10 }, (_, i) => pull(isolated, 10 + i)), pull(isolated, 30, "6"), pull(home, 40), pull(home, 41)];
}

describe("the Standard counter in pull history", () => {
    for (const isolated of ["RETURN_EN_41_0_1", "ATTAIN_EN_40_0_2"]) {
        it(`is neither accumulated nor reset by ${isolated}`, () => {
            expect(sharedPity(interleaved("SINGLE_EN_1", isolated), BANNERS)).toBe(3);
        });
    }
});

describe("Orienteering in pull history", () => {
    it("accumulates and resets the Standard counter", () => {
        const items = [pull("SINGLE_EN_1", 1, "6"), pull("SPECIAL_EN_40_0_6", 2), pull("SINGLE_EN_1", 3)];
        expect(sharedPity(items, BANNERS)).toBe(2);
        expect(sharedPity([...items, pull("SPECIAL_EN_40_0_6", 4, "6"), pull("SINGLE_EN_1", 5)], BANNERS)).toBe(1);
    });
});

describe("the Kernel counter in pull history", () => {
    for (const isolated of ["CLASSIC_ATTAIN_EN_40_0_1", "ATTAIN_EN_40_0_2"]) {
        it(`is neither accumulated nor reset by ${isolated}`, () => {
            expect(sharedPity(interleaved("CLASSIC_EN_1", isolated), BANNERS)).toBe(3);
        });
    }
});

describe("classifying a pull's pool", () => {
    it("reads the rule type from the banner list", () => {
        expect(isIsolatedPull(pull("ATTAIN_EN_40_0_2", 1), BANNERS)).toBe(true);
        expect(isIsolatedPull(pull("CLASSIC_ATTAIN_EN_40_0_1", 1), BANNERS)).toBe(true);
        expect(isIsolatedPull(pull("SPECIAL_EN_40_0_6", 1), BANNERS)).toBe(false);
        expect(isIsolatedPull(pull("SINGLE_EN_1", 1), BANNERS)).toBe(false);
        expect(isIsolatedPull(pull("CLASSIC_EN_1", 1), BANNERS)).toBe(false);
    });

    it("falls back to the pool id while the banner list is empty", () => {
        const empty = new Map<string, IBanner>();
        for (const id of ["RETURN_71_0_1", "ATTAIN_EN_40_0_2", "CLASSIC_ATTAIN_EN_40_0_1"]) expect(isIsolatedPull(pull(id, 1), empty)).toBe(true);
        for (const id of ["NORM_EN_41_0_3", "SPECIAL_EN_40_0_6", "CLASSIC_DOUBLE_EN_41_0_2", "FESCLASSIC_EN_40_0_3", "BOOT_0_1_1"]) expect(isIsolatedPull(pull(id, 1), empty)).toBe(false);
    });

    it("counts every pull when no isolated pool is in the history", () => {
        expect(sharedPity([pull("SINGLE_EN_1", 1, "6"), pull("SINGLE_EN_1", 2), pull("SINGLE_EN_1", 3)], BANNERS)).toBe(2);
    });
});
