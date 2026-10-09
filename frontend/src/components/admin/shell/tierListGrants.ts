import { useQuery } from "@tanstack/react-query";
import type { TierListPermissionLevel } from "#/lib/api/admin";
import { grantedTierListsQueryOptions } from "#/lib/api/tier-lists";

export interface IMyTierListGrant {
    slug: string;
    permission: TierListPermissionLevel;
}

/**
 * The signed-in user's own per-list tier-list grants (non-staff only: staff
 * reach every list at `admin` without one). This is the ONLY place the shell
 * reads tier-list grants.
 *
 * `undefined` = still loading; `[]` = loaded, none, or the request failed
 * (a failure must not leave the caller loading forever).
 */
export function useMyTierListGrants(enabled: boolean): readonly IMyTierListGrant[] | undefined {
    const query = useQuery({ ...grantedTierListsQueryOptions(enabled), enabled });
    if (!enabled) return [];
    return query.data ?? (query.isError ? [] : undefined);
}
