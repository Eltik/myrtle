import { useQueries } from "@tanstack/react-query";
import { useMemo } from "react";
import { dpsCalculateQueryOptions, type IDpsCalculateResponse } from "#/lib/api/dps";
import { instanceToRequest } from "./state";
import type { IDpsInstance, IEnemyConfig } from "./types";

export interface IInstanceSnapshot {
    uid: string;
    data: IDpsCalculateResponse | undefined;
    isPending: boolean;
    error: Error | null;
}

/**
 * Per-instance snapshot at the current enemy DEF/RES - powers the KPI panel.
 * Cache is keyed by `dpsRequestKey` so editing one instance does not refetch
 * the others.
 */
export function useDpsResults(instances: IDpsInstance[], enemy: IEnemyConfig): IInstanceSnapshot[] {
    const queryConfigs = useMemo(() => instances.map((inst) => dpsCalculateQueryOptions(instanceToRequest(inst, enemy))), [instances, enemy]);
    const queries = useQueries({ queries: queryConfigs });
    return useMemo(
        () =>
            instances.map((inst, i) => ({
                uid: inst.uid,
                data: queries[i]?.data,
                isPending: queries[i]?.isPending ?? false,
                error: (queries[i]?.error as Error | null) ?? null,
            })),
        [instances, queries],
    );
}
