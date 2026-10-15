import { describe, expect, it } from "vitest";
import { classifyClientGachaGroup, type IBanner, type IGachaItem } from "#/lib/api/gacha";
import { isIsolatedPull, pityCounterKey, pityWithLimitedReset, sharedPity, sixStarPullDetails } from "./pity";

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

describe("Starter Headhunting in pull history", () => {
    it("neither accumulates nor resets the Kernel counter", () => {
        expect(sharedPity(interleaved("CLASSIC_EN_1", "BOOT_0_1_1"), BANNERS)).toBe(3);
    });

    it("is isolated whether or not the banner list has loaded", () => {
        // BOOT_ pools are never in the banner list: they live in `NewbeeGachaPoolClient`.
        expect(isIsolatedPull(pull("BOOT_0_1_2", 1), BANNERS)).toBe(true);
        expect(isIsolatedPull(pull("BOOT_0_1_3", 1), new Map())).toBe(true);
    });

    it("gets its own bucket, never Kernel", () => {
        expect(classifyClientGachaGroup({ poolId: "BOOT_0_1_1", typeName: "boot" })).toBe("boot");
        // Older rows stored BOOT pulls under the "regular" wire type; the pool id still decides.
        expect(classifyClientGachaGroup({ poolId: "BOOT_0_1_1", typeName: "regular" })).toBe("boot");
        expect(classifyClientGachaGroup({ poolId: "", typeName: "boot" })).toBe("boot");
        expect(classifyClientGachaGroup({ poolId: "CLASSIC_EN_1", typeName: "classic" })).toBe("special");
    });
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
        for (const id of ["NORM_EN_41_0_3", "SPECIAL_EN_40_0_6", "CLASSIC_DOUBLE_EN_41_0_2", "FESCLASSIC_EN_40_0_3"]) expect(isIsolatedPull(pull(id, 1), empty)).toBe(false);
    });

    it("counts every pull when no isolated pool is in the history", () => {
        expect(sharedPity([pull("SINGLE_EN_1", 1, "6"), pull("SINGLE_EN_1", 2), pull("SINGLE_EN_1", 3)], BANNERS)).toBe(2);
    });
});

describe("Limited and Collab pity", () => {
    const limited = (id: string, endTime: number) => [id, { gachaPoolId: id, gachaRuleType: "LIMITED", endTime, gachaPoolName: id, returning: false } as unknown as IBanner] as const;
    const banners = new Map<string, IBanner>([limited("LIMITED_A", 100), limited("LIMITED_B", 300)]);

    it("counts only the newest pull's banner", () => {
        const items = [pull("LIMITED_A", 1), pull("LIMITED_A", 2), pull("LIMITED_A", 3), pull("LIMITED_B", 4), pull("LIMITED_B", 5)];
        expect(pityWithLimitedReset(items, banners, 200)).toEqual({ pity: 2, reset: false });
    });

    it("is 0 once that banner has ended", () => {
        expect(pityWithLimitedReset([pull("LIMITED_B", 4)], banners, 400)).toEqual({ pity: 0, reset: true, resetReason: "LIMITED_B" });
    });
});

describe("each 6* pull's pity count", () => {
    const LIMITED = new Map<string, IBanner>([...BANNERS, ["LIMITED_EN_1", { gachaPoolId: "LIMITED_EN_1", gachaRuleType: "LIMITED", returning: false, openTime: 100 } as unknown as IBanner]]);

    it("carries the Standard counter across Standard banners and Orienteering", () => {
        const first = pull("SINGLE_EN_1", 1, "6");
        const second = pull("SINGLE_EN_1", 30, "6");
        const items = [first, ...Array.from({ length: 5 }, (_, i) => pull("SINGLE_EN_1", 2 + i)), ...Array.from({ length: 3 }, (_, i) => pull("SPECIAL_EN_40_0_6", 10 + i)), second];
        const counts = sixStarPullDetails(items, BANNERS);
        expect(counts.get(second)).toMatchObject({ count: 9, exact: true });
    });

    it("keeps Kernel, isolated and Limited pulls off the Standard counter", () => {
        const first = pull("SINGLE_EN_1", 1, "6");
        const second = pull("SINGLE_EN_1", 50, "6");
        const items = [first, pull("CLASSIC_EN_1", 2), pull("ATTAIN_EN_40_0_2", 3), pull("RETURN_EN_41_0_1", 4), pull("LIMITED_EN_1", 200_000), pull("SINGLE_EN_1", 5), second];
        expect(sixStarPullDetails(items, LIMITED).get(second)).toMatchObject({ count: 2, exact: true });
    });

    it("reads a ten-pull newest first, as the wire lists it", () => {
        // One ten-pull at t=10, wire order newest first: its 6* is the 8th row, so the 3rd pull rolled.
        const before = pull("SINGLE_EN_1", 1, "6");
        const batch = Array.from({ length: 10 }, (_, i) => pull("SINGLE_EN_1", 10, i === 7 ? "6" : "3"));
        const counts = sixStarPullDetails([...batch, before], BANNERS);
        expect(counts.get(batch[7])?.count).toBe(3);
    });

    it("marks the first 6* on a shared counter as a lower bound", () => {
        const first = pull("CLASSIC_EN_1", 5, "6");
        const items = [pull("CLASSIC_EN_1", 1), pull("CLASSIC_EN_1", 2), first];
        expect(sixStarPullDetails(items, BANNERS).get(first)).toMatchObject({ count: 3, exact: false });
    });

    it("trusts a pool's first 6* only when the history starts before the pool opened", () => {
        // openTime 100 s = 100,000 ms.
        const sixStar = pull("LIMITED_EN_1", 200_000, "6");
        expect(sixStarPullDetails([pull("SINGLE_EN_1", 1), pull("LIMITED_EN_1", 150_000), sixStar], LIMITED).get(sixStar)).toMatchObject({ count: 2, exact: true });
        expect(sixStarPullDetails([pull("LIMITED_EN_1", 150_000), sixStar], LIMITED).get(sixStar)).toMatchObject({ count: 2, exact: false });
    });

    it("falls back to the pull's bucket while the banner list is empty", () => {
        expect(pityCounterKey(pull("SINGLE_EN_1", 1), new Map())).toBe("standard");
        expect(pityCounterKey(pull("FESCLASSIC_EN_1", 1), new Map())).toBe("kernel");
        expect(pityCounterKey(pull("LIMITED_EN_1", 1), new Map())).toBe("pool:LIMITED_EN_1");
    });
});

describe("each 6* pull's detail", () => {
    const LIMITED = new Map<string, IBanner>([...BANNERS, ["LIMITED_EN_1", { gachaPoolId: "LIMITED_EN_1", gachaRuleType: "LIMITED", returning: false, openTime: 100 } as unknown as IBanner]]);

    it("links the previous 6* on the same counter across banners", () => {
        const first = pull("SINGLE_EN_1", 1, "6");
        const second = pull("SPECIAL_EN_40_0_6", 10, "6");
        const details = sixStarPullDetails([first, pull("CLASSIC_EN_1", 5, "6"), second], BANNERS);
        expect(details.get(first)).toMatchObject({ scope: "standard", previous: null });
        expect(details.get(second)).toMatchObject({ scope: "standard", previous: first });
    });

    it("names the counter each scope rolls on", () => {
        const kernel = pull("CLASSIC_EN_1", 1, "6");
        const attain = pull("ATTAIN_EN_40_0_2", 2, "6");
        const limited = pull("LIMITED_EN_1", 200_000, "6");
        const details = sixStarPullDetails([kernel, attain, limited], LIMITED);
        expect(details.get(kernel)?.scope).toBe("kernel");
        expect(details.get(attain)?.scope).toBe("isolated");
        expect(details.get(limited)?.scope).toBe("isolated");
    });

    it("counts the rarities between the previous 6* (excluded) and this one", () => {
        const first = pull("SINGLE_EN_1", 1, "6");
        const second = pull("SINGLE_EN_1", 20, "6");
        const items = [first, pull("SINGLE_EN_1", 2, "5"), pull("SINGLE_EN_1", 3, "4"), pull("SINGLE_EN_1", 4, "4"), pull("SINGLE_EN_1", 5), pull("CLASSIC_EN_1", 6, "5"), second];
        const detail = sixStarPullDetails(items, BANNERS).get(second);
        expect(detail?.stretch).toEqual({ five: 1, four: 2, three: 1 });
        expect(detail?.count).toBe(5);
        expect(sixStarPullDetails(items, BANNERS).get(first)?.stretch).toEqual({ five: 0, four: 0, three: 0 });
    });
});
