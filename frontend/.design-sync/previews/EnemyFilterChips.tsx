import { EnemyFilterChips } from "frontend";

// The chip rail above the enemy grid. Every chip is a controlled toggle, so a
// static preview just hands it a filter state and no-op setters.
const noop = () => undefined;

const emptyFilters = {
    q: "",
    levels: [] as string[],
    damageTypes: [] as string[],
    attackTypes: [] as string[],
    races: [] as string[],
    appearsIn: [] as string[],
    sortBy: "index",
    sortOrder: "asc",
};

// `raceData` from /static/enemies, sorted by display name the way the list does.
const RACES = [
    { id: "animated", label: "Apparition" },
    { id: "originiumartscraft", label: "Arts Creation" },
    { id: "collapsal", label: "Collapsal" },
    { id: "drone", label: "Drone" },
    { id: "infection", label: "Infected Creature" },
    { id: "machine", label: "Machina" },
    { id: "origen", label: "Originium Creation" },
    { id: "mutant", label: "Possessed" },
    { id: "sarkaz", label: "Sarkaz" },
    { id: "seamonster", label: "Sea Monster" },
    { id: "wildanimal", label: "Wild Beast" },
];

export const NoFiltersActive = () => <EnemyFilterChips filters={emptyFilters} setLevels={noop} setDamageTypes={noop} setAttackTypes={noop} setRaces={noop} races={[]} />;

export const ThreatAndDamageSelected = () => <EnemyFilterChips filters={{ ...emptyFilters, levels: ["ELITE", "BOSS"], damageTypes: ["MAGIC"], attackTypes: ["RANGED"] }} setLevels={noop} setDamageTypes={noop} setAttackTypes={noop} setRaces={noop} races={[]} />;

export const WithRaceOptions = () => <EnemyFilterChips filters={emptyFilters} setLevels={noop} setDamageTypes={noop} setAttackTypes={noop} setRaces={noop} races={RACES} />;

export const RaceSelected = () => <EnemyFilterChips filters={{ ...emptyFilters, levels: ["BOSS"], races: ["sarkaz", "collapsal"] }} setLevels={noop} setDamageTypes={noop} setAttackTypes={noop} setRaces={noop} races={RACES} />;
