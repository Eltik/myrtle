import { useCallback, useEffect, useState } from "react";
import type { UnownedOperator } from "~/types/api/impl/user";

interface UseUnownedOperatorsResult {
    unownedOperators: UnownedOperator[] | null;
    isLoading: boolean;
    error: string | null;
}

/**
 * Hook to fetch unowned operators client-side.
 * Only fires when enabled=true (user selects "All" or "Unowned" filter).
 */
export function useUnownedOperators(userId: string, enabled: boolean): UseUnownedOperatorsResult {
    const [unownedOperators, setUnownedOperators] = useState<UnownedOperator[] | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchUnowned = useCallback(async () => {
        if (!userId) return;

        setIsLoading(true);
        setError(null);

        try {
            const response = await fetch(`/api/user/${userId}/unowned`);

            if (!response.ok) {
                const data = await response.json();
                throw new Error(data.error || "Failed to fetch unowned operators");
            }

            const data = await response.json();
            setUnownedOperators(data.unowned);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Unknown error");
            setUnownedOperators(null);
        } finally {
            setIsLoading(false);
        }
    }, [userId]);

    useEffect(() => {
        if (enabled && !unownedOperators && !isLoading) {
            void fetchUnowned();
        }
    }, [enabled, unownedOperators, isLoading, fetchUnowned]);

    return { unownedOperators, isLoading, error };
}
