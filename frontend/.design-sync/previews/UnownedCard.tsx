import { UnownedCard } from "frontend";

// `IUnownedEntry` — what the Roster tab builds for an operator in the index
// that the Doctor doesn't have. There is no roster row, so everything past the
// index metadata is blank by design.
const unowned = (operator_id: string, name: string, rarity: number, profession: string, subProfessionId: string) => ({
    isOwned: false,
    operator_id,
    name,
    rarity,
    meta: { id: operator_id, name, appellation: name, rarity, profession, subProfessionId, position: "MELEE", tagList: [], nationId: "rhodes", isNotObtainable: false, groupId: null, teamId: null, artists: [], portrait: null, gender: "", race: "", placeOfBirth: "" },
    static: null,
});

const texasAlter = unowned("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor");
const dusk = unowned("char_2015_dusk", "Dusk", 6, "CASTER", "splashcaster");
const heidi = unowned("char_4045_heidi", "Heidi", 5, "SUPPORT", "bard");

// Grid card: full art, desaturated, with a "Not Owned" ribbon and every stat
// row stubbed to "--".
export const DetailedUnowned = () => (
    <div className="mx-auto max-w-sm">
        <UnownedCard entry={texasAlter} viewMode="detailed" />
    </div>
);

// Compact card: the same greyed treatment at roster-grid density.
export const CompactUnowned = () => (
    <div className="pt-2 pl-2">
        <UnownedCard entry={dusk} viewMode="compact" />
    </div>
);

// How a run of unowned operators reads inside the compact roster grid.
export const CompactRow = () => (
    <div className="grid grid-cols-[repeat(auto-fill,minmax(6.5rem,1fr))] gap-3 pt-2 sm:grid-cols-[repeat(auto-fill,minmax(9rem,1fr))]">
        <UnownedCard entry={texasAlter} viewMode="compact" />
        <UnownedCard entry={dusk} viewMode="compact" />
        <UnownedCard entry={heidi} viewMode="compact" />
    </div>
);
