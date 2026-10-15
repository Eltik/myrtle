import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { useAuth } from "#/hooks/use-auth";
import { isSuperAdmin, isTierListAdmin, localesQueryOptions, myTranslationPermissionsQueryOptions } from "#/lib/api/admin";
import { bestLocaleGrants, computeSectionAccess, type ILocaleGrant, type SectionAccess } from "./model";
import { type IMyTierListGrant, useMyTierListGrants } from "./tierListGrants";

export interface IAdminAccess {
    role: string | null;
    isSuper: boolean;
    /** `super_admin` or `tier_list_admin`. */
    staff: boolean;
    /** May change roles and grants (super only). */
    canAssign: boolean;
    /** `"all"` for staff (every list at `admin`); otherwise the user's own grants. */
    myTl: readonly IMyTierListGrant[] | "all";
    /** Every locale at `admin` for super; anyone else's own locale grants, any role. */
    myLoc: readonly ILocaleGrant[];
    can: SectionAccess;
    /** True while the tier-list grants behind `can.tierlists` are in flight. */
    tierListGrantsLoading: boolean;
    /** True while the locale grants behind `myLoc` / `can.translations` are in flight. */
    localeGrantsLoading: boolean;
    /**
     * Either of the above. The Translations gate should read
     * `localeGrantsLoading` instead: tier-list grants never decide it.
     */
    grantsLoading: boolean;
}

/**
 * The signed-in user's role model (spec section 0). Every query here is
 * gated to the roles that may call it, so no role fires a request that 403s.
 */
export function useAdminAccess(): IAdminAccess {
    const { user, isAuthenticated } = useAuth();
    const role = user?.role ?? null;
    const userId = user?.id ?? null;
    const isSuper = isSuperAdmin(role);
    const staff = isTierListAdmin(role);

    const tierListGrants = useMyTierListGrants(isAuthenticated && !staff);
    const localesQuery = useQuery({ ...localesQueryOptions(isAuthenticated), enabled: isAuthenticated && isSuper });
    // Self-scoped, so it is safe for every role: a locale grant is held by
    // admins, editors and plain users alike, not just `translator`.
    const wantMyLoc = isAuthenticated && !isSuper && userId != null;
    const myLocQuery = useQuery({ ...myTranslationPermissionsQueryOptions(userId), enabled: wantMyLoc });

    return useMemo((): IAdminAccess => {
        const myTl: IAdminAccess["myTl"] = staff ? "all" : (tierListGrants ?? []);
        let myLoc: ILocaleGrant[] = [];
        if (isSuper) myLoc = (localesQuery.data ?? []).map((l) => ({ code: l.code, level: "admin" }));
        else myLoc = bestLocaleGrants(myLocQuery.data ?? []);
        const tierListsLoading = !staff && tierListGrants === undefined;
        const localesLoading = wantMyLoc && myLocQuery.isPending;
        return {
            role,
            isSuper,
            staff,
            canAssign: isSuper,
            myTl,
            myLoc,
            can: computeSectionAccess(role, myTl === "all" ? 0 : myTl.length, myLoc.length),
            tierListGrantsLoading: tierListsLoading,
            localeGrantsLoading: localesLoading,
            grantsLoading: tierListsLoading || localesLoading,
        };
    }, [role, isSuper, staff, wantMyLoc, tierListGrants, localesQuery.data, myLocQuery.data, myLocQuery.isPending]);
}
