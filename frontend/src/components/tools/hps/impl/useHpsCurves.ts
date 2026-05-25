import { useQueries } from "@tanstack/react-query";
import { useMemo } from "react";
import { buildSweepPoints } from "#/components/tools/shared/constants";
import type { ICurvePoint, ISweepRange } from "#/components/tools/shared/types";
import { hpsCalculateQueryOptions } from "#/lib/api/hps";
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

export function useHpsCurves(state: IHpsState, buffs: IHpsBuffConfig): IUseHpsCurvesResult {
    const range: ISweepRange = state.sweep[state.xAxis];
    const isInteger = state.xAxis === "targets";
    const points = useMemo(() => buildSweepPoints(range, isInteger), [range, isInteger]);

    const visibleInstances = useMemo(() => state.instances.filter((i) => i.visible), [state.instances]);

    const { plans, queryConfigs } = useMemo(() => {
        const plans: ICurvePlan[] = [];
        const queryConfigs = visibleInstances.flatMap((inst) =>
            points.map((x, idx) => {
                plans.push({ instUid: inst.uid, pointIdx: idx });
                return hpsCalculateQueryOptions(instanceToRequest(inst, buffs, axisOverride(state.xAxis, x)));
            }),
        );
        return { plans, queryConfigs };
    }, [visibleInstances, points, buffs, state.xAxis]);

    const queries = useQueries({ queries: queryConfigs });

    return useMemo(() => {
        const rows: ICurvePoint[] = points.map((x) => ({ x }));
        let isPending = false;
        for (let i = 0; i < plans.length; i++) {
            const plan = plans[i];
            const q = queries[i];
            const row = rows[plan.pointIdx];
            if (!row) continue;
            if (q?.isPending) isPending = true;
            const data = q?.data;
            row[plan.instUid] = data ? selectMetric(data, state.yMetric) : null;
        }
        return { points, rows, isPending };
    }, [plans, queries, points, state.yMetric]);
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
