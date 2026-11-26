import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { useAuth } from "#/hooks/use-auth";
import { userRosterQueryOptions } from "#/lib/api/user";

export interface IRecruitRoster {
    /** Somebody is signed in, so the roster toggles mean something. */
    signedIn: boolean;
    /** The roster request is still in flight; cards render without the overlay until it lands. */
    loading: boolean;
    /** Operator id -> 0-based potential (0 = P1, 5 = P6). Empty until loaded or when signed out. */
    potentialByOperator: ReadonlyMap<string, number>;
    /** The roster request failed: the empty map would read every operator as unowned, so nothing owned-dependent may show. */
    failed: boolean;
    refetch: () => void;
}

const EMPTY: ReadonlyMap<string, number> = new Map();

/**
 * The signed-in user's roster reduced to the one field this tool reads. The
 * query is the same one the planner and the profile use, so the roster is
 * fetched once per session and shared through the react-query cache; the
 * reduction to a map is the only work done here.
 */
export function useRecruitRoster(): IRecruitRoster {
    const { user } = useAuth();
    const uid = user?.uid ?? "";
    const { data, isLoading, isError, refetch } = useQuery({
        ...userRosterQueryOptions(uid),
        enabled: uid.length > 0,
    });

    const potentialByOperator = React.useMemo(() => {
        if (!data) return EMPTY;
        const out = new Map<string, number>();
        for (const entry of data) out.set(entry.operator_id, entry.potential);
        return out;
    }, [data]);

    const failed = uid.length > 0 && isError && !data;
    return { signedIn: uid.length > 0, loading: uid.length > 0 && isLoading, potentialByOperator, failed, refetch: () => void refetch() };
}
