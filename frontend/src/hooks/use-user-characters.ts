import { useCallback, useEffect, useState } from "react";
import type { CharacterData } from "~/types/api/impl/user";

interface UseUserCharactersResult {
    characters: Record<string, CharacterData> | null;
    isLoading: boolean;
    error: string | null;
    refetch: () => void;
}

/**
 * Hook to fetch user character data client-side.
 * Used to lazy-load the Characters tab instead of passing data from SSR.
 */
export function useUserCharacters(userId: string): UseUserCharactersResult {
    const [characters, setCharacters] = useState<Record<string, CharacterData> | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchCharacters = useCallback(async () => {
        if (!userId) {
            setIsLoading(false);
            return;
        }

        setIsLoading(true);
        setError(null);

        try {
            const response = await fetch(`/api/user/${userId}/characters`);

            if (!response.ok) {
                const data = await response.json();
                throw new Error(data.error || "Failed to fetch characters");
            }

            const data = await response.json();
            setCharacters(data.characters);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Unknown error");
            setCharacters(null);
        } finally {
            setIsLoading(false);
        }
    }, [userId]);

    useEffect(() => {
        void fetchCharacters();
    }, [fetchCharacters]);

    return { characters, isLoading, error, refetch: fetchCharacters };
}
