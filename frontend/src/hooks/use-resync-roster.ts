import { useMutation, useQueryClient } from "@tanstack/react-query";
import { refreshRosterFn } from "#/lib/api/auth";
import { forgetSession } from "#/lib/root-context";

/**
 * Returns a function that marks the signed-in user's profile as stale: it drops
 * the memoised session (which carries the profile row) and refetches every
 * `["user", ...]` query. Call it after anything that rewrites the profile.
 */
export function useInvalidateProfile(): () => Promise<void> {
    const queryClient = useQueryClient();
    return () => {
        forgetSession();
        return queryClient.invalidateQueries({ queryKey: ["user"] });
    };
}

interface IResyncRosterCallbacks {
    /** Runs once the fresh roster has been refetched. */
    onSuccess?: () => void;
    onError?: (err: unknown) => void;
}

/**
 * Pull a fresh snapshot of the signed-in user's game data from Yostar.
 *
 * The profile refresh is built in, so callers only add their own feedback
 * (a toast, inline text) through the callbacks or the mutation's state.
 */
export function useResyncRoster({ onSuccess, onError }: IResyncRosterCallbacks = {}) {
    const invalidateProfile = useInvalidateProfile();
    return useMutation({
        mutationFn: () => refreshRosterFn(),
        onSuccess: async () => {
            await invalidateProfile();
            onSuccess?.();
        },
        onError,
    });
}
