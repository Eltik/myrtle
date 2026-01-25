import type { CollectiveStats, GachaEnhancedStats, OperatorPopularity, PullTimingData } from "~/types/api";
import { DAY_NAMES, EXPECTED_RATES, RARITY_COLORS } from "./constants";

// Type for actual rates with all rarities
export interface ActualRates {
    6: number;
    5: number;
    4: number;
    3: number;
}

// Calculate actual rates from collective stats
export function calculateActualRates(collectiveStats: CollectiveStats): ActualRates {
    const totalPulls = collectiveStats.totalPulls;
    return {
        6: totalPulls > 0 ? collectiveStats.totalSixStars / totalPulls : 0,
        5: totalPulls > 0 ? collectiveStats.totalFiveStars / totalPulls : 0,
        4: totalPulls > 0 ? collectiveStats.totalFourStars / totalPulls : 0,
        3: totalPulls > 0 ? collectiveStats.totalThreeStars / totalPulls : 0,
    };
}

// Calculate overall luck score (weighted average of rate differences)
// Positive = lucky, Negative = unlucky
export function calculateLuckScore(actualRates: ActualRates) {
    return (
        ((actualRates[6] - EXPECTED_RATES[6]) / EXPECTED_RATES[6]) * 0.5 + // 6★ weighted heavily
        ((actualRates[5] - EXPECTED_RATES[5]) / EXPECTED_RATES[5]) * 0.3 + // 5★ weighted moderately
        ((actualRates[4] - EXPECTED_RATES[4]) / EXPECTED_RATES[4]) * 0.1 + // 4★ weighted lightly
        ((actualRates[3] - EXPECTED_RATES[3]) / EXPECTED_RATES[3]) * 0.1 // 3★ weighted lightly
    );
}

// Build rate comparison data for progress bars
export function buildRateComparisonData(actualRates: ActualRates) {
    return [
        { rarity: 6, label: "6★", actual: actualRates[6], expected: EXPECTED_RATES[6], color: RARITY_COLORS[6].hex, bgColor: RARITY_COLORS[6].bgClass },
        { rarity: 5, label: "5★", actual: actualRates[5], expected: EXPECTED_RATES[5], color: RARITY_COLORS[5].hex, bgColor: RARITY_COLORS[5].bgClass },
        { rarity: 4, label: "4★", actual: actualRates[4], expected: EXPECTED_RATES[4], color: RARITY_COLORS[4].hex, bgColor: RARITY_COLORS[4].bgClass },
        { rarity: 3, label: "3★", actual: actualRates[3], expected: EXPECTED_RATES[3], color: RARITY_COLORS[3].hex, bgColor: RARITY_COLORS[3].bgClass },
    ] as const;
}

// Group operators by rarity for display
export function groupOperatorsByRarity(operators: OperatorPopularity[]) {
    return {
        6: operators.filter((op) => op.rarity === 6),
        5: operators.filter((op) => op.rarity === 5),
        4: operators.filter((op) => op.rarity === 4),
        3: operators.filter((op) => op.rarity === 3),
    };
}

// Transform hourly timing data for charts
export function transformHourlyData(pullTiming: PullTimingData | undefined) {
    return (
        pullTiming?.byHour.map((item) => ({
            hour: `${item.hour}:00`,
            pulls: item.pullCount,
            percentage: item.percentage,
        })) ?? []
    );
}

// Transform daily timing data for charts
export function transformDailyData(pullTiming: PullTimingData | undefined) {
    return (
        pullTiming?.byDayOfWeek.map((item) => ({
            day: DAY_NAMES[item.day] ?? item.dayName,
            pulls: item.pullCount,
            percentage: item.percentage,
        })) ?? []
    );
}

// Transform date timing data for line chart
export function transformDateData(pullTiming: PullTimingData | undefined) {
    return (
        pullTiming?.byDate?.map((item) => ({
            date: new Date(item.date).toLocaleDateString(undefined, { month: "short", day: "numeric" }),
            fullDate: item.date,
            pulls: item.pullCount,
        })) ?? []
    );
}

// Build rarity distribution data for pie chart
export function buildRarityData(collectiveStats: CollectiveStats) {
    return [
        { name: "6-Star", value: collectiveStats.totalSixStars, color: RARITY_COLORS[6].hex },
        { name: "5-Star", value: collectiveStats.totalFiveStars, color: RARITY_COLORS[5].hex },
        { name: "4-Star", value: collectiveStats.totalFourStars, color: RARITY_COLORS[4].hex },
        { name: "3-Star", value: collectiveStats.totalThreeStars, color: RARITY_COLORS[3].hex },
    ];
}

// Calculate all derived data from stats
export function calculateDerivedData(stats: GachaEnhancedStats) {
    const actualRates = calculateActualRates(stats.collectiveStats);
    const luckScore = calculateLuckScore(actualRates);
    const rateComparisonData = buildRateComparisonData(actualRates);
    const operatorsByRarity = groupOperatorsByRarity(stats.mostCommonOperators);
    const hourlyData = transformHourlyData(stats.pullTiming);
    const dailyData = transformDailyData(stats.pullTiming);
    const dateData = transformDateData(stats.pullTiming);
    const rarityData = buildRarityData(stats.collectiveStats);

    return {
        actualRates,
        luckScore,
        rateComparisonData,
        operatorsByRarity,
        hourlyData,
        dailyData,
        dateData,
        rarityData,
    };
}
