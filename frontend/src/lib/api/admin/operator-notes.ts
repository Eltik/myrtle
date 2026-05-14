import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";
import type { IOperatorNote } from "../operator-notes";

export interface IUpdateOperatorNoteInput {
    operatorId: string;
    pros?: string | null;
    cons?: string | null;
    notes?: string | null;
    trivia?: string | null;
    summary?: string | null;
    tags?: unknown;
}

export interface IOperatorNoteAuditEntry {
    id: number;
    note_id: string;
    field_name: string;
    old_value: string | null;
    new_value: string | null;
    changed_by: string;
    changed_at: string;
}

export const updateOperatorNoteFn = createServerFn({ method: "POST" })
    .inputValidator((data: IUpdateOperatorNoteInput) => data)
    .handler(async ({ data }): Promise<IOperatorNote> => {
        const token = requireSiteToken();
        const { operatorId, ...fields } = data;
        const res = await backendFetch(`/operator-notes/${encodeURIComponent(operatorId)}`, {
            method: "PUT",
            bearerToken: token,
            body: JSON.stringify(fields),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IOperatorNote;
    });

export const getOperatorNoteAuditLogFn = createServerFn({ method: "GET" })
    .inputValidator((operatorId: string) => operatorId)
    .handler(async ({ data: operatorId }): Promise<IOperatorNoteAuditEntry[]> => {
        const res = await backendFetch(`/operator-notes/${encodeURIComponent(operatorId)}/audit`);
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IOperatorNoteAuditEntry[];
    });

export function operatorNoteAuditLogQueryOptions(operatorId: string) {
    return queryOptions({
        queryKey: ["admin", "operator-notes", "audit", operatorId],
        queryFn: () => getOperatorNoteAuditLogFn({ data: operatorId }),
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
