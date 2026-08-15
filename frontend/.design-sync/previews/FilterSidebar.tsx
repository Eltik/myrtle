import { FilterSidebar } from "frontend";

// The persistent filter rail from the birthday page. Its root is `hidden lg:flex`,
// so at the 900px capture viewport it collapses to nothing. `DesktopRail` is an
// unlayered CSS rule (Tailwind utilities live in @layer utilities, so it wins)
// that pins the rail to its >=1024px appearance - the only state it ever has.
const DesktopRail = () => <style>{".rail-stage aside{display:flex}"}</style>;
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

export const AllOperators = () => (
    <div className="rail-stage w-80">
        <DesktopRail />
        <FilterSidebar filters={filters("", [], [], [])} onChange={noop} nations={NATIONS} matched={382} total={382} onReset={noop} />
    </div>
);

export const FiltersApplied = () => (
    <div className="rail-stage w-80">
        <DesktopRail />
        <FilterSidebar filters={filters("", [6, 5], ["MEDIC", "SUPPORT"], ["columbia"])} onChange={noop} nations={NATIONS} matched={14} total={382} onReset={noop} />
    </div>
);

export const NoMatches = () => (
    <div className="rail-stage w-80">
        <DesktopRail />
        <FilterSidebar filters={filters("mlynar", [3], [], [])} onChange={noop} nations={NATIONS} matched={0} total={382} onReset={noop} />
    </div>
);
