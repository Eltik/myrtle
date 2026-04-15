import { useCallback, useEffect, useState } from "react";
import type { EnrichedRosterEntry } from "~/types/api/impl/user";

interface UseUserCharactersResult {
    characters: EnrichedRosterEntry[] | null;
    isLoading: boolean;
    error: string | null;
    refetch: () => void;
}

/**
 * Hook to fetch user character data client-side.
 * Used to lazy-load the Characters tab instead of passing data from SSR.
 * v3: Returns EnrichedRosterEntry[] (RosterEntry + static data) instead of Record<string, CharacterData>.
 */
export function useUserCharacters(userId: string): UseUserCharactersResult {
    const [characters, setCharacters] = useState<EnrichedRosterEntry[] | null>(null);
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
