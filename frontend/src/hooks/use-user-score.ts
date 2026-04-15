import { useCallback, useEffect, useState } from "react";

/** Full score breakdown from the backend `user_scores` table. */
export interface UserScoreBreakdown {
    total_score: number | null;
    operator_score: number | null;
    stage_score: number | null;
    roguelike_score: number | null;
    sandbox_score: number | null;
    medal_score: number | null;
    base_score: number | null;
    skin_score: number | null;
    grade: string | null;
    calculated_at: string | null;
}

interface UseUserScoreResult {
    score: UserScoreBreakdown | null;
    isLoading: boolean;
    error: string | null;
    refetch: () => void;
}

/**
 * Fetch a user's score breakdown client-side.
 * v3: proxies `/api/user/{id}/score` → backend `/get-user-score?uid=…`,
 * which returns the whole `user_scores` row or `null` if unscored.
 */
export function useUserScore(userId: string): UseUserScoreResult {
    const [score, setScore] = useState<UserScoreBreakdown | null>(null);
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
                const data = await response.json().catch(() => ({}));
                throw new Error(data.error || "Failed to fetch score");
            }

            const data = (await response.json()) as UserScoreBreakdown | null;
            setScore(data);
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
