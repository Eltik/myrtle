import { migrateInstance } from "#/components/tools/shared/instance";
import { usePersistedReducer } from "#/components/tools/shared/usePersistedReducer";
import { hpsReducer, INITIAL_STATE } from "./state";
import type { HpsXAxis, HpsYMetric, IHpsState } from "./types";

const STORAGE_KEY = "hps-calculator-state-v1";

const VALID_X_AXES: ReadonlySet<HpsXAxis> = new Set<HpsXAxis>(["targets", "atk", "aspd"]);
const VALID_Y_METRICS: ReadonlySet<HpsYMetric> = new Set<HpsYMetric>(["skill_hps", "base_hps", "avg_hps"]);

export function useHpsState() {
    return usePersistedReducer<IHpsState, Parameters<typeof hpsReducer>[1]>(STORAGE_KEY, hpsReducer, INITIAL_STATE, mergeWithDefaults, (state) => ({ type: "HYDRATE", state }));
}

function mergeWithDefaults(value: unknown): IHpsState | null {
    if (!value || typeof value !== "object") return null;
    const v = value as Partial<IHpsState>;
    if (!Array.isArray(v.instances)) return null;
    const instances = v.instances.map(migrateInstance).filter((i): i is NonNullable<typeof i> => i !== null);
    return {
        instances,
        buffs: {
            ...INITIAL_STATE.buffs,
            ...(v.buffs ?? {}),
            buffs: { ...INITIAL_STATE.buffs.buffs, ...(v.buffs?.buffs ?? {}) },
        },
        xAxis: VALID_X_AXES.has(v.xAxis as HpsXAxis) ? (v.xAxis as HpsXAxis) : INITIAL_STATE.xAxis,
        yMetric: VALID_Y_METRICS.has(v.yMetric as HpsYMetric) ? (v.yMetric as HpsYMetric) : INITIAL_STATE.yMetric,
        sweep: {
            targets: { ...INITIAL_STATE.sweep.targets, ...(v.sweep?.targets ?? {}) },
            atk: { ...INITIAL_STATE.sweep.atk, ...(v.sweep?.atk ?? {}) },
            aspd: { ...INITIAL_STATE.sweep.aspd, ...(v.sweep?.aspd ?? {}) },
        },
    };
}
