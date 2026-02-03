import { useEffect, useState } from "react";
import type { StatsResponse } from "~/types/api";
import { STAT_CONFIGS } from "./constants";

export interface DisplayStat {
    value: number;
    suffix: string;
    label: string;
    description: string;
    decimals?: number;
}

interface UseStatsResult {
    stats: DisplayStat[];
    isLoading: boolean;
}

function mapResponseToStats(response: StatsResponse): DisplayStat[] {
    return STAT_CONFIGS.map((config) => {
        let rawValue: number;

        switch (config.key) {
            case "operators":
                rawValue = response.gameData.operators;
                break;
            case "users":
                rawValue = response.users.total;
                break;
            case "gachaPulls":
                rawValue = response.gacha.totalPulls;
                break;
            case "tierLists":
                rawValue = response.tierLists.total + response.tierLists.communityCount;
                break;
            default:
                rawValue = config.fallbackValue;
        }

        if (config.formatValue) {
            const formatted = config.formatValue(rawValue);
            return {
                value: formatted.value,
                suffix: formatted.suffix,
                label: config.label,
                description: config.description,
                decimals: formatted.value % 1 !== 0 ? 1 : 0,
            };
        }

        return {
            value: rawValue,
            suffix: config.suffix,
            label: config.label,
            description: config.description,
        };
    });
}

function getFallbackStats(): DisplayStat[] {
    return STAT_CONFIGS.map((config) => {
        if (config.formatValue) {
            const formatted = config.formatValue(config.fallbackValue);
            return {
                value: formatted.value,
                suffix: formatted.suffix,
                label: config.label,
                description: config.description,
                decimals: formatted.value % 1 !== 0 ? 1 : 0,
            };
        }
        return {
            value: config.fallbackValue,
            suffix: config.suffix,
            label: config.label,
            description: config.description,
        };
    });
}

export function useStats(): UseStatsResult {
    const [stats, setStats] = useState<DisplayStat[]>(getFallbackStats());
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        async function fetchStats() {
            try {
                const response = await fetch("/api/stats");
                if (!response.ok) {
                    throw new Error("Failed to fetch stats");
                }
                const data = (await response.json()) as StatsResponse;
                setStats(mapResponseToStats(data));
            } catch (err) {
                console.error("Error fetching stats:", err);
                // Keep fallback stats on error
            } finally {
                setIsLoading(false);
            }
        }

        void fetchStats();
    }, []);

    return { stats, isLoading };
}
