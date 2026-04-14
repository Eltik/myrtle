import { useCallback, useEffect, useState } from "react";

interface UserScore {
    total_score: number | null;
    grade: string | null;
}

interface UseUserScoreResult {
    score: UserScore | null;
    isLoading: boolean;
    error: string | null;
    refetch: () => void;
}

/**
 * Hook to fetch user score data client-side.
 * Used to lazy-load the Score tab instead of passing data from SSR.
 * v3: Returns { total_score, grade } directly instead of StoredUserScore.
 */
export function useUserScore(userId: string): UseUserScoreResult {
    const [score, setScore] = useState<UserScore | null>(null);
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
            setScore({
                total_score: data.total_score ?? null,
                grade: data.grade ?? null,
            });
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
