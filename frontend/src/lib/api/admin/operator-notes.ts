import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { sanitizeMarkdownForStorage, sanitizePlainName } from "#/lib/markdown/sanitize-input";
import type { AuditLogActor } from "#/types/generated/AuditLogActor";
import type { GlobalAuditLogResponse } from "#/types/generated/GlobalAuditLogResponse";
import type { OperatorNoteAuditActor } from "#/types/generated/OperatorNoteAuditActor";
import type { OperatorNoteAuditEntry } from "#/types/generated/OperatorNoteAuditEntry";
import type { OperatorNoteAuditLogEntry } from "#/types/generated/OperatorNoteAuditLogEntry";
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

export type IOperatorNoteAuditEntry = OperatorNoteAuditEntry;
export type IOperatorNoteAuditActor = OperatorNoteAuditActor;

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

export type IAuditLogActor = AuditLogActor;
export type IAuditLogEntry = OperatorNoteAuditLogEntry;
export type IGlobalAuditLogResponse = GlobalAuditLogResponse;

export interface IGlobalAuditLogInput {
    limit?: number;
    before?: string;
    /**
     * `"me"` or an account id: only that account's edits. A tier list editor
     * always gets their own rows (naming anyone else is 403), so editors may
     * call this too; admins get everyone's when it is absent.
     */
    actor?: string;
}

export const getGlobalAuditLogFn = createServerFn({ method: "GET" })
    .inputValidator((data: IGlobalAuditLogInput) => data)
    .handler(async ({ data }): Promise<IGlobalAuditLogResponse> => {
        const token = requireSiteToken();
        const params = new URLSearchParams();
        if (data.limit !== undefined) params.set("limit", String(data.limit));
        if (data.before) params.set("before", data.before);
        if (data.actor) params.set("actor", data.actor);
        const query = params.toString();
        const res = await backendFetch(`/admin/operator-notes/audit${query ? `?${query}` : ""}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IGlobalAuditLogResponse;
    });

/**
 * `viewerId` identifies whose rows `actor: "me"` resolved to, so two accounts
 * in one tab never share a cache entry. Optional for older callers.
 */
export function globalAuditLogQueryOptions(input: IGlobalAuditLogInput = {}, viewerId?: string | null) {
    return queryOptions({
        queryKey: ["admin", "operator-notes", "audit", "global", input, input.actor === "me" ? (viewerId ?? null) : null],
        queryFn: () => getGlobalAuditLogFn({ data: input }),
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
