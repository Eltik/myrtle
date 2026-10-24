import { type UseMutationResult, type UseQueryResult, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo } from "react";
import { useAuth } from "#/hooks/use-auth";
import type { IBackendStatus } from "#/lib/api/_shared";
import { isSuperAdmin } from "#/lib/api/admin";
import { deleteReleaseOverrideFn, type IReleaseOverrideKey, putReleaseOverrideFn, releaseOverridesQueryOptions } from "#/lib/api/release";
import type { PutOverride } from "#/types/generated/PutOverride";
import type { ReleaseOverride } from "#/types/generated/ReleaseOverride";
import { overrideMap } from "./overrides";

export interface IReleaseOverrides {
    /** Super admins only; everyone else sees the planner exactly as before. */
    canEdit: boolean;
    query: UseQueryResult<ReleaseOverride[]>;
    /** Keyed by `overrideKey(kind, cnId)`. */
    overrides: Map<string, ReleaseOverride>;
    save: UseMutationResult<ReleaseOverride, unknown, PutOverride>;
    remove: UseMutationResult<IBackendStatus, unknown, IReleaseOverrideKey>;
}

/**
 * The manual release-date overrides and the writes on them. A write invalidates all of
 * `["release"]`: the backend drops its own release cache on a write, so events, banners, skins
 * and the lag model refetch with the moved row, and this list refetches with them.
 */
export function useReleaseOverrides(): IReleaseOverrides {
    const { user } = useAuth();
    const canEdit = isSuperAdmin(user?.role);
    const queryClient = useQueryClient();
    const query = useQuery({ ...releaseOverridesQueryOptions(), enabled: canEdit });
    const overrides = useMemo(() => overrideMap(query.data), [query.data]);
    const invalidate = (): void => void queryClient.invalidateQueries({ queryKey: ["release"] });
    const save = useMutation({ mutationFn: (input: PutOverride) => putReleaseOverrideFn({ data: input }), onSuccess: invalidate });
    const remove = useMutation({ mutationFn: (key: IReleaseOverrideKey) => deleteReleaseOverrideFn({ data: key }), onSuccess: invalidate });
    return { canEdit, query, overrides, save, remove };
}
