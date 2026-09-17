/**
 * The ledger that turns a list of banners into a plan.
 *
 * Without this the tab could only answer "how many rolls will I have by date X",
 * which it answered identically for every banner, as though spending on one had no
 * bearing on the next. A plan is the opposite claim: rolls are a single pool, and
 * committing them to an early banner is exactly what makes a later one unaffordable.
 *
 * Two things are carried forward down the timeline. Rolls, which is the obvious one.
 * And PITY, which is not: a banner sharing a counter with the one before it does not
 * start where the user's account started, it starts wherever the previous banner left
 * off. That leftover is a distribution rather than a number, because after spending
 * sixty rolls the player is somewhere across a spread of counters, and collapsing it
 * to its mean would throw away the soft-pity curvature that makes those rolls worth
 * what they are worth. `odds.ts` hands the distribution back for exactly this.
 */

import type { LagModel } from "#/types/generated/LagModel";
import type { ReleaseBanner } from "#/types/generated/ReleaseBanner";
import { isPast, resolvedEnStart, sortKey } from "../helpers";
import { dayAt, type IProjectedDay } from "./income";
import { type IGoalEstimate, type IOddsResult, type ITargetEstimate, MAX_POT_COPIES, pityAt, pullOdds, pullsToGoal, pullsToMaxPot, pullsToTarget } from "./odds";
import { bannerModel, freePullsFor, type IBannerModel, type PityScope } from "./rates";

/**
 * How far the cost estimate looks before reporting the leftover tail.
 *
 * Chosen by sweeping the worst case, a CLASSIC pool with no forced rate-up and only a
 * quarter of the 6* band on the target. Its mean converges from below:
 *
 *     horizon   400      600      800     1000     1200
 *     mean      132.50   137.17   138.13   138.33   138.37
 *     tail      4.637%   0.951%   0.195%   0.040%   0.008%
 *
 * 800 leaves the mean 0.24 rolls short of converged and the tail under a fifth of a
 * percent, which is finer than a planner can act on, and it costs 6.5 ms a walk.
 * Every banner with a guarantee resolves far sooner and never reaches this.
 */
const ESTIMATE_HORIZON = 800;

export interface IPlanRow {
    key: string;
    banner: ReleaseBanner;
    /** Unix seconds of the banner's resolved EN start. */
    enStart: number;
    model: IBannerModel;
    /** The 6* operators this banner features, EN list preferred over CN. */
    featured: string[];
    /** Rolls the user asked to commit here. */
    allocated: number;
    /** Rolls banked and still unspent when this banner opens. */
    available: number;
    /** Rolls actually committed: the allocation, capped by what is in hand. */
    spent: number;
    /** How far the allocation overran the bank. Zero when the plan is affordable. */
    shortfall: number;
    /** Rolls still unspent after this banner. */
    leftover: number;
    /**
     * Free rolls this banner hands out, which expire with it. Counted here rather
     * than as income precisely because they cannot be saved for a later banner.
     */
    freePulls: number;
    /** Rolls actually thrown at this banner: what was committed plus what was free. */
    totalPulls: number;
    /** Odds at `spent`, not at the whole bank. */
    odds: IOddsResult;
    /** Rolls this banner is expected to need for the rate-up, from where it starts. */
    estimate: ITargetEstimate;
    /** Rolls to take the primary rate-up to maximum potential, six copies. */
    maxPot: IGoalEstimate;
    /** Copies wanted per featured operator. Absent or zero means not chased. */
    targets: Record<string, number>;
    /** Copies wanted across the banner, which is what the custom goal costs. */
    totalCopies: number;
    /**
     * Rolls to reach exactly what the user asked for on this banner, or null when
     * they have asked for nothing. This is the figure the plan is really about; the
     * four reference estimates are presets beside it.
     */
    goalEstimate: IGoalEstimate | null;
    /**
     * The counter this banner starts from, as the distribution the odds used. Handed
     * out so a caller can price a different goal on this banner without rebuilding
     * the plan, which is what the potential stepper needs on a click.
     */
    pityDist: Float64Array | null;
    /** Whether this banner reaches its outright exchange, when it has one. */
    sparkMet: boolean;
}

/**
 * Rolls to maximum potential, cached across the whole session.
 *
 * This walk is the dearest thing the tab does, 40 to 90 ms against 6.5 ms for the
 * other goals, and its answer depends only on the banner's model because it takes no
 * carried pity. A forty-row window holds a handful of distinct models, so this turns
 * forty expensive walks into about five, once.
 */
const MAX_POT_CACHE = new Map<string, IGoalEstimate>();

export function maxPotFor(model: IBannerModel): IGoalEstimate {
    const key = `${model.ruleType}|${model.featuredCount}|${model.shareEach}|${model.guarantee.kind}|${model.spark ?? ""}`;
    let hit = MAX_POT_CACHE.get(key);
    if (hit === undefined) {
        hit = pullsToMaxPot(model, { copies: MAX_POT_COPIES });
        MAX_POT_CACHE.set(key, hit);
    }
    return hit;
}

export interface IPlanTotals {
    /** Rolls accrued across the whole horizon. */
    income: number;
    allocated: number;
    spent: number;
    /** Free rolls across every banner in the window. */
    freePulls: number;
    /** Rolls left over at the end of the horizon. */
    remaining: number;
    /** Total by which allocations overran the bank, summed over banners. */
    shortfall: number;
    /** Banners the plan cannot pay for in full. */
    shortBanners: number;
}

export interface IPlanInput {
    banners: ReleaseBanner[];
    days: IProjectedDay[];
    model: LagModel | null;
    today: Date;
    /** Rolls since the user's last 6* on the shared counters, at the start. */
    pity: number;
    /** Rolls committed per banner key. Absent means nothing committed. */
    allocations: Record<string, number | undefined>;
    /** Copies wanted per operator, per banner key. Absent means none picked. */
    targets: Record<string, Record<string, number> | undefined>;
    /** Whether Originite Prime counts toward the bank. */
    spendOriginite: boolean;
    /** Whether a banner's own free pulls are counted toward its odds. */
    countFreePulls: boolean;
    limit?: number;
}

export interface IPlan {
    rows: IPlanRow[];
    totals: IPlanTotals;
}

/**
 * `enFeatured6` is the EN-server roster once it is known; `featured6` is the CN one.
 * The EN list wins when present because it is what the player will actually see.
 */
export function featuredOf(banner: ReleaseBanner): string[] {
    if (banner.enFeatured6.length > 0) return banner.enFeatured6;
    if (banner.overrideFeatured.length > 0) return banner.overrideFeatured;
    return banner.featured6;
}

/**
 * The bank a banner has to draw on, read on the day it opens.
 *
 * A banner that has ALREADY opened and is still running falls back to today, which it
 * did not before: `dayAt` has no day earlier than today to return, so a live banner
 * read as zero available no matter what the player was holding. Everything committed
 * to it was then an overrun that never left the pool, and the next banner down the
 * list went on reporting the same figure however much was poured into the live one.
 * Today is the honest answer for a banner already in progress: it is what the player
 * can actually spend on it right now.
 */
function bankAt(days: IProjectedDay[], at: number, spendOriginite: boolean): number {
    const day = dayAt(days, at) ?? days[0];
    if (!day) return 0;
    return spendOriginite ? day.pullsWithOriginite : day.pulls;
}

export function buildPlan({ banners, days, model, today, pity, allocations, targets, spendOriginite, countFreePulls, limit = 40 }: IPlanInput): IPlan {
    const upcoming: { banner: ReleaseBanner; enStart: number }[] = [];
    for (const banner of banners) {
        if (banner.standing) continue;
        if (isPast(sortKey(banner.resolution, banner.cnOpen, model), today)) continue;
        const enStart = resolvedEnStart(banner.resolution);
        if (enStart === null) continue;
        upcoming.push({ banner, enStart });
    }
    upcoming.sort((a, b) => a.enStart - b.enStart || a.banner.cnPoolId.localeCompare(b.banner.cnPoolId));
    const window = upcoming.slice(0, limit);

    // One carried counter per shared scope. An isolated banner reads neither and
    // writes neither, which is the whole content of "cleared at the end".
    const carried: Record<Exclude<PityScope, "isolated">, Float64Array> = {
        standard: pityAt(pity),
        kernel: pityAt(pity),
    };

    /**
     * The cost estimate depends only on the banner model and the counter it starts
     * from, and `carried` keeps the SAME array until a shared-scope banner actually
     * spends rolls. So consecutive rows on an untouched counter are the identical
     * question, and asking it once takes forty walks down to a handful.
     */
    const estimates = new Map<string, ITargetEstimate>();
    const distIds = new Map<Float64Array, number>();
    const idOf = (d: Float64Array | null): string => {
        if (!d) return "cold";
        let id = distIds.get(d);
        if (id === undefined) {
            id = distIds.size + 1;
            distIds.set(d, id);
        }
        return `d${id}`;
    };

    /**
     * The custom goal is the dearest walk the tab can be asked for: six copies of
     * each operator measures 297 ms. It depends only on the banner model, the two
     * counts and the counter it starts from, so the same question asked by two rows
     * is answered once.
     */
    const goals = new Map<string, IGoalEstimate>();
    const goalCached = (bm: IBannerModel, request: { copiesA: number; copiesB: number }, dist: Float64Array | null, distKey: string): IGoalEstimate => {
        const key = `${bm.ruleType}|${bm.featuredCount}|${bm.shareEach}|${request.copiesA}|${request.copiesB}|${distKey}`;
        let hit = goals.get(key);
        if (hit === undefined) {
            hit = pullsToGoal(bm, request, { startPityDist: dist, startPity: 0 });
            goals.set(key, hit);
        }
        return hit;
    };

    let committed = 0;
    const rows: IPlanRow[] = [];
    let totalAllocated = 0;
    let totalShortfall = 0;
    let totalFree = 0;
    let shortBanners = 0;

    for (const { banner, enStart } of window) {
        const featured = featuredOf(banner);
        // A pick only counts while the operator is still on the banner, so a roster
        // correction cannot leave a stale id steering the estimate.
        const picked: Record<string, number> = {};
        for (const [id, n] of Object.entries(targets[banner.cnPoolId] ?? {})) {
            if (featured.includes(id) && n > 0) picked[id] = Math.min(MAX_POT_COPIES, Math.floor(n));
        }
        const totalCopies = Object.values(picked).reduce((sum, n) => sum + n, 0);
        const bm = bannerModel({ ruleType: banner.ruleType, featuredCount: Math.max(1, featured.length), poolId: banner.cnPoolId });

        const available = Math.max(0, bankAt(days, enStart, spendOriginite) - committed);
        const allocated = Math.max(0, Math.floor(allocations[banner.cnPoolId] ?? 0));
        const spent = Math.min(allocated, available);
        const shortfall = allocated - spent;
        const freePulls = countFreePulls ? freePullsFor(banner.ruleType, banner.cnOpen, banner.cnEnd) : 0;
        // The free rolls are spent on this banner whether or not the player commits
        // anything, so the odds are computed over both.
        const totalPulls = spent + freePulls;

        const shared = bm.carryOver ? carried[bm.scope === "kernel" ? "kernel" : "standard"] : null;
        const odds = pullOdds(bm, totalPulls, {
            startPityDist: shared,
            startPity: 0,
            maxCopies: 1,
        });
        // What the banner is expected to COST, measured from the same place the
        // odds start, so a banner inheriting deep pity honestly reads cheaper.
        const estimateKey = `${bm.ruleType}|${bm.featuredCount}|${bm.shareEach}|${idOf(shared)}`;
        let estimate = estimates.get(estimateKey);
        if (estimate === undefined) {
            estimate = pullsToTarget(bm, { startPityDist: shared, startPity: 0, horizon: ESTIMATE_HORIZON });
            estimates.set(estimateKey, estimate);
        }

        // Only a shared counter keeps what this banner did to it.
        if (shared) carried[bm.scope === "kernel" ? "kernel" : "standard"] = odds.endPity;

        committed += spent;
        totalAllocated += allocated;
        totalShortfall += shortfall;
        totalFree += freePulls;
        if (shortfall > 0) shortBanners += 1;

        rows.push({
            key: banner.cnPoolId,
            banner,
            enStart,
            model: bm,
            featured,
            allocated,
            available,
            spent,
            shortfall,
            leftover: available - spent,
            freePulls,
            totalPulls,
            odds,
            estimate,
            maxPot: maxPotFor(bm),
            targets: picked,
            totalCopies,
            goalEstimate:
                totalCopies === 0
                    ? null
                    : goalCached(
                          bm,
                          {
                              // Slot A is the banner's first featured operator and slot B
                              // its second, which is the pairing the grid tracks.
                              copiesA: picked[featured[0]] ?? 0,
                              copiesB: featured.length > 1 ? (picked[featured[1]] ?? 0) : 0,
                          },
                          shared,
                          idOf(shared),
                      ),
            pityDist: shared,
            // The spark counts every roll made on the banner, free ones included.
            sparkMet: bm.spark !== null && totalPulls >= bm.spark,
        });
    }

    const horizonBank = days.length > 0 ? (spendOriginite ? days[days.length - 1].pullsWithOriginite : days[days.length - 1].pulls) : 0;
    return {
        rows,
        totals: {
            income: horizonBank,
            allocated: totalAllocated,
            spent: committed,
            freePulls: totalFree,
            remaining: Math.max(0, horizonBank - committed),
            shortfall: totalShortfall,
            shortBanners,
        },
    };
}

/**
 * The sensible commitments for a banner, offered as one-click targets.
 * `spark` is the outright exchange where one exists, `guarantee` the roll at which a
 * forced rate-up binds, and `max` everything still in the bank.
 */
export function planTargets(row: IPlanRow): { spark: number | null; guarantee: number | null; max: number } {
    const g = row.model.guarantee;
    const guarantee = g.kind === "linkage" ? (g.at ?? null) : g.kind === "selection" ? (g.first ?? null) : null;
    return { spark: row.model.spark, guarantee, max: row.available };
}

/** The plan as CSV, one row per banner, for taking the numbers elsewhere. */
export function planToCsv(rows: IPlanRow[], formatDate: (at: number) => string): string {
    const head = ["Banner", "Rule", "EN date", "Available", "Allocated", "Spent", "Leftover", "Chance", "Any rate-up"];
    const lines = [head.join(",")];
    for (const r of rows) {
        const cells = [r.banner.nameCn, r.banner.ruleType, formatDate(r.enStart), String(r.available), String(r.allocated), String(r.spent), String(r.leftover), (r.odds.specific * 100).toFixed(2), (r.odds.any * 100).toFixed(2)];
        lines.push(cells.map((c) => (c.includes(",") || c.includes('"') ? `"${c.replaceAll('"', '""')}"` : c)).join(","));
    }
    return lines.join("\n");
}
