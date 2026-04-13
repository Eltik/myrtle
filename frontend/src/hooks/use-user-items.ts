import { useCallback, useEffect, useState } from "react";
import type { InventoryItem } from "~/types/api/impl/user";

interface UseUserItemsResult {
    inventory: Record<string, InventoryItem> | null;
    isLoading: boolean;
    error: string | null;
    unavailable: boolean;
    refetch: () => void;
}

/**
 * Hook to fetch user inventory data client-side.
 * Used to lazy-load the Items tab instead of passing data from SSR.
 * v3: Items endpoint may return { inventory: null, unavailable: true } when not available.
 */
export function useUserItems(userId: string): UseUserItemsResult {
    const [inventory, setInventory] = useState<Record<string, InventoryItem> | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [unavailable, setUnavailable] = useState(false);

    const fetchItems = useCallback(async () => {
        if (!userId) {
            setIsLoading(false);
            return;
        }

        setIsLoading(true);
        setError(null);
        setUnavailable(false);

        try {
            const response = await fetch(`/api/user/${userId}/items`);

            if (!response.ok) {
                const data = await response.json();
                throw new Error(data.error || "Failed to fetch items");
            }

            const data = await response.json();

            if (data.unavailable) {
                setUnavailable(true);
                setInventory(null);
            } else {
                setInventory(data.inventory);
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : "Unknown error");
            setInventory(null);
        } finally {
            setIsLoading(false);
        }
    }, [userId]);

    useEffect(() => {
        void fetchItems();
    }, [fetchItems]);

    return { inventory, isLoading, error, unavailable, refetch: fetchItems };
}
