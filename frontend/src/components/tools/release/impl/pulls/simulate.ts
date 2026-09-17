/**
 * Rolling the banner for real.
 *
 * `odds.ts` answers "what are my chances" exactly; this answers "show me what
 * happens", which is a different question and needs sampling. The puller is
 * stateful on purpose so the UI can hand the user a button and let them pull one at
 * a time, with pity and the once-only guarantees carrying across clicks exactly as
 * they do in game.
 */

import { BASE_FIVE_RATE, BASE_FOUR_RATE, BASE_THREE_RATE, GUARANTEE_FIVE_COUNT, type IBannerModel, sixStarRate } from "./rates";

export type PullSlot = "featuredA" | "featuredB" | "offRate" | "five" | "four" | "three";

export interface IPullOutcome {
    /** 1-based index of this roll within the session. */
    index: number;
    rarity: 6 | 5 | 4 | 3;
    slot: PullSlot;
    /** Rolls since the previous 6*, counting this one. */
    pityAt: number;
    /** True when a banner guarantee, not the base odds, decided this outcome. */
    guaranteed: boolean;
}

export interface IPuller {
    pull(): IPullOutcome;
    pullMany(n: number): IPullOutcome[];
    readonly state: {
        pulls: number;
        pity: number;
        copiesA: number;
        copiesB: number;
        sixStars: number;
        fiveStars: number;
    };
}

type Rng = () => number;

/** The non-6* mass renormalised over the 5/4/3 bands, which keep their 8:50:40 ratio. */
const LOWER_TOTAL = BASE_FIVE_RATE + BASE_FOUR_RATE + BASE_THREE_RATE;

export function createPuller(model: IBannerModel, startPity = 0, spentOnBanner = 0, rng: Rng = Math.random): IPuller {
    let pity = Math.max(0, Math.floor(startPity));
    let pulls = Math.max(0, Math.floor(spentOnBanner));
    let sinceFive = 0;
    let copiesA = 0;
    let copiesB = 0;
    let sixStars = 0;
    let fiveStars = 0;
    const twoFeatured = model.featuredCount >= 2;
    const g = model.guarantee;
    let attainSpent = false;

    function resolveSix(onBanner: number): { slot: PullSlot; guaranteed: boolean } {
        if (g.kind === "selection") {
            const have = (copiesA > 0 ? 1 : 0) + (copiesB > 0 ? 1 : 0);
            if (have === 0 && onBanner >= (g.first ?? Number.POSITIVE_INFINITY)) {
                if (!twoFeatured) return { slot: "featuredA", guaranteed: true };
                return { slot: rng() < 0.5 ? "featuredA" : "featuredB", guaranteed: true };
            }
            // Only when there IS another rate-up to be forced into.
            if (twoFeatured && have === 1 && onBanner >= (g.second ?? Number.POSITIVE_INFINITY)) {
                return { slot: copiesA > 0 ? "featuredB" : "featuredA", guaranteed: true };
            }
        }
        if (g.kind === "attain" && !attainSpent) {
            attainSpent = true;
            return { slot: "offRate", guaranteed: true };
        }
        const r = rng();
        if (r < model.shareEach) return { slot: "featuredA", guaranteed: false };
        if (twoFeatured && r < model.shareEach * 2) return { slot: "featuredB", guaranteed: false };
        return { slot: "offRate", guaranteed: false };
    }

    function pull(): IPullOutcome {
        pulls += 1;
        const pityAt = pity + 1;
        const p6 = sixStarRate(pityAt);

        // The collab handover lands on its roll regardless of what that roll rolled.
        if (g.kind === "linkage" && copiesA === 0 && pulls === (g.at ?? -1)) {
            pity = 0;
            sinceFive = 0;
            sixStars += 1;
            copiesA += 1;
            return { index: pulls, rarity: 6, slot: "featuredA", pityAt, guaranteed: true };
        }

        if (rng() < p6) {
            pity = 0;
            sinceFive = 0;
            sixStars += 1;
            const { slot, guaranteed } = resolveSix(pulls);
            if (slot === "featuredA") copiesA += 1;
            else if (slot === "featuredB") copiesB += 1;
            return { index: pulls, rarity: 6, slot, pityAt, guaranteed };
        }

        pity = pityAt;
        sinceFive += 1;
        // Every pool ships Guarantee5Avail 1 / Guarantee5Count 10: a 5* or better
        // must appear within any ten rolls.
        if (sinceFive >= GUARANTEE_FIVE_COUNT) {
            sinceFive = 0;
            fiveStars += 1;
            return { index: pulls, rarity: 5, slot: "five", pityAt, guaranteed: true };
        }

        const rest = 1 - p6;
        const r = rng() * rest;
        const five = (BASE_FIVE_RATE / LOWER_TOTAL) * rest;
        const four = (BASE_FOUR_RATE / LOWER_TOTAL) * rest;
        if (r < five) {
            sinceFive = 0;
            fiveStars += 1;
            return { index: pulls, rarity: 5, slot: "five", pityAt, guaranteed: false };
        }
        if (r < five + four) return { index: pulls, rarity: 4, slot: "four", pityAt, guaranteed: false };
        return { index: pulls, rarity: 3, slot: "three", pityAt, guaranteed: false };
    }

    return {
        pull,
        pullMany(n: number): IPullOutcome[] {
            const out: IPullOutcome[] = [];
            for (let i = 0; i < Math.max(0, Math.floor(n)); i++) out.push(pull());
            return out;
        },
        get state() {
            return { pulls, pity, copiesA, copiesB, sixStars, fiveStars };
        },
    };
}

export interface ITargetStats {
    runs: number;
    /** Share of runs that reached `copies` within `maxPulls`. */
    success: number;
    /** Rolls used, by percentile, over successful runs. */
    p50: number;
    p75: number;
    p90: number;
    p95: number;
    mean: number;
    /** `histogram[i]` counts runs finishing in bucket `i` of `bucket` rolls. */
    histogram: number[];
    bucket: number;
}

function percentile(sorted: number[], q: number): number {
    if (sorted.length === 0) return 0;
    const i = Math.min(sorted.length - 1, Math.max(0, Math.ceil(q * sorted.length) - 1));
    return sorted[i];
}

/**
 * How many rolls it takes to reach `copies` of the featured operator, sampled.
 *
 * The exact answer for one copy is in `odds.ts`; this exists for the copy counts and
 * the shape of the tail, which is what a player actually wants to look at.
 */
export function simulateToTarget(model: IBannerModel, opts: { runs?: number; copies?: number; maxPulls?: number; startPity?: number; spentOnBanner?: number; bucket?: number; rng?: Rng } = {}): ITargetStats {
    const runs = Math.max(1, Math.floor(opts.runs ?? 20_000));
    const copies = Math.max(1, Math.floor(opts.copies ?? 1));
    const maxPulls = Math.max(1, Math.floor(opts.maxPulls ?? 1000));
    const bucket = Math.max(1, Math.floor(opts.bucket ?? 10));
    const rng = opts.rng ?? Math.random;
    const used: number[] = [];
    const histogram = new Array(Math.ceil(maxPulls / bucket)).fill(0);
    let success = 0;

    for (let r = 0; r < runs; r++) {
        const puller = createPuller(model, opts.startPity ?? 0, opts.spentOnBanner ?? 0, rng);
        let spent = 0;
        let got = 0;
        // Counting the primary copies here rather than reading `puller.state` each
        // iteration: that getter builds a fresh six-field object per call, which at
        // six copies over ten thousand runs is millions of throwaway allocations.
        while (spent < maxPulls && got < copies) {
            if (puller.pull().slot === "featuredA") got += 1;
            spent += 1;
        }
        if (got >= copies) {
            success += 1;
            used.push(spent);
            histogram[Math.min(histogram.length - 1, Math.floor((spent - 1) / bucket))] += 1;
        }
    }

    used.sort((a, b) => a - b);
    const mean = used.length > 0 ? used.reduce((s, v) => s + v, 0) / used.length : 0;
    return {
        runs,
        success: success / runs,
        p50: percentile(used, 0.5),
        p75: percentile(used, 0.75),
        p90: percentile(used, 0.9),
        p95: percentile(used, 0.95),
        mean,
        histogram,
        bucket,
    };
}
