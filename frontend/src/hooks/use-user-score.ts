import { useCallback, useEffect, useState } from "react";
import type { StoredUserScore } from "~/types/api/impl/user";

interface UseUserScoreResult {
    score: StoredUserScore | null;
    isLoading: boolean;
    error: string | null;
    refetch: () => void;
}

/**
 * Hook to fetch user score data client-side.
 * Used to lazy-load the Score tab instead of passing data from SSR.
 */
export function useUserScore(userId: string): UseUserScoreResult {
    const [score, setScore] = useState<StoredUserScore | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchScore = useCallback(async () => {
        if (!userId) {
            setIsLoading(false);
            return;
        }

        setIsLoading(true);
        setError(null);

        try {
            const response = await fetch(`/api/user/${userId}/score`);

            if (!response.ok) {
                const data = await response.json();
                throw new Error(data.error || "Failed to fetch score");
            }

            const data = await response.json();
            setScore(data.score);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Unknown error");
            setScore(null);
        } finally {
            setIsLoading(false);
        }
    }, [userId]);

    useEffect(() => {
        void fetchScore();
    }, [fetchScore]);

    return { score, isLoading, error, refetch: fetchScore };
}
