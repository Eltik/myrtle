import { useCallback, useState } from "react";
import type { GachaFetchResult, GachaGlobalStats, GachaHistoryParams, GachaRecord, GachaStats } from "~/types/api/impl/gacha";

export interface UseGachaReturn {
    // Data state
    records: GachaRecord[];
    userStats: GachaStats | null;
    globalStats: GachaGlobalStats | null;
    fetchResult: GachaFetchResult | null;

    // Loading states
    loading: boolean;
    loadingHistory: boolean;
    loadingUserStats: boolean;
    loadingGlobalStats: boolean;

    // Error state
    error: string | null;

    // Actions
    triggerFetch: () => Promise<GachaFetchResult | null>;
    fetchHistory: (params?: GachaHistoryParams) => Promise<GachaRecord[]>;
    fetchUserStats: () => Promise<GachaStats | null>;
    fetchGlobalStats: () => Promise<GachaGlobalStats | null>;

    // Utility
    clearError: () => void;
}

/**
 * Hook for managing gacha data fetching and state.
 * v3: Simplified API surface - triggerFetch, fetchHistory, fetchUserStats, fetchGlobalStats.
 */
export function useGacha(): UseGachaReturn {
    const [records, setRecords] = useState<GachaRecord[]>([]);
    const [userStats, setUserStats] = useState<GachaStats | null>(null);
    const [globalStats, setGlobalStats] = useState<GachaGlobalStats | null>(null);
    const [fetchResult, setFetchResult] = useState<GachaFetchResult | null>(null);

    const [loading, setLoading] = useState(false);
    const [loadingHistory, setLoadingHistory] = useState(false);
    const [loadingUserStats, setLoadingUserStats] = useState(false);
    const [loadingGlobalStats, setLoadingGlobalStats] = useState(false);

    const [error, setError] = useState<string | null>(null);

    /**
     * Trigger a backend gacha fetch (pulls new records from game server).
     * POST /api/gacha -> returns GachaFetchResult
     */
    const triggerFetch = useCallback(async (): Promise<GachaFetchResult | null> => {
        setLoading(true);
        setError(null);

        try {
            const response = await fetch("/api/gacha", { method: "POST" });

            if (!response.ok) {
                const data = await response.json().catch(() => ({}));
                setError(data.error || "Failed to fetch gacha records");
                return null;
            }

            const data: GachaFetchResult = await response.json();
            setFetchResult(data);
            return data;
        } catch (err) {
            console.error("Error triggering gacha fetch:", err);
            setError("An error occurred while fetching gacha records");
            return null;
        } finally {
            setLoading(false);
        }
    }, []);

    /**
     * Fetch gacha history with optional filters.
     * GET /api/gacha/history?rarity=&limit=&offset= -> returns GachaRecord[]
     */
    const fetchHistory = useCallback(async (params?: GachaHistoryParams): Promise<GachaRecord[]> => {
        setLoadingHistory(true);

        try {
            const searchParams = new URLSearchParams();
            if (params?.rarity !== undefined) searchParams.set("rarity", String(params.rarity));
            if (params?.limit !== undefined) searchParams.set("limit", String(params.limit));
            if (params?.offset !== undefined) searchParams.set("offset", String(params.offset));

            const queryString = searchParams.toString();
            const url = queryString ? `/api/gacha/history?${queryString}` : "/api/gacha/history";

            const response = await fetch(url);

            if (!response.ok) {
                const data = await response.json().catch(() => ({}));
                setError(data.error || "Failed to fetch gacha history");
                return [];
            }

            const data: GachaRecord[] = await response.json();
            setRecords(data);
            return data;
        } catch (err) {
            console.error("Error fetching gacha history:", err);
            setError("An error occurred while fetching gacha history");
            return [];
        } finally {
            setLoadingHistory(false);
        }
    }, []);

    /**
     * Fetch user-specific gacha statistics.
     * GET /api/gacha/stats?user=true -> returns GachaStats
     */
    const fetchUserStats = useCallback(async (): Promise<GachaStats | null> => {
        setLoadingUserStats(true);

        try {
            const response = await fetch("/api/gacha/stats?user=true");

            if (!response.ok) {
                console.warn("Failed to fetch user gacha stats");
                return null;
            }

            const data: GachaStats = await response.json();
            setUserStats(data);
            return data;
        } catch (err) {
            console.error("Error fetching user gacha stats:", err);
            return null;
        } finally {
            setLoadingUserStats(false);
        }
    }, []);

    /**
     * Fetch global anonymous gacha statistics.
     * GET /api/gacha/stats -> returns GachaGlobalStats
     */
    const fetchGlobalStats = useCallback(async (): Promise<GachaGlobalStats | null> => {
        setLoadingGlobalStats(true);

        try {
            const response = await fetch("/api/gacha/stats");

            if (!response.ok) {
                console.warn("Failed to fetch global gacha stats");
                return null;
            }

            const data: GachaGlobalStats = await response.json();
            setGlobalStats(data);
            return data;
        } catch (err) {
            console.error("Error fetching global gacha stats:", err);
            return null;
        } finally {
            setLoadingGlobalStats(false);
        }
    }, []);

    /**
     * Clear the current error state.
     */
    const clearError = useCallback(() => {
        setError(null);
    }, []);

    return {
        records,
        userStats,
        globalStats,
        fetchResult,

        loading,
        loadingHistory,
        loadingUserStats,
        loadingGlobalStats,

        error,

        triggerFetch,
        fetchHistory,
        fetchUserStats,
        fetchGlobalStats,
        clearError,
    };
}
