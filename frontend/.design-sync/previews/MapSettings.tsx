import { MapSettings, TileLegend } from "frontend";

const DEFAULTS = { showRoutes: true, showEnemyIcons: false, showMovement: false, showTimers: true, walkingChibis: true };
const noop = () => {};

/** What the map opens with: routes and timers on, chibis walking, icons off. */
export const Defaults = () => (
    <div className="max-w-3xl">
        <MapSettings settings={DEFAULTS} onChange={noop} />
    </div>
);

/** Toggling 3D off swaps walking chibis for animated route icons. */
export const TwoDimensionalView = () => (
    <div className="max-w-3xl">
        <MapSettings settings={{ showRoutes: true, showEnemyIcons: true, showMovement: true, showTimers: true, walkingChibis: false }} onChange={noop} />
    </div>
);

/** Everything off — a bare board with no overlays. */
export const AllOff = () => (
    <div className="max-w-3xl">
        <MapSettings settings={{ showRoutes: false, showEnemyIcons: false, showMovement: false, showTimers: false, walkingChibis: false }} onChange={noop} />
    </div>
);

/** Where the panel lives: under the board, beneath the tile legend. */
export const BelowTheBoard = () => (
    <div className="flex max-w-3xl flex-col gap-3">
        <TileLegend />
        <MapSettings settings={DEFAULTS} onChange={noop} />
    </div>
);
