import { useQueries } from "@tanstack/react-query";
import { useMemo, useRef } from "react";
import { chunk, flattenChunks } from "#/components/tools/shared/batch";
import { buildSweepPoints } from "#/components/tools/shared/constants";
import type { ICurvePoint, ISweepRange } from "#/components/tools/shared/types";
import { dpsBatchQueryOptions } from "#/lib/api/dps";
import type { BatchItem } from "#/types/generated/BatchItem";
import type { Sweep } from "#/types/generated/Sweep";
import type { SweepPoint } from "#/types/generated/SweepPoint";
import { instanceToRequest } from "./state";
import type { IDpsState, YMetric } from "./types";
import type { ITeamBuffsContext } from "./useTeamBuffs";

interface IUseDpsCurvesResult {
    points: number[];
    rows: ICurvePoint[];
    isPending: boolean;
}

/**
 * Every visible card's curve from `POST /dps/batch` with a sweep: one call per 64 cards,
 * the backend evaluating each card at every point. It replaced one `/dps/calculate` per
 * (card, point): an 8-card, 41-point chart was 328 requests and is now 1.
 */
export function useDpsCurves(state: IDpsState, ctx: ITeamBuffsContext): IUseDpsCurvesResult {
    const range: ISweepRange = state.sweep[state.xAxis];
    const points = useMemo(() => buildSweepPoints(range), [range]);
    const { sources, waiting, error: teamError, professionOf } = ctx;

    const visibleInstances = useMemo(() => state.instances.filter((i) => i.visible), [state.instances]);

    // The same points `buildSweepPoints` lays out, so every card's x lines up on the chart rows.
    const sweep = useMemo<Sweep>(() => ({ axis: state.xAxis === "defense" ? "def" : "res", min: points[0] ?? 0, max: points[points.length - 1] ?? 0, steps: points.length }), [points, state.xAxis]);

    const chunks = useMemo(() => chunk(visibleInstances.map((inst) => instanceToRequest(inst, state.enemy, state.team, sources, professionOf(inst.op.id)))), [visibleInstances, state.enemy, state.team, sources, professionOf]);
    const queries = useQueries({ queries: chunks.map((requests) => ({ ...dpsBatchQueryOptions({ requests, sweep }), enabled: !waiting && !teamError })) });
    // The last curve per card, drawn while a new batch runs (one edited card re-sends them all).
    const lastRef = useRef(new Map<string, SweepPoint[]>());

    return useMemo(() => {
        const slots = flattenChunks(
            chunks.map((c) => c.length),
            queries.map((q) => ({ data: q.data, isPending: q.isPending, error: q.error })),
            (item: BatchItem) => item.points,
        );
        const rows: ICurvePoint[] = points.map((x) => ({ x }));
        // Forget cards that are gone or hidden, so their uid holds no stale curve.
        const live = new Set(visibleInstances.map((i) => i.uid));
        for (const uid of lastRef.current.keys()) if (!live.has(uid)) lastRef.current.delete(uid);
        if (teamError) return { points, rows, isPending: false };
        let isPending = false;
        visibleInstances.forEach((inst, i) => {
            const slot = slots[i];
            if (slot?.isPending) isPending = true;
            if (slot?.data) lastRef.current.set(inst.uid, slot.data);
            const curve = slot?.data ?? (slot?.isPending ? lastRef.current.get(inst.uid) : undefined);
            rows.forEach((row, p) => {
                // A held curve from a sweep with another point count is not drawn against these rows.
                // Its x can differ from the row's in the third decimal: `buildSweepPoints` rounds to 0.01.
                const point = curve?.length === points.length ? curve[p] : undefined;
                row[inst.uid] = point ? selectMetric(point, state.yMetric) : null;
            });
        });
        return { points, rows, isPending };
    }, [chunks, queries, points, visibleInstances, state.yMetric, teamError]);
}

function selectMetric(d: SweepPoint, metric: YMetric): number {
    switch (metric) {
        case "skill_dps":
            return d.skill_dps;
        case "average_dps":
            return d.average_dps;
        case "total_damage":
            return d.total_damage;
    }
}
