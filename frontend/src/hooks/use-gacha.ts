import { useCallback, useState } from "react";
import type { GachaGlobalStats, GachaHistoryParams, GachaHistoryResponse, GachaRecordEntry, GachaRecords, GachaSettings } from "~/types/api";

export interface UseGachaReturn {
    // Data state
    records: GachaRecords | null;
    settings: GachaSettings | null;
    globalStats: GachaGlobalStats | null;
    history: GachaHistoryResponse | null;
    /** Grouped stored records (limited/regular/special) for statistics */
    storedRecords: GachaRecords | null;

    // Loading states
    loading: boolean;
    loadingRecords: boolean;
    loadingSettings: boolean;
    loadingStats: boolean;
    loadingHistory: boolean;
    loadingStoredRecords: boolean;

    // Error state
    error: string | null;

    // Record actions
    fetchAllRecords: () => Promise<GachaRecords | null>;
    refetch: () => Promise<void>;

    // Settings actions
    fetchSettings: () => Promise<GachaSettings | null>;
    updateSettings: (settings: { storeRecords?: boolean; shareAnonymousStats?: boolean }) => Promise<GachaSettings | null>;

    // Stats actions
    fetchGlobalStats: () => Promise<GachaGlobalStats | null>;

    // History actions
    fetchHistory: (params?: GachaHistoryParams) => Promise<GachaHistoryResponse | null>;
    fetchOperatorHistory: (charId: string) => Promise<GachaRecordEntry[] | null>;
    fetchStoredRecords: () => Promise<GachaRecords | null>;

    clearError: () => void;
}

/**
 * Hook for managing gacha data fetching and state.
 * v3: backed by /api/gacha, /api/gacha/stored-records, /api/gacha/history, /api/gacha/settings.
 */
export function useGacha(): UseGachaReturn {
    const [records, setRecords] = useState<GachaRecords | null>(null);
    const [settings, setSettings] = useState<GachaSettings | null>(null);
    const [globalStats, setGlobalStats] = useState<GachaGlobalStats | null>(null);
    const [storedRecords, setStoredRecords] = useState<GachaRecords | null>(null);
    const [history, setHistory] = useState<GachaHistoryResponse | null>(null);

    const [loading, setLoading] = useState(false);
    const [loadingSettings, setLoadingSettings] = useState(false);
    const [loadingStats, setLoadingStats] = useState(false);
    const [loadingHistory, setLoadingHistory] = useState(false);
    const [loadingStoredRecords, setLoadingStoredRecords] = useState(false);

    const [error, setError] = useState<string | null>(null);

    /** Fetch all gacha records via a live Yostar sync. */
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

    const refetch = useCallback(async (): Promise<void> => {
        await fetchAllRecords();
    }, [fetchAllRecords]);

    const fetchSettings = useCallback(async (): Promise<GachaSettings | null> => {
        setLoadingSettings(true);

        try {
            const response = await fetch("/api/gacha/settings");
            const data = await response.json();

            if (data.success) {
                setSettings(data.settings);
                return data.settings;
            }

            console.warn("Failed to fetch gacha settings:", data.error);
            return null;
        } catch (err) {
            console.error("Error fetching gacha settings:", err);
            return null;
        } finally {
            setLoadingSettings(false);
        }
    }, []);

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

    const fetchGlobalStats = useCallback(async (): Promise<GachaGlobalStats | null> => {
        setLoadingStats(true);

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
            setLoadingStats(false);
        }
    }, []);

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

    const fetchStoredRecords = useCallback(async (): Promise<GachaRecords | null> => {
        setLoadingStoredRecords(true);

        try {
            const response = await fetch("/api/gacha/stored-records");
            const data = await response.json();

            if (data.success && data.data) {
                setStoredRecords(data.data);
                return data.data;
            }

            console.warn("Failed to fetch stored records:", data.error);
            return null;
        } catch (err) {
            console.error("Error fetching stored records:", err);
            return null;
        } finally {
            setLoadingStoredRecords(false);
        }
    }, []);

    const clearError = useCallback(() => {
        setError(null);
    }, []);

    return {
        records,
        settings,
        globalStats,
        history,
        storedRecords,

        loading,
        loadingRecords: loading,
        loadingSettings,
        loadingStats,
        loadingHistory,
        loadingStoredRecords,

        error,

        fetchAllRecords,
        refetch,
        fetchSettings,
        updateSettings,
        fetchGlobalStats,
        fetchHistory,
        fetchStoredRecords,
        fetchOperatorHistory,
        clearError,
    };
}
