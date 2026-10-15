import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useAuth } from "#/hooks/use-auth";
import { adminUsersQueryOptions, allTierListGrantsQueryOptions, type IAdminUser, type ITierListGrant, translationPermissionsQueryOptions } from "#/lib/api/admin";
import type { TranslationGrant } from "#/types/generated/TranslationGrant";
import { type IPersonGrants, indexGrantsByUser } from "./warnings";

/** The backend's page-size cap on `GET /admin/users`. */
const ROSTER_LIMIT = 200;

export interface IPeopleGrants {
    tierLists: ITierListGrant[];
    locales: TranslationGrant[];
    byUser: Map<string, IPersonGrants>;
}

/**
 * Every tier-list and language grant, bucketed by holder. Staff only (the
 * tier-list endpoint is tier list admin+); `undefined` while either loads.
 */
export function usePeopleGrants(enabled: boolean): IPeopleGrants | undefined {
    const { isAuthenticated, user } = useAuth();
    const on = enabled && isAuthenticated;
    const tl = useQuery({ ...allTierListGrantsQueryOptions({}, on), enabled: on });
    const loc = useQuery({ ...translationPermissionsQueryOptions(undefined, on, user?.id), enabled: on });
    return useMemo(() => {
        if (!tl.data || !loc.data) return undefined;
        return { tierLists: tl.data, locales: loc.data, byUser: indexGrantsByUser(tl.data, loc.data) };
    }, [tl.data, loc.data]);
}

export interface IPeopleRoster {
    /** Every staff member and translator, by account id. */
    byId: Map<string, IAdminUser>;
}

/** Everyone whose role is not Player. Staff only; `undefined` while loading. */
export function usePeopleRoster(enabled: boolean): IPeopleRoster | undefined {
    const { isAuthenticated } = useAuth();
    const on = enabled && isAuthenticated;
    const staff = useQuery({ ...adminUsersQueryOptions({ role: "staff", limit: ROSTER_LIMIT }, on), enabled: on });
    const translators = useQuery({ ...adminUsersQueryOptions({ role: "translators", limit: ROSTER_LIMIT }, on), enabled: on });
    return useMemo(() => {
        if (!staff.data || !translators.data) return undefined;
        const byId = new Map<string, IAdminUser>();
        for (const u of [...staff.data.users, ...translators.data.users]) byId.set(u.id, u);
        return { byId };
    }, [staff.data, translators.data]);
}
