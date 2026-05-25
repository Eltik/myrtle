import { useQueries } from "@tanstack/react-query";
import { useMemo } from "react";
import type { IInstanceSnapshot } from "#/components/tools/shared/KpiPanel";
import { hpsCalculateQueryOptions } from "#/lib/api/hps";
import { instanceToRequest } from "./state";
import type { IHpsBuffConfig, IHpsInstance } from "./types";

/** Per-instance snapshot at the current team buffs / target count. */
export function useHpsResults(instances: IHpsInstance[], buffs: IHpsBuffConfig): IInstanceSnapshot[] {
    const queryConfigs = useMemo(() => instances.map((inst) => hpsCalculateQueryOptions(instanceToRequest(inst, buffs))), [instances, buffs]);
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
