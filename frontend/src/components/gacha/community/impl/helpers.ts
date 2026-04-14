import type { GachaGlobalStats } from "~/types/api/impl/gacha";
import { EXPECTED_RATES, RARITY_COLORS } from "./constants";

// Type for actual rates with all rarities
export interface ActualRates {
    6: number;
    5: number;
    4: number;
    3: number;
}

// Calculate actual rates from global stats
export function calculateActualRates(stats: GachaGlobalStats): ActualRates {
    return {
        6: stats.sixStarRate,
        5: stats.fiveStarRate,
        4: 0, // Not available in v3 global stats
        3: 0, // Not available in v3 global stats
    };
}

// Calculate overall luck score (weighted average of rate differences)
// Positive = lucky, Negative = unlucky
export function calculateLuckScore(actualRates: ActualRates) {
    return (
        ((actualRates[6] - EXPECTED_RATES[6]) / EXPECTED_RATES[6]) * 0.5 + // 6-star weighted heavily
        ((actualRates[5] - EXPECTED_RATES[5]) / EXPECTED_RATES[5]) * 0.3 // 5-star weighted moderately
    );
}

// Build rate comparison data for progress bars
export function buildRateComparisonData(actualRates: ActualRates) {
    return [
        { rarity: 6, label: "6-Star", actual: actualRates[6], expected: EXPECTED_RATES[6], color: RARITY_COLORS[6].hex, bgColor: RARITY_COLORS[6].bgClass },
        { rarity: 5, label: "5-Star", actual: actualRates[5], expected: EXPECTED_RATES[5], color: RARITY_COLORS[5].hex, bgColor: RARITY_COLORS[5].bgClass },
    ] as const;
}

// Group operators by rarity for display (stub - enhanced stats not available in v3)
export function groupOperatorsByRarity(operators: Array<{ rarity: number }>) {
    return {
        6: operators.filter((op) => op.rarity === 6),
        5: operators.filter((op) => op.rarity === 5),
        4: operators.filter((op) => op.rarity === 4),
        3: operators.filter((op) => op.rarity === 3),
    };
}

// Build rarity distribution data (stub - detailed counts not available in v3)
export function buildRarityData(stats: GachaGlobalStats) {
    return [
        { name: "6-Star", value: Math.round(stats.sixStarRate * stats.totalPulls), color: RARITY_COLORS[6].hex },
        { name: "5-Star", value: Math.round(stats.fiveStarRate * stats.totalPulls), color: RARITY_COLORS[5].hex },
    ];
}

// Stub functions for compatibility - these features require enhanced stats not in v3
export function transformHourlyData(_pullTiming: unknown) { return []; }
export function transformDailyData(_pullTiming: unknown) { return []; }
export function transformDateData(_pullTiming: unknown) { return []; }
export function calculateDerivedData(stats: GachaGlobalStats) {
    const actualRates = calculateActualRates(stats);
    const luckScore = calculateLuckScore(actualRates);
    const rateComparisonData = buildRateComparisonData(actualRates);
    const rarityData = buildRarityData(stats);

    return {
        actualRates,
        luckScore,
        rateComparisonData,
        operatorsByRarity: { 6: [], 5: [], 4: [], 3: [] },
        hourlyData: [],
        dailyData: [],
        dateData: [],
        rarityData,
    };
}
