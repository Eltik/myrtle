import { useCallback, useState } from "react";
import type { GachaGlobalStats, GachaRecords, GachaSettings } from "~/types/api";

export interface UseGachaReturn {
    // Data state
    records: GachaRecords | null;
    settings: GachaSettings | null;
    globalStats: GachaGlobalStats | null;

    // Loading states
    loading: boolean;
    loadingSettings: boolean;
    loadingStats: boolean;

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

    // Utility
    clearError: () => void;
}

/**
 * Hook for managing gacha data fetching and state.
 * Provides functions for fetching records, settings, and global stats.
 */
export function useGacha(): UseGachaReturn {
    // Data state
    const [records, setRecords] = useState<GachaRecords | null>(null);
    const [settings, setSettings] = useState<GachaSettings | null>(null);
    const [globalStats, setGlobalStats] = useState<GachaGlobalStats | null>(null);

    // Loading states
    const [loading, setLoading] = useState(false);
    const [loadingSettings, setLoadingSettings] = useState(false);
    const [loadingStats, setLoadingStats] = useState(false);

    // Error state
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
     * Clear the current error state.
     */
    const clearError = useCallback(() => {
        setError(null);
    }, []);

    return {
        // Data
        records,
        settings,
        globalStats,

        // Loading states
        loading,
        loadingSettings,
        loadingStats,

        // Error
        error,

        // Functions
        fetchAllRecords,
        refetch,
        fetchSettings,
        updateSettings,
        fetchGlobalStats,
        clearError,
    };
}
