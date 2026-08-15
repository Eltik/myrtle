import { ResultsList } from "frontend";

// The results column of the recruitment calculator: one ResultCard per tag
// combination, ranked by 5★ chance then by the pool's worst outcome. It owns
// both empty states — nothing picked yet, and picked-but-no-match.
//
// The combinations below are the real output of `calculateResults` for the tag
// set "Senior Operator + Medic + Healing".

const op = (id: string, name: string, rarity: number, profession: string, position: string, tagList: string[]) => ({ id, name, rarity, profession, position, tagList });

const PTILOPSIS = op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "RANGED", ["Ranged", "Medic", "Senior Operator", "Healing", "Support"]);
const SILENCE = op("char_108_silent", "Silence", 5, "MEDIC", "RANGED", ["Ranged", "Medic", "Senior Operator", "Healing"]);
const WARFARIN = op("char_171_bldsk", "Warfarin", 5, "MEDIC", "RANGED", ["Ranged", "Medic", "Senior Operator", "Healing", "Support"]);
const WHISPERAIN = op("char_436_whispr", "Whisperain", 5, "MEDIC", "RANGED", ["Ranged", "Medic", "Senior Operator", "Healing"]);

const RESULTS = [
    { tags: [14, 4], tagNames: ["Senior Operator", "Medic"], guaranteedRarity: 5, maxRarity: 5, fiveStarCount: 4, operators: [PTILOPSIS, SILENCE, WARFARIN, WHISPERAIN] },
    { tags: [14, 4, 15], tagNames: ["Senior Operator", "Medic", "Healing"], guaranteedRarity: 5, maxRarity: 5, fiveStarCount: 4, operators: [PTILOPSIS, SILENCE, WARFARIN, WHISPERAIN] },
    {
        tags: [14, 15],
        tagNames: ["Senior Operator", "Healing"],
        guaranteedRarity: 5,
        maxRarity: 5,
        fiveStarCount: 7,
        operators: [
            op("char_226_hmau", "Hung", 5, "TANK", "MELEE", ["Melee", "Defender", "Senior Operator", "Defense", "Healing"]),
            op("char_148_nearl", "Nearl", 5, "TANK", "MELEE", ["Melee", "Defender", "Senior Operator", "Defense", "Healing"]),
            op("char_164_nightm", "Nightmare", 5, "CASTER", "RANGED", ["Ranged", "Caster", "Senior Operator", "DPS", "Healing", "Slow"]),
            PTILOPSIS,
            SILENCE,
            WARFARIN,
            WHISPERAIN,
        ],
    },
    {
        tags: [4, 15],
        tagNames: ["Medic", "Healing"],
        guaranteedRarity: 1,
        maxRarity: 5,
        fiveStarCount: 4,
        operators: [
            PTILOPSIS,
            SILENCE,
            WARFARIN,
            WHISPERAIN,
            op("char_187_ccheal", "Gavial", 4, "MEDIC", "RANGED", ["Ranged", "Medic", "Healing"]),
            op("char_181_flower", "Perfumer", 4, "MEDIC", "RANGED", ["Ranged", "Medic", "Healing"]),
            op("char_212_ansel", "Ansel", 3, "MEDIC", "RANGED", ["Ranged", "Medic", "Healing"]),
            op("char_120_hibisc", "Hibiscus", 3, "MEDIC", "RANGED", ["Ranged", "Medic", "Healing"]),
            op("char_285_medic2", "Lancet-2", 1, "MEDIC", "RANGED", ["Ranged", "Medic", "Robot", "Healing"]),
        ],
    },
];

export const Combinations = () => <ResultsList hasSelection results={RESULTS} />;

// The landing state before any tag is picked.
export const NoTagsPicked = () => <ResultsList hasSelection={false} results={[]} />;

// Tags are picked but the inclusion options filtered every candidate out.
export const NoMatches = () => <ResultsList hasSelection results={[]} />;
