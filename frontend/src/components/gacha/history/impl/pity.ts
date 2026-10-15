import { type PityScope, scopeFor } from "#/components/tools/release/impl/pulls/rates";
import { classifyClientGachaGroup, type IBanner, type IGachaItem } from "#/lib/api/gacha";

/**
 * Pool-id prefixes of the isolated pools that land in the Standard or Kernel bucket,
 * used ONLY when the banner list cannot answer. It cannot in two cases: while the
 * `/static/banners` query is loading or has failed (the map is empty), and for a pool
 * newer than the static data.
 */
const ISOLATED_PREFIXES = ["RETURN_", "ATTAIN_", "CLASSIC_ATTAIN_"];

/**
 * Starter Headhunting. Its pools are absent from the banner list (they live in
 * gacha_table's `NewbeeGachaPoolClient`), so the prefix always decides. Each pool
 * is 21 pulls at 380 Orundum (`GachaTimes`, `GachaPrice`) and its own text says
 * "All events guarantee at least 1 6★ Operator": the 6* it hands out is that
 * guarantee, not a Kernel pity reset. Measured on local Postgres (2026-10-10),
 * counting these in the Kernel bucket changed 76 users' Kernel pity.
 */
const STARTER_PREFIX = "BOOT_";

/**
 * Whether a pull was made on a pool whose pity is its own, so it must neither
 * accumulate nor reset the Standard or Kernel counter.
 *
 * Measured on local Postgres (2026-10-10), these would otherwise land in the
 * shared buckets: ATTAIN (7,329 rows) in Standard, because its wire `typeName`
 * is "normal"; CLASSIC_ATTAIN (4,841) in Kernel, by its `CLASSIC_` prefix;
 * BACKFLOW (210) in Standard. SPECIAL (Orienteering) is Standard Headhunting
 * and counts. The banner's rule type decides through the planner's own
 * `scopeFor`, plus the pool's `returning` flag.
 */
export function isIsolatedPull(item: IGachaItem, bannersById: Map<string, IBanner>): boolean {
    if (item.poolId.startsWith(STARTER_PREFIX)) return true;
    const banner = bannersById.get(item.poolId);
    if (banner) return banner.returning || scopeFor(banner.gachaRuleType) === "isolated";
    return ISOLATED_PREFIXES.some((prefix) => item.poolId.startsWith(prefix));
}

/** Rolls since the last 6*, walking back from the newest pull. */
export function computePity(items: IGachaItem[]): number {
    const sorted = [...items].sort((a, b) => b.at - a.at);
    let pity = 0;
    for (const item of sorted) {
        if (item.star === "6") break;
        pity++;
    }
    return pity;
}

/** The Standard or Kernel counter: pulls on isolated pools are skipped entirely. */
export function sharedPity(items: IGachaItem[], bannersById: Map<string, IBanner>): number {
    return computePity(items.filter((item) => !isIsolatedPull(item, bannersById)));
}

/**
 * Limited and Collab pity does NOT carry between banners in-game: each pool has
 * its own counter that vanishes when the banner closes. So the bucket's pity is
 * the newest pull's banner alone, and 0 once that banner has ended.
 */
export function pityWithLimitedReset(items: IGachaItem[], bannersById: Map<string, IBanner>, nowSec: number): { pity: number; reset: boolean; resetReason?: string } {
    if (items.length === 0) return { pity: 0, reset: false };
    const sorted = [...items].sort((a, b) => b.at - a.at);
    const lastPullPoolId = sorted[0].poolId;
    const banner = bannersById.get(lastPullPoolId);
    // Static-data endTime is unix-seconds; pull `at` is unix-ms.
    if (banner && banner.endTime > 0 && nowSec > banner.endTime) {
        return { pity: 0, reset: true, resetReason: banner.gachaPoolName };
    }
    return { pity: computePity(items.filter((item) => item.poolId === lastPullPoolId)), reset: false };
}

/**
 * The pity counter a pull rolls on: the shared Standard or Kernel counter, or the
 * pool's own (Limited, Collab and every isolated pool). With no banner entry the
 * pull's bucket decides, as `classifyClientGachaGroup` sorts it.
 */
export function pityCounterKey(item: IGachaItem, bannersById: Map<string, IBanner>): string {
    if (isIsolatedPull(item, bannersById)) return `pool:${item.poolId}`;
    const banner = bannersById.get(item.poolId);
    if (banner) {
        const scope = scopeFor(banner.gachaRuleType);
        return scope === "isolated" ? `pool:${item.poolId}` : scope;
    }
    const group = classifyClientGachaGroup(item);
    if (group === "regular") return "standard";
    if (group === "special") return "kernel";
    return `pool:${item.poolId}`;
}

export interface ISixStarPity {
    /** Pulls on the counter since its previous 6*, this pull included. */
    count: number;
    /**
     * False when the counter's earlier pulls may be missing from the history, so
     * `count` is a lower bound. That is the first 6* on a shared counter (it carries
     * from the account's first pull), and the first on a pool that opened before the
     * history begins.
     */
    exact: boolean;
}

/** What came on one counter between two 6* pulls: the previous 6* excluded, this one included. */
export interface IStretchCounts {
    five: number;
    four: number;
    three: number;
}

/** One 6* pull's pity plus what the popover on its card shows. */
export interface ISixStarPullDetail extends ISixStarPity {
    /** The counter it rolled on: Standard and Kernel carry across banners, every other pool is its own. */
    scope: PityScope;
    /** The previous 6* on the same counter, or null when none is in the history. */
    previous: IGachaItem | null;
    /** Rarity counts of the pulls that make up `count`. */
    stretch: IStretchCounts;
}

function scopeOfKey(key: string): PityScope {
    return key === "standard" || key === "kernel" ? key : "isolated";
}

/**
 * Every 6* pull's detail, keyed by the pull object itself.
 *
 * Order inside a ten-pull: its ten rows share one `at`, and the wire lists them
 * newest first, like the rest of the history (the backend sorts by `at` DESC, then
 * `batch_index` ASC, the Yostar row order). Measured on local Postgres (2026-10-15,
 * 5,429 runs between 6* pulls): read newest first, the run lengths follow the game's
 * 50/99 curve (one run past 75 against 0.3 expected); read oldest first they do not
 * (32 runs past 75, and 41 to 50 overfilled by 290). So the chronological order is
 * `at` ascending, and inside one `at` the reverse of the input order.
 */
export function sixStarPullDetails(items: IGachaItem[], bannersById: Map<string, IBanner>): Map<IGachaItem, ISixStarPullDetail> {
    const chronological = items.map((item, index) => ({ item, index })).sort((a, b) => a.item.at - b.item.at || b.index - a.index);
    const historyStart = chronological.length > 0 ? chronological[0].item.at : 0;
    const counters = new Map<string, { count: number; previous: IGachaItem | null; stretch: IStretchCounts }>();
    const result = new Map<IGachaItem, ISixStarPullDetail>();
    for (const { item } of chronological) {
        const key = pityCounterKey(item, bannersById);
        let counter = counters.get(key);
        if (!counter) {
            counter = { count: 0, previous: null, stretch: { five: 0, four: 0, three: 0 } };
            counters.set(key, counter);
        }
        counter.count++;
        if (item.star === "5") counter.stretch.five++;
        else if (item.star === "4") counter.stretch.four++;
        else if (item.star === "3") counter.stretch.three++;
        if (item.star === "6") {
            let exact = counter.previous !== null;
            if (!exact && key.startsWith("pool:")) {
                // A pool's counter starts at its opening, so a history that begins
                // earlier holds every pull on it.
                const openTime = bannersById.get(item.poolId)?.openTime ?? 0;
                exact = openTime > 0 && historyStart < openTime * 1000;
            }
            result.set(item, { count: counter.count, exact, scope: scopeOfKey(key), previous: counter.previous, stretch: counter.stretch });
            counter.count = 0;
            counter.previous = item;
            counter.stretch = { five: 0, four: 0, three: 0 };
        }
    }
    return result;
}
