import { useCallback, useEffect, useState } from "react";
import type { UserStatsResponse } from "~/types/api/impl/stats";

interface UseUserStatsResult {
    stats: UserStatsResponse | null;
    isLoading: boolean;
    error: string | null;
    refetch: () => void;
}

/**
 * Hook to fetch user stats data client-side.
 * Used to lazy-load the Stats tab instead of passing data from SSR.
 */
export function useUserStats(userId: string): UseUserStatsResult {
    const [stats, setStats] = useState<UserStatsResponse | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchStats = useCallback(async () => {
        if (!userId) {
            setIsLoading(false);
            return;
        }

        setIsLoading(true);
        setError(null);

        try {
            const response = await fetch(`/api/user/${userId}/stats`);

            if (!response.ok) {
                const data = await response.json();
                throw new Error(data.error || "Failed to fetch stats");
            }

            const data = await response.json();
            setStats(data);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Unknown error");
            setStats(null);
        } finally {
            setIsLoading(false);
        }
    }, [userId]);

    useEffect(() => {
        void fetchStats();
    }, [fetchStats]);

    return { stats, isLoading, error, refetch: fetchStats };
}
