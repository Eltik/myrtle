import type { GachaItem, GachaRecordEntry, GachaRecords } from "~/types/api";

/**
 * Pity system constants
 */
export const SOFT_PITY_START = 50;
export const HARD_PITY = 99;

/**
 * Statistics calculated from gacha records
 */
export interface GachaStats {
    totalPulls: number;
    sixStarCount: number;
    fiveStarCount: number;
    fourStarCount: number;
    threeStarCount: number;
    sixStarRate: number;
    fiveStarRate: number;
    pityCount: number;
}

/**
 * Parse the star/rarity string to a number.
 * Handles formats like "6", "TIER_6", etc.
 */
export function parseRarity(star: string): number {
    const match = star.match(/\d+/);
    return match ? Number.parseInt(match[0], 10) : 3;
}

/**
 * Calculate pity counter (pulls since last 6-star).
 * Records should be sorted newest first.
 */
export function calculatePity(records: GachaItem[]): number {
    let pity = 0;
    for (const record of records) {
        const rarity = parseRarity(record.star);
        if (rarity === 6) {
            break;
        }
        pity++;
    }
    return pity;
}

/**
 * Calculate comprehensive statistics from gacha records.
 */
export function calculateStats(records: GachaItem[]): GachaStats {
    const totalPulls = records.length;
    let sixStarCount = 0;
    let fiveStarCount = 0;
    let fourStarCount = 0;
    let threeStarCount = 0;

    for (const record of records) {
        const rarity = parseRarity(record.star);
        switch (rarity) {
            case 6:
                sixStarCount++;
                break;
            case 5:
                fiveStarCount++;
                break;
            case 4:
                fourStarCount++;
                break;
            default:
                threeStarCount++;
                break;
        }
    }

    return {
        totalPulls,
        sixStarCount,
        fiveStarCount,
        fourStarCount,
        threeStarCount,
        sixStarRate: totalPulls > 0 ? sixStarCount / totalPulls : 0,
        fiveStarRate: totalPulls > 0 ? fiveStarCount / totalPulls : 0,
        pityCount: calculatePity(records),
    };
}

/**
 * Get total pulls across all gacha types.
 */
export function getTotalPulls(gachaRecords: GachaRecords): number {
    return gachaRecords.limited.total + gachaRecords.regular.total + gachaRecords.special.total;
}

/**
 * Get all records from all gacha types combined.
 */
export function getAllRecords(gachaRecords: GachaRecords): GachaItem[] {
    return [...gachaRecords.limited.records, ...gachaRecords.regular.records, ...gachaRecords.special.records];
}

/**
 * Filter records by rarity.
 */
export function filterByRarity(records: GachaItem[], rarity: number): GachaItem[] {
    return records.filter((record) => parseRarity(record.star) === rarity);
}

/**
 * Get the most recent N pulls.
 * Assumes records are already sorted newest first.
 */
export function getRecentPulls(records: GachaItem[], count: number): GachaItem[] {
    return records.slice(0, count);
}

/**
 * Group records by pool ID.
 */
export function groupByPool(records: GachaItem[]): Map<string, GachaItem[]> {
    const grouped = new Map<string, GachaItem[]>();
    for (const record of records) {
        const existing = grouped.get(record.poolId);
        if (existing) {
            existing.push(record);
        } else {
            grouped.set(record.poolId, [record]);
        }
    }
    return grouped;
}

/**
 * Sort records by timestamp.
 */
export function sortByTime(records: GachaItem[], order: "asc" | "desc"): GachaItem[] {
    return [...records].sort((a, b) => {
        return order === "asc" ? a.at - b.at : b.at - a.at;
    });
}

/**
 * Get Tailwind color class for rarity display.
 */
export function getRarityColor(star: string): string {
    const rarity = parseRarity(star);
    switch (rarity) {
        case 6:
            return "text-orange-500";
        case 5:
            return "text-yellow-500";
        case 4:
            return "text-purple-500";
        default:
            return "text-blue-500";
    }
}

/**
 * Get background color class for rarity.
 */
export function getRarityBgColor(star: string): string {
    const rarity = parseRarity(star);
    switch (rarity) {
        case 6:
            return "bg-orange-500/10";
        case 5:
            return "bg-yellow-500/10";
        case 4:
            return "bg-purple-500/10";
        default:
            return "bg-blue-500/10";
    }
}

/**
 * Format pull timestamp for display.
 */
export function formatPullDate(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
    });
}

/**
 * Format rate as percentage string.
 */
export function formatRate(rate: number): string {
    return `${(rate * 100).toFixed(2)}%`;
}

/**
 * Calculate average pulls per 6-star.
 */
export function calculateAvgPullsPerSixStar(totalPulls: number, sixStarCount: number): number {
    if (sixStarCount === 0) return 0;
    return totalPulls / sixStarCount;
}

/**
 * Count 6-stars obtained in soft pity range (pulls 50-99).
 * This analyzes the pull history to find the pity count when each 6-star was obtained.
 */
export function countSixStarsInSoftPity(records: GachaItem[]): number {
    let softPityCount = 0;
    let pullsSinceLastSixStar = 0;
    const sortedRecords = [...records].sort((a, b) => a.at - b.at); // oldest first

    for (const record of sortedRecords) {
        pullsSinceLastSixStar++;
        if (parseRarity(record.star) === 6) {
            if (pullsSinceLastSixStar >= SOFT_PITY_START) {
                softPityCount++;
            }
            pullsSinceLastSixStar = 0;
        }
    }
    return softPityCount;
}

/**
 * Operator count entry for most common operators
 */
export interface OperatorCount {
    charId: string;
    charName: string;
    count: number;
    rarity: number;
}

/**
 * Get most common operators by rarity.
 * Returns top N operators for each rarity level.
 */
export function getMostCommonOperatorsByRarity(records: GachaItem[], topN = 3): Record<number, OperatorCount[]> {
    const countsByRarity: Record<number, Map<string, { charName: string; count: number }>> = {
        6: new Map(),
        5: new Map(),
        4: new Map(),
        3: new Map(),
    };

    for (const record of records) {
        const rarity = parseRarity(record.star);
        if (rarity >= 3 && rarity <= 6) {
            const map = countsByRarity[rarity];
            if (!map) continue;
            const existing = map.get(record.charId);
            if (existing) {
                existing.count++;
            } else {
                map.set(record.charId, { charName: record.charName, count: 1 });
            }
        }
    }

    const result: Record<number, OperatorCount[]> = {};
    for (const rarity of [6, 5, 4, 3]) {
        const map = countsByRarity[rarity];
        if (!map) continue;
        const sorted = Array.from(map.entries())
            .map(([charId, { charName, count }]) => ({ charId, charName, count, rarity }))
            .sort((a, b) => b.count - a.count)
            .slice(0, topN);
        result[rarity] = sorted;
    }

    return result;
}

/** Pity info for a single 6-star pull */
export interface SixStarPityEntry {
    charId: string;
    charName: string;
    pityCount: number;
    timestamp: number;
    inSoftPity: boolean;
}

/** Summary statistics for a single banner (pool) */
export interface BannerBreakdownEntry {
    poolId: string;
    poolName: string;
    gachaType: string;
    totalPulls: number;
    firstPullTimestamp: number;
    lastPullTimestamp: number;
    sixStarCount: number;
    fiveStarCount: number;
    fourStarCount: number;
    threeStarCount: number;
    notablePulls: Array<{ charId: string; charName: string; count: number }>;
    /** All pulls on this banner sorted oldest first */
    pulls: GachaItem[];
    /** Pity count for each 6-star obtained on this banner */
    pityHistory: SixStarPityEntry[];
    /** Current pity (pulls since last 6-star on this banner) */
    currentPity: number;
}

export type BannerSortMode = "recent" | "pullCount" | "sixStarRate";

/**
 * Build per-banner breakdown from gacha records.
 * Groups by poolId, accumulates stats including full pull history and pity tracking.
 * Returns sorted by most recent pull.
 */
export function buildBannerBreakdown(records: GachaItem[]): BannerBreakdownEntry[] {
    const poolMap = new Map<
        string,
        {
            poolName: string;
            gachaType: string;
            pulls: GachaItem[];
            sixStars: Map<string, { charName: string; count: number }>;
        }
    >();

    for (const record of records) {
        let entry = poolMap.get(record.poolId);
        if (!entry) {
            entry = {
                poolName: record.poolName,
                gachaType: record.typeName,
                pulls: [],
                sixStars: new Map(),
            };
            poolMap.set(record.poolId, entry);
        }

        entry.pulls.push(record);

        if (parseRarity(record.star) === 6) {
            const existing = entry.sixStars.get(record.charId);
            if (existing) existing.count++;
            else entry.sixStars.set(record.charId, { charName: record.charName, count: 1 });
        }
    }

    return Array.from(poolMap.entries())
        .map(([poolId, data]) => {
            // Sort pulls oldest first for pity calculation
            const sortedPulls = [...data.pulls].sort((a, b) => a.at - b.at);

            let sixStarCount = 0;
            let fiveStarCount = 0;
            let fourStarCount = 0;
            let threeStarCount = 0;
            let pullsSinceLast6 = 0;
            const pityHistory: SixStarPityEntry[] = [];

            for (const pull of sortedPulls) {
                pullsSinceLast6++;
                const rarity = parseRarity(pull.star);
                switch (rarity) {
                    case 6: {
                        sixStarCount++;
                        pityHistory.push({
                            charId: pull.charId,
                            charName: pull.charName,
                            pityCount: pullsSinceLast6,
                            timestamp: pull.at,
                            inSoftPity: pullsSinceLast6 >= SOFT_PITY_START,
                        });
                        pullsSinceLast6 = 0;
                        break;
                    }
                    case 5:
                        fiveStarCount++;
                        break;
                    case 4:
                        fourStarCount++;
                        break;
                    default:
                        threeStarCount++;
                        break;
                }
            }

            const firstPull = sortedPulls[0]?.at ?? 0;
            const lastPull = sortedPulls[sortedPulls.length - 1]?.at ?? 0;

            return {
                poolId,
                poolName: data.poolName,
                gachaType: data.gachaType,
                totalPulls: sortedPulls.length,
                firstPullTimestamp: firstPull,
                lastPullTimestamp: lastPull,
                sixStarCount,
                fiveStarCount,
                fourStarCount,
                threeStarCount,
                notablePulls: Array.from(data.sixStars.entries())
                    .map(([charId, { charName, count }]) => ({ charId, charName, count }))
                    .sort((a, b) => b.count - a.count),
                pulls: sortedPulls,
                pityHistory,
                currentPity: pullsSinceLast6,
            };
        })
        .sort((a, b) => b.lastPullTimestamp - a.lastPullTimestamp);
}

/**
 * Sort banner breakdown entries by the given mode.
 */
export function sortBannerBreakdown(entries: BannerBreakdownEntry[], mode: BannerSortMode): BannerBreakdownEntry[] {
    return [...entries].sort((a, b) => {
        switch (mode) {
            case "recent":
                return b.lastPullTimestamp - a.lastPullTimestamp;
            case "pullCount":
                return b.totalPulls - a.totalPulls;
            case "sixStarRate": {
                const rateA = a.totalPulls > 0 ? a.sixStarCount / a.totalPulls : 0;
                const rateB = b.totalPulls > 0 ? b.sixStarCount / b.totalPulls : 0;
                return rateB - rateA;
            }
            default:
                return 0;
        }
    });
}

/**
 * Check if a pool is a collab/linkage banner
 */
export function isCollabBanner(poolId: string): boolean {
    return poolId.startsWith("LINKAGE_");
}

/**
 * Map of operator rarity tier strings to numeric values.
 * Game data uses "TIER_6" for 6-star, etc.
 */
export const RARITY_TIER_MAP: Record<string, number> = {
    TIER_6: 6,
    TIER_5: 5,
    TIER_4: 4,
    TIER_3: 3,
    TIER_2: 2,
    TIER_1: 1,
};

/** Operator lookup entry for enriching gacha records */
export interface OperatorLookupEntry {
    name: string;
    rarity: number;
    profession: string;
}

/** Build an operator lookup map from static operator data */
export function buildOperatorLookup(operators: Array<{ id: string | null; name: string; rarity: string; profession: string }>): Map<string, OperatorLookupEntry> {
    const map = new Map<string, OperatorLookupEntry>();
    for (const op of operators) {
        if (!op.id) continue;
        map.set(op.id, {
            name: op.name,
            rarity: RARITY_TIER_MAP[op.rarity] ?? parseRarity(op.rarity),
            profession: op.profession,
        });
    }
    return map;
}

/** Enrich gacha record entries with correct rarity and name from operator data */
export function enrichRecordEntries(records: GachaRecordEntry[], operatorMap: Map<string, OperatorLookupEntry>): GachaRecordEntry[] {
    return records.map((record) => {
        const op = operatorMap.get(record.charId);
        if (op) {
            return { ...record, rarity: op.rarity, charName: op.name };
        }
        return record;
    });
}

/** Enrich GachaRecords with correct rarity and name from operator data */
export function enrichGachaRecords(records: GachaRecords, operatorMap: Map<string, OperatorLookupEntry>): GachaRecords {
    const enrichItems = (items: GachaItem[]) =>
        items.map((item) => {
            const op = operatorMap.get(item.charId);
            if (op) {
                return { ...item, star: String(op.rarity), charName: op.name };
            }
            return item;
        });

    return {
        limited: { ...records.limited, records: enrichItems(records.limited.records) },
        regular: { ...records.regular, records: enrichItems(records.regular.records) },
        special: { ...records.special, records: enrichItems(records.special.records) },
    };
}

/**
 * Convert GachaRecordEntry array to GachaItem array.
 * Used when computing statistics from database records.
 */
export function convertRecordEntryToGachaItem(entry: { charId: string; charName: string; rarity: number; poolId: string; poolName: string; gachaType: string; pullTimestamp: number; pullTimestampStr: string | null }): GachaItem {
    return {
        charId: entry.charId,
        charName: entry.charName,
        star: String(entry.rarity), // Convert numeric rarity to string for parseRarity()
        color: "", // Not available from database, but not used in stats
        poolId: entry.poolId,
        poolName: entry.poolName,
        typeName: entry.gachaType, // Use gacha type as type name
        at: entry.pullTimestamp,
        atStr: entry.pullTimestampStr ?? "",
    };
}

/**
 * Convert an array of GachaRecordEntry to GachaRecords structure.
 * Groups records by gacha_type and creates the expected format for StatsOverview.
 */
export function convertHistoryToRecords(
    entries: Array<{
        charId: string;
        charName: string;
        rarity: number;
        poolId: string;
        poolName: string;
        gachaType: string;
        pullTimestamp: number;
        pullTimestampStr: string | null;
    }>,
): GachaRecords {
    const limited: GachaItem[] = [];
    const regular: GachaItem[] = [];
    const special: GachaItem[] = [];

    for (const entry of entries) {
        const item = convertRecordEntryToGachaItem(entry);
        switch (entry.gachaType) {
            case "limited":
                limited.push(item);
                break;
            case "regular":
                regular.push(item);
                break;
            case "special":
                special.push(item);
                break;
        }
    }

    return {
        limited: {
            gacha_type: "limited",
            records: limited,
            total: limited.length,
        },
        regular: {
            gacha_type: "regular",
            records: regular,
            total: regular.length,
        },
        special: {
            gacha_type: "special",
            records: special,
            total: special.length,
        },
    };
}
