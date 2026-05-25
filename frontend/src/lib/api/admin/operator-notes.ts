import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { sanitizeMarkdownForStorage, sanitizePlainName } from "#/lib/markdown/sanitize-input";
import { parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";
import type { IOperatorNote } from "../operator-notes";

const SUMMARY_LIMIT = 280;
const PROS_CONS_LIMIT = 2000;
const NOTES_LIMIT = 8000;
const TRIVIA_LIMIT = 2000;
const TAG_LIMIT = 32;
const TAG_COUNT_LIMIT = 16;

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
        const { operatorId, summary, pros, cons, notes, trivia, tags } = data;
        const cleanTags = Array.isArray(tags)
            ? (tags as unknown[])
                  .filter((t): t is string => typeof t === "string")
                  .map((t) => sanitizePlainName(t, TAG_LIMIT))
                  .filter((t) => t.length > 0)
                  .slice(0, TAG_COUNT_LIMIT)
            : undefined;
        const payload = {
            summary: summary === undefined ? undefined : sanitizePlainName(summary, SUMMARY_LIMIT) || null,
            pros: pros === undefined ? undefined : sanitizeMarkdownForStorage(pros, { maxLength: PROS_CONS_LIMIT, nullOnEmpty: true }),
            cons: cons === undefined ? undefined : sanitizeMarkdownForStorage(cons, { maxLength: PROS_CONS_LIMIT, nullOnEmpty: true }),
            notes: notes === undefined ? undefined : sanitizeMarkdownForStorage(notes, { maxLength: NOTES_LIMIT, nullOnEmpty: true }),
            trivia: trivia === undefined ? undefined : sanitizeMarkdownForStorage(trivia, { maxLength: TRIVIA_LIMIT, nullOnEmpty: true }),
            tags: cleanTags,
        };
        const res = await backendFetch(`/operator-notes/${encodeURIComponent(operatorId)}`, {
            method: "PUT",
            bearerToken: token,
            body: JSON.stringify(payload),
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

export interface IAuditLogActor {
    user_id: string;
    uid: string | null;
    nickname: string | null;
    secretary: string | null;
    secretary_skin_id: string | null;
}

export interface IAuditLogEntry {
    id: number;
    note_id: string;
    operator_id: string;
    field_name: string;
    old_value: string | null;
    new_value: string | null;
    changed_at: string;
    actor: IAuditLogActor;
}

export interface IGlobalAuditLogResponse {
    entries: IAuditLogEntry[];
    total: number;
}

export interface IGlobalAuditLogInput {
    limit?: number;
    before?: string;
}

export const getGlobalAuditLogFn = createServerFn({ method: "GET" })
    .inputValidator((data: IGlobalAuditLogInput) => data)
    .handler(async ({ data }): Promise<IGlobalAuditLogResponse> => {
        const token = requireSiteToken();
        const params = new URLSearchParams();
        if (data.limit !== undefined) params.set("limit", String(data.limit));
        if (data.before) params.set("before", data.before);
        const query = params.toString();
        const res = await backendFetch(`/admin/operator-notes/audit${query ? `?${query}` : ""}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IGlobalAuditLogResponse;
    });

export function globalAuditLogQueryOptions(input: IGlobalAuditLogInput = {}) {
    return queryOptions({
        queryKey: ["admin", "operator-notes", "audit", "global", input],
        queryFn: () => getGlobalAuditLogFn({ data: input }),
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
