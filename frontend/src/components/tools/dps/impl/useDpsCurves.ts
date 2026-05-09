import { useQueries } from "@tanstack/react-query";
import { useMemo } from "react";
import { dpsCalculateQueryOptions } from "#/lib/api/dps";
import { buildSweepPoints } from "./constants";
import { instanceToRequest } from "./state";
import type { ICurvePoint, IDpsState, IEnemyConfig, ISweepRange, YMetric } from "./types";

interface ICurvePlan {
    instUid: string;
    pointIdx: number;
}

interface IUseDpsCurvesResult {
    points: number[];
    /** Recharts-shaped: one row per sweep point, keyed by instance uid → metric value (or null while loading). */
    rows: ICurvePoint[];
    isPending: boolean;
}

export function useDpsCurves(state: IDpsState, enemy: IEnemyConfig): IUseDpsCurvesResult {
    const range: ISweepRange = state.sweep[state.xAxis];
    const points = useMemo(() => buildSweepPoints(range), [range]);

    const visibleInstances = useMemo(() => state.instances.filter((i) => i.visible), [state.instances]);

    const { plans, queryConfigs } = useMemo(() => {
        const plans: ICurvePlan[] = [];
        const queryConfigs = visibleInstances.flatMap((inst) =>
            points.map((x, idx) => {
                plans.push({ instUid: inst.uid, pointIdx: idx });
                const override: { defense?: number; res?: number } = {};
                if (state.xAxis === "defense") override.defense = x;
                else override.res = x;
                return dpsCalculateQueryOptions(instanceToRequest(inst, enemy, override));
            }),
        );
        return { plans, queryConfigs };
    }, [visibleInstances, points, enemy, state.xAxis]);

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

function selectMetric(d: { skill_dps: number; total_damage: number; average_dps: number }, metric: YMetric): number {
    switch (metric) {
        case "skill_dps":
            return d.skill_dps;
        case "average_dps":
            return d.average_dps;
        case "total_damage":
            return d.total_damage;
    }
}
