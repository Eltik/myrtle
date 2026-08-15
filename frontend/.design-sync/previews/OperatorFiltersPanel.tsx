import { OperatorFiltersPanel } from "frontend";

// The "Operators" tab of the randomizer settings sheet: class and rarity toggle
// groups, the squad-size slider, and four rule switches. The last two rules are
// profile-gated — `hasProfile={false}` locks them shut with a padlock.

const ALL_CLASSES = ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"];
const ALL_RARITIES = [6, 5, 4, 3, 2, 1];

const settings = (over: Record<string, unknown> = {}) => ({
    allowedClasses: ALL_CLASSES,
    allowedRarities: ALL_RARITIES,
    allowedZoneTypes: ["MAINLINE", "ACTIVITY"],
    squadSize: 12,
    allowDuplicates: false,
    hideUnplayableOperators: true,
    onlyOwnedOperators: false,
    onlyCompletedStages: false,
    onlyAvailableStages: true,
    onlyE2Operators: false,
    deselectedStageIds: [],
    ...over,
});

const noop = () => {};

export const Defaults = () => (
    <div className="max-w-md">
        <OperatorFiltersPanel hasProfile onChange={noop} settings={settings()} />
    </div>
);

// A four-class, high-rarity draw with a six-operator squad.
export const NarrowedDraw = () => (
    <div className="max-w-md">
        <OperatorFiltersPanel hasProfile onChange={noop} settings={settings({ allowedClasses: ["PIONEER", "WARRIOR", "SNIPER", "MEDIC"], allowedRarities: [6, 5], squadSize: 6 })} />
    </div>
);

// Signed in with the roster synced: both profile switches are on.
export const OwnedE2Only = () => (
    <div className="max-w-md">
        <OperatorFiltersPanel hasProfile onChange={noop} settings={settings({ onlyOwnedOperators: true, onlyE2Operators: true, allowDuplicates: true, squadSize: 8 })} />
    </div>
);

// Signed out: the two roster rules render locked and forced off.
export const SignedOut = () => (
    <div className="max-w-md">
        <OperatorFiltersPanel hasProfile={false} onChange={noop} settings={settings()} />
    </div>
);
