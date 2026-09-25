import { usePersistedReducer } from "#/components/tools/shared/usePersistedReducer";
import type { ICalculatorSettings, IRosterViewOptions, OperatorSortMode, ResultLayout } from "./types";

const STORAGE_KEY = "recruitment-calculator-options-v1";

export interface IRecruitOptions {
    settings: ICalculatorSettings;
    rosterView: IRosterViewOptions;
    layout: ResultLayout;
}

export const DEFAULT_SETTINGS: ICalculatorSettings = {
    includeRobots: true,
    includeTwoStars: true,
    includeThreeStars: true,
    operatorSortMode: "rarity-desc",
};

// Both overlays on by default: a signed-in reader opened the tool to see them.
const DEFAULT_ROSTER_VIEW: IRosterViewOptions = { showPotentials: true, showNextUpgrade: true };

const DEFAULT_LAYOUT: ResultLayout = "compact";

const INITIAL: IRecruitOptions = { settings: DEFAULT_SETTINGS, rosterView: DEFAULT_ROSTER_VIEW, layout: DEFAULT_LAYOUT };

type Action = { type: "HYDRATE"; state: IRecruitOptions } | { type: "SETTINGS"; patch: Partial<ICalculatorSettings> } | { type: "ROSTER_VIEW"; patch: Partial<IRosterViewOptions> } | { type: "LAYOUT"; layout: ResultLayout };

function reducer(state: IRecruitOptions, action: Action): IRecruitOptions {
    switch (action.type) {
        case "HYDRATE":
            return action.state;
        case "SETTINGS":
            return { ...state, settings: { ...state.settings, ...action.patch } };
        case "ROSTER_VIEW":
            return { ...state, rosterView: { ...state.rosterView, ...action.patch } };
        case "LAYOUT":
            return { ...state, layout: action.layout };
    }
}

const SORT_MODES: ReadonlySet<OperatorSortMode> = new Set(["rarity-desc", "common-first", "potential-asc"]);
const LAYOUTS: ReadonlySet<ResultLayout> = new Set(["compact", "detailed"]);

function pickBool(raw: Record<string, unknown> | undefined, key: string, fallback: boolean): boolean {
    const v = raw?.[key];
    return typeof v === "boolean" ? v : fallback;
}

/** Field by field over the defaults, so a stored blob from an older shape keeps what still parses and drops the rest. */
function merge(raw: unknown): IRecruitOptions | null {
    if (!raw || typeof raw !== "object") return null;
    const r = raw as { settings?: Record<string, unknown>; rosterView?: Record<string, unknown>; layout?: unknown };
    const sort = r.settings?.operatorSortMode;
    const layout = r.layout;
    return {
        settings: {
            includeRobots: pickBool(r.settings, "includeRobots", DEFAULT_SETTINGS.includeRobots),
            includeTwoStars: pickBool(r.settings, "includeTwoStars", DEFAULT_SETTINGS.includeTwoStars),
            includeThreeStars: pickBool(r.settings, "includeThreeStars", DEFAULT_SETTINGS.includeThreeStars),
            operatorSortMode: typeof sort === "string" && SORT_MODES.has(sort as OperatorSortMode) ? (sort as OperatorSortMode) : DEFAULT_SETTINGS.operatorSortMode,
        },
        rosterView: {
            showPotentials: pickBool(r.rosterView, "showPotentials", DEFAULT_ROSTER_VIEW.showPotentials),
            showNextUpgrade: pickBool(r.rosterView, "showNextUpgrade", DEFAULT_ROSTER_VIEW.showNextUpgrade),
        },
        layout: typeof layout === "string" && LAYOUTS.has(layout as ResultLayout) ? (layout as ResultLayout) : DEFAULT_LAYOUT,
    };
}

/** The options panel's state, kept in `localStorage` per browser like the DPS/HPS tools' state. */
export function useRecruitOptions() {
    return usePersistedReducer<IRecruitOptions, Action>(STORAGE_KEY, reducer, INITIAL, merge, (state) => ({ type: "HYDRATE", state }));
}
