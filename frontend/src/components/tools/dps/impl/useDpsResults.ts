import { useQueries } from "@tanstack/react-query";
import { useMemo } from "react";
import type { IInstanceSnapshot } from "#/components/tools/shared/KpiPanel";
import { dpsCalculateQueryOptions } from "#/lib/api/dps";
import { instanceToRequest } from "./state";
import type { IDpsInstance, IEnemyConfig } from "./types";

/** Per-instance snapshot at the current enemy DEF/RES - powers the KPI panel. */
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
