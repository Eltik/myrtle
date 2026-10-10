import { scopeFor } from "#/components/tools/release/impl/pulls/rates";
import type { IBanner, IGachaItem } from "#/lib/api/gacha";

/**
 * Pool-id prefixes of the isolated pools that land in the Standard or Kernel bucket,
 * used ONLY when the banner list cannot answer. It cannot in two cases: while the
 * `/static/banners` query is loading or has failed (the map is empty), and for a pool
 * newer than the static data. The newbie BOOT_ pools are also absent from the list
 * (they live in `NewbeeGachaPoolClient`), and match no prefix here, so they keep
 * counting in the Kernel bucket as before.
 */
const ISOLATED_PREFIXES = ["RETURN_", "ATTAIN_", "CLASSIC_ATTAIN_"];

/**
 * Whether a pull was made on a pool whose pity is its own, so it must neither
 * accumulate nor reset the Standard or Kernel counter.
 *
 * Measured on local Postgres (2026-10-10), these land in the shared buckets today:
 * ATTAIN (7,329 rows) in Standard, because its wire `typeName` is "normal";
 * CLASSIC_ATTAIN (4,841) in Kernel, by its `CLASSIC_` prefix; BACKFLOW (210) in
 * Standard. SPECIAL (Orienteering) is Standard Headhunting and counts. The banner's rule type decides through the
 * planner's own `scopeFor`, plus the pool's `returning` flag.
 */
export function isIsolatedPull(item: IGachaItem, bannersById: Map<string, IBanner>): boolean {
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
 * Limited and Collab pity does NOT carry between banners in-game - each
 * pool has its own independent pity counter that vanishes when the banner
 * closes. So if the user's most recent pull on a Limited/Collab bucket was
 * on a banner that has now ended, treat the bucket's pity as 0.
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
    return { pity: computePity(items), reset: false };
}
