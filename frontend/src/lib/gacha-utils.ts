import type { GachaRecord } from "~/types/api/impl/gacha";

/**
 * Pity system constants
 */
export const SOFT_PITY_START = 50;
export const HARD_PITY = 99;

/**
 * Parse the star/rarity string to a number.
 * Handles formats like "6", "TIER_6", etc.
 */
export function parseRarity(star: string | number): number {
    if (typeof star === "number") return star;
    const match = star.match(/\d+/);
    return match ? Number.parseInt(match[0], 10) : 3;
}

/**
 * Format rate as percentage string.
 */
export function formatRate(rate: number): string {
    return `${(rate * 100).toFixed(2)}%`;
}

/**
 * Format pull timestamp for display.
 */
export function formatPullDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
    });
}

/**
 * Calculate average pulls per 6-star.
 */
export function calculateAvgPullsPerSixStar(totalPulls: number, sixStarCount: number): number {
    if (sixStarCount === 0) return 0;
    return totalPulls / sixStarCount;
}

/**
 * Get Tailwind color class for rarity display.
 */
export function getRarityColor(rarity: number): string {
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

/** Enrich v3 GachaRecord entries with operator names from lookup */
export function enrichGachaRecords(records: GachaRecord[], operatorMap: Map<string, OperatorLookupEntry>): GachaRecord[] {
    return records.map((record) => {
        const op = operatorMap.get(record.char_id);
        if (op) {
            return { ...record, rarity: op.rarity };
        }
        return record;
    });
}
