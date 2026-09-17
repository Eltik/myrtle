import { describe, expect, it } from "vitest";
import { certPurchases, GREEN_PHASE1_BUYOUT, originiteWarning, projectIncome } from "./income";
import { longRunSixStar, oddsCurve, pityAt, pullOdds, pullsToTarget } from "./odds";
import { bannerModel, HARD_PITY_PULL, sixStarRate } from "./rates";
import { createPuller, simulateToTarget } from "./simulate";

/** Deterministic generator so the sampling assertions cannot flake. */
function mulberry32(seed: number): () => number {
    let a = seed >>> 0;
    return () => {
        a = (a + 0x6d2b79f5) >>> 0;
        let t = a;
        t = Math.imul(t ^ (t >>> 15), t | 1);
        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
}

/** Income settings with every optional source off, so a test moves one thing. */
function base() {
    return {
        orundum: 0,
        permits: 0,
        tenPermits: 0,
        originite: 0,
        spendOriginite: true,
        monthlyCard: false,
        annihilation: 1800,
        extraPerDay: 0,
        store: false,
        // The certificate shops are exercised in their own block; these tests are
        // about the base income and would only be noised up by them.
        goldCertsPerDay: 0,
        goldCertShop: false,
        greenCertsPerWeek: 0,
        greenCertShop: "off" as const,
        originiteForSkins: 0,
    };
}

const single = bannerModel({ ruleType: "SINGLE", featuredCount: 1 });
const limited = bannerModel({ ruleType: "LIMITED", featuredCount: 2 });
const double = bannerModel({ ruleType: "DOUBLE", featuredCount: 2 });
const linkage = bannerModel({ ruleType: "LINKAGE", featuredCount: 1 });

describe("the declared schedule", () => {
    it("is flat at 2% through roll 50 and additive after", () => {
        expect(sixStarRate(1)).toBeCloseTo(0.02, 12);
        expect(sixStarRate(50)).toBeCloseTo(0.02, 12);
        expect(sixStarRate(51)).toBeCloseTo(0.04, 12);
        expect(sixStarRate(52)).toBeCloseTo(0.06, 12);
        expect(sixStarRate(74)).toBeCloseTo(0.5, 12);
    });

    it("reaches certainty on roll 99, derived rather than asserted", () => {
        expect(HARD_PITY_PULL).toBe(99);
        expect(sixStarRate(99)).toBe(1);
        expect(sixStarRate(98)).toBeCloseTo(0.98, 12);
    });

    it("implies 34.594555 rolls per 6*, a realised rate of 2.8906%", () => {
        const { expectedPulls, rate } = longRunSixStar();
        expect(expectedPulls).toBeCloseTo(34.594555, 5);
        expect(rate).toBeCloseTo(0.028906, 6);
    });
});

describe("rate-up shares come out of the gamedata split", () => {
    it("gives a lone rate-up half the 6* band", () => {
        expect(single.shareEach).toBeCloseTo(0.5, 12);
        expect(single.shareTotal).toBeCloseTo(0.5, 12);
    });

    it("gives a limited pair 0.35 each for 0.70 together", () => {
        expect(limited.shareEach).toBeCloseTo(0.35, 12);
        expect(limited.shareTotal).toBeCloseTo(0.7, 12);
    });

    it("gives a standard pair 0.25 each for 0.50 together", () => {
        expect(double.shareEach).toBeCloseTo(0.25, 12);
        expect(double.shareTotal).toBeCloseTo(0.5, 12);
    });

    it("keeps the NORMAL pools that really use the limited split, on both servers", () => {
        // "Earthborn Metals" predates the LIMITED rule type. The two servers give it
        // unrelated ids, so each is read from its own sidecar rather than guessed:
        // CN is NORM_6_0_4, EN is NORM_EN_6_0_6.
        for (const poolId of ["NORM_EN_6_0_6", "NORM_6_0_4"]) {
            expect(bannerModel({ ruleType: "NORMAL", featuredCount: 2, poolId }).shareTotal).toBeCloseTo(0.7, 12);
        }
        expect(bannerModel({ ruleType: "NORMAL", featuredCount: 2, poolId: "NORM_6_0_6" }).shareTotal).toBeCloseTo(0.5, 12);
    });

    it("prefers a declared share over the rule type when the sidecar supplied one", () => {
        const declared = bannerModel({ ruleType: "NORMAL", featuredCount: 2, declaredShareEach: 0.35 });
        expect(declared.shareEach).toBeCloseTo(0.35, 12);
        expect(declared.inferred).toBe(false);
        expect(double.inferred).toBe(true);
    });
});

describe("pity scope follows the game's own rule text", () => {
    it("carries across standard and kernel, and never off an isolated banner", () => {
        expect(bannerModel({ ruleType: "NORMAL", featuredCount: 2 }).carryOver).toBe(true);
        expect(bannerModel({ ruleType: "CLASSIC", featuredCount: 2 }).carryOver).toBe(true);
        expect(bannerModel({ ruleType: "FESCLASSIC", featuredCount: 2 }).carryOver).toBe(true);
        expect(limited.carryOver).toBe(false);
        expect(linkage.carryOver).toBe(false);
        expect(bannerModel({ ruleType: "ATTAIN", featuredCount: 1 }).carryOver).toBe(false);
    });

    it("puts kernel banners on the same 50 / 99 schedule as standard", () => {
        // PityPanel.tsx currently hardcodes 45 / 80 for the kernel bucket. The
        // game's Kernel Headhunting text is byte-for-byte the standard text.
        const kernel = bannerModel({ ruleType: "CLASSIC", featuredCount: 2 });
        expect(kernel.scope).toBe("kernel");
        expect(pullOdds(kernel, 99).expectedSix).toBeGreaterThan(1);
    });
});

describe("exact odds reproduce an independent implementation", () => {
    // imivi's arknights-pulls-calculator ships a precomputed pull_odds.json whose
    // generator is not in that repo. These are its published values.
    it("matches the debut curve on rolls 1, 2 and 3", () => {
        expect(pullOdds(single, 1).specific * 100).toBeCloseTo(1.0, 6);
        expect(pullOdds(single, 2).specific * 100).toBeCloseTo(1.99, 6);
        expect(pullOdds(single, 3).specific * 100).toBeCloseTo(2.9701, 4);
    });

    it("matches the limited banner at 300 rolls to within 0.08 points", () => {
        const r = pullOdds(limited, 300);
        expect(r.any * 100).toBeCloseTo(99.98, 1);
        expect(r.specific * 100).toBeCloseTo(96.38, 0);
        expect(r.both * 100).toBeCloseTo(92.79, 0);
    });
});

describe("odds are internally coherent", () => {
    it("orders specific <= any and both <= specific", () => {
        for (const n of [1, 10, 50, 120, 300, 600]) {
            const r = pullOdds(limited, n);
            expect(r.both).toBeLessThanOrEqual(r.specific + 1e-12);
            expect(r.specific).toBeLessThanOrEqual(r.any + 1e-12);
            expect(r.any).toBeLessThanOrEqual(1 + 1e-12);
        }
    });

    it("rises monotonically in the number of rolls", () => {
        const curve = oddsCurve(limited, 400);
        for (let i = 1; i < curve.length; i++) {
            expect(curve[i].specific).toBeGreaterThanOrEqual(curve[i - 1].specific - 1e-12);
            expect(curve[i].expectedSix).toBeGreaterThanOrEqual(curve[i - 1].expectedSix - 1e-12);
        }
    });

    it("agrees with the single-point answer at every sampled length", () => {
        const curve = oddsCurve(limited, 200);
        for (const n of [1, 17, 99, 150, 200]) {
            expect(curve[n].specific).toBeCloseTo(pullOdds(limited, n).specific, 12);
            expect(curve[n].both).toBeCloseTo(pullOdds(limited, n).both, 12);
        }
    });

    it("never creates probability mass, whatever share it is handed", () => {
        // `shareEach` is read from gamedata. A pool expressing it as a percentage
        // rather than a fraction used to push the three branches past 1 and inflate
        // the grid: copies summed to 2.83 instead of 1.
        for (const share of [0.25, 0.35, 0.5, 0.6, 1.2]) {
            const m = bannerModel({ ruleType: "NORMAL", featuredCount: 2, declaredShareEach: share });
            const total = pullOdds(m, 200, { maxCopies: 3 }).copies.reduce((s2, v) => s2 + v, 0);
            expect(total).toBeCloseTo(1, 9);
        }
    });

    it("counts the collab handover toward expected 6*", () => {
        // The handover grants a 6* outside the pity schedule. Computing expected 6*
        // on a separate recursion missed it and undercounted by about 0.098.
        const at119 = pullOdds(linkage, 119);
        const step = pullOdds(linkage, 120).expectedSix - at119.expectedSix;
        // Every unit of mass that does not yet have the operator receives one on
        // roll 120, so the step is at least the probability of not yet having them.
        const missing = 1 - at119.specific;
        expect(missing).toBeGreaterThan(0.1);
        expect(step).toBeGreaterThanOrEqual(missing - 1e-12);
        // And the schedule alone could never produce a step that large here.
        const plainStep = pullOdds(single, 120).expectedSix - pullOdds(single, 119).expectedSix;
        expect(step).toBeGreaterThan(plainStep * 3);
    });

    it("agrees with sampling on expected 6*, including LINKAGE", () => {
        for (const [m, n] of [
            [single, 200],
            [linkage, 200],
            [limited, 200],
        ] as [ReturnType<typeof bannerModel>, number][]) {
            const exact = pullOdds(m, n).expectedSix;
            const rng = mulberry32(99);
            let total = 0;
            const runs = 4000;
            for (let r = 0; r < runs; r++) {
                const p = createPuller(m, 0, 0, rng);
                p.pullMany(n);
                total += p.state.sixStars;
            }
            expect(Math.abs(exact - total / runs)).toBeLessThan(0.15);
        }
    });

    it("reports the same expectedCopies from the curve as from the point", () => {
        const curve = oddsCurve(limited, 200, { maxCopies: 6 });
        const point = pullOdds(limited, 200, { maxCopies: 6 });
        expect(curve[200].expectedCopies).toBeCloseTo(point.expectedCopies, 12);
        expect(curve[200].expectedSix).toBeCloseTo(point.expectedSix, 12);
    });

    it("keeps `any` consistent with the declared band share on three rate-ups", () => {
        // The second slot aggregates "any featured other than the primary", so a
        // three-rate-up pool still spends its whole declared share.
        const three = bannerModel({ ruleType: "NORMAL", featuredCount: 3 });
        const two = bannerModel({ ruleType: "NORMAL", featuredCount: 2 });
        expect(three.shareTotal).toBeCloseTo(two.shareTotal, 12);
        expect(pullOdds(three, 300).any).toBeCloseTo(pullOdds(two, 300).any, 9);
        expect(pullOdds(three, 300).specific).toBeLessThan(pullOdds(two, 300).specific);
    });

    it("keeps the copy distribution a distribution", () => {
        const r = pullOdds(limited, 300, { maxCopies: 6 });
        const total = r.copies.reduce((s, v) => s + v, 0);
        expect(total).toBeCloseTo(1, 10);
        expect(1 - r.copies[0]).toBeCloseTo(r.specific, 12);
        expect(r.expectedCopies).toBeGreaterThan(1);
    });

    it("starts from carried pity rather than ignoring it", () => {
        const cold = pullOdds(single, 10).specific;
        const warm = pullOdds(single, 10, { startPity: 60 }).specific;
        expect(warm).toBeGreaterThan(cold * 2);
    });
});

describe("banner guarantees actually bind", () => {
    it("hands over the collab operator ON roll 120, not merely redirects one", () => {
        // "guaranteed to receive [X] within 120 attempts": the operator arrives on
        // the 120th roll whether or not that roll was itself a 6*.
        expect(pullOdds(linkage, 119).specific).toBeLessThan(1);
        expect(pullOdds(linkage, 120).specific).toBeCloseTo(1, 10);
        // A banner without the handover is still short at the same length.
        expect(pullOdds(single, 120).specific).toBeLessThan(0.93);
    });

    it("forces the NEXT 6* on-rate past 150, and the other one past 300", () => {
        // The Standard Selection text redirects "the next 6 star operator received",
        // so it binds only once a 6* actually drops. Hard pity puts that within 99
        // further rolls, which is why 150 alone is not yet certain and 249 is.
        expect(pullOdds(double, 150).any).toBeLessThan(1);
        expect(pullOdds(double, 150 + 99).any).toBeCloseTo(1, 6);
        expect(pullOdds(double, 300).both).toBeLessThan(1);
        expect(pullOdds(double, 300 + 99).both).toBeCloseTo(1, 6);
    });

    it("beats an unguaranteed banner on the same share", () => {
        const plain = bannerModel({ ruleType: "NORMAL", featuredCount: 2 });
        expect(plain.shareTotal).toBeCloseTo(double.shareTotal, 12);
        expect(pullOdds(double, 200).any).toBeGreaterThan(pullOdds(plain, 200).any);
    });

    it("spends the first 6* off-rate on an attain banner, ONCE", () => {
        const attain = bannerModel({ ruleType: "ATTAIN", featuredCount: 1 });
        const plain = bannerModel({ ruleType: "SINGLE", featuredCount: 1 });
        // Keying the guarantee off "no copies yet" rather than a spent flag makes it
        // re-fire on every 6* and pins the featured operator at zero forever. That
        // shipped once; a strictly-less-than assertion could not see it, because
        // zero satisfies it for free.
        expect(pullOdds(attain, 60).specific).toBeLessThan(pullOdds(plain, 60).specific);
        expect(pullOdds(attain, 60).specific).toBeGreaterThan(0.15);
        expect(pullOdds(attain, 600).specific).toBeGreaterThan(0.99);
        expect(pullOdds(attain, 1).specific).toBe(0);
    });

    it("forces the lone rate-up at 150 on a SINGLE pool", () => {
        // Focused Selection, present in the rule text of all 30 SINGLE pools:
        // "if you make 150 headhunting attempts without receiving the current rate-up
        // 6-star operator, the next 6-star operator received is guaranteed to be" it.
        // Treating SINGLE as unguaranteed understated it by 5.5 points at roll 249.
        expect(single.guarantee.kind).toBe("selection");
        expect(single.guarantee.first).toBe(150);
        expect(single.guarantee.second).toBeUndefined();
        const plain = bannerModel({ ruleType: "NORMAL", featuredCount: 1 });
        expect(plain.guarantee.kind).toBe("none");
        expect(pullOdds(single, 149).specific).toBeCloseTo(pullOdds(plain, 149).specific, 12);
        expect(pullOdds(single, 249).specific).toBeGreaterThan(pullOdds(plain, 249).specific);
        expect(pullOdds(single, 150 + 99).specific).toBeCloseTo(1, 6);
    });

    it("leaves LIMITED without a forced rate-up, because its text has none", () => {
        // The limited 300 is the Data Contract exchange, a currency mechanism, not a
        // redirected roll. Modelling it as pity would overstate the banner.
        expect(limited.guarantee.kind).toBe("none");
        expect(limited.spark).toBe(300);
        expect(pullOdds(limited, 300).specific).toBeLessThan(0.97);
    });

    it("keeps a single-rate-up DOUBLE pool moving past roll 300", () => {
        // The second selection rule forces "the other" rate-up. With only one
        // featured operator there is no other, and applying it anyway wrote the
        // mass straight back into the cell it came from, freezing the distribution.
        const lone = bannerModel({ ruleType: "DOUBLE", featuredCount: 1 });
        const at300 = pullOdds(lone, 300, { maxCopies: 3 }).expectedCopies;
        const at600 = pullOdds(lone, 600, { maxCopies: 3 }).expectedCopies;
        expect(at600).toBeGreaterThan(at300);
    });

    it("leaves CLASSIC_ATTAIN isolated despite the CLASSIC prefix", () => {
        // Its own rule text keeps its count separate from other banners.
        expect(bannerModel({ ruleType: "CLASSIC_ATTAIN", featuredCount: 1 }).carryOver).toBe(false);
        expect(bannerModel({ ruleType: "CLASSIC_DOUBLE", featuredCount: 2 }).carryOver).toBe(true);
    });
});

describe("sampling agrees with the exact answer", () => {
    it("lands within Monte Carlo noise on four configurations", () => {
        const cases: [ReturnType<typeof bannerModel>, number][] = [
            [single, 100],
            [limited, 200],
            [double, 150],
            [linkage, 130],
        ];
        for (const [model, n] of cases) {
            const exact = pullOdds(model, n).specific;
            const sampled = simulateToTarget(model, { runs: 20_000, maxPulls: n, rng: mulberry32(12345) }).success;
            expect(Math.abs(exact - sampled)).toBeLessThan(0.015);
        }
    });

    it("reports percentiles in order", () => {
        const s = simulateToTarget(limited, { runs: 5000, maxPulls: 600, rng: mulberry32(7) });
        expect(s.p50).toBeLessThanOrEqual(s.p75);
        expect(s.p75).toBeLessThanOrEqual(s.p90);
        expect(s.p90).toBeLessThanOrEqual(s.p95);
        expect(s.success).toBeGreaterThan(0.99);
    });
});

describe("income projection", () => {
    const from = new Date(Date.UTC(2026, 8, 16));

    it("books nothing on the day the user enters their balance", () => {
        const days = projectIncome({ ...base(), orundum: 6000 }, from, from);
        expect(days).toHaveLength(1);
        expect(days[0].orundum).toBe(6000);
        expect(days[0].pulls).toBe(10);
    });

    it("adds exactly the daily mission on a plain day", () => {
        const days = projectIncome(base(), from, new Date(Date.UTC(2026, 8, 17)));
        expect(days[1].orundum - days[0].orundum).toBe(100);
    });

    it("adds weekly missions and annihilation on the Monday reset", () => {
        // 2026-09-21 is a Monday.
        const days = projectIncome(base(), from, new Date(Date.UTC(2026, 8, 21)));
        const monday = days[days.length - 1];
        const sunday = days[days.length - 2];
        expect(monday.orundum - sunday.orundum).toBe(100 + 500 + 1800);
    });

    it("counts the monthly card at 200 a day on top of the daily", () => {
        const plain = projectIncome(base(), from, new Date(Date.UTC(2026, 8, 17)));
        const card = projectIncome({ ...base(), monthlyCard: true }, from, new Date(Date.UTC(2026, 8, 17)));
        expect(card[1].orundum - plain[1].orundum).toBe(200);
    });

    it("converts Originite Prime at 180 only when asked", () => {
        const off = projectIncome({ ...base(), originite: 60, spendOriginite: false }, from, from);
        const on = projectIncome({ ...base(), originite: 60, spendOriginite: true }, from, from);
        expect(off[0].pullsWithOriginite).toBe(0);
        // 60 * 180 = 10800 orundum, which is 18 rolls.
        expect(on[0].pullsWithOriginite).toBe(18);
    });

    it("counts a ten-roll permit as ten rolls", () => {
        const days = projectIncome({ ...base(), tenPermits: 2, permits: 3 }, from, from);
        expect(days[0].pulls).toBe(23);
    });
});

describe("estimated pulls to the rate-up", () => {
    it("agrees with the gacha-experiments simulator on a SINGLE banner", () => {
        // That repo's run_experiment records pulls-to-first-rate-up and prints the
        // mean with percentiles. Same model, read off exactly instead of sampled.
        // Its 400,000-run figures at this horizon: mean 66.08, median 57, 90th 142.
        const e = pullsToTarget(single, { horizon: 400 }).specific;
        expect(e.mean).toBeGreaterThan(65);
        expect(e.mean).toBeLessThan(67);
        expect(e.p50).toBe(57);
        expect(e.p90).toBe(142);
        expect(e.unresolved).toBeLessThan(0.0005);
    });

    it("puts each median where its own curve first crosses a half", () => {
        for (const m of [single, limited, double, linkage]) {
            const e = pullsToTarget(m, { horizon: 400 });
            expect(pullOdds(m, e.specific.p50).specific).toBeGreaterThanOrEqual(0.5);
            expect(pullOdds(m, e.specific.p50 - 1).specific).toBeLessThan(0.5);
            expect(pullOdds(m, e.specific.p90).specific).toBeGreaterThanOrEqual(0.9);
            if (e.both !== null) {
                expect(pullOdds(m, e.any.p50).any).toBeGreaterThanOrEqual(0.5);
                expect(pullOdds(m, e.both.p50).both).toBeGreaterThanOrEqual(0.5);
            }
        }
    });

    it("never quotes a 90th percentile below the median", () => {
        for (const m of [single, limited, double, linkage]) {
            const e = pullsToTarget(m, { horizon: 400 });
            for (const goal of [e.specific, e.any, e.both]) {
                if (goal === null) continue;
                expect(goal.p90).toBeGreaterThanOrEqual(goal.p50);
                expect(goal.mean).toBeGreaterThan(0);
            }
        }
    });

    it("orders the three goals: either is cheapest, both is dearest", () => {
        // Quoting one figure for a two-rate-up banner would flatter a player chasing
        // whichever comes first and mislead one chasing the pair.
        for (const m of [limited, double]) {
            const e = pullsToTarget(m, { horizon: 800 });
            expect(e.both).not.toBeNull();
            const both = e.both as NonNullable<typeof e.both>;
            expect(e.any.p50).toBeLessThan(e.specific.p50);
            expect(e.specific.p50).toBeLessThan(both.p50);
            expect(e.any.mean).toBeLessThan(e.specific.mean);
            expect(e.specific.mean).toBeLessThan(both.mean);
        }
    });

    it("reports the measured medians for the two-rate-up pools", () => {
        const d = pullsToTarget(double, { horizon: 800 });
        expect([d.any.p50, d.specific.p50, (d.both as NonNullable<typeof d.both>).p50]).toEqual([57, 102, 174]);
        const l = pullsToTarget(limited, { horizon: 800 });
        expect([l.any.p50, l.specific.p50, (l.both as NonNullable<typeof l.both>).p50]).toEqual([50, 70, 126]);
    });

    it("leaves `both` null on a banner with one rate-up, and `any` equal to it", () => {
        const e = pullsToTarget(single, { horizon: 400 });
        expect(e.both).toBeNull();
        expect(e.any.p50).toBe(e.specific.p50);
        expect(e.any.mean).toBeCloseTo(e.specific.mean, 12);
    });

    it("caps a collab at its handover, because it cannot cost more", () => {
        const e = pullsToTarget(linkage, { horizon: 400 }).specific;
        expect(e.p90).toBeLessThanOrEqual(120);
        // Exhausted to floating-point residue, not to an exact zero.
        expect(e.unresolved).toBeCloseTo(0, 12);
    });

    it("reads a banner cheaper when it inherits pity", () => {
        const cold = pullsToTarget(single, { horizon: 400 }).specific;
        const warm = pullsToTarget(single, { horizon: 400, startPityDist: pityAt(45) }).specific;
        expect(warm.p50).toBeLessThan(cold.p50);
        expect(warm.mean).toBeLessThan(cold.mean);
    });

    it("reports the tail it truncated rather than hiding it", () => {
        const classic = bannerModel({ ruleType: "CLASSIC", featuredCount: 2 });
        const short = pullsToTarget(classic, { horizon: 400 }).specific;
        const long = pullsToTarget(classic, { horizon: 800 }).specific;
        // A truncated mean is an understatement, and `unresolved` is its size.
        expect(short.mean).toBeLessThan(long.mean);
        expect(short.unresolved).toBeGreaterThan(long.unresolved);
        expect(long.unresolved).toBeLessThan(0.005);
        // `both` has the longest tail of the three, and 800 still covers it.
        const both = pullsToTarget(classic, { horizon: 800 }).both;
        expect((both as NonNullable<typeof both>).unresolved).toBeLessThan(0.005);
    });
});

describe("certificate shops", () => {
    it("walks the gold ladder cheapest rung first, to 38 permits for 258 certs", () => {
        // The cumulative table imivi hardcodes: 10/28/68/138/258 certs buy 1/3/8/18/38.
        expect(certPurchases(9, 0, "off").permits).toBe(0);
        expect(certPurchases(10, 0, "off").permits).toBe(1);
        expect(certPurchases(28, 0, "off").permits).toBe(3);
        expect(certPurchases(68, 0, "off").permits).toBe(8);
        expect(certPurchases(138, 0, "off").permits).toBe(18);
        expect(certPurchases(258, 0, "off").permits).toBe(38);
        expect(certPurchases(258, 0, "off").goldSpent).toBe(258);
    });

    it("stops at 38 permits however many gold certs are banked", () => {
        expect(certPurchases(10_000, 0, "off").permits).toBe(38);
        expect(certPurchases(10_000, 0, "off").goldSpent).toBe(258);
    });

    it("buys nothing green while the shop is off", () => {
        const none = certPurchases(0, 5000, "off");
        expect(none.permits).toBe(0);
        expect(none.orundum).toBe(0);
        expect(none.greenSpent).toBe(0);
    });

    it("sells 2 permits and 600 Orundum in green phase 1", () => {
        const full = certPurchases(0, 720, "phase1");
        expect(full.permits).toBe(2);
        expect(full.orundum).toBe(600);
        expect(full.greenSpent).toBe(720);
    });

    it("prices a green phase 1 permit and its Orundum identically", () => {
        // 240 certs buys one permit, which IS 600 Orundum, or 600 Orundum outright.
        const permitOnly = certPurchases(0, 240, "phase1");
        expect(permitOnly.permits).toBe(1);
        expect(permitOnly.orundum).toBe(0);
        const asOrundum = certPurchases(0, 240, "phase1").permits * 600;
        expect(asOrundum).toBe(600);
    });

    it("keeps phase 2 shut until phase 1 is bought out entirely", () => {
        // Weekly missions pay 20 a week, roughly 87 a month, so this gate is real.
        const short = certPurchases(0, GREEN_PHASE1_BUYOUT - 1, "phase2");
        const open = certPurchases(0, GREEN_PHASE1_BUYOUT + 900, "phase2");
        expect(open.permits).toBeGreaterThan(short.permits);
        expect(open.permits - short.permits).toBe(2);
    });

    it("feeds bought permits into the projection on the monthly reset", () => {
        const from = new Date(Date.UTC(2026, 8, 16));
        const to = new Date(Date.UTC(2026, 9, 2));
        const without = projectIncome({ ...base(), goldCertsPerDay: 20, goldCertShop: false }, from, to);
        const with_ = projectIncome({ ...base(), goldCertsPerDay: 20, goldCertShop: true }, from, to);
        expect(with_[with_.length - 1].permits).toBeGreaterThan(without[without.length - 1].permits);
    });

    it("leaves outfit spending out of the projection entirely", () => {
        // The projection converts every Originite Prime the player holds. What the
        // skins planner wants is reported as a clash, not quietly deducted here.
        const from = new Date(Date.UTC(2026, 8, 16));
        const days = projectIncome({ ...base(), originite: 60, spendOriginite: true }, from, from);
        expect(days[0].pullsWithOriginite).toBe(18);
    });
});

describe("the Originite Prime the outfit picks reserve", () => {
    const from = new Date(Date.UTC(2026, 8, 16));

    it("converts only what the outfits leave over", () => {
        // 840 OP is 252 rolls; 66 reserved leaves 774 OP, which is 232 rolls (774 * 180 / 600 = 232.2).
        const days = projectIncome({ ...base(), originite: 840, spendOriginite: true }, from, from, 66);
        expect(days[0].pullsWithOriginite).toBe(232);
        // The raw balance is still reported, for the warning to compare against.
        expect(days[0].originite).toBe(840);
    });

    it("reserves nothing when nothing is picked, bit for bit", () => {
        const plain = projectIncome({ ...base(), originite: 840, spendOriginite: true }, from, new Date(Date.UTC(2026, 9, 2)));
        const zero = projectIncome({ ...base(), originite: 840, spendOriginite: true }, from, new Date(Date.UTC(2026, 9, 2)), 0);
        expect(zero).toEqual(plain);
    });

    it("never converts a balance the outfits already exceed", () => {
        const days = projectIncome({ ...base(), orundum: 6000, originite: 40, spendOriginite: true }, from, from, 66);
        expect(days[0].pullsWithOriginite).toBe(days[0].pulls);
    });

    it("lets the monthly card's Originite grow back past the reservation", () => {
        // 2026-10-01 pays 6 OP on the card; with 40 held and 42 reserved, that is the
        // first day any of it converts: 46 - 42 = 4 OP = 720 orundum = 1 roll.
        const days = projectIncome({ ...base(), originite: 40, spendOriginite: true, monthlyCard: true }, from, new Date(Date.UTC(2026, 9, 1)), 42);
        const before = days[days.length - 2];
        const first = days[days.length - 1];
        expect(before.pullsWithOriginite).toBe(before.pulls);
        expect(first.pullsWithOriginite - first.pulls).toBe(1);
    });

    it("does not touch the no-conversion series", () => {
        const days = projectIncome({ ...base(), orundum: 6000, originite: 840, spendOriginite: false }, from, from, 66);
        expect(days[0].pulls).toBe(10);
        expect(days[0].pullsWithOriginite).toBe(10);
    });
});

describe("the Originite Prime warning", () => {
    it("says nothing when no outfits are picked", () => {
        expect(originiteWarning(0, 100)).toBeNull();
        expect(originiteWarning(0, 0)).toBeNull();
    });

    it("stays quiet when the outfits fit, whether or not pulls convert the rest", () => {
        expect(originiteWarning(40, 60)).toBeNull();
        expect(originiteWarning(40, 10_000)).toBeNull();
    });

    it("warns only when the outfits alone cost more than will be held", () => {
        const w = originiteWarning(80, 60);
        expect(w).not.toBeNull();
        expect(w?.short).toBe(20);
        expect(w?.needed).toBe(80);
        expect(w?.available).toBe(60);
    });
});
