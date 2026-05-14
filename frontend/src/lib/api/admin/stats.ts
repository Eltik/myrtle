import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";
import type { IStatsResponse } from "../stats";

export interface IRoleBreakdown {
    user: number;
    tierListEditor: number;
    tierListAdmin: number;
    superAdmin: number;
}

export interface IRecentUser {
    uid: string;
    serverId: number;
    nickname: string | null;
    level: number | null;
    createdAt: string;
}

export interface IAdminStatsResponse extends IStatsResponse {
    usersByRole: IRoleBreakdown;
    recentUsers: IRecentUser[];
}

export const getAdminStatsFn = createServerFn({ method: "GET" }).handler(async (): Promise<IAdminStatsResponse> => {
    const token = requireSiteToken();
    const res = await backendFetch("/admin/stats", { bearerToken: token });
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as IAdminStatsResponse;
});

export function adminStatsQueryOptions(authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "stats", authed ? "auth" : "anon"],
        queryFn: () => getAdminStatsFn(),
        enabled: authed,
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
