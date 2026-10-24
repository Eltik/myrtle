import { useQueries } from "@tanstack/react-query";
import { useMemo } from "react";
import { chunk, flattenChunks } from "#/components/tools/shared/batch";
import { buildSweepPoints } from "#/components/tools/shared/constants";
import type { ICurvePoint, ISweepRange } from "#/components/tools/shared/types";
import { hpsBatchQueryOptions } from "#/lib/api/hps";
import type { HpsBatchItem } from "#/types/generated/HpsBatchItem";
import { type IAxisOverride, instanceToRequest } from "./state";
import type { HpsXAxis, HpsYMetric, IHpsBuffConfig, IHpsState } from "./types";

interface ICurvePlan {
    instUid: string;
    pointIdx: number;
}

interface IUseHpsCurvesResult {
    points: number[];
    rows: ICurvePoint[];
    isPending: boolean;
}

/** Translate a swept X value (display units) into a per-request override. */
function axisOverride(axis: HpsXAxis, x: number): IAxisOverride {
    switch (axis) {
        case "targets":
            return { targets: x };
        case "atk":
            return { atkPercent: x };
        case "aspd":
            return { aspd: x };
    }
}

/**
 * Every (card, point) as one request, sent through `POST /hps/batch` in runs of 64. The HPS
 * batch has no sweep (the axes are buffs and targets, not enemy stats), so an 8-card,
 * 41-point chart is 328 requests in 6 calls instead of 328 calls.
 */
export function useHpsCurves(state: IHpsState, buffs: IHpsBuffConfig): IUseHpsCurvesResult {
    const range: ISweepRange = state.sweep[state.xAxis];
    const isInteger = state.xAxis === "targets";
    const points = useMemo(() => buildSweepPoints(range, isInteger), [range, isInteger]);

    const visibleInstances = useMemo(() => state.instances.filter((i) => i.visible), [state.instances]);

    const { plans, chunks } = useMemo(() => {
        const plans: ICurvePlan[] = [];
        const requests = visibleInstances.flatMap((inst) =>
            points.map((x, idx) => {
                plans.push({ instUid: inst.uid, pointIdx: idx });
                return instanceToRequest(inst, buffs, axisOverride(state.xAxis, x));
            }),
        );
        return { plans, chunks: chunk(requests) };
    }, [visibleInstances, points, buffs, state.xAxis]);

    const queries = useQueries({ queries: chunks.map((requests) => hpsBatchQueryOptions({ requests })) });

    return useMemo(() => {
        const slots = flattenChunks(
            chunks.map((c) => c.length),
            queries,
            (item: HpsBatchItem) => item.result,
        );
        const rows: ICurvePoint[] = points.map((x) => ({ x }));
        let isPending = false;
        for (let i = 0; i < plans.length; i++) {
            const plan = plans[i];
            const slot = slots[i];
            const row = rows[plan.pointIdx];
            if (!row) continue;
            if (slot?.isPending) isPending = true;
            const data = slot?.data;
            row[plan.instUid] = data ? selectMetric(data, state.yMetric) : null;
        }
        return { points, rows, isPending };
    }, [plans, chunks, queries, points, state.yMetric]);
}

function selectMetric(d: { skill_hps: number; base_hps: number; avg_hps: number }, metric: HpsYMetric): number {
    switch (metric) {
        case "skill_hps":
            return d.skill_hps;
        case "base_hps":
            return d.base_hps;
        case "avg_hps":
            return d.avg_hps;
    }
}
