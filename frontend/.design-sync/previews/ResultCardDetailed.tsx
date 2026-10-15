import { ResultCardDetailed } from "frontend";

// The recruitment calculator's "Detailed" layout: one card per tag
// combination, the tags and the guaranteed floor in the header, each operator
// a wide rarity-tinted row (portrait, name, stars, class, and the next
// potential's gain when a signed-in roster asks for it). The tags each
// operator carries open on hover; interaction only, not captured.
//
// Pools are the real output of `calculateResults` over the EN gacha table;
// potentials are read off the EN character table.

type Potential = { kind: "stat"; attribute: string; value: number } | { kind: "talent"; index: number; of: number };
const cost = (value: number): Potential => ({ kind: "stat", attribute: "COST", value });
const respawn = (value: number): Potential => ({ kind: "stat", attribute: "RESPAWN_TIME", value });
const atk = (value: number): Potential => ({ kind: "stat", attribute: "ATK", value });
const talent = (index: number, of: number): Potential => ({ kind: "talent", index, of });

const op = (id: string, name: string, rarity: number, profession: string, professionName: string, tagList: string[], potentials: Potential[]) => ({ id, name, rarity, profession, professionName, position: tagList[0] === "Melee" ? "MELEE" : "RANGED", tagList, potentials });

const seniorMedic = {
    tags: [14, 4],
    tagNames: ["Senior Operator", "Medic"],
    guaranteedRarity: 5,
    maxRarity: 5,
    fiveStarCount: 4,
    operators: [
        op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "Medic", ["Ranged", "Medic", "Senior Operator", "Healing", "Support"], [cost(-1), respawn(-4), atk(21), respawn(-6), cost(-1)]),
        op("char_108_silent", "Silence", 5, "MEDIC", "Medic", ["Ranged", "Medic", "Senior Operator", "Healing"], [cost(-1), respawn(-4), atk(24), talent(0, 1), cost(-1)]),
        op("char_171_bldsk", "Warfarin", 5, "MEDIC", "Medic", ["Ranged", "Medic", "Senior Operator", "Healing", "Support"], [cost(-1), respawn(-4), atk(27), respawn(-6), cost(-1)]),
        op("char_436_whispr", "Whisperain", 5, "MEDIC", "Medic", ["Ranged", "Medic", "Senior Operator", "Healing"], [cost(-1), respawn(-4), atk(23), talent(0, 1), cost(-1)]),
    ],
};

const topSniper = {
    tags: [11, 2],
    tagNames: ["Top Operator", "Sniper"],
    guaranteedRarity: 6,
    maxRarity: 6,
    fiveStarCount: 4,
    operators: [
        op("char_332_archet", "Archetto", 6, "SNIPER", "Sniper", ["Ranged", "Sniper", "Top Operator", "DPS"], [cost(-1), respawn(-4), atk(27), talent(1, 2), cost(-1)]),
        op("char_103_angel", "Exusiai", 6, "SNIPER", "Sniper", ["Ranged", "Sniper", "Top Operator", "DPS"], [cost(-1), talent(0, 2), atk(27), cost(-1), talent(1, 2)]),
        op("char_197_poca", "Rosa", 6, "SNIPER", "Sniper", ["Ranged", "Sniper", "Top Operator", "DPS", "Crowd-Control"], [cost(-1), respawn(-4), atk(34), talent(1, 2), cost(-1)]),
        op("char_340_shwaz", "Schwarz", 6, "SNIPER", "Sniper", ["Ranged", "Sniper", "Top Operator", "DPS"], [cost(-1), respawn(-4), atk(30), cost(-1), talent(1, 2)]),
    ],
};

const slowRanged = {
    tags: [23, 10],
    tagNames: ["Slow", "Ranged"],
    guaranteedRarity: 3,
    maxRarity: 5,
    fiveStarCount: 4,
    operators: [
        op("char_218_cuttle", "Andreana", 5, "SNIPER", "Sniper", ["Ranged", "Sniper", "Senior Operator", "DPS", "Slow"], []),
        op("char_326_glacus", "Glaucus", 5, "SUPPORT", "Supporter", ["Ranged", "Supporter", "Senior Operator", "Slow", "Crowd-Control"], []),
        op("char_195_glassb", "Istina", 5, "SUPPORT", "Supporter", ["Ranged", "Supporter", "Senior Operator", "Slow", "DPS"], []),
        op("char_164_nightm", "Nightmare", 5, "CASTER", "Caster", ["Ranged", "Caster", "Senior Operator", "DPS", "Healing", "Slow"], []),
        op("char_302_glaze", "Ambriel", 4, "SNIPER", "Sniper", ["Ranged", "Sniper", "DPS", "Slow"], []),
        op("char_183_skgoat", "Earthspirit", 4, "SUPPORT", "Supporter", ["Ranged", "Supporter", "Slow"], []),
        op("char_253_greyy", "Greyy", 4, "CASTER", "Caster", ["Ranged", "Caster", "AoE", "Slow"], []),
        op("char_133_mm", "May", 4, "SNIPER", "Sniper", ["Ranged", "Sniper", "DPS", "Slow"], []),
        op("char_258_podego", "Podenco", 4, "SUPPORT", "Supporter", ["Ranged", "Supporter", "Slow", "Healing"], []),
        op("char_118_yuki", "Shirayuki", 4, "SNIPER", "Sniper", ["Ranged", "Sniper", "AoE", "Slow"], []),
        op("char_278_orchid", "Orchid", 3, "SUPPORT", "Supporter", ["Ranged", "Supporter", "Slow"], []),
    ],
};

const noRoster = { showPotentials: false, showNextUpgrade: false, potentialByOperator: new Map<string, number>() };

/** A Senior Operator lock: filled 5★ badge, four amber rows, no roster overlay. */
export const FiveStarLock = () => <ResultCardDetailed result={seniorMedic} roster={noRoster} />;

/** A Top Operator lock with no roster overlay. */
export const SixStarLock = () => <ResultCardDetailed result={topSniper} roster={noRoster} />;

/** No lock: the quiet "Min 3★" label, and a pool that runs from 5★ down to 3★. */
export const MixedPool = () => <ResultCardDetailed result={slowRanged} roster={noRoster} />;

/**
 * Signed in with both overlays on: owned operators carry their potential
 * badge and the next rank's gain; Rosa is unowned, so she greys out and
 * reads as the recruit itself being the gain.
 */
export const WithRoster = () => (
    <ResultCardDetailed
        result={topSniper}
        roster={{
            showPotentials: true,
            showNextUpgrade: true,
            potentialByOperator: new Map([
                ["char_332_archet", 2],
                ["char_103_angel", 0],
                ["char_340_shwaz", 5],
            ]),
        }}
    />
);
