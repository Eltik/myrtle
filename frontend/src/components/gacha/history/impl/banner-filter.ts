import type { IBanner, IGachaItem, IGachaRecords, IGachaTypeRecords } from "#/lib/api/gacha";

/**
 * The most banners one filter carries, matching the backend's cap on
 * `/gacha/public-records?pools=` (`MAX_POOL_FILTER`), so a shared link is
 * never refused for its length.
 */
export const MAX_BANNER_FILTER = 64;

/** A pool id as the backend accepts it: letters, digits and `_`, at most 64 (`LIMITED_EN_40_0_4`). */
const POOL_ID = /^[A-Za-z0-9_]{1,64}$/;

/**
 * The `banners` search param as a list of distinct pool ids, in the order given. Ids the
 * backend would refuse are dropped rather than failing the page, so a hand-edited link
 * still opens; past {@link MAX_BANNER_FILTER} the rest are dropped too. Empty is "every banner".
 */
export function parseBannerParam(raw: unknown): string[] {
    if (typeof raw !== "string") return [];
    const out: string[] = [];
    for (const part of raw.split(",")) {
        const id = part.trim();
        if (!POOL_ID.test(id) || out.includes(id)) continue;
        out.push(id);
        if (out.length === MAX_BANNER_FILTER) break;
    }
    return out;
}

/** The `banners` search param for `pools`, `undefined` (no param) for every banner. */
export function formatBannerParam(pools: readonly string[]): string | undefined {
    return pools.length > 0 ? pools.join(",") : undefined;
}

/**
 * `records` keeping only the pulls from `pools`, with each group's `total` recounted.
 * An empty `pools` is no filter, and returns `records` itself.
 */
export function filterRecordsByPools(records: IGachaRecords, pools: readonly string[]): IGachaRecords {
    if (pools.length === 0) return records;
    const keep = new Set(pools);
    const group = (g: IGachaTypeRecords): IGachaTypeRecords => {
        const kept = g.records.filter((r) => keep.has(r.poolId));
        return { ...g, records: kept, total: kept.length };
    };
    return { limited: group(records.limited), regular: group(records.regular), special: group(records.special) };
}

/** One choice of the banner filter: a banner the account pulled on. */
export interface IBannerOption {
    poolId: string;
    label: string;
    pulls: number;
    /** The newest pull on it, Unix seconds. */
    lastAt: number;
}

/**
 * Every banner `records` has a pull on, newest last pull first, so the banner just pulled on
 * leads the menu. The label is the game data's banner name, then the name stored with the
 * pull, then the bare pool id.
 */
export function bannerOptions(records: IGachaRecords | null, bannersById: ReadonlyMap<string, IBanner>): IBannerOption[] {
    if (!records) return [];
    const byPool = new Map<string, { item: IGachaItem; pulls: number; lastAt: number }>();
    for (const item of [...records.limited.records, ...records.regular.records, ...records.special.records]) {
        const seen = byPool.get(item.poolId);
        if (seen) {
            seen.pulls += 1;
            seen.lastAt = Math.max(seen.lastAt, item.at);
        } else {
            byPool.set(item.poolId, { item, pulls: 1, lastAt: item.at });
        }
    }
    return [...byPool.entries()].map(([poolId, { item, pulls, lastAt }]) => ({ poolId, label: bannersById.get(poolId)?.gachaPoolName || item.poolName || poolId, pulls, lastAt })).sort((a, b) => b.lastAt - a.lastAt || a.poolId.localeCompare(b.poolId));
}
