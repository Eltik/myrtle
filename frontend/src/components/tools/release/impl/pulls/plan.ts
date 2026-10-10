/**
 * The ledger that turns a list of banners into a plan.
 *
 * Rolls are a single pool: committing them to an early banner is what makes a later one
 * unaffordable. Two things carry forward down the timeline, rolls and PITY. A banner sharing
 * a counter with the one before it starts where that one left off. The leftover is a
 * distribution, not a number: collapsing it to its mean loses the soft-pity curvature.
 * `odds.ts` hands the distribution back for this.
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
    /** Whether this banner reaches its bonus copy of the limited operator, when it has one. */
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

/**
 * Everything about a model that changes a walk's answer, as a cache key. Two collab
 * pools differ only in whether the handover repeats, and two keys that drop that
 * would hand one pool the other's estimate.
 */
function modelKey(model: IBannerModel): string {
    const g = model.guarantee;
    return `${model.ruleType}|${model.rateUp}|${model.featuredCount}|${model.shareEach}|${g.kind}|${g.repeat ? "loop" : ""}|${model.spark ?? ""}`;
}

/** The answer for a goal that cannot be reached: a banner with no rate-up has no operator to aim at. */
const UNREACHABLE: IGoalEstimate = { p50: 0, p90: 0, mean: 0, unresolved: 1 };

/**
 * A pity distribution's CONTENT, as a string.
 *
 * The goal walk is the dearest thing on this tab, up to 297 ms for six copies of each, and the
 * rebuild after a stepper press repeated it. The walk's third input is a `Float64Array` that
 * the rebuild hands back as an equal-but-NEW object, so identity said "different question"
 * where content said "same". Hashing the bytes costs ~800 imuls against 297 ms, and the
 * `WeakMap` hashes each array once however many goals are priced against it.
 */
const DIST_KEYS = new WeakMap<Float64Array, string>();

function distKey(dist: Float64Array | null): string {
    if (!dist) return "cold";
    const hit = DIST_KEYS.get(dist);
    if (hit !== undefined) return hit;
    const bytes = new Uint8Array(dist.buffer, dist.byteOffset, dist.byteLength);
    let h1 = 0x811c9dc5;
    let h2 = 0x01000193;
    for (let i = 0; i < bytes.length; i++) {
        h1 = Math.imul(h1 ^ bytes[i], 0x01000193);
        h2 = Math.imul(h2 + bytes[i] + i, 0x85ebca6b);
    }
    const key = `${(h1 >>> 0).toString(36)}.${(h2 >>> 0).toString(36)}.${dist.length}`;
    DIST_KEYS.set(dist, key);
    return key;
}

const GOAL_CACHE = new Map<string, IGoalEstimate>();
/** Forty rows times a handful of distinct goals each; the bound is slack, not tight. */
const GOAL_CACHE_MAX = 512;

/**
 * Rolls to reach a specific pair of copy counts, memoised on (model, request, start).
 *
 * Both the click handler and the rebuild go through here, so stepping a potential
 * from 2 to 3 and back to 2 costs one walk rather than four.
 */
export function goalEstimateFor(model: IBannerModel, request: { copiesA: number; copiesB: number }, dist: Float64Array | null): IGoalEstimate {
    const key = `${modelKey(model)}|${request.copiesA}|${request.copiesB}|${distKey(dist)}`;
    let hit = GOAL_CACHE.get(key);
    if (hit === undefined) {
        hit = pullsToGoal(model, request, { startPityDist: dist, startPity: 0 });
        if (GOAL_CACHE.size >= GOAL_CACHE_MAX) GOAL_CACHE.clear();
        GOAL_CACHE.set(key, hit);
    }
    return hit;
}

export function maxPotFor(model: IBannerModel): IGoalEstimate {
    if (model.rateUp === "none") return UNREACHABLE;
    const key = modelKey(model);
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
    /** Rolls since the user's last 6* on the Standard counter, at the start. */
    standardPity: number;
    /** Rolls since the user's last 6* on the Kernel counter, at the start. A separate counter. */
    kernelPity: number;
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
 * A banner already open and still running falls back to today: `dayAt` has no day earlier than
 * today, so a live banner read as zero available whatever the player held, every commitment
 * to it was an overrun that never left the pool, and the next banner reported the same figure.
 * Today is what the player can spend on it right now.
 */
function bankAt(days: IProjectedDay[], at: number, spendOriginite: boolean): number {
    const day = dayAt(days, at) ?? days[0];
    if (!day) return 0;
    return spendOriginite ? day.pullsWithOriginite : day.pulls;
}

export function buildPlan({ banners, days, model, today, standardPity, kernelPity, allocations, targets, spendOriginite, countFreePulls, limit = 40 }: IPlanInput): IPlan {
    const upcoming: { banner: ReleaseBanner; enStart: number }[] = [];
    for (const banner of banners) {
        if (banner.returning) continue;
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
        standard: pityAt(standardPity),
        kernel: pityAt(kernelPity),
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

    let committed = 0;
    const rows: IPlanRow[] = [];
    let totalAllocated = 0;
    let totalShortfall = 0;
    let totalFree = 0;
    let shortBanners = 0;

    for (const { banner, enStart } of window) {
        const featured = featuredOf(banner);
        // The published share and loop come off the pool's own data; without them the share
        // is inferred from the rule type. Orienteering and ATTAIN keep their own split.
        const bm = bannerModel({
            ruleType: banner.ruleType,
            featuredCount: Math.max(1, featured.length),
            poolId: banner.cnPoolId,
            declaredShareEach: banner.declaredShareEach ?? undefined,
            linkageLoopAt: banner.linkageLoopAt ?? undefined,
        });
        // A pick only counts while the operator is still on the banner, so a roster
        // correction cannot leave a stale id steering the estimate.
        const picked: Record<string, number> = {};
        // A banner with no rate-up has no operator a potential could be aimed at.
        for (const [id, n] of Object.entries(bm.rateUp === "none" ? {} : (targets[banner.cnPoolId] ?? {}))) {
            if (featured.includes(id) && n > 0) picked[id] = Math.min(MAX_POT_COPIES, Math.floor(n));
        }
        const totalCopies = Object.values(picked).reduce((sum, n) => sum + n, 0);

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
        const estimateKey = `${modelKey(bm)}|${idOf(shared)}`;
        let estimate = estimates.get(estimateKey);
        if (estimate === undefined) {
            estimate = bm.rateUp === "none" ? { horizon: ESTIMATE_HORIZON, specific: UNREACHABLE, any: UNREACHABLE, both: null } : pullsToTarget(bm, { startPityDist: shared, startPity: 0, horizon: ESTIMATE_HORIZON });
            estimates.set(estimateKey, estimate);
        }

        // Only a shared counter keeps what this banner did to it, and a banner the
        // player skips (no rolls at all) did nothing to it: the counter carries past
        // it as the SAME array, neither reset nor accumulated. Writing back the
        // zero-roll `endPity` would hand over a renormalised copy instead.
        if (shared && totalPulls > 0) carried[bm.scope === "kernel" ? "kernel" : "standard"] = odds.endPity;

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
                    : goalEstimateFor(
                          bm,
                          {
                              // Slot A is the banner's first featured operator and slot B
                              // its second, which is the pairing the grid tracks. On a
                              // LIMITED pool slot A is also the one the 300-roll bonus
                              // copy goes to: the backend lists `limitParam.limitedCharId`
                              // first, and the sidecar's `limitedChar` is
                              // `upCharInfo.charIdList[0]` on all 24 EN and 26 CN pools.
                              copiesA: picked[featured[0]] ?? 0,
                              copiesB: featured.length > 1 ? (picked[featured[1]] ?? 0) : 0,
                          },
                          shared,
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
 * `spark` is the roll that brings the Limited bonus copy where there is one, `guarantee`
 * the roll at which a forced rate-up binds, and `max` everything still in the bank.
 *
 * Both thresholds count every roll made on the banner, free ones included: `sparkMet` above
 * tests `spent + freePulls >= spark`, and the odds are read at `totalPulls`. What the player
 * has to COMMIT is the threshold less what the banner gives them (a LIMITED banner with 24
 * free rolls would otherwise offer "Spark 300" and spend 324 against a 300-roll bonus).
 */
export function planTargets(row: IPlanRow): { spark: number | null; guarantee: number | null; max: number } {
    const g = row.model.guarantee;
    const rawGuarantee = g.kind === "linkage" ? (g.at ?? null) : g.kind === "selection" ? (g.first ?? null) : null;
    const commit = (threshold: number | null): number | null => (threshold === null ? null : Math.max(0, threshold - row.freePulls));
    return { spark: commit(row.model.spark), guarantee: commit(rawGuarantee), max: row.available };
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
