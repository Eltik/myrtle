import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import type { AdminUserEntry } from "#/types/generated/AdminUserEntry";
import type { AdminUsersPage } from "#/types/generated/AdminUsersPage";
import { parseError } from "../_shared";
import { requireSiteToken } from "../_shared.server";
import type { AdminUserRoleFilter, AdminUserServer, UserRole } from "./types";

export interface IAdminUsersInput {
    /** Nickname substring (case-insensitive) or UID prefix; an exact UID sorts first. */
    q?: string;
    role?: AdminUserRoleFilter;
    server?: AdminUserServer;
    /** Defaults to 50 on the backend, capped at 200. */
    limit?: number;
    offset?: number;
}

export interface IAdminUser {
    id: string;
    uid: string;
    /** `servers.code`, e.g. `EN`. */
    server: string;
    nickname: string | null;
    nickNumber: string | null;
    level: number | null;
    role: UserRole;
    avatarId: string | null;
    secretary: string | null;
    secretarySkinId: string | null;
    /** `false` also for an account that never saved settings, which the profile gate treats as private. */
    publicProfile: boolean;
    totalScore: number | null;
    grade: string | null;
    operatorCount: number;
    itemCount: number;
    skinCount: number;
    createdAt: string;
    updatedAt: string;
}

export interface IAdminUsersPage {
    users: IAdminUser[];
    /** Every account the filters admit, across all pages. */
    total: number;
}

function mapAdminUser(raw: AdminUserEntry): IAdminUser {
    return {
        id: raw.id,
        uid: raw.uid,
        server: raw.server,
        nickname: raw.nickname,
        nickNumber: raw.nick_number,
        level: raw.level,
        role: raw.role as UserRole,
        avatarId: raw.avatar_id,
        secretary: raw.secretary,
        secretarySkinId: raw.secretary_skin_id,
        publicProfile: raw.public_profile,
        totalScore: raw.total_score,
        grade: raw.grade,
        operatorCount: raw.operator_count,
        itemCount: raw.item_count,
        skinCount: raw.skin_count,
        createdAt: raw.created_at,
        updatedAt: raw.updated_at,
    };
}

export const listAdminUsersFn = createServerFn({ method: "GET" })
    .inputValidator((data: IAdminUsersInput) => data)
    .handler(async ({ data }): Promise<IAdminUsersPage> => {
        const token = requireSiteToken();
        const params = new URLSearchParams();
        const q = data.q?.trim();
        if (q) params.set("q", q);
        if (data.role && data.role !== "all") params.set("role", data.role);
        if (data.server) params.set("server", data.server);
        if (data.limit !== undefined) params.set("limit", String(data.limit));
        if (data.offset !== undefined) params.set("offset", String(data.offset));
        const query = params.toString();
        const res = await backendFetch(`/admin/users${query ? `?${query}` : ""}`, { bearerToken: token });
        if (!res.ok) throw await parseError(res);
        const raw = (await res.json()) as AdminUsersPage;
        return { users: raw.users.map(mapAdminUser), total: raw.total };
    });

/** Tier list admin or super-admin only; pass `authed` as that role check so other roles never fire a 403. */
export function adminUsersQueryOptions(input: IAdminUsersInput, authed: boolean) {
    return queryOptions({
        queryKey: ["admin", "users", input, authed ? "auth" : "anon"],
        queryFn: () => listAdminUsersFn({ data: input }),
        enabled: authed,
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
