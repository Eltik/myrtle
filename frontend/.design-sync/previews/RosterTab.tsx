import { RosterTab } from "frontend";
import { type ReactNode, useState } from "react";

// Index rows drive the name/rarity/class shown on every card; the static rows
// carry the phases, skills and modules the detailed card derives stats from.
const index = (id: string, name: string, rarity: number, profession: string, subProfessionId: string) => ({ id, name, appellation: name, rarity, profession, subProfessionId, position: "MELEE", tagList: [], nationId: "rhodes", isNotObtainable: false, groupId: null, teamId: null, artists: [], portrait: null, gender: "", race: "", placeOfBirth: "" });

// The index carries the whole operator list — the three the Doctor owns plus
// the ones they don't, which is what the "Unowned" filter reads.
const operatorsIndex = [
    index("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator"),
    index("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless"),
    index("char_140_whitew", "Lappland", 5, "WARRIOR", "lord"),
    index("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor"),
    index("char_2015_dusk", "Dusk", 6, "CASTER", "splashcaster"),
    index("char_4045_heidi", "Heidi", 5, "SUPPORT", "bard"),
];

const MLYNAR_STATIC = {
    id: "char_4064_mlynar",
    name: "Młynar",
    rarity: "TIER_6",
    profession: "WARRIOR",
    subProfessionId: "librator",
    portrait: "/portraits/char_4064_mlynar_2.png",
    skin: "/textures/chararts/char_4064_mlynar/char_4064_mlynar_2.png",
    phases: [
        { maxLevel: 50, attributesKeyFrames: [{ level: 1, data: { maxHp: 1945, atk: 161, def: 239, magicResistance: 15, cost: 10, blockCnt: 2 } }, { level: 50, data: { maxHp: 2560, atk: 231, def: 332, magicResistance: 15, cost: 10, blockCnt: 2 } }] },
        { maxLevel: 80, attributesKeyFrames: [{ level: 1, data: { maxHp: 2560, atk: 231, def: 332, magicResistance: 15, cost: 12, blockCnt: 2 } }, { level: 80, data: { maxHp: 3241, atk: 301, def: 426, magicResistance: 15, cost: 12, blockCnt: 2 } }] },
        { maxLevel: 90, attributesKeyFrames: [{ level: 1, data: { maxHp: 3241, atk: 301, def: 426, magicResistance: 15, cost: 12, blockCnt: 3 } }, { level: 90, data: { maxHp: 3906, atk: 355, def: 502, magicResistance: 15, cost: 12, blockCnt: 3 } }] },
    ],
    skills: [
        { skillId: "skchr_mlynar_1", static: { skillId: "skchr_mlynar_1", iconId: null, image: "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_1.png", levels: [{ name: "Unvoiced Anger" }] } },
        { skillId: "skchr_mlynar_2", static: { skillId: "skchr_mlynar_2", iconId: null, image: "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_2.png", levels: [{ name: "Unresolved Sorrow" }] } },
        { skillId: "skchr_mlynar_3", static: { skillId: "skchr_mlynar_3", iconId: null, image: "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_3.png", levels: [{ name: "Unbrilliant Glory" }] } },
    ],
    modules: [
        { uniEquipId: "uniequip_001_mlynar", uniEquipName: "Młynar's Badge", uniEquipIcon: "uniequip_001_mlynar", image: null, typeName1: "ORIGINAL", typeName2: null, type: "INITIAL", data: null },
        { uniEquipId: "uniequip_002_mlynar", uniEquipName: "'Man in Scabbard'", uniEquipIcon: "uniequip_002_mlynar", image: "/textures/spritepack/ui_equip_big_img_hub_19/uniequip_002_mlynar.png", typeName1: "LIB", typeName2: "X", type: "ADVANCED", data: { phases: [{ attributeBlackboard: [{ key: "max_hp", value: 150 }] }, { attributeBlackboard: [{ key: "max_hp", value: 250 }] }, { attributeBlackboard: [{ key: "max_hp", value: 360 }, { key: "atk", value: 35 }] }] } },
    ],
    potentialRanks: [
        { type: "BUFF", description: "DP Cost -1", buff: { attributes: { attributeModifiers: [{ attributeType: "COST", value: -1 }] } } },
        { type: "CUSTOM", description: "Improves First Talent", buff: null },
        { type: "BUFF", description: "ATK +25", buff: { attributes: { attributeModifiers: [{ attributeType: "ATK", value: 25 }] } } },
        { type: "CUSTOM", description: "Improves Second Talent", buff: null },
        { type: "BUFF", description: "DP Cost -1", buff: { attributes: { attributeModifiers: [{ attributeType: "COST", value: -1 }] } } },
    ],
    favorKeyFrames: [{ level: 0, data: { maxHp: 0, atk: 0, def: 0 } }, { level: 50, data: { maxHp: 360, atk: 30, def: 0 } }],
};

const SKADI_STATIC = {
    id: "char_263_skadi",
    name: "Skadi",
    rarity: "TIER_6",
    profession: "WARRIOR",
    subProfessionId: "fearless",
    portrait: "/portraits/char_263_skadi_2.png",
    skin: "/textures/chararts/char_263_skadi/char_263_skadi_2.png",
    phases: [
        { maxLevel: 50, attributesKeyFrames: [{ level: 1, data: { maxHp: 1521, atk: 452, def: 116, magicResistance: 0, cost: 17, blockCnt: 1 } }, { level: 50, data: { maxHp: 2174, atk: 665, def: 167, magicResistance: 0, cost: 17, blockCnt: 1 } }] },
        { maxLevel: 80, attributesKeyFrames: [{ level: 1, data: { maxHp: 2174, atk: 665, def: 167, magicResistance: 0, cost: 19, blockCnt: 1 } }, { level: 80, data: { maxHp: 2938, atk: 842, def: 220, magicResistance: 0, cost: 19, blockCnt: 1 } }] },
        { maxLevel: 90, attributesKeyFrames: [{ level: 1, data: { maxHp: 2938, atk: 842, def: 220, magicResistance: 0, cost: 19, blockCnt: 1 } }, { level: 90, data: { maxHp: 3866, atk: 1015, def: 263, magicResistance: 0, cost: 19, blockCnt: 1 } }] },
    ],
    skills: [
        { skillId: "skcom_quickattack[3]", static: { skillId: "skcom_quickattack[3]", iconId: null, image: "/textures/spritepack/skill_icons_1/skill_icon_skcom_quickattack[3].png", levels: [{ name: "Swift Strike γ" }] } },
        { skillId: "skchr_skadi_2", static: { skillId: "skchr_skadi_2", iconId: null, image: "/textures/spritepack/skill_icons_1/skill_icon_skchr_skadi_2.png", levels: [{ name: "Wave Strike" }] } },
        { skillId: "skchr_skadi_3", static: { skillId: "skchr_skadi_3", iconId: null, image: "/textures/spritepack/skill_icons_1/skill_icon_skchr_skadi_3.png", levels: [{ name: "Tidal Elegy" }] } },
    ],
    modules: [
        { uniEquipId: "uniequip_001_skadi", uniEquipName: "Skadi's Badge", uniEquipIcon: "uniequip_001_skadi", image: null, typeName1: "ORIGINAL", typeName2: null, type: "INITIAL", data: null },
        { uniEquipId: "uniequip_003_skadi", uniEquipName: "No Ending to This Dream", uniEquipIcon: "uniequip_003_skadi", image: "/textures/spritepack/ui_equip_big_img_hub_12/uniequip_003_skadi.png", typeName1: "DRE", typeName2: "X", type: "ADVANCED", data: { phases: [{ attributeBlackboard: [{ key: "atk", value: 30 }] }, { attributeBlackboard: [{ key: "atk", value: 50 }] }, { attributeBlackboard: [{ key: "atk", value: 75 }, { key: "def", value: 40 }] }] } },
    ],
    potentialRanks: [
        { type: "BUFF", description: "DP Cost -1", buff: { attributes: { attributeModifiers: [{ attributeType: "COST", value: -1 }] } } },
        { type: "BUFF", description: "Redeployment Time -4 sec", buff: { attributes: { attributeModifiers: [{ attributeType: "RESPAWN_TIME", value: -4 }] } } },
        { type: "BUFF", description: "ATK +33", buff: { attributes: { attributeModifiers: [{ attributeType: "ATK", value: 33 }] } } },
        { type: "CUSTOM", description: "Improves First Talent", buff: null },
        { type: "BUFF", description: "DP Cost -1", buff: { attributes: { attributeModifiers: [{ attributeType: "COST", value: -1 }] } } },
    ],
    favorKeyFrames: [{ level: 0, data: { maxHp: 0, atk: 0, def: 0 } }, { level: 50, data: { maxHp: 0, atk: 80, def: 40 } }],
};

const LAPPLAND_STATIC = {
    id: "char_140_whitew",
    name: "Lappland",
    rarity: "TIER_5",
    profession: "WARRIOR",
    subProfessionId: "lord",
    portrait: "/portraits/char_140_whitew_2.png",
    skin: "/textures/chararts/char_140_whitew/char_140_whitew_2.png",
    phases: [
        { maxLevel: 50, attributesKeyFrames: [{ level: 1, data: { maxHp: 987, atk: 285, def: 173, magicResistance: 10, cost: 17, blockCnt: 2 } }, { level: 50, data: { maxHp: 1410, atk: 426, def: 238, magicResistance: 10, cost: 17, blockCnt: 2 } }] },
        { maxLevel: 70, attributesKeyFrames: [{ level: 1, data: { maxHp: 1410, atk: 426, def: 238, magicResistance: 10, cost: 19, blockCnt: 2 } }, { level: 70, data: { maxHp: 1856, atk: 554, def: 302, magicResistance: 10, cost: 19, blockCnt: 2 } }] },
        { maxLevel: 80, attributesKeyFrames: [{ level: 1, data: { maxHp: 1856, atk: 554, def: 302, magicResistance: 15, cost: 19, blockCnt: 2 } }, { level: 80, data: { maxHp: 2350, atk: 685, def: 365, magicResistance: 15, cost: 19, blockCnt: 2 } }] },
    ],
    skills: [
        { skillId: "skchr_whitew_1", static: { skillId: "skchr_whitew_1", iconId: null, image: "/textures/spritepack/skill_icons_1/skill_icon_skchr_whitew_1.png", levels: [{ name: "Sundial" }] } },
        { skillId: "skchr_whitew_2", static: { skillId: "skchr_whitew_2", iconId: null, image: "/textures/spritepack/skill_icons_1/skill_icon_skchr_whitew_2.png", levels: [{ name: "Wolf Spirit" }] } },
    ],
    modules: [
        { uniEquipId: "uniequip_001_whitew", uniEquipName: "Lappland's Badge", uniEquipIcon: "uniequip_001_whitew", image: null, typeName1: "ORIGINAL", typeName2: null, type: "INITIAL", data: null },
        { uniEquipId: "uniequip_002_whitew", uniEquipName: "'The Young Fang'", uniEquipIcon: "uniequip_002_whitew", image: "/textures/spritepack/ui_equip_big_img_hub_14/uniequip_002_whitew.png", typeName1: "LOR", typeName2: "X", type: "ADVANCED", data: { phases: [{ attributeBlackboard: [{ key: "atk", value: 20 }] }, { attributeBlackboard: [{ key: "atk", value: 34 }] }, { attributeBlackboard: [{ key: "atk", value: 50 }] }] } },
    ],
    potentialRanks: [
        { type: "BUFF", description: "DP Cost -1", buff: { attributes: { attributeModifiers: [{ attributeType: "COST", value: -1 }] } } },
        { type: "BUFF", description: "Redeployment Time -4 sec", buff: { attributes: { attributeModifiers: [{ attributeType: "RESPAWN_TIME", value: -4 }] } } },
        { type: "BUFF", description: "ATK +25", buff: { attributes: { attributeModifiers: [{ attributeType: "ATK", value: 25 }] } } },
        { type: "CUSTOM", description: "Improves Talent", buff: null },
        { type: "BUFF", description: "DP Cost -1", buff: { attributes: { attributeModifiers: [{ attributeType: "COST", value: -1 }] } } },
    ],
    favorKeyFrames: [{ level: 0, data: { maxHp: 0, atk: 0, def: 0 } }, { level: 50, data: { maxHp: 0, atk: 75, def: 0 } }],
};

const operatorsStatic = [MLYNAR_STATIC, SKADI_STATIC, LAPPLAND_STATIC];

const roster = [
    { user_id: "1000048871", operator_id: "char_4064_mlynar", elite: 2, level: 90, exp: 0, potential: 5, skill_level: 7, favor_point: 25570, skin_id: null, default_skill: 2, voice_lan: "JP", current_equip: "uniequip_002_mlynar", current_tmpl: null, obtained_at: 1671840000, masteries: [{ index: 0, mastery: 3 }, { index: 1, mastery: 3 }, { index: 2, mastery: 3 }], modules: [{ id: "uniequip_002_mlynar", level: 3, locked: false }] },
    { user_id: "1000048871", operator_id: "char_263_skadi", elite: 2, level: 82, exp: 0, potential: 2, skill_level: 7, favor_point: 18400, skin_id: null, default_skill: 2, voice_lan: "CN_MANDARIN", current_equip: "uniequip_003_skadi", current_tmpl: null, obtained_at: 1580515200, masteries: [{ index: 0, mastery: 0 }, { index: 1, mastery: 3 }, { index: 2, mastery: 1 }], modules: [{ id: "uniequip_003_skadi", level: 3, locked: false }] },
    { user_id: "1000048871", operator_id: "char_140_whitew", elite: 2, level: 70, exp: 0, potential: 3, skill_level: 7, favor_point: 12800, skin_id: null, default_skill: 1, voice_lan: "JP", current_equip: "uniequip_002_whitew", current_tmpl: null, obtained_at: 1587772800, masteries: [{ index: 0, mastery: 1 }, { index: 1, mastery: 2 }], modules: [{ id: "uniequip_002_whitew", level: 2, locked: false }] },
];

// The ownership filter is persisted (`useRoster` → `useLocalStorageState`), not
// a prop, so each story seeds the stored filter state before the tab mounts —
// a parent's `useState` initializer runs ahead of the child's. Both stories set
// it explicitly, because localStorage survives between story navigations.
// (`viewMode` is NOT settable this way: `useRoster` force-sets it to "detailed"
// on mount at >=768px, which is every capture viewport.)
const Filters = ({ ownership, children }: { ownership: "owned" | "unowned" | "all"; children: ReactNode }) => {
    useState(() => {
        localStorage.setItem("user:roster:filters", JSON.stringify({ search: "", ownership, rarity: "all", sortBy: "rarity", sortOrder: "desc", viewMode: "detailed" }));
        return null;
    });
    return <>{children}</>;
};

// Search, ownership / sort / rarity selects, sort direction and the view toggle
// above the operator grid — the tab's default "owned, by rarity" view.
export const OwnedRoster = () => (
    <Filters ownership="owned">
        <RosterTab roster={roster} operatorsIndex={operatorsIndex} operatorsStatic={operatorsStatic} />
    </Filters>
);

// The same tab switched to "Unowned": every operator in the index the Doctor is
// missing, rendered as desaturated "Not Owned" cards with stubbed stat rows.
export const UnownedOperators = () => (
    <Filters ownership="unowned">
        <RosterTab roster={roster} operatorsIndex={operatorsIndex} operatorsStatic={operatorsStatic} />
    </Filters>
);
