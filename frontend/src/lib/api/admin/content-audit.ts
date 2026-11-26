import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { ContentAuditEntry } from "#/types/generated/ContentAuditEntry";
import type { ContentAuditResponse } from "#/types/generated/ContentAuditResponse";
import type { ContentTargetKind } from "#/types/generated/ContentTargetKind";
import { parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";

export type IContentAuditEntry = ContentAuditEntry;
export type IContentAuditResponse = ContentAuditResponse;
export type { ContentTargetKind };

export interface IContentAuditInput {
    limit?: number;
    /** The oldest `changed_at` already loaded, for the next page. */
    before?: string;
    /** An account id: only that account's actions. */
    actor?: string;
    targetKind?: ContentTargetKind;
    /** A tier list or grid id: only rows about it. */
    targetId?: string;
}

/**
 * Who created, edited, published, hid, flaired, shared or deleted which tier
 * list or grid. Staff only. A sitting of edits is one row (`event_count`
 * requests), so autosave does not flood it.
 */
export const getContentAuditLogFn = createServerFn({ method: "GET" })
    .inputValidator((data: IContentAuditInput) => data)
    .handler(async ({ data }): Promise<IContentAuditResponse> => {
        const token = requireSiteToken();
        const params = new URLSearchParams();
        if (data.limit !== undefined) params.set("limit", String(data.limit));
        if (data.before) params.set("before", data.before);
        if (data.actor) params.set("actor", data.actor);
        if (data.targetKind) params.set("target_kind", data.targetKind);
        if (data.targetId) params.set("target_id", data.targetId);
        const query = params.toString();
        const res = await backendFetch(`/admin/content-audit${query ? `?${query}` : ""}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IContentAuditResponse;
    });
