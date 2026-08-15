import { RosterPicker } from "frontend";

// The "Roster" tab of the randomizer settings sheet: search, bulk actions, and a
// rarity-grouped grid of operator chips. 4★ and below start collapsed, so the
// captured card shows the 6★ and 5★ grids expanded. All ids are verified against
// https://api.myrtle.moe/api/operators/index.

type Op = {
    id: string;
    name: string;
    rarity: number;
    profession: string;
    subProfessionId: string;
    position: string;
    race: string;
    hasOffensiveRecovery: boolean;
    hasDefensiveRecovery: boolean;
    allSkillsManual: boolean;
};

const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, position: string, race: string): Op => ({
    id,
    name,
    rarity,
    profession,
    subProfessionId,
    position,
    race,
    hasOffensiveRecovery: false,
    hasDefensiveRecovery: false,
    allSkillsManual: false,
});

const SIX: Op[] = [
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "Kuranta"),
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless", "MELEE", "Unknown"),
    op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "RANGED", "Caprinae"),
    op("char_249_mlyss", "Muelsyse", 6, "PIONEER", "tactician", "RANGED", "Elf"),
    op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "RANGED", "Sankta"),
    op("char_172_svrash", "SilverAsh", 6, "WARRIOR", "lord", "MELEE", "Feline"),
    op("char_293_thorns", "Thorns", 6, "WARRIOR", "lord", "MELEE", "Ægir"),
    op("char_202_demkni", "Saria", 6, "TANK", "guardian", "MELEE", "Vouivre"),
    op("char_179_cgbird", "Nightingale", 6, "MEDIC", "ringhealer", "RANGED", "Sarkaz"),
    op("char_222_bpipe", "Bagpipe", 6, "PIONEER", "charger", "MELEE", "Vouivre"),
    op("char_017_huang", "Blaze", 6, "WARRIOR", "centurion", "MELEE", "Feline"),
    op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter", "MELEE", "Sarkaz"),
];

const FIVE: Op[] = [
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "Lupo"),
    op("char_140_whitew", "Lappland", 5, "WARRIOR", "lord", "MELEE", "Lupo"),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "Liberi"),
    op("char_143_ghost", "Specter", 5, "WARRIOR", "centurion", "MELEE", "Undisclosed"),
    op("char_145_prove", "Provence", 5, "SNIPER", "closerange", "RANGED", "Lupo"),
    op("char_163_hpsts", "Vulcan", 5, "TANK", "unyield", "MELEE", "Forte"),
    op("char_106_franka", "Franka", 5, "WARRIOR", "fearless", "MELEE", "Vulpo"),
    op("char_107_liskam", "Liskarm", 5, "TANK", "shotprotector", "MELEE", "Vouivre"),
    op("char_171_bldsk", "Warfarin", 5, "MEDIC", "physician", "RANGED", "Sarkaz"),
    op("char_115_headbr", "Zima", 5, "PIONEER", "pioneer", "MELEE", "Ursus"),
];

const FOUR: Op[] = [
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "Durin"),
    op("char_150_snakek", "Cuora", 4, "TANK", "protector", "MELEE", "Petram"),
    op("char_187_ccheal", "Gavial", 4, "MEDIC", "physician", "RANGED", "Archosauria"),
    op("char_181_flower", "Perfumer", 4, "MEDIC", "ringhealer", "RANGED", "Vulpo"),
    op("char_199_yak", "Matterhorn", 4, "TANK", "protector", "MELEE", "Forte"),
    op("char_118_yuki", "Shirayuki", 4, "SNIPER", "aoesniper", "RANGED", "Anaty"),
    op("char_290_vigna", "Vigna", 4, "PIONEER", "charger", "MELEE", "Sarkaz"),
    op("char_109_fmout", "Gitano", 4, "CASTER", "splashcaster", "RANGED", "Elafia"),
];

const THREE: Op[] = [
    op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "RANGED", "Cautus"),
    op("char_123_fang", "Fang", 3, "PIONEER", "pioneer", "MELEE", "Kuranta"),
    op("char_212_ansel", "Ansel", 3, "MEDIC", "physician", "RANGED", "Cautus"),
    op("char_120_hibisc", "Hibiscus", 3, "MEDIC", "physician", "RANGED", "Sarkaz"),
    op("char_208_melan", "Melantha", 3, "WARRIOR", "fearless", "MELEE", "Feline"),
    op("char_122_beagle", "Beagle", 3, "TANK", "protector", "MELEE", "Perro"),
    op("char_192_falco", "Plume", 3, "PIONEER", "charger", "MELEE", "Liberi"),
    op("char_282_catap", "Catapult", 3, "SNIPER", "aoesniper", "RANGED", "Kuranta"),
];

const TWO: Op[] = [
    op("char_502_nblade", "Yato", 2, "PIONEER", "pioneer", "MELEE", "Oni"),
    op("char_500_noirc", "Noir Corne", 2, "TANK", "protector", "MELEE", "Oni"),
    op("char_501_durin", "Durin", 2, "CASTER", "corecaster", "RANGED", "Durin"),
    op("char_503_rang", "Rangers", 2, "SNIPER", "fastshot", "RANGED", "Savra"),
    op("char_009_12fce", "12F", 2, "CASTER", "splashcaster", "RANGED", "Savra"),
];

const ALL: Op[] = [...SIX, ...FIVE, ...FOUR, ...THREE, ...TWO];
const allIds = (ops: Op[]) => new Set(ops.map((o) => o.id));

const settings = (over: Record<string, unknown> = {}) => ({
    allowedClasses: ["PIONEER", "WARRIOR", "TANK", "SNIPER", "CASTER", "MEDIC", "SUPPORT", "SPECIAL"],
    allowedRarities: [6, 5, 4, 3, 2, 1],
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

const EMPTY_INDEX = { owned: new Set<string>(), e2: new Set<string>() };
const OWNED_INDEX = { owned: allIds([...SIX.slice(0, 9), ...FIVE, ...FOUR, ...THREE]), e2: allIds([...SIX.slice(0, 6), ...FIVE.slice(0, 4)]) };

const noop = () => {};

// Default state: nothing pruned, so All is disabled and there is no Reset.
export const EveryOperator = () => (
    <div className="max-w-md">
        <RosterPicker allOperators={ALL} hasProfile={false} isExplicit={false} onChange={noop} onReset={noop} rosterIndex={EMPTY_INDEX} selected={allIds(ALL)} settings={settings()} visibleOperators={ALL} />
    </div>
);

// Class + rarity filters are on in the Operators tab, so the panel explains what
// is hidden and the counts split into "in view" and "total in roster".
export const FilteredByClass = () => {
    const visible = [...SIX, ...FIVE].filter((o) => ["WARRIOR", "SNIPER", "CASTER"].includes(o.profession));
    return (
        <div className="max-w-md">
            <RosterPicker
                allOperators={ALL}
                hasProfile={false}
                isExplicit
                onChange={noop}
                onReset={noop}
                rosterIndex={EMPTY_INDEX}
                selected={allIds([...visible.slice(0, 5), ...FOUR])}
                settings={settings({ allowedClasses: ["WARRIOR", "SNIPER", "CASTER"], allowedRarities: [6, 5] })}
                visibleOperators={visible}
            />
        </div>
    );
};

// Signed in with an imported roster: "Sync profile" offers to replace the
// selection with the 33 operators the account actually owns.
export const ProfileLinked = () => (
    <div className="max-w-md">
        <RosterPicker allOperators={ALL} hasProfile isExplicit onChange={noop} onReset={noop} rosterIndex={OWNED_INDEX} selected={allIds([...SIX.slice(0, 8), ...FIVE.slice(0, 6)])} settings={settings()} visibleOperators={ALL} />
    </div>
);

// Filters that leave nothing behind — 1★ Supporters, of which there are none in
// the visible pool.
export const NothingVisible = () => (
    <div className="max-w-md">
        <RosterPicker allOperators={ALL} hasProfile={false} isExplicit onChange={noop} onReset={noop} rosterIndex={EMPTY_INDEX} selected={allIds(ALL)} settings={settings({ allowedClasses: ["SUPPORT"], allowedRarities: [1] })} visibleOperators={[]} />
    </div>
);
