import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { type IBackendStatus, parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";
import type { TierListPermissionLevel } from "./types";

export interface ITierListPermissionEntry {
    tierListId: string;
    userId: string;
    permission: TierListPermissionLevel;
    grantedBy: string | null;
    grantedAt: string;
}

export interface IGrantTierListPermissionInput {
    slug: string;
    userId: string;
    permission: TierListPermissionLevel;
}

export interface IRevokeTierListPermissionInput {
    slug: string;
    userId: string;
    permission: TierListPermissionLevel;
}

interface IBackendTierListPermission {
    tier_list_id: string;
    user_id: string;
    permission: TierListPermissionLevel;
    granted_by: string | null;
    granted_at: string;
}

function mapTierListPermission(raw: IBackendTierListPermission): ITierListPermissionEntry {
    return {
        tierListId: raw.tier_list_id,
        userId: raw.user_id,
        permission: raw.permission,
        grantedBy: raw.granted_by,
        grantedAt: raw.granted_at,
    };
}

export const getTierListPermissionsFn = createServerFn({ method: "GET" })
    .inputValidator((slug: string) => slug)
    .handler(async ({ data: slug }): Promise<ITierListPermissionEntry[]> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/tier-lists/${encodeURIComponent(slug)}/permissions`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        const raw = (await res.json()) as IBackendTierListPermission[];
        return raw.map(mapTierListPermission);
    });

export function tierListPermissionsQueryOptions(slug: string, authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "tier-lists", "permissions", slug, authed ? "auth" : "anon"],
        queryFn: () => getTierListPermissionsFn({ data: slug }),
        enabled: authed,
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const grantTierListPermissionFn = createServerFn({ method: "POST" })
    .inputValidator((data: IGrantTierListPermissionInput) => data)
    .handler(async ({ data }): Promise<IBackendStatus> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/tier-lists/${encodeURIComponent(data.slug)}/permissions`, {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ user_id: data.userId, permission: data.permission }),
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IBackendStatus;
    });

export const revokeTierListPermissionFn = createServerFn({ method: "POST" })
    .inputValidator((data: IRevokeTierListPermissionInput) => data)
    .handler(async ({ data }): Promise<IBackendStatus> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/tier-lists/${encodeURIComponent(data.slug)}/permissions/${encodeURIComponent(data.userId)}/${encodeURIComponent(data.permission)}`, {
            method: "DELETE",
            bearerToken: token,
        });
        if (!res.ok) throw await parseError(res);
        return (await res.json()) as IBackendStatus;
    });
