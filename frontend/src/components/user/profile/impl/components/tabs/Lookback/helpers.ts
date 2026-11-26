import type { LookbackOperator } from "#/types/generated/LookbackOperator";
import type { LookbackSnapshot } from "#/types/generated/LookbackSnapshot";
import type { LookbackSummary } from "#/types/generated/LookbackSummary";
import type { OwnershipPoint } from "#/types/generated/OwnershipPoint";

/** One skill whose mastery rose between the two builds. */
export interface IMasteryStep {
    operatorId: string;
    /** 0-based skill index. */
    skill: number;
    from: number;
    to: number;
}

/** One module whose stage rose; `from` is 0 for a module unlocked in between. */
export interface IModuleStep {
    operatorId: string;
    moduleId: string;
    from: number;
    to: number;
}

/** What changed from one build to a later one. Only gains: a roster never loses an operator or a rank. */
export interface IBuildDiff {
    gained: string[];
    /** At Elite 2 now and not before, the newly obtained included. */
    elite2: string[];
    masteries: IMasteryStep[];
    modules: IModuleStep[];
}

/** `after` against `before`, every list in `after`'s operator order. */
export function diffBuilds(before: readonly LookbackOperator[], after: readonly LookbackOperator[]): IBuildDiff {
    const prior = new Map(before.map((op) => [op.operator_id, op]));
    const diff: IBuildDiff = { gained: [], elite2: [], masteries: [], modules: [] };
    for (const op of after) {
        const was = prior.get(op.operator_id);
        if (!was) diff.gained.push(op.operator_id);
        if (op.elite === 2 && (was?.elite ?? 0) < 2) diff.elite2.push(op.operator_id);
        op.masteries.forEach((to, skill) => {
            const from = was?.masteries[skill] ?? 0;
            if (to > from) diff.masteries.push({ operatorId: op.operator_id, skill, from, to });
        });
        for (const mod of op.modules) {
            const from = was?.modules.find((m) => m.module_id === mod.module_id)?.stage ?? 0;
            if (mod.stage > from) diff.modules.push({ operatorId: op.operator_id, moduleId: mod.module_id, from, to: mod.stage });
        }
    }
    return diff;
}

/** The summary counts the chart and the delta tiles offer, in display order. */
export const METRICS = ["operators", "elite2", "max_level", "mastery3", "modules", "skins"] as const satisfies readonly (keyof LookbackSummary)[];
export type Metric = (typeof METRICS)[number];

/** One chart point: a snapshot, or the live roster at the right edge. */
export interface ISeriesPoint {
    ts: number;
    value: number;
    label: string | null;
}

/**
 * `metric` over the snapshots, oldest first, with `live` (the current roster's
 * summary) appended at `liveTs` when given and later than the last snapshot.
 */
export function metricSeries(snapshots: readonly LookbackSnapshot[], metric: Metric, live?: { ts: number; summary: LookbackSummary } | null): ISeriesPoint[] {
    const points = snapshots.map((s) => ({ ts: new Date(s.period_start).getTime(), value: s.summary[metric], label: s.activity_name }));
    if (live && (points.length === 0 || live.ts > points[points.length - 1].ts)) points.push({ ts: live.ts, value: live.summary[metric], label: null });
    return points.filter((p) => !Number.isNaN(p.ts));
}

/** The reconstructed ownership line in milliseconds, for a time axis. */
export function ownershipSeries(points: readonly OwnershipPoint[]): ISeriesPoint[] {
    return points.map((p) => ({ ts: p.ts * 1000, value: p.count, label: null }));
}

/** `after - before` for each metric. */
export function summaryDelta(before: LookbackSummary, after: LookbackSummary): Record<Metric, number> {
    return Object.fromEntries(METRICS.map((m) => [m, after[m] - before[m]])) as Record<Metric, number>;
}
