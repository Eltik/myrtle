import { describe, expect, it } from "vitest";
import { EMPTY_SHARED_FILTERS, toFilterSets } from "#/components/operators/list/impl/shared-filters";
import { filterEntries, investmentScore, operatorSource, sortEntries } from "./helpers";
import type { IDisplayEntry, IOwnedEntry } from "./types";

const EMPTY_SETS = toFilterSets(EMPTY_SHARED_FILTERS);

function owned(partial: Partial<IOwnedEntry> & { operator_id: string; name: string; rarity: number }): IOwnedEntry {
    return {
        isOwned: true as const,
        user_id: "u",
        elite: 0,
        level: 1,
        exp: 0,
        potential: 0,
        skill_level: 1,
        favor_point: 0,
        skin_id: null,
        default_skill: null,
        voice_lan: null,
        current_equip: null,
        current_tmpl: null,
        obtained_at: 0,
        masteries: [],
        modules: [],
        meta: null,
        static: null,
        voiceActors: [],
        ...partial,
    };
}

/** A 5-star with everything: E2 80, S7, three M3s, a level-3 module, P5. */
const maxedFiveStar = owned({
    operator_id: "char_greyy2",
    name: "Greyy the Lightningbearer",
    rarity: 5,
    elite: 2,
    level: 80,
    skill_level: 7,
    potential: 5,
    masteries: [
        { index: 0, mastery: 3 },
        { index: 1, mastery: 3 },
        { index: 2, mastery: 3 },
    ],
    modules: [{ id: "uniequip_002_greyy2", level: 3, locked: false }],
});

/** A brand-new 6-star: E0 1, S1, nothing else. */
const untouchedSixStar = owned({ operator_id: "char_lumen", name: "Lumen", rarity: 6 });

/** A 6-star part-way through: E2 60, S7, no masteries yet. */
const midSixStar = owned({ operator_id: "char_mid", name: "Aardvark", rarity: 6, elite: 2, level: 60, skill_level: 7, potential: 0 });

/** A 3-star at its own ceiling: E1 55, S7, P5. It can never master or module. */
const maxedThreeStar = owned({ operator_id: "char_myrtle", name: "Myrtle", rarity: 3, elite: 1, level: 55, skill_level: 7, potential: 5 });

/** A 1-star at its own ceiling: no promotion, no skills. */
const maxedOneStar = owned({ operator_id: "char_yato", name: "Yato", rarity: 1, elite: 0, level: 30, potential: 5 });

describe("investmentScore", () => {
    it("scores a fully-invested 5-star above a fully-uninvested 6-star", () => {
        expect(investmentScore(maxedFiveStar)).toBeGreaterThan(investmentScore(untouchedSixStar));
    });

    it("stays inside [0, 1] for owned operators and returns -1 for unowned", () => {
        for (const e of [maxedFiveStar, untouchedSixStar, midSixStar, maxedThreeStar, maxedOneStar]) {
            expect(investmentScore(e)).toBeGreaterThanOrEqual(0);
            expect(investmentScore(e)).toBeLessThanOrEqual(1);
        }
        const unowned: IDisplayEntry = { isOwned: false, operator_id: "x", name: "X", rarity: 6, meta: null as never, static: null, voiceActors: [] };
        expect(investmentScore(unowned)).toBe(-1);
    });

    it("puts a fully-maxed operator at the top of the range, whatever its rarity", () => {
        // Masteries and modules are inapplicable to a 3-star and a 1-star, so
        // their weight has to redistribute rather than be forfeited, or these
        // could never reach 1.
        expect(investmentScore(maxedFiveStar)).toBeCloseTo(1, 5);
        expect(investmentScore(maxedThreeStar)).toBeCloseTo(1, 5);
        expect(investmentScore(maxedOneStar)).toBeCloseTo(1, 5);
    });

    it("puts a fully-uninvested operator at the bottom of the range", () => {
        expect(investmentScore(untouchedSixStar)).toBeCloseTo(0, 5);
    });

    it("never drops when investment goes up (E1 max -> E2 1)", () => {
        // The regression this pins: masteries are inapplicable below E2 and
        // merely empty at E2, so scoring the inapplicable as complete made
        // promotion LOWER the score.
        const e1Max = owned({ operator_id: "char_a", name: "A", rarity: 6, elite: 1, level: 80, skill_level: 7 });
        const e2Fresh = owned({ operator_id: "char_a", name: "A", rarity: 6, elite: 2, level: 1, skill_level: 7 });
        expect(investmentScore(e2Fresh)).toBeGreaterThan(investmentScore(e1Max));
    });

    it("rises monotonically along one operator's whole progression", () => {
        const steps = [
            owned({ operator_id: "char_a", name: "A", rarity: 6 }),
            owned({ operator_id: "char_a", name: "A", rarity: 6, elite: 0, level: 50 }),
            owned({ operator_id: "char_a", name: "A", rarity: 6, elite: 1, level: 1 }),
            owned({ operator_id: "char_a", name: "A", rarity: 6, elite: 1, level: 80, skill_level: 7 }),
            owned({ operator_id: "char_a", name: "A", rarity: 6, elite: 2, level: 1, skill_level: 7 }),
            owned({ operator_id: "char_a", name: "A", rarity: 6, elite: 2, level: 90, skill_level: 7 }),
            owned({
                operator_id: "char_a",
                name: "A",
                rarity: 6,
                elite: 2,
                level: 90,
                skill_level: 7,
                masteries: [
                    { index: 0, mastery: 3 },
                    { index: 1, mastery: 3 },
                    { index: 2, mastery: 3 },
                ],
            }),
            owned({
                operator_id: "char_a",
                name: "A",
                rarity: 6,
                elite: 2,
                level: 90,
                skill_level: 7,
                potential: 5,
                masteries: [
                    { index: 0, mastery: 3 },
                    { index: 1, mastery: 3 },
                    { index: 2, mastery: 3 },
                ],
            }),
        ];
        const scores = steps.map(investmentScore);
        for (let i = 1; i < scores.length; i += 1) {
            expect(scores[i]).toBeGreaterThan(scores[i - 1] as number);
        }
        expect(scores.at(-1)).toBeCloseTo(1, 5);
    });
});

describe("sortEntries", () => {
    const all = [untouchedSixStar, maxedFiveStar, midSixStar];

    it("orders by investment, most invested first, when sorting by investment desc", () => {
        const names = sortEntries(all, "investment", "desc").map((e) => e.name);
        expect(names).toEqual(["Greyy the Lightningbearer", "Aardvark", "Lumen"]);
    });

    it("breaks a rarity tie by investment rather than alphabetically", () => {
        // Same rarity, opposite alphabetical and investment orders: "Aardvark"
        // sorts first by name and second by investment, so the assertion below
        // fails on the old name-only tie-break.
        const tied = [untouchedSixStar, midSixStar];
        const names = sortEntries(tied, "rarity", "desc").map((e) => e.name);
        expect(names).toEqual(["Aardvark", "Lumen"]);
    });

    it("keeps the tie-break pointing the same way as the primary sort", () => {
        const asc = sortEntries([untouchedSixStar, midSixStar], "rarity", "asc").map((e) => e.name);
        expect(asc).toEqual(["Lumen", "Aardvark"]);
    });

    it("falls back to the name only once investment and rarity are both equal", () => {
        const a = owned({ operator_id: "char_b", name: "Beagle", rarity: 4 });
        const b = owned({ operator_id: "char_a", name: "Ansel", rarity: 4 });
        expect(sortEntries([a, b], "rarity", "desc").map((e) => e.name)).toEqual(["Ansel", "Beagle"]);
    });
});

describe("filterEntries by source", () => {
    const gacha = owned({ operator_id: "char_103_angel", name: "Exusiai", rarity: 6, static: { itemObtainApproach: "Recruitment & Headhunting" } as IOwnedEntry["static"] });
    // Texas is the one gacha operator whose approach is not the bare string.
    const texas = owned({ operator_id: "char_102_texas", name: "Texas", rarity: 5, static: { itemObtainApproach: "Recruitment & Headhunting, Pinboard Missions" } as IOwnedEntry["static"] });
    const event = owned({ operator_id: "char_427_vigil", name: "Vigil", rarity: 6, static: { itemObtainApproach: "Event Reward" } as IOwnedEntry["static"] });
    const voucher = owned({ operator_id: "char_275_breeze", name: "Breeze", rarity: 5, static: { itemObtainApproach: "Voucher Exchange" } as IOwnedEntry["static"] });
    const story = owned({ operator_id: "char_002_amiya", name: "Amiya", rarity: 5, static: { itemObtainApproach: "Main Theme Story" } as IOwnedEntry["static"] });
    // The backend serialises a missing approach as "": Reserve Operators and IS-only temporaries.
    const reserve = owned({ operator_id: "char_504_rguard", name: "Reserve Operator - Melee", rarity: 3, static: { itemObtainApproach: "" } as IOwnedEntry["static"] });
    const unloaded = owned({ operator_id: "char_4195_radian", name: "Raidian", rarity: 6, static: null });
    const all: IDisplayEntry[] = [gacha, texas, event, voucher, story, reserve, unloaded];
    const names = (entries: IDisplayEntry[]) => entries.map((e) => e.name);

    it("classifies by substring, so Texas's pinboard suffix still reads as headhunting", () => {
        expect(operatorSource("Recruitment & Headhunting")).toBe("headhunting");
        expect(operatorSource("Recruitment & Headhunting, Pinboard Missions")).toBe("headhunting");
        expect(operatorSource("Event Reward")).toBe("welfare");
        expect(operatorSource("Obtained from Integrated Strategies")).toBe("welfare");
        expect(operatorSource("")).toBeNull();
        expect(operatorSource(undefined)).toBeNull();
    });

    it("returns the same array untouched for the default", () => {
        expect(filterEntries(all, "", EMPTY_SETS)).toBe(all);
        expect(filterEntries(all, "", EMPTY_SETS, "any")).toBe(all);
    });

    it("headhunting keeps the gacha pool and drops welfare plus the sourceless", () => {
        expect(names(filterEntries(all, "", EMPTY_SETS, "headhunting"))).toEqual(["Exusiai", "Texas"]);
    });

    it("welfare keeps event, voucher and story operators only", () => {
        expect(names(filterEntries(all, "", EMPTY_SETS, "welfare"))).toEqual(["Vigil", "Breeze", "Amiya"]);
    });

    it("composes with search", () => {
        expect(names(filterEntries(all, "i", EMPTY_SETS, "welfare"))).toEqual(["Vigil", "Amiya"]);
    });
});
