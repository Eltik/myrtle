import { ResultCard } from "frontend";

// One tag combination in the recruitment results. The header lists the combo's
// tags and, at 5★+, the guaranteed-rarity badge; the body is the operator pool,
// each row tinted by rarity. Hovering a row opens its tag list — interaction
// only, not captured.
//
// Every combination below is the real output of `calculateResults` over
// /api/static/gacha + /api/static/operators, so the pools are exact.

const seniorMedic = {
    tags: [14, 4],
    tagNames: ["Senior Operator", "Medic"],
    guaranteedRarity: 5,
    maxRarity: 5,
    fiveStarCount: 4,
    operators: [
        { id: "char_128_plosis", name: "Ptilopsis", rarity: 5, profession: "MEDIC", position: "RANGED", tagList: ["Ranged", "Medic", "Senior Operator", "Healing", "Support"] },
        { id: "char_108_silent", name: "Silence", rarity: 5, profession: "MEDIC", position: "RANGED", tagList: ["Ranged", "Medic", "Senior Operator", "Healing"] },
        { id: "char_171_bldsk", name: "Warfarin", rarity: 5, profession: "MEDIC", position: "RANGED", tagList: ["Ranged", "Medic", "Senior Operator", "Healing", "Support"] },
        { id: "char_436_whispr", name: "Whisperain", rarity: 5, profession: "MEDIC", position: "RANGED", tagList: ["Ranged", "Medic", "Senior Operator", "Healing"] },
    ],
};

const topSniper = {
    tags: [11, 2],
    tagNames: ["Top Operator", "Sniper"],
    guaranteedRarity: 6,
    maxRarity: 6,
    fiveStarCount: 4,
    operators: [
        { id: "char_332_archet", name: "Archetto", rarity: 6, profession: "SNIPER", position: "RANGED", tagList: ["Ranged", "Sniper", "Top Operator", "DPS"] },
        { id: "char_103_angel", name: "Exusiai", rarity: 6, profession: "SNIPER", position: "RANGED", tagList: ["Ranged", "Sniper", "Top Operator", "DPS"] },
        { id: "char_197_poca", name: "Rosa", rarity: 6, profession: "SNIPER", position: "RANGED", tagList: ["Ranged", "Sniper", "Top Operator", "DPS", "Crowd-Control"] },
        { id: "char_340_shwaz", name: "Schwarz", rarity: 6, profession: "SNIPER", position: "RANGED", tagList: ["Ranged", "Sniper", "Top Operator", "DPS"] },
    ],
};

const slowRanged = {
    tags: [23, 10],
    tagNames: ["Slow", "Ranged"],
    guaranteedRarity: 3,
    maxRarity: 5,
    fiveStarCount: 4,
    operators: [
        { id: "char_218_cuttle", name: "Andreana", rarity: 5, profession: "SNIPER", position: "RANGED", tagList: ["Ranged", "Sniper", "Senior Operator", "DPS", "Slow"] },
        { id: "char_326_glacus", name: "Glaucus", rarity: 5, profession: "SUPPORT", position: "RANGED", tagList: ["Ranged", "Supporter", "Senior Operator", "Slow", "Crowd-Control"] },
        { id: "char_195_glassb", name: "Istina", rarity: 5, profession: "SUPPORT", position: "RANGED", tagList: ["Ranged", "Supporter", "Senior Operator", "Slow", "DPS"] },
        { id: "char_164_nightm", name: "Nightmare", rarity: 5, profession: "CASTER", position: "RANGED", tagList: ["Ranged", "Caster", "Senior Operator", "DPS", "Healing", "Slow"] },
        { id: "char_302_glaze", name: "Ambriel", rarity: 4, profession: "SNIPER", position: "RANGED", tagList: ["Ranged", "Sniper", "DPS", "Slow"] },
        { id: "char_183_skgoat", name: "Earthspirit", rarity: 4, profession: "SUPPORT", position: "RANGED", tagList: ["Ranged", "Supporter", "Slow"] },
        { id: "char_253_greyy", name: "Greyy", rarity: 4, profession: "CASTER", position: "RANGED", tagList: ["Ranged", "Caster", "AoE", "Slow"] },
        { id: "char_133_mm", name: "May", rarity: 4, profession: "SNIPER", position: "RANGED", tagList: ["Ranged", "Sniper", "DPS", "Slow"] },
        { id: "char_258_podego", name: "Podenco", rarity: 4, profession: "SUPPORT", position: "RANGED", tagList: ["Ranged", "Supporter", "Slow", "Healing"] },
        { id: "char_118_yuki", name: "Shirayuki", rarity: 4, profession: "SNIPER", position: "RANGED", tagList: ["Ranged", "Sniper", "AoE", "Slow"] },
        { id: "char_278_orchid", name: "Orchid", rarity: 3, profession: "SUPPORT", position: "RANGED", tagList: ["Ranged", "Supporter", "Slow"] },
    ],
};

const robotOnly = {
    tags: [28],
    tagNames: ["Robot"],
    guaranteedRarity: 1,
    maxRarity: 1,
    fiveStarCount: 0,
    operators: [
        { id: "char_286_cast3", name: "Castle-3", rarity: 1, profession: "WARRIOR", position: "MELEE", tagList: ["Melee", "Guard", "Robot", "Support"] },
        { id: "char_4188_confes", name: "CONFESS-47", rarity: 1, profession: "PIONEER", position: "MELEE", tagList: ["Melee", "Vanguard", "Robot", "Crowd-Control"] },
        { id: "char_4093_frston", name: "Friston-3", rarity: 1, profession: "TANK", position: "MELEE", tagList: ["Melee", "Defender", "Robot", "Defense"] },
        { id: "char_285_medic2", name: "Lancet-2", rarity: 1, profession: "MEDIC", position: "RANGED", tagList: ["Ranged", "Medic", "Robot", "Healing"] },
        { id: "char_4136_phonor", name: "PhonoR-0", rarity: 1, profession: "SUPPORT", position: "RANGED", tagList: ["Ranged", "Supporter", "Robot", "Elemental"] },
        { id: "char_376_therex", name: "THRM-EX", rarity: 1, profession: "SPECIAL", position: "MELEE", tagList: ["Melee", "Specialist", "Robot", "Nuker"] },
    ],
};

export const GuaranteedSixStar = () => <ResultCard result={topSniper} />;

export const GuaranteedFiveStar = () => <ResultCard result={seniorMedic} />;

// No qualification tag in the combo, so there is no guarantee badge and the pool
// runs all the way down to a 3★.
export const MixedRarityPool = () => <ResultCard result={slowRanged} />;

// A Robot lock: six 1★s, all tinted zinc, no badge.
export const RobotLock = () => <ResultCard result={robotOnly} />;
