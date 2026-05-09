import { type Dispatch, useEffect, useReducer, useRef, useState } from "react";
import { dpsReducer, INITIAL_STATE } from "./state";
import type { DpsAction, IDpsState, XAxisKind, YMetric } from "./types";

const STORAGE_KEY = "dps-calculator-state-v1";

const VALID_X_AXES: ReadonlySet<XAxisKind> = new Set(["defense", "res"]);
const VALID_Y_METRICS: ReadonlySet<YMetric> = new Set(["skill_dps", "average_dps", "total_damage"]);

export interface IDpsStateHandle {
    state: IDpsState;
    dispatch: Dispatch<DpsAction>;
    /**
     * Increments once after hydration from `localStorage` completes. Children
     * with uncontrolled inputs (e.g. `<NumberField defaultValue>`) can include
     * this in their `key` so they remount with the hydrated value rather than
     * holding the initial-render placeholder.
     */
    hydrationToken: number;
}

/**
 * Wraps the DPS reducer with `localStorage` persistence so users keep their
 * configured operators and enemy setup across reloads. Hydration happens in an
 * effect on mount to avoid SSR mismatches; until it runs, the page renders
 * with `INITIAL_STATE` (empty calculator).
 */
export function useDpsState(): IDpsStateHandle {
    const [state, dispatch] = useReducer(dpsReducer, INITIAL_STATE);
    const hydratedRef = useRef(false);
    const [hydrationToken, setHydrationToken] = useState(0);

    useEffect(() => {
        if (typeof window === "undefined") return;
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (raw == null) {
            hydratedRef.current = true;
            setHydrationToken((t) => t + 1);
            return;
        }
        try {
            const parsed = JSON.parse(raw) as unknown;
            const merged = mergeWithDefaults(parsed);
            if (merged) dispatch({ type: "HYDRATE", state: merged });
        } catch {
            // Malformed stored value — ignore and fall back to the default state.
        }
        hydratedRef.current = true;
        setHydrationToken((t) => t + 1);
    }, []);

    useEffect(() => {
        if (typeof window === "undefined" || !hydratedRef.current) return;
        try {
            window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
        } catch {
            // Storage quota exhausted or other failure — silently ignore.
        }
    }, [state]);

    return { state, dispatch, hydrationToken };
}

/**
 * Validates the shape of a stored state and fills in any missing fields from
 * `INITIAL_STATE`. Returns `null` when the value is unusable (not an object,
 * `instances` not an array, etc.). Tolerant of forward changes in `IDpsState`
 * — new top-level fields fall back to defaults instead of dropping the save.
 */
function mergeWithDefaults(value: unknown): IDpsState | null {
    if (!value || typeof value !== "object") return null;
    const v = value as Partial<IDpsState>;
    if (!Array.isArray(v.instances)) return null;
    return {
        instances: v.instances,
        enemy: { ...INITIAL_STATE.enemy, ...(v.enemy ?? {}) },
        xAxis: VALID_X_AXES.has(v.xAxis as XAxisKind) ? (v.xAxis as XAxisKind) : INITIAL_STATE.xAxis,
        yMetric: VALID_Y_METRICS.has(v.yMetric as YMetric) ? (v.yMetric as YMetric) : INITIAL_STATE.yMetric,
        sweep: {
            defense: { ...INITIAL_STATE.sweep.defense, ...(v.sweep?.defense ?? {}) },
            res: { ...INITIAL_STATE.sweep.res, ...(v.sweep?.res ?? {}) },
        },
    };
}
