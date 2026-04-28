import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";

export interface IOperatorNote {
    id: string;
    operator_id: string;
    pros: string | null;
    cons: string | null;
    notes: string | null;
    trivia: string | null;
    summary: string | null;
    tags: string[] | null;
    created_at: string;
    updated_at: string;
}

export const getOperatorNotesFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/operator-notes");
    if (!res.ok) throw new Error(`Failed to load operator notes: ${res.status}`);
    return (await res.json()) as IOperatorNote[];
});

export function operatorNotesListQueryOptions() {
    return queryOptions({
        queryKey: ["operator-notes", "list"],
        queryFn: () => getOperatorNotesFn(),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

export function noteHasContent(note: IOperatorNote): boolean {
    if (note.pros?.trim()) return true;
    if (note.cons?.trim()) return true;
    if (note.notes?.trim()) return true;
    if (note.trivia?.trim()) return true;
    if (note.summary?.trim()) return true;
    if (Array.isArray(note.tags) && note.tags.length > 0) return true;
    return false;
}
