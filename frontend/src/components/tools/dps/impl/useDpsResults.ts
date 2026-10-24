import { useQueries } from "@tanstack/react-query";
import { useMemo, useRef } from "react";
import { chunk, flattenChunks } from "#/components/tools/shared/batch";
import { dpsBatchQueryOptions, type IDpsCalculateResponse } from "#/lib/api/dps";
import type { BatchItem } from "#/types/generated/BatchItem";
import { instanceToRequest } from "./state";
import type { IDpsInstance, IEnemyConfig, ITeamBuffs } from "./types";
import type { ITeamBuffsContext } from "./useTeamBuffs";

/** One card's result at the current enemy, for the KPI panel. */
export interface IDpsSnapshot {
    uid: string;
    data: IDpsCalculateResponse | undefined;
    isPending: boolean;
    error: Error | null;
}

/**
 * Per-instance snapshot at the current enemy DEF/RES, every card in one
 * `POST /dps/batch` (chunked at 64), instead of one `/dps/calculate` per card.
 */
export function useDpsResults(instances: IDpsInstance[], enemy: IEnemyConfig, team: ITeamBuffs, ctx: ITeamBuffsContext): IDpsSnapshot[] {
    const { sources, waiting, error: teamError, professionOf } = ctx;
    const chunks = useMemo(() => chunk(instances.map((inst) => instanceToRequest(inst, enemy, team, sources, professionOf(inst.op.id)))), [instances, enemy, team, sources, professionOf]);
    const queries = useQueries({ queries: chunks.map((requests) => ({ ...dpsBatchQueryOptions({ requests }), enabled: !waiting && !teamError })) });
    // The last answer per card, shown while a new batch runs: one edited card re-sends the
    // whole batch, and the other cards' numbers should not blank meanwhile.
    const lastRef = useRef(new Map<string, IDpsCalculateResponse>());
    return useMemo(() => {
        const slots = flattenChunks(
            chunks.map((c) => c.length),
            queries.map((q) => ({ data: q.data, isPending: q.isPending, error: q.error })),
            (item: BatchItem) => item.result,
        );
        // Forget cards that are gone, so a removed card's uid holds nothing.
        const live = new Set(instances.map((i) => i.uid));
        for (const uid of lastRef.current.keys()) if (!live.has(uid)) lastRef.current.delete(uid);
        if (teamError) return instances.map((inst) => ({ uid: inst.uid, data: undefined, isPending: false, error: teamError }));
        return instances.map((inst, i) => {
            const slot = slots[i];
            if (slot?.data) lastRef.current.set(inst.uid, slot.data);
            const data = slot?.data ?? (slot?.isPending ? lastRef.current.get(inst.uid) : undefined);
            return { uid: inst.uid, data, isPending: slot?.isPending ?? true, error: slot?.error ?? null };
        });
    }, [instances, chunks, queries, teamError]);
}
