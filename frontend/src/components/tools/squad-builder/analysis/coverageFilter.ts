/**
 * The Coverage panel's filter tabs as a pure function over the response's
 * groups and the picked stage's threats.
 */

import type { CoverageGroup } from "#/types/generated/CoverageGroup";
import type { CoverageRowKey } from "#/types/generated/CoverageRowKey";
import type { IStageContext, IStageThreat } from "../model";

export type CoverageFilter = "all" | "covered" | "missing" | "stage";

/** The stage threats each coverage row answers; empty without a stage. */
export function neededRows(stage: Pick<IStageContext, "threats"> | null): Map<CoverageRowKey, IStageThreat[]> {
    const out = new Map<CoverageRowKey, IStageThreat[]>();
    if (!stage) return out;
    for (const threat of stage.threats) {
        for (const row of threat.rows) {
            const list = out.get(row);
            if (list) list.push(threat);
            else out.set(row, [threat]);
        }
    }
    return out;
}

/**
 * The groups with only the rows the filter keeps, groups left empty dropped.
 * `stage` without a stage keeps nothing, the same as a stage that names no row.
 */
export function filterCoverage(groups: ReadonlyArray<CoverageGroup>, filter: CoverageFilter, stage: Pick<IStageContext, "threats"> | null): CoverageGroup[] {
    const needed = neededRows(stage);
    const keep = (row: CoverageGroup["rows"][number]): boolean => {
        switch (filter) {
            case "all":
                return true;
            case "covered":
                return row.ops.length > 0;
            case "missing":
                return row.ops.length === 0;
            case "stage":
                return needed.has(row.key);
        }
    };
    const out: CoverageGroup[] = [];
    for (const g of groups) {
        const rows = g.rows.filter(keep);
        if (rows.length > 0) out.push({ key: g.key, rows });
    }
    return out;
}
