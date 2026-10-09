import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { TierListGrant } from "#/types/generated/TierListGrant";
import { type IBackendStatus, parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";
import type { TierListPermissionLevel } from "./types";

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

/** One grant on an active list, with the list, grantee and granter named. */
export interface ITierListGrant {
    tierListId: string;
    slug: string;
    title: string;
    listType: "official" | "community";
    userId: string;
    userUid: string;
    userNickname: string | null;
    permission: TierListPermissionLevel;
    grantedBy: string | null;
    grantedByNickname: string | null;
    grantedAt: string;
}

function mapTierListGrant(raw: TierListGrant): ITierListGrant {
    return {
        tierListId: raw.tier_list_id,
        slug: raw.slug,
        title: raw.title,
        listType: raw.list_type === "official" ? "official" : "community",
        userId: raw.user_id,
        userUid: raw.user_uid,
        userNickname: raw.user_nickname,
        permission: raw.permission as TierListPermissionLevel,
        grantedBy: raw.granted_by,
        grantedByNickname: raw.granted_by_nickname,
        grantedAt: raw.granted_at,
    };
}

export interface IAllTierListGrantsInput {
    /** Only this account's grants. */
    userId?: string;
}

export const getAllTierListGrantsFn = createServerFn({ method: "GET" })
    .inputValidator((data: IAllTierListGrantsInput) => data)
    .handler(async ({ data }): Promise<ITierListGrant[]> => {
        const token = requireSiteToken();
        const query = data.userId ? `?${new URLSearchParams({ user_id: data.userId }).toString()}` : "";
        const res = await backendFetch(`/admin/tier-lists/permissions${query}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        const raw = (await res.json()) as TierListGrant[];
        return raw.map(mapTierListGrant);
    });

/**
 * Every tier-list grant across every active list, newest first (`GET
 * /admin/tier-lists/permissions`). Tier list admin or super-admin only; pass
 * `authed` as that role check so other roles never fire a 403.
 */
export function allTierListGrantsQueryOptions(input: IAllTierListGrantsInput, authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "tier-lists", "permissions", "all", input, authed ? "auth" : "anon"],
        queryFn: () => getAllTierListGrantsFn({ data: input }),
        enabled: authed,
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
