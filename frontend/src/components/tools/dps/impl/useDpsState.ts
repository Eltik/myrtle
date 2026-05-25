import { migrateInstance } from "#/components/tools/shared/instance";
import { usePersistedReducer } from "#/components/tools/shared/usePersistedReducer";
import { dpsReducer, INITIAL_STATE } from "./state";
import type { IDpsState, XAxisKind, YMetric } from "./types";

const STORAGE_KEY = "dps-calculator-state-v1";

const VALID_X_AXES: ReadonlySet<XAxisKind> = new Set<XAxisKind>(["defense", "res"]);
const VALID_Y_METRICS: ReadonlySet<YMetric> = new Set<YMetric>(["skill_dps", "average_dps", "total_damage"]);

export function useDpsState() {
    return usePersistedReducer<IDpsState, Parameters<typeof dpsReducer>[1]>(STORAGE_KEY, dpsReducer, INITIAL_STATE, mergeWithDefaults, (state) => ({ type: "HYDRATE", state }));
}

/**
 * Validate stored state and fill missing fields from `INITIAL_STATE`. Tolerant
 * of forward changes; per-instance configs are normalised to the current shape.
 */
function mergeWithDefaults(value: unknown): IDpsState | null {
    if (!value || typeof value !== "object") return null;
    const v = value as Partial<IDpsState>;
    if (!Array.isArray(v.instances)) return null;
    const instances = v.instances.map(migrateInstance).filter((i): i is NonNullable<typeof i> => i !== null);
    return {
        instances,
        enemy: { ...INITIAL_STATE.enemy, ...(v.enemy ?? {}) },
        xAxis: VALID_X_AXES.has(v.xAxis as XAxisKind) ? (v.xAxis as XAxisKind) : INITIAL_STATE.xAxis,
        yMetric: VALID_Y_METRICS.has(v.yMetric as YMetric) ? (v.yMetric as YMetric) : INITIAL_STATE.yMetric,
        sweep: {
            defense: { ...INITIAL_STATE.sweep.defense, ...(v.sweep?.defense ?? {}) },
            res: { ...INITIAL_STATE.sweep.res, ...(v.sweep?.res ?? {}) },
        },
    };
}
