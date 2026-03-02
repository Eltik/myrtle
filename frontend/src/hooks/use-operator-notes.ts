import { useCallback, useEffect, useState } from "react";
import type { OperatorNote } from "~/types/api/impl/operator-notes";

interface UseOperatorNotesResult {
    notes: OperatorNote | null;
    isLoading: boolean;
    error: string | null;
    refetch: () => void;
}

/**
 * Hook to fetch operator notes client-side.
 * Treats 404 as notes = null (no error) since many operators won't have content yet.
 */
export function useOperatorNotes(operatorId: string | null): UseOperatorNotesResult {
    const [notes, setNotes] = useState<OperatorNote | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const fetchNotes = useCallback(async () => {
        if (!operatorId) {
            setIsLoading(false);
            return;
        }

        setIsLoading(true);
        setError(null);

        try {
            const response = await fetch(`/api/operator-notes/${operatorId}`);

            if (response.status === 404) {
                setNotes(null);
                return;
            }

            if (!response.ok) {
                const data = await response.json();
                throw new Error(data.error || "Failed to fetch operator notes");
            }

            const data = await response.json();
            setNotes(data);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Unknown error");
            setNotes(null);
        } finally {
            setIsLoading(false);
        }
    }, [operatorId]);

    useEffect(() => {
        void fetchNotes();
    }, [fetchNotes]);

    return { notes, isLoading, error, refetch: fetchNotes };
}
