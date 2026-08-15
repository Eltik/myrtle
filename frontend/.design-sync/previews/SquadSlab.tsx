import { SquadSlab } from "frontend";

// Slot 02 of a randomizer roll. Operator ids are checked against
// https://api.myrtle.moe/api/operators/index, so the tiles load real avatars and
// the rarity hairline under each tile picks up `--rarity-<n>`.

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

const MLYNAR = op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "Kuranta");
const EYJA = op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "RANGED", "Caprinae");
const SARIA = op("char_202_demkni", "Saria", 6, "TANK", "guardian", "MELEE", "Vouivre");
const TEXAS = op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "Lupo");
const LAPPLAND = op("char_140_whitew", "Lappland", 5, "WARRIOR", "lord", "MELEE", "Lupo");
const PTILOPSIS = op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "Liberi");
const MYRTLE = op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "Durin");
const CUORA = op("char_150_snakek", "Cuora", 4, "TANK", "protector", "MELEE", "Petram");
const GAVIAL = op("char_187_ccheal", "Gavial", 4, "MEDIC", "physician", "RANGED", "Archosauria");
const KROOS = op("char_124_kroos", "Kroos", 3, "SNIPER", "fastshot", "RANGED", "Cautus");
const MELANTHA = op("char_208_melan", "Melantha", 3, "WARRIOR", "fearless", "MELEE", "Feline");
const YATO = op("char_502_nblade", "Yato", 2, "PIONEER", "pioneer", "MELEE", "Oni");

const noop = () => {};

export const FullSquad = () => <SquadSlab onReroll={noop} operators={[MLYNAR, EYJA, SARIA, TEXAS, LAPPLAND, PTILOPSIS, MYRTLE, CUORA, GAVIAL, KROOS, MELANTHA, YATO]} squadSize={12} />;

// A "Ranged only" modifier cut the pool, so the draw came up short of the
// configured squad size — the counter reads 5/12.
export const PartialDraw = () => <SquadSlab onReroll={noop} operators={[EYJA, PTILOPSIS, GAVIAL, KROOS, op("char_278_orchid", "Orchid", 3, "SUPPORT", "slower", "RANGED", "Liberi")]} squadSize={12} />;

// A six-slot squad size, all six filled.
export const SmallSquad = () => <SquadSlab onReroll={noop} operators={[MLYNAR, SARIA, TEXAS, PTILOPSIS, MYRTLE, KROOS]} squadSize={6} />;

// Filters excluded every operator: the grid collapses to its inline hint.
export const NoMatchingOperators = () => <SquadSlab onReroll={noop} operators={[]} squadSize={12} />;
