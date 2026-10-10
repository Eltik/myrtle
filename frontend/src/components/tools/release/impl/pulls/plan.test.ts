import { describe, expect, it } from "vitest";
import { projectIncome } from "./income";
import { meanPity, pityAt, pullOdds } from "./odds";

/** The carried counter as one number, derived rather than stored on the row. */
function carriedPity(dist: Float64Array | null): number {
    return dist === null ? 0 : meanPity(dist);
}

import { buildPlan, goalEstimateFor, type IPlanInput, maxPotFor, planTargets, planToCsv } from "./plan";
import { bannerModel, freePullsFor, startPityFor } from "./rates";

const DAY = 86_400;

/** Narrows an estimate the test has already established is present. */
function nn<T>(v: T | null): T {
    expect(v).not.toBeNull();
    return v as T;
}
const TODAY = new Date(Date.UTC(2026, 8, 16));

/** A forecast banner, reduced to the fields the ledger reads. */
function banner(id: string, ruleType: string, daysOut: number, featured: string[] = ["char_a", "char_b"], over: { declaredShareEach?: number; linkageLoopAt?: number } = {}) {
    const enStart = Math.floor(Date.UTC(2026, 8, 16) / 1000) + daysOut * DAY;
    return {
        cnPoolId: id,
        ruleType,
        nameCn: id,
        cnOpen: enStart,
        cnEnd: enStart + 14 * DAY,
        returning: false,
        featured6: featured,
        featured5: [],
        debutChars: [],
        anchorActivity: null,
        alignment: { enPoolId: null, method: "none" },
        enFeatured6: featured,
        overrideFeatured: [],
        nameEnAuto: null,
        imagePath: null,
        resolution: { status: "confirmed", enId: id, enStart, enEnd: enStart + 14 * DAY },
        declaredShareEach: over.declaredShareEach ?? null,
        linkageLoopAt: over.linkageLoopAt ?? null,
    } as unknown as IPlanInput["banners"][number];
}

function plan(over: Partial<IPlanInput> = {}, orundum = 60_000) {
    const days = projectIncome(
        {
            orundum,
            permits: 0,
            tenPermits: 0,
            originite: 0,
            spendOriginite: false,
            monthlyCard: false,
            annihilation: 1800,
            extraPerDay: 0,
            store: false,
            goldCertsPerDay: 0,
            goldCertShop: false,
            greenCertsPerWeek: 0,
            greenCertShop: "off",
        },
        TODAY,
        new Date(Date.UTC(2027, 2, 16)),
    );
    return buildPlan({
        banners: [banner("A", "LIMITED", 10), banner("B", "SINGLE", 40, ["char_c"]), banner("C", "LIMITED", 80)],
        days,
        model: null,
        today: TODAY,
        standardPity: 0,
        kernelPity: 0,
        allocations: {},
        targets: {},
        spendOriginite: false,
        countFreePulls: false,
        ...over,
    });
}

describe("the ledger", () => {
    it("leaves out a returning-players-only pool", () => {
        const returning = { ...banner("R", "BACKFLOW", 5), returning: true };
        const { rows } = plan({ banners: [returning, banner("A", "LIMITED", 10)] });
        expect(rows.map((r) => r.banner.cnPoolId)).toEqual(["A"]);
    });

    it("shows the whole bank on the first banner when nothing is committed", () => {
        const { rows, totals } = plan();
        expect(rows).toHaveLength(3);
        // 60,000 Orundum is 100 rolls, plus ten days of income before the banner
        // opens, so this is strictly more than the opening balance.
        expect(rows[0].available).toBeGreaterThan(100);
        expect(totals.allocated).toBe(0);
        expect(totals.spent).toBe(0);
    });

    it("takes what an earlier banner spends away from a later one", () => {
        const none = plan();
        const some = plan({ allocations: { A: 60 } });
        expect(some.rows[0].spent).toBe(60);
        expect(some.rows[1].available).toBe(none.rows[1].available - 60);
        expect(some.rows[2].available).toBe(none.rows[2].available - 60);
        // Without the ledger every banner reported the same bank, as though spending
        // had no consequence.
        expect(none.rows[1].available).toBe(none.rows[1].available);
    });

    it("prices a banner that is already running against today rather than at zero", () => {
        // Three days into its run, and still inside the past grace window, so the
        // ledger keeps it. `dayAt` has no day earlier than today to hand back, so this
        // row used to report nothing available: every commitment to it became an
        // overrun that never left the pool, and the banner after it never moved however
        // much was poured into this one.
        const live = banner("L", "SINGLE", -3, ["char_d"]);
        const later = banner("B", "SINGLE", 40, ["char_c"]);
        const none = plan({ banners: [live, later] });
        // 60,000 Orundum is exactly 100 rolls and today has earned nothing yet.
        expect(none.rows[0].available).toBe(100);

        const some = plan({ banners: [live, later], allocations: { L: 60 } });
        expect(some.rows[0].spent).toBe(60);
        expect(some.rows[0].shortfall).toBe(0);
        expect(some.rows[1].available).toBe(none.rows[1].available - 60);
    });

    it("carries leftovers forward rather than resetting the bank", () => {
        const { rows } = plan({ allocations: { A: 20 } });
        expect(rows[0].leftover).toBe(rows[0].available - 20);
        expect(rows[1].available).toBeGreaterThan(rows[0].leftover);
    });

    it("caps a commitment at what is actually in hand and reports the shortfall", () => {
        const { rows, totals } = plan({ allocations: { A: 500 } });
        const bank = rows[0].available;
        expect(bank).toBeLessThan(500);
        expect(rows[0].allocated).toBe(500);
        expect(rows[0].spent).toBe(bank);
        expect(rows[0].shortfall).toBe(500 - bank);
        expect(rows[0].leftover).toBe(0);
        expect(totals.shortBanners).toBe(1);
        expect(totals.spent).toBe(bank);
    });

    it("never lets a later banner go negative when an earlier one overran", () => {
        const { rows } = plan({ allocations: { A: 9999, B: 9999, C: 9999 } });
        for (const row of rows) {
            expect(row.available).toBeGreaterThanOrEqual(0);
            expect(row.spent).toBeLessThanOrEqual(row.available);
        }
    });

    it("computes odds at what is committed, not at the whole bank", () => {
        const { rows } = plan({ allocations: { A: 10 } });
        const direct = pullOdds(rows[0].model, 10, { startPity: 0 });
        expect(rows[0].odds.specific).toBeCloseTo(direct.specific, 12);
        expect(rows[0].odds.specific).toBeLessThan(0.2);
    });

    it("gives a banner with nothing committed odds of exactly zero", () => {
        const { rows } = plan();
        expect(rows[0].odds.specific).toBe(0);
        expect(rows[0].odds.any).toBe(0);
    });
});

describe("pity carried between banners", () => {
    it("starts a shared-counter banner where the previous one left off", () => {
        // Two SINGLE banners share the standard counter. Spending 40 rolls on the
        // first without hitting a 6* leaves the second deep into the ramp.
        const rows = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), banner("S2", "SINGLE", 40, ["char_d"])],
            allocations: { S1: 40 },
        }).rows;
        expect(carriedPity(rows[1].pityDist)).toBeGreaterThan(0);
        const cold = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), banner("S2", "SINGLE", 40, ["char_d"])],
            allocations: {},
        }).rows;
        expect(carriedPity(cold[1].pityDist)).toBe(0);
    });

    it("does not leak carried pity onto an isolated banner", () => {
        const rows = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), banner("L1", "LIMITED", 40)],
            allocations: { S1: 40 },
            standardPity: 30,
            kernelPity: 30,
        }).rows;
        expect(rows[1].model.carryOver).toBe(false);
        expect(carriedPity(rows[1].pityDist)).toBe(0);
    });

    it("seeds the first shared banner from the user's own counter", () => {
        const warm = plan({ banners: [banner("S1", "SINGLE", 10, ["char_c"])], allocations: { S1: 10 }, standardPity: 60 }).rows;
        const cold = plan({ banners: [banner("S1", "SINGLE", 10, ["char_c"])], allocations: { S1: 10 }, standardPity: 0 }).rows;
        expect(warm[0].odds.specific).toBeGreaterThan(cold[0].odds.specific * 2);
    });

    it("keeps standard and kernel counters apart", () => {
        const rows = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), banner("K1", "CLASSIC", 40), banner("S2", "SINGLE", 70, ["char_d"])],
            allocations: { S1: 40, K1: 40 },
            standardPity: 0,
            kernelPity: 0,
        }).rows;
        // The kernel banner starts cold despite 40 rolls spent on standard first.
        expect(carriedPity(rows[1].pityDist)).toBe(0);
        // And the second standard banner does not inherit the kernel banner's rolls.
        expect(carriedPity(rows[2].pityDist)).toBeGreaterThan(0);
        expect(carriedPity(rows[2].pityDist)).toBeLessThan(80);
    });

    it("carries Kernel pity across Kernel banners by the same rule as standard", () => {
        // Both families' pool details say the rolls "carry over to subsequent" banners
        // of the same family; neither text says the two families share one counter.
        const rows = plan({
            banners: [banner("K1", "CLASSIC", 10), banner("K2", "CLASSIC_DOUBLE", 40)],
            allocations: { K1: 40 },
        }).rows;
        expect(rows[1].model.carryOver).toBe(true);
        expect(carriedPity(rows[1].pityDist)).toBeGreaterThan(0);
    });

    it("never feeds a Returning Headhunting pool into the standard counter", () => {
        // Even if one reached the plan, its rolls would not move the standard counter.
        const returning = { ...banner("R1", "BACKFLOW", 20, ["char_r"]), returning: false } as IPlanInput["banners"][number];
        const rows = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), returning, banner("S2", "SINGLE", 40, ["char_d"])],
            allocations: { S1: 40, R1: 60 },
        }).rows;
        expect(rows[1].model.scope).toBe("isolated");
        expect(rows[1].pityDist).toBeNull();
        expect(rows[2].pityDist).toBe(rows[0].odds.endPity);
    });
});

describe("a skipped banner leaves the shared counter untouched", () => {
    it("carries standard pity past a skipped standard banner as the same distribution", () => {
        const rows = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), banner("S2", "DOUBLE", 25), banner("S3", "SINGLE", 40, ["char_d"])],
            allocations: { S1: 40 },
        }).rows;
        expect(rows[1].totalPulls).toBe(0);
        expect(rows[1].pityDist).toBe(rows[0].odds.endPity);
        expect(rows[2].pityDist).toBe(rows[1].pityDist);
    });

    it("carries the user's own counter past a skipped first banner unchanged", () => {
        const rows = plan({
            banners: [banner("S1", "NORMAL", 10, ["char_c"]), banner("S2", "SINGLE", 40, ["char_d"])],
            allocations: {},
            standardPity: 30,
        }).rows;
        expect(Array.from(nn(rows[1].pityDist))).toEqual(Array.from(pityAt(30)));
        expect(rows[1].pityDist).toBe(rows[0].pityDist);
    });

    it("carries Kernel pity past a skipped Kernel banner, and standard pity past it too", () => {
        const rows = plan({
            banners: [banner("K1", "CLASSIC", 10), banner("S1", "SINGLE", 15, ["char_c"]), banner("K2", "FESCLASSIC", 25), banner("K3", "CLASSIC", 40), banner("S2", "SINGLE", 55, ["char_d"])],
            allocations: { K1: 30, S1: 45 },
        }).rows;
        expect(rows[2].totalPulls).toBe(0);
        expect(rows[2].pityDist).toBe(rows[0].odds.endPity);
        expect(rows[3].pityDist).toBe(rows[0].odds.endPity);
        expect(rows[4].pityDist).toBe(rows[1].odds.endPity);
    });

    it("carries standard pity past a returning pool the plan leaves out", () => {
        const returning = { ...banner("R1", "BACKFLOW", 20, ["char_r"]), returning: true } as IPlanInput["banners"][number];
        const withIt = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), returning, banner("S2", "SINGLE", 40, ["char_d"])],
            allocations: { S1: 40, R1: 60 },
        }).rows;
        const without = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), banner("S2", "SINGLE", 40, ["char_d"])],
            allocations: { S1: 40 },
        }).rows;
        expect(withIt.map((r) => r.key)).toEqual(["S1", "S2"]);
        expect(withIt[1].pityDist).toBe(withIt[0].odds.endPity);
        expect(Array.from(nn(withIt[1].pityDist))).toEqual(Array.from(nn(without[1].pityDist)));
    });
});

describe("pity scope per rule type", () => {
    it("scopes every rule type the way its pool detail text does", () => {
        const scope = (ruleType: string) => {
            const m = bannerModel({ ruleType, featuredCount: 2 });
            return `${m.scope}/${m.carryOver}`;
        };
        expect(["NORMAL", "SINGLE", "DOUBLE", "SPECIAL"].map(scope)).toEqual(["standard/true", "standard/true", "standard/true", "standard/true"]);
        expect(["CLASSIC", "CLASSIC_DOUBLE", "FESCLASSIC"].map(scope)).toEqual(["kernel/true", "kernel/true", "kernel/true"]);
        const isolated = ["ATTAIN", "CLASSIC_ATTAIN", "BACKFLOW", "LIMITED", "LINKAGE"];
        expect(isolated.map(scope)).toEqual(isolated.map(() => "isolated/false"));
    });

    it("seeds Standard and Kernel from their own counters", () => {
        const rows = plan({
            banners: [banner("S1", "SINGLE", 10, ["char_c"]), banner("K1", "CLASSIC", 40), banner("O1", "SPECIAL", 50), banner("A1", "ATTAIN", 60, ["char_e"])],
            standardPity: 30,
            kernelPity: 70,
        }).rows;
        expect(Array.from(nn(rows[0].pityDist))).toEqual(Array.from(pityAt(30)));
        expect(Array.from(nn(rows[1].pityDist))).toEqual(Array.from(pityAt(70)));
        expect(rows[2].pityDist).not.toBeNull();
        expect(rows[3].pityDist).toBeNull();
    });

    it("starts the odds and simulator archetypes on the right counter", () => {
        const pity = { standard: 30, kernel: 70 };
        const start = (ruleType: string) => startPityFor(bannerModel({ ruleType, featuredCount: 2 }), pity);
        expect(["NORMAL", "DOUBLE", "CLASSIC", "SPECIAL", "ATTAIN", "LIMITED", "LINKAGE"].map(start)).toEqual([30, 30, 70, 30, 0, 0, 0]);
    });
});

describe("a distribution handed between banners", () => {
    it("hands back a normalised distribution that means what pityAt means", () => {
        const fresh = pullOdds(bannerModel({ ruleType: "SINGLE", featuredCount: 1 }), 0, { startPityDist: pityAt(25) });
        expect(meanPity(fresh.endPity)).toBeCloseTo(25, 9);
        let mass = 0;
        for (const v of fresh.endPity) mass += v;
        expect(mass).toBeCloseTo(1, 12);
    });
});

describe("totals and export", () => {
    it("reports remaining as the bank less everything committed", () => {
        const { totals } = plan({ allocations: { A: 50, B: 30 } });
        expect(totals.spent).toBe(80);
        expect(totals.remaining).toBe(totals.income - 80);
        expect(totals.shortfall).toBe(0);
    });

    it("quotes every field and escapes a name containing a comma", () => {
        const { rows } = plan({ banners: [banner("Hi, there", "LIMITED", 10)], allocations: { "Hi, there": 10 } });
        const csv = planToCsv(rows, () => "2026-09-26");
        expect(csv.split("\n")).toHaveLength(2);
        expect(csv).toContain('"Hi, there"');
    });
});

describe("choosing a potential per operator", () => {
    it("carries the counts onto the row and totals them", () => {
        const row = plan({ targets: { A: { char_a: 3, char_b: 2 } } }).rows[0];
        expect(row.targets).toEqual({ char_a: 3, char_b: 2 });
        expect(row.totalCopies).toBe(5);
        expect(row.goalEstimate).not.toBeNull();
    });

    it("has no goal until something is asked for", () => {
        expect(plan().rows[0].goalEstimate).toBeNull();
        expect(plan().rows[0].totalCopies).toBe(0);
    });

    it("costs more for more copies of the same operator", () => {
        const one = plan({ targets: { A: { char_a: 1 } } }).rows[0].goalEstimate;
        const three = plan({ targets: { A: { char_a: 3 } } }).rows[0].goalEstimate;
        const six = plan({ targets: { A: { char_a: 6 } } }).rows[0].goalEstimate;
        expect(nn(three).p50).toBeGreaterThan(nn(one).p50);
        expect(nn(six).p50).toBeGreaterThan(nn(three).p50);
    });

    it("is cheaper to spread across two operators than to stack one", () => {
        // Two rate-ups share the 6* pool, so one copy of each costs less than two of
        // the same: 126 against 171 on this banner.
        const two = nn(plan({ targets: { A: { char_a: 2 } } }).rows[0].goalEstimate).p50;
        const oneEach = nn(plan({ targets: { A: { char_a: 1, char_b: 1 } } }).rows[0].goalEstimate).p50;
        expect(oneEach).toBeLessThan(two);
    });

    it("does not let medians be added, because they do not add", () => {
        // A tempting shortcut is to price N copies as N times one copy. That is wrong
        // in both directions and the right answer needs the joint walk. Stacking one
        // operator costs MORE than twice a single copy (171 against 2 x 70), because
        // the median of a sum is pulled toward the mean of a right-skewed wait.
        // Spreading across two costs LESS (126 against 140).
        const one = nn(plan({ targets: { A: { char_a: 1 } } }).rows[0].goalEstimate).p50;
        const two = nn(plan({ targets: { A: { char_a: 2 } } }).rows[0].goalEstimate).p50;
        const oneEach = nn(plan({ targets: { A: { char_a: 1, char_b: 1 } } }).rows[0].goalEstimate).p50;
        expect(two).toBeGreaterThan(one * 2);
        expect(oneEach).toBeLessThan(one * 2);
    });

    it("keeps MEANS additive for copies of one operator, unlike medians", () => {
        // Expectation is linear even though the median is not: the wait for a second
        // copy starts from a reset counter, so its mean is the first one's again.
        const one = nn(plan({ targets: { A: { char_a: 1 } } }).rows[0].goalEstimate).mean;
        const two = nn(plan({ targets: { A: { char_a: 2 } } }).rows[0].goalEstimate).mean;
        expect(two).toBeGreaterThan(one * 1.9);
        expect(two).toBeLessThan(one * 2.1);
    });

    it("reproduces the reference goals it generalises", () => {
        const row = plan({ targets: { A: { char_a: 1 } } }).rows[0];
        expect(nn(row.goalEstimate).p50).toBe(row.estimate.specific.p50);
        const both = plan({ targets: { A: { char_a: 1, char_b: 1 } } }).rows[0];
        expect(nn(both.goalEstimate).p50).toBe(nn(both.estimate.both).p50);
        const pot = plan({ targets: { A: { char_a: 6 } } }).rows[0];
        expect(nn(pot.goalEstimate).p50).toBe(pot.maxPot.p50);
    });

    it("is symmetric between the two rate-ups, which share a rate", () => {
        const a = nn(plan({ targets: { A: { char_a: 2 } } }).rows[0].goalEstimate).p50;
        const b = nn(plan({ targets: { A: { char_b: 2 } } }).rows[0].goalEstimate).p50;
        expect(a).toBe(b);
    });

    it("drops a count for an operator no longer on the banner", () => {
        const row = plan({ targets: { A: { char_a: 2, char_gone: 4 } } }).rows[0];
        expect(row.targets).toEqual({ char_a: 2 });
        expect(row.totalCopies).toBe(2);
    });

    it("ignores counts aimed at a banner that is not in the window", () => {
        const { rows, totals } = plan({ targets: { NOPE: { char_a: 3 } } });
        expect(rows.every((r) => r.totalCopies === 0)).toBe(true);
        expect(totals.allocated).toBe(0);
    });

    it("caps a request at maximum potential", () => {
        const row = plan({ targets: { A: { char_a: 99 } } }).rows[0];
        expect(row.targets.char_a).toBe(6);
    });
});

describe("maximum potential", () => {
    it("costs far more than one copy, and more than both operators", () => {
        // Six copies is the target kukkiforarknights/gacha-experiments simulates.
        const row = plan().rows[0];
        const one = row.estimate.specific.p50;
        const both = (row.estimate.both as NonNullable<typeof row.estimate.both>).p50;
        expect(row.maxPot.p50).toBeGreaterThan(both);
        expect(both).toBeGreaterThan(one);
        // Six copies of one operator is dearer than one each of two.
        expect(row.maxPot.p50).toBeGreaterThan(one * 4);
    });

    it("reports the measured medians per banner type", () => {
        // 565 before the 300-roll bonus copy was counted.
        expect(maxPotFor(bannerModel({ ruleType: "LIMITED", featuredCount: 2 })).p50).toBe(467);
        expect(maxPotFor(bannerModel({ ruleType: "DOUBLE", featuredCount: 2 })).p50).toBe(783);
        expect(maxPotFor(bannerModel({ ruleType: "SINGLE", featuredCount: 1 })).p50).toBe(397);
    });

    it("resolves inside its horizon", () => {
        for (const rt of ["LIMITED", "DOUBLE", "CLASSIC", "SINGLE"]) {
            const e = maxPotFor(bannerModel({ ruleType: rt, featuredCount: rt === "SINGLE" ? 1 : 2 }));
            expect(e.unresolved).toBeLessThan(0.005);
            expect(e.p90).toBeGreaterThan(e.p50);
            expect(e.mean).toBeGreaterThan(0);
        }
    });

    it("returns the cached instance for the same banner model", () => {
        const m = bannerModel({ ruleType: "LIMITED", featuredCount: 2 });
        expect(maxPotFor(m)).toBe(maxPotFor(bannerModel({ ruleType: "LIMITED", featuredCount: 2 })));
    });
});

describe("rule details carried onto the rows", () => {
    it("makes the limited operator certain once the banner reaches 300, free pulls included", () => {
        const rich = plan({ allocations: { A: 276 }, countFreePulls: true }, 200_000);
        expect(rich.rows[0].totalPulls).toBe(300);
        expect(rich.rows[0].odds.specific).toBeCloseTo(1, 12);
        const short = plan({ allocations: { A: 275 }, countFreePulls: true }, 200_000);
        expect(short.rows[0].odds.specific).toBeLessThan(0.97);
    });

    it("reads the banner's declared share, and Orienteering keeps its three picks", () => {
        const p = plan({ banners: [banner("N", "NORMAL", 10, ["char_a", "char_b"], { declaredShareEach: 0.35 }), banner("S", "SPECIAL", 20, ["c1", "c2", "c3"], { declaredShareEach: 0.5 })] });
        const [normal, special] = p.rows;
        expect(normal.model.shareEach).toBeCloseTo(0.35, 12);
        expect(normal.model.inferred).toBe(false);
        expect(special.model.shareEach).toBeCloseTo(1 / 3, 12);
    });

    it("splits Orienteering over three picks, not over the six candidates listed", () => {
        const six = ["c1", "c2", "c3", "c4", "c5", "c6"];
        const p = plan({ banners: [banner("S", "SPECIAL", 10, six)], allocations: { S: 100 } });
        const row = p.rows[0];
        expect(row.model.shareEach).toBeCloseTo(1 / 3, 12);
        expect(row.model.featuredCount).toBe(3);
        expect(row.model.inferred).toBe(false);
        // A third each: 28.43% at 50 pulls and 60.89% at 100 (8.00% and 19.60% at the old twelfth).
        expect(row.odds.specific).toBeCloseTo(0.6089, 4);
        expect(pullOdds(row.model, 50).specific).toBeCloseTo(0.2843, 4);
    });

    it("gives an ATTAIN row no goals and drops potentials aimed at it", () => {
        const p = plan({ banners: [banner("T", "ATTAIN", 10)], allocations: { T: 100 }, targets: { T: { char_a: 2 } } });
        const row = p.rows[0];
        expect(row.model.rateUp).toBe("none");
        expect(row.odds.specific).toBe(0);
        expect(row.totalCopies).toBe(0);
        expect(row.goalEstimate).toBeNull();
        expect(row.estimate.both).toBeNull();
        expect(row.maxPot.unresolved).toBe(1);
        expect(row.model.inferred).toBe(false);
        expect(planTargets(row).guarantee).toBeNull();
    });

    it("keeps a looping collab's estimates apart from a once-only one's", () => {
        const p = plan({ banners: [banner("LINKAGE_74_0_1", "LINKAGE", 10, ["char_x"], { linkageLoopAt: 120 }), banner("L", "LINKAGE", 40, ["char_y"])], targets: { LINKAGE_74_0_1: { char_x: 2 }, L: { char_y: 2 } } });
        const [loop, once] = p.rows;
        expect(loop.model.guarantee.repeat).toBe(true);
        expect(once.model.guarantee.repeat).toBeUndefined();
        expect(nn(loop.goalEstimate).p90).toBeLessThanOrEqual(240);
        expect(nn(once.goalEstimate).p90).toBeGreaterThan(nn(loop.goalEstimate).p90);
        expect(loop.maxPot.p50).toBeLessThan(once.maxPot.p50);
    });
});

describe("free pulls a banner gives away", () => {
    it("derives a Limited banner's 24 from its own run length", () => {
        const day = 86_400;
        // A free ten-roll plus one single a day over the 14-day run every Limited
        // pool in the data uses.
        expect(freePullsFor("LIMITED", 0, 14 * day)).toBe(24);
        // The one 13-day Limited pool in the data correctly yields 23, which is why
        // this is derived rather than hardcoded at 24.
        expect(freePullsFor("LIMITED", 0, 13 * day)).toBe(23);
    });

    it("gives a collab two ten-rolls and no dailies", () => {
        expect(freePullsFor("LINKAGE", 0, 14 * 86_400)).toBe(20);
    });

    it("pays every Limited banner, because they do not rerun", () => {
        // No rerun means no case where a Limited pool pays nothing, so the rule needs
        // no rerun flag to consult.
        expect(freePullsFor("LIMITED", 0, 14 * 86_400)).toBeGreaterThan(0);
    });

    it("gives an ordinary banner nothing", () => {
        for (const rt of ["NORMAL", "SINGLE", "DOUBLE", "CLASSIC", "ATTAIN"]) {
            expect(freePullsFor(rt, 0, 14 * 86_400)).toBe(0);
        }
    });

    it("adds them to a banner's odds without banking them", () => {
        const off = plan({ allocations: { A: 0 } });
        const on = plan({ allocations: { A: 0 }, countFreePulls: true });
        expect(on.rows[0].freePulls).toBe(24);
        expect(on.rows[0].totalPulls).toBe(24);
        expect(on.rows[0].odds.specific).toBeGreaterThan(off.rows[0].odds.specific);
        // The key property: a free pull never becomes savings for a later banner.
        expect(on.rows[1].available).toBe(off.rows[1].available);
        expect(on.totals.spent).toBe(off.totals.spent);
    });

    it("counts free pulls toward the spark, because the game does", () => {
        // 276 committed plus the banner's own 24 reaches the 300-roll bonus copy, so a
        // bank that can actually pay 276 is needed for the case to mean anything.
        const rich = plan({ allocations: { A: 276 }, countFreePulls: true }, 200_000);
        expect(rich.rows[0].spent).toBe(276);
        expect(rich.rows[0].freePulls).toBe(24);
        expect(rich.rows[0].totalPulls).toBe(300);
        expect(rich.rows[0].sparkMet).toBe(true);
        // And 275 committed falls one short of it.
        const short = plan({ allocations: { A: 275 }, countFreePulls: true }, 200_000);
        expect(short.rows[0].sparkMet).toBe(false);
    });

    it("never lets a free pull inflate what was actually spent", () => {
        const on = plan({ allocations: { A: 9999 }, countFreePulls: true });
        expect(on.rows[0].spent).toBe(on.rows[0].available);
        expect(on.rows[0].totalPulls).toBe(on.rows[0].spent + on.rows[0].freePulls);
    });

    it("totals them separately from what was committed", () => {
        const { totals } = plan({ countFreePulls: true });
        expect(totals.freePulls).toBeGreaterThan(0);
        expect(totals.spent).toBe(0);
    });
});

describe("the one-click presets", () => {
    it("asks for the spark less what the banner gives away", () => {
        // The ledger above already settles the arithmetic: 276 committed plus this
        // banner's own 24 free rolls is the 300 the bonus copy wants. The preset used to
        // offer a flat 300 and so bought 24 rolls of nothing.
        const rich = plan({ countFreePulls: true }, 200_000);
        const limited = rich.rows[0];
        expect(limited.freePulls).toBe(24);
        expect(planTargets(limited).spark).toBe(276);

        // With the free rolls switched off there is nothing to subtract.
        const plain = plan({ countFreePulls: false }, 200_000);
        expect(plain.rows[0].freePulls).toBe(0);
        expect(planTargets(plain.rows[0]).spark).toBe(300);
    });

    it("never asks for a negative commitment", () => {
        const rich = plan({ countFreePulls: true }, 200_000);
        for (const row of rich.rows) {
            const { spark, guarantee } = planTargets(row);
            if (spark !== null) expect(spark).toBeGreaterThanOrEqual(0);
            if (guarantee !== null) expect(guarantee).toBeGreaterThanOrEqual(0);
        }
    });

    it("offers the whole bank as `max`", () => {
        const p = plan();
        expect(planTargets(p.rows[0]).max).toBe(p.rows[0].available);
    });
});

describe("the memoised goal estimate", () => {
    it("answers the same question with the same figure, whichever array carries the counter", () => {
        const model = bannerModel({ ruleType: "LIMITED", featuredCount: 2, poolId: "A" });
        const request = { copiesA: 2, copiesB: 1 };
        // Two DIFFERENT arrays holding the same distribution: the cache is keyed on the
        // content, because a rebuild hands back an equal-but-new array and identity
        // would call that a different question and pay for the walk twice.
        const first = pityAt(17);
        const second = pityAt(17);
        expect(first).not.toBe(second);
        expect(goalEstimateFor(model, request, first)).toEqual(goalEstimateFor(model, request, second));

        // And a genuinely different counter is a different answer.
        const deep = goalEstimateFor(model, request, pityAt(60));
        expect(deep.p50).toBeLessThan(goalEstimateFor(model, request, first).p50);
    });
});
