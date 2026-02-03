export interface StatConfig {
    key: string;
    label: string;
    description: string;
    suffix: string;
    fallbackValue: number;
    formatValue?: (value: number) => { value: number; suffix: string };
}

/**
 * Format large numbers with K/M suffix
 */
export function formatLargeNumber(value: number): { value: number; suffix: string } {
    if (value >= 1_000_000) {
        return { value: Math.floor(value / 100_000) / 10, suffix: "M+" };
    }
    if (value >= 1_000) {
        return { value: Math.floor(value / 100) / 10, suffix: "K+" };
    }
    return { value, suffix: "+" };
}

export const STAT_CONFIGS: StatConfig[] = [
    {
        key: "operators",
        label: "Operators",
        description: "Complete database",
        suffix: "+",
        fallbackValue: 300,
    },
    {
        key: "users",
        label: "Registered Users",
        description: "Global community",
        suffix: "+",
        fallbackValue: 50000,
        formatValue: formatLargeNumber,
    },
    {
        key: "gachaPulls",
        label: "Gacha Pulls",
        description: "Community tracked",
        suffix: "+",
        fallbackValue: 1000000,
        formatValue: formatLargeNumber,
    },
    {
        key: "tierLists",
        label: "Tier Lists",
        description: "Community created",
        suffix: "+",
        fallbackValue: 10,
    },
];
