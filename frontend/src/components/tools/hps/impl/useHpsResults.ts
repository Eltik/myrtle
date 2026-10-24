import { useQueries } from "@tanstack/react-query";
import { useMemo } from "react";
import { chunk, flattenChunks } from "#/components/tools/shared/batch";
import type { IInstanceSnapshot } from "#/components/tools/shared/KpiPanel";
import { hpsBatchQueryOptions } from "#/lib/api/hps";
import type { HpsBatchItem } from "#/types/generated/HpsBatchItem";
import { instanceToRequest } from "./state";
import type { IHpsBuffConfig, IHpsInstance } from "./types";

/** Per-instance snapshot at the current team buffs / target count, every card in one `POST /hps/batch` (chunked at 64). */
export function useHpsResults(instances: IHpsInstance[], buffs: IHpsBuffConfig): IInstanceSnapshot[] {
    const chunks = useMemo(() => chunk(instances.map((inst) => instanceToRequest(inst, buffs))), [instances, buffs]);
    const queries = useQueries({ queries: chunks.map((requests) => hpsBatchQueryOptions({ requests })) });
    return useMemo(() => {
        const slots = flattenChunks(
            chunks.map((c) => c.length),
            queries,
            (item: HpsBatchItem) => item.result,
        );
        return instances.map((inst, i) => ({ uid: inst.uid, data: slots[i]?.data, isPending: slots[i]?.isPending ?? true, error: slots[i]?.error ?? null }));
    }, [instances, chunks, queries]);
}
