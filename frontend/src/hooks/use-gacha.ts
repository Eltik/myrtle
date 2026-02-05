import { useCallback, useState } from "react";
import { convertHistoryToRecords } from "~/lib/gacha-utils";
import type { GachaEnhancedStats, GachaEnhancedStatsParams, GachaGlobalStats, GachaHistoryParams, GachaHistoryResponse, GachaRecordEntry, GachaRecords, GachaSettings } from "~/types/api";

export interface UseGachaReturn {
    // Data state
    records: GachaRecords | null;
    settings: GachaSettings | null;
    globalStats: GachaGlobalStats | null;
    history: GachaHistoryResponse | null;
    enhancedStats: GachaEnhancedStats | null;
    /** All history records converted to GachaRecords format for statistics */
    storedRecords: GachaRecords | null;

    // Loading states
    loading: boolean;
    loadingSettings: boolean;
    loadingStats: boolean;
    loadingHistory: boolean;
    loadingEnhancedStats: boolean;
    loadingStoredRecords: boolean;

    // Error state
    error: string | null;

    // Record functions
    fetchAllRecords: () => Promise<GachaRecords | null>;
    refetch: () => Promise<void>;

    // Settings functions
    fetchSettings: () => Promise<GachaSettings | null>;
    updateSettings: (settings: { storeRecords?: boolean; shareAnonymousStats?: boolean }) => Promise<GachaSettings | null>;

    // Stats functions
    fetchGlobalStats: () => Promise<GachaGlobalStats | null>;
    fetchEnhancedStats: (params?: GachaEnhancedStatsParams) => Promise<GachaEnhancedStats | null>;

    // History functions
    fetchHistory: (params?: GachaHistoryParams) => Promise<GachaHistoryResponse | null>;
    fetchOperatorHistory: (charId: string) => Promise<GachaRecordEntry[] | null>;
    /** Fetch all stored records for statistics calculation */
    fetchStoredRecords: () => Promise<GachaRecords | null>;

    // Utility
    clearError: () => void;
}

/**
 * Hook for managing gacha data fetching and state.
 * Provides functions for fetching records, settings, and global stats.
 */
export function useGacha(): UseGachaReturn {
    const [records, setRecords] = useState<GachaRecords | null>(null);
    const [settings, setSettings] = useState<GachaSettings | null>(null);
    const [globalStats, setGlobalStats] = useState<GachaGlobalStats | null>(null);
    const [storedRecords, setStoredRecords] = useState<GachaRecords | null>(null);
    const [history, setHistory] = useState<GachaHistoryResponse | null>(null);
    const [enhancedStats, setEnhancedStats] = useState<GachaEnhancedStats | null>(null);

    const [loading, setLoading] = useState(false);
    const [loadingSettings, setLoadingSettings] = useState(false);
    const [loadingStats, setLoadingStats] = useState(false);
    const [loadingHistory, setLoadingHistory] = useState(false);
    const [loadingEnhancedStats, setLoadingEnhancedStats] = useState(false);
    const [loadingStoredRecords, setLoadingStoredRecords] = useState(false);

    const [error, setError] = useState<string | null>(null);

    /**
     * Fetch all gacha records (limited, regular, special).
     * Requires authentication with Yostar credentials.
     */
    const fetchAllRecords = useCallback(async (): Promise<GachaRecords | null> => {
        setLoading(true);
        setError(null);

        try {
            const response = await fetch("/api/gacha");
            const data = await response.json();

            if (data.success) {
                setRecords(data.data);
                return data.data;
            }

            setError(data.error || "Failed to fetch gacha records");
            return null;
        } catch (err) {
            console.error("Error fetching gacha records:", err);
            setError("An error occurred while fetching gacha records");
            return null;
        } finally {
            setLoading(false);
        }
    }, []);

    /**
     * Re-fetch all records. Alias for fetchAllRecords with void return.
     */
    const refetch = useCallback(async (): Promise<void> => {
        await fetchAllRecords();
    }, [fetchAllRecords]);

    /**
     * Fetch user's gacha settings.
     * Requires authentication.
     */
    const fetchSettings = useCallback(async (): Promise<GachaSettings | null> => {
        setLoadingSettings(true);

        try {
            const response = await fetch("/api/gacha/settings");
            const data = await response.json();

            if (data.success) {
                setSettings(data.settings);
                return data.settings;
            }

            // Don't set error for settings - it's optional data
            console.warn("Failed to fetch gacha settings:", data.error);
            return null;
        } catch (err) {
            console.error("Error fetching gacha settings:", err);
            return null;
        } finally {
            setLoadingSettings(false);
        }
    }, []);

    /**
     * Update user's gacha settings.
     * Requires authentication.
     */
    const updateSettings = useCallback(async (newSettings: { storeRecords?: boolean; shareAnonymousStats?: boolean }): Promise<GachaSettings | null> => {
        setLoadingSettings(true);

        try {
            const response = await fetch("/api/gacha/settings", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(newSettings),
            });
            const data = await response.json();

            if (data.success) {
                setSettings(data.settings);
                return data.settings;
            }

            setError(data.error || "Failed to update gacha settings");
            return null;
        } catch (err) {
            console.error("Error updating gacha settings:", err);
            setError("An error occurred while updating gacha settings");
            return null;
        } finally {
            setLoadingSettings(false);
        }
    }, []);

    /**
     * Fetch global anonymous pull rate statistics.
     * This is a public endpoint - no authentication required.
     */
    const fetchGlobalStats = useCallback(async (): Promise<GachaGlobalStats | null> => {
        setLoadingStats(true);

        try {
            const response = await fetch("/api/gacha/stats");
            const data = await response.json();

            if (data.success) {
                setGlobalStats(data.data);
                return data.data;
            }

            // Don't set error for stats - it's optional data
            console.warn("Failed to fetch global gacha stats:", data.error);
            return null;
        } catch (err) {
            console.error("Error fetching global gacha stats:", err);
            return null;
        } finally {
            setLoadingStats(false);
        }
    }, []);

    /**
     * Fetch user's pull history with pagination and filters.
     * Requires authentication.
     */
    const fetchHistory = useCallback(async (params?: GachaHistoryParams): Promise<GachaHistoryResponse | null> => {
        setLoadingHistory(true);

        try {
            const searchParams = new URLSearchParams();
            if (params?.limit !== undefined) searchParams.set("limit", String(params.limit));
            if (params?.offset !== undefined) searchParams.set("offset", String(params.offset));
            if (params?.rarity !== undefined) searchParams.set("rarity", String(params.rarity));
            if (params?.gachaType !== undefined) searchParams.set("gachaType", params.gachaType);
            if (params?.charId !== undefined) searchParams.set("charId", params.charId);
            if (params?.from !== undefined) searchParams.set("from", String(params.from));
            if (params?.to !== undefined) searchParams.set("to", String(params.to));
            if (params?.order !== undefined) searchParams.set("order", params.order);

            const queryString = searchParams.toString();
            const url = queryString ? `/api/gacha/history?${queryString}` : "/api/gacha/history";

            const response = await fetch(url);
            const data = await response.json();

            if (data.success) {
                setHistory(data.data);
                return data.data;
            }

            setError(data.error || "Failed to fetch gacha history");
            return null;
        } catch (err) {
            console.error("Error fetching gacha history:", err);
            setError("An error occurred while fetching gacha history");
            return null;
        } finally {
            setLoadingHistory(false);
        }
    }, []);

    /**
     * Fetch all pulls of a specific operator for the user.
     * Requires authentication.
     */
    const fetchOperatorHistory = useCallback(async (charId: string): Promise<GachaRecordEntry[] | null> => {
        setLoadingHistory(true);

        try {
            const response = await fetch(`/api/gacha/history/${encodeURIComponent(charId)}`);
            const data = await response.json();

            if (data.success) {
                return data.data;
            }

            setError(data.error || "Failed to fetch operator history");
            return null;
        } catch (err) {
            console.error("Error fetching operator history:", err);
            setError("An error occurred while fetching operator history");
            return null;
        } finally {
            setLoadingHistory(false);
        }
    }, []);

    /**
     * Fetch comprehensive global statistics.
     * This is a public endpoint - no authentication required.
     */
    const fetchEnhancedStats = useCallback(async (params?: GachaEnhancedStatsParams): Promise<GachaEnhancedStats | null> => {
        setLoadingEnhancedStats(true);

        try {
            const searchParams = new URLSearchParams();
            if (params?.topN !== undefined) searchParams.set("topN", String(params.topN));
            if (params?.includeTiming !== undefined) searchParams.set("includeTiming", String(params.includeTiming));

            const queryString = searchParams.toString();
            const url = queryString ? `/api/gacha/stats/enhanced?${queryString}` : "/api/gacha/stats/enhanced";

            const response = await fetch(url);
            const data = await response.json();

            if (data.success) {
                setEnhancedStats(data.data);
                return data.data;
            }

            // Don't set error for stats - it's optional data
            console.warn("Failed to fetch enhanced gacha stats:", data.error);
            return null;
        } catch (err) {
            console.error("Error fetching enhanced gacha stats:", err);
            return null;
        } finally {
            setLoadingEnhancedStats(false);
        }
    }, []);

    /**
     * Fetch all stored records from database for statistics calculation.
     * Fetches up to 5000 records to compute accurate statistics.
     */
    const fetchStoredRecords = useCallback(async (): Promise<GachaRecords | null> => {
        setLoadingStoredRecords(true);

        try {
            const response = await fetch("/api/gacha/history?limit=5000&offset=0&order=desc");
            const data = await response.json();

            if (data.success && data.data?.records) {
                const converted = convertHistoryToRecords(data.data.records);
                setStoredRecords(converted);
                return converted;
            }

            console.warn("Failed to fetch stored records for stats:", data.error);
            return null;
        } catch (err) {
            console.error("Error fetching stored records:", err);
            return null;
        } finally {
            setLoadingStoredRecords(false);
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
        settings,
        globalStats,
        history,
        enhancedStats,
        storedRecords,

        loading,
        loadingSettings,
        loadingStats,
        loadingHistory,
        loadingEnhancedStats,
        loadingStoredRecords,

        error,

        fetchAllRecords,
        refetch,
        fetchSettings,
        updateSettings,
        fetchGlobalStats,
        fetchEnhancedStats,
        fetchHistory,
        fetchStoredRecords,
        fetchOperatorHistory,
        clearError,
    };
}
