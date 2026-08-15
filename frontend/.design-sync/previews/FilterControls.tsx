import { FilterControls } from "frontend";

// The filter form as it sits in the birthday sidebar (a ~300px rail).
// `filters` is a controlled value object; rarities/professions/nations are Sets.
const NATIONS: [string, string][] = [
    ["columbia", "Columbia"],
    ["higashi", "Higashi"],
    ["kazimierz", "Kazimierz"],
    ["kjerag", "Kjerag"],
    ["laterano", "Laterano"],
    ["lungmen", "Lungmen"],
    ["rhodes", "Rhodes Island"],
    ["sargon", "Sargon"],
    ["siracusa", "Siracusa"],
    ["ursus", "Ursus"],
    ["victoria", "Victoria"],
    ["yan", "Yan"],
];

const filters = (query: string, rarities: number[], professions: string[], nations: string[]) => ({
    query,
    rarities: new Set(rarities),
    professions: new Set(professions),
    nations: new Set(nations),
});

const noop = () => {};

export const NoFiltersApplied = () => (
    <div className="w-80">
        <FilterControls filters={filters("", [], [], [])} onChange={noop} nations={NATIONS} />
    </div>
);

export const RarityAndClassSelected = () => (
    <div className="w-80">
        <FilterControls filters={filters("", [6, 5], ["MEDIC", "SUPPORT"], [])} onChange={noop} nations={NATIONS} />
    </div>
);

export const SearchAndNation = () => (
    <div className="w-80">
        <FilterControls filters={filters("silence", [], [], ["columbia", "victoria"])} onChange={noop} nations={NATIONS} />
    </div>
);
