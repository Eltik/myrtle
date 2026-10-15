import { CalculatorOptionsPanel } from "frontend";

// The "Options" card of the recruitment calculator: three inclusion switches,
// the operator sort order, the result layout, and the two roster overlays
// (potentials, next upgrade). Signed out, the roster rows render disabled with a
// sign-in hint and the roster-backed "potential" sort is not offered.

const noop = () => {};

const settings = (over: Record<string, unknown> = {}) => ({
    includeRobots: true,
    includeTwoStars: true,
    includeThreeStars: true,
    operatorSortMode: "rarity-desc" as const,
    ...over,
});

const handlers = { onChangeSettings: noop, onChangeRosterView: noop, onChangeLayout: noop };

/** Signed out, defaults: roster overlays disabled with the hint. */
export const Defaults = () => (
    <div className="max-w-sm">
        <CalculatorOptionsPanel settings={settings()} rosterView={{ showPotentials: false, showNextUpgrade: false }} rosterAvailable={false} layout="compact" {...handlers} />
    </div>
);

// Chasing 4★+ only: the low-rarity fillers are switched off.
export const HighRarityOnly = () => (
    <div className="max-w-sm">
        <CalculatorOptionsPanel settings={settings({ includeRobots: false, includeTwoStars: false, includeThreeStars: false })} rosterView={{ showPotentials: false, showNextUpgrade: false }} rosterAvailable={false} layout="compact" {...handlers} />
    </div>
);

// Robot-hunting: 2★/3★ stay in, and results lead with the most common pulls.
export const CommonFirst = () => (
    <div className="max-w-sm">
        <CalculatorOptionsPanel settings={settings({ operatorSortMode: "common-first" })} rosterView={{ showPotentials: false, showNextUpgrade: false }} rosterAvailable={false} layout="detailed" {...handlers} />
    </div>
);

/** Signed in: the overlays live, sorted by lowest potential first, detailed cards. */
export const SignedInRoster = () => (
    <div className="max-w-sm">
        <CalculatorOptionsPanel settings={settings({ operatorSortMode: "potential-asc" })} rosterView={{ showPotentials: true, showNextUpgrade: true }} rosterAvailable layout="detailed" {...handlers} />
    </div>
);
