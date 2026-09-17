/**
 * The gacha model for one banner, derived from the game's own declared data.
 *
 * Every constant here is READ OFF the shipped gamedata rather than assumed. The
 * base band rates come from `availCharInfo.perAvailList` and are identical on all
 * 360 EN pools that carry a pool-detail sidecar: 6* 0.02, 5* 0.08, 4* 0.50,
 * 3* 0.40. The rate-up shares come from `upCharInfo.perCharList`, where `percent`
 * is the share taken by EACH featured character and `count` equals the length of
 * `charIdList`, so the band's total rate-up share is `percent * count`. Tallied
 * over the sidecar that resolves to exactly two values and nothing else:
 *
 *     LIMITED                      0.35 x 2 = 0.70
 *     SINGLE, LINKAGE              0.50 x 1 = 0.50
 *     NORMAL, DOUBLE, CLASSIC,     0.25 x 2 = 0.50
 *     CLASSIC_DOUBLE
 *
 * One pool deviates: `NORM_EN_6_0_6` ("Earthborn Metals", Nian and Haak) is typed
 * NORMAL but carries the limited 0.35 x 2 = 0.70 split. It is a 2019 banner, so it
 * never reaches a forward-looking planner, but `LIMITED_SHARE_POOL_IDS` keeps the
 * derivation honest rather than silently wrong.
 */

/** Per-roll 6* rate before soft pity, from `perAvailList[rarityRank=5].totalPercent`. */
export const BASE_SIX_RATE = 0.02;
/** Per-roll 5* rate, from `perAvailList[rarityRank=4].totalPercent`. */
export const BASE_FIVE_RATE = 0.08;
/** Per-roll 4* rate, from `perAvailList[rarityRank=3].totalPercent`. */
export const BASE_FOUR_RATE = 0.5;
/** Per-roll 3* rate, from `perAvailList[rarityRank=2].totalPercent`. */
export const BASE_THREE_RATE = 0.4;

/** Rolls without a 6* before the rate starts climbing. */
export const SOFT_PITY_START = 50;
/** Percentage points added per roll once soft pity is active. Additive, not compounding. */
export const SOFT_PITY_INCREMENT = 0.02;
/**
 * The roll on which the schedule reaches 100%. Derived, not asserted:
 * `0.02 + 0.02 * (n - 50) = 1` solves to n = 99.
 */
export const HARD_PITY_PULL = SOFT_PITY_START + Math.round((1 - BASE_SIX_RATE) / SOFT_PITY_INCREMENT);

/** Every pool on both servers ships `Guarantee5Avail: 1, Guarantee5Count: 10`. */
export const GUARANTEE_FIVE_COUNT = 10;

/** Orundum for one roll. */
export const ORUNDUM_PER_PULL = 600;
/** Orundum one Originite Prime converts into. */
export const ORUNDUM_PER_ORIGINITE = 180;

/** The banner families that share a pity counter, per the game's own rule text. */
export type PityScope = "standard" | "kernel" | "isolated";

export type GuaranteeKind = "linkage" | "selection" | "attain" | "none";

export interface IGuarantee {
    kind: GuaranteeKind;
    /** LINKAGE: the roll by which the collab 6* is handed over. */
    at?: number;
    /** DOUBLE / CLASSIC_DOUBLE: rolls after which the next 6* is forced on-rate. */
    first?: number;
    /** DOUBLE / CLASSIC_DOUBLE: rolls after which the OTHER rate-up is forced. */
    second?: number;
}

export interface IBannerModel {
    ruleType: string;
    /** Share of the 6* band taken by EACH featured operator. */
    shareEach: number;
    /** Share of the 6* band taken by all featured operators together. */
    shareTotal: number;
    /** How many 6* operators are rate-up. */
    featuredCount: number;
    scope: PityScope;
    /** Whether pity survives the banner closing. False means it resets to zero. */
    carryOver: boolean;
    guarantee: IGuarantee;
    /** Rolls that buy the featured operator outright in the banner shop, or null. */
    spark: number | null;
    /** True when the featured share could not be read and was inferred from the rule type. */
    inferred: boolean;
}

/**
 * Pools typed NORMAL that nonetheless use the 0.70 limited-style split.
 *
 * One banner on each server, "Earthborn Metals" (地生五金, Nian and Haak), a 2019
 * pool that predates the LIMITED rule type. Both ids are READ from the shipped
 * sidecars rather than derived: the CN id is `NORM_6_0_4` and the EN one is
 * `NORM_EN_6_0_6`, which do not correspond, so guessing one from the other gets it
 * wrong. Tallying `percent * count` over all 446 CN and 360 EN pools turns up these
 * two and nothing else.
 */
const LIMITED_SHARE_POOL_IDS = new Set(["NORM_EN_6_0_6", "NORM_6_0_4"]);

const STANDARD_RULES = new Set(["NORMAL", "SINGLE", "DOUBLE", "BACKFLOW"]);
/**
 * CLASSIC_ATTAIN is deliberately NOT here despite the CLASSIC prefix. Its own rule
 * text says the cumulative rolls "do not carry over between different
 * [Celebration - Kernel] banners" and are "kept separate from accumulated rolls on
 * other banners", so it is isolated, not kernel. The prefix is a naming accident.
 */
const KERNEL_RULES = new Set(["CLASSIC", "CLASSIC_DOUBLE", "FESCLASSIC"]);

function scopeFor(ruleType: string): PityScope {
    if (STANDARD_RULES.has(ruleType)) return "standard";
    if (KERNEL_RULES.has(ruleType)) return "kernel";
    return "isolated";
}

/**
 * Which forced-rate-up rule a banner carries, MEASURED from the rule text shipped in
 * each pool's own detail rather than reasoned about. Counting pools whose text names
 * each mechanism, over the whole EN sidecar:
 *
 *     SINGLE           [Focused Selection]  at 150            30 of 30
 *     DOUBLE           [Standard Selection] at 150 then 300   23 of 23
 *     CLASSIC_DOUBLE   [Kernel Selection]   at 150 then 300     2 of 2
 *     LINKAGE          handover within 120                      6 of 6
 *     NORMAL, CLASSIC, FESCLASSIC, LIMITED, ATTAIN, CLASSIC_ATTAIN: no such text
 *
 * SINGLE is the one that is easy to get wrong, and this code did get it wrong at
 * first. Its text reads "if you make 150 headhunting attempts without receiving the
 * current rate-up 6-star operator, the next 6-star operator received is guaranteed to
 * be the current rate-up 6-star operator", which is the same threshold the two-rate-up
 * pools use, on a banner with only one operator to force. LIMITED, by contrast,
 * carries NO forced-rate-up text: its 300 is the Data Contract exchange, a separate
 * currency mechanism, not a redirected roll.
 */
function guaranteeFor(ruleType: string): IGuarantee {
    switch (ruleType) {
        case "LINKAGE":
            // "you are guaranteed to receive [X] within 120 attempts, only once"
            return { kind: "linkage", at: 120 };
        case "SINGLE":
            // One rate-up, so there is no "other one" for a second threshold to force.
            return { kind: "selection", first: 150 };
        case "DOUBLE":
        case "CLASSIC_DOUBLE":
            // "after a total of 150 attempts the next 6 star is guaranteed to be one of
            // the rate-ups; after 300, the other one". Cleared when the banner ends.
            return { kind: "selection", first: 150, second: 300 };
        case "ATTAIN":
        case "CLASSIC_ATTAIN":
            // "the first 6-star Operator obtained is guaranteed to be an unowned 6-star".
            return { kind: "attain" };
        default:
            return { kind: "none" };
    }
}

/**
 * The share of the 6* band the featured operators take together.
 *
 * Read from the pool detail when it is present. The sidecar lags brand-new pools
 * (41 of 401 EN pools had no entry at the time of writing), so the rule type is the
 * fallback and the result is flagged `inferred` when that path is taken.
 */
function shareTotalFor(ruleType: string, poolId: string | null): number {
    if (poolId !== null && LIMITED_SHARE_POOL_IDS.has(poolId)) return 0.7;
    return ruleType === "LIMITED" ? 0.7 : 0.5;
}

export interface IBannerModelInput {
    ruleType: string;
    featuredCount: number;
    poolId?: string | null;
    /** `upCharInfo.perCharList[rarityRank=5].percent` when the sidecar supplied it. */
    declaredShareEach?: number | null;
}

export function bannerModel({ ruleType, featuredCount, poolId = null, declaredShareEach = null }: IBannerModelInput): IBannerModel {
    const count = Math.max(1, featuredCount);
    const declared = declaredShareEach !== null && Number.isFinite(declaredShareEach) && declaredShareEach > 0;
    const shareEach = declared ? (declaredShareEach as number) : shareTotalFor(ruleType, poolId) / count;
    return {
        ruleType,
        shareEach,
        shareTotal: Math.min(1, shareEach * count),
        featuredCount: count,
        scope: scopeFor(ruleType),
        carryOver: scopeFor(ruleType) !== "isolated",
        guarantee: guaranteeFor(ruleType),
        spark: ruleType === "LIMITED" ? 300 : null,
        inferred: !declared,
    };
}

/**
 * The 6* rate on the `n`th roll counting from the last 6*, where `n` is 1-based.
 * Flat at the base rate through `SOFT_PITY_START`, then additive to a 100% ceiling.
 */
export function sixStarRate(n: number): number {
    if (n <= SOFT_PITY_START) return BASE_SIX_RATE;
    return Math.min(1, BASE_SIX_RATE + SOFT_PITY_INCREMENT * (n - SOFT_PITY_START));
}

/** One-time free ten-roll permit handed out on a Limited or collab banner. */
export const FREE_TEN_ROLL = 10;

/**
 * Free headhunting a banner gives away, derived from its own run length.
 *
 * A Limited banner gives a one-time ten-roll permit plus one free single a day for as
 * long as it runs, which on the 14-day run every Limited pool in the data uses comes
 * to 10 + 14 = 24. That is the "+24 free pulls" figure, and deriving it from the
 * banner's real dates beats hardcoding it: the data holds one 13-day Limited pool,
 * which correctly yields 23.
 *
 * A collab gives two ten-roll permits, on day 1 and day 8, and no dailies: 20.
 *
 * These do NOT roll into the bank, which is the whole reason they are counted per
 * banner rather than as income. The daily single expires at the next daily reset and
 * the permits expire with the banner, so an unspent free pull is simply lost.
 *
 * An earlier version of this comment flagged reruns as an open question, on the
 * grounds that a rerun would pay none of this and the feed carries no rerun flag.
 * SETTLED: Limited banners do not rerun, so every Limited pool is a first run and
 * every one of them pays. The data agrees, in that all 26 Limited pools in CN carry
 * their own `limitedCharId`, a debut operator.
 */
export function freePullsFor(ruleType: string, openTime: number, endTime: number): number {
    const days = Math.max(1, Math.round((endTime - openTime) / 86_400));
    if (ruleType === "LIMITED") return FREE_TEN_ROLL + days;
    if (ruleType === "LINKAGE") return FREE_TEN_ROLL * 2;
    return 0;
}
