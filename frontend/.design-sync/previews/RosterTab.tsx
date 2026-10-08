import { RosterTab } from "frontend";
import { type ReactNode, useState } from "react";

// Index rows drive the name/rarity/class shown on every card; the static rows
// carry the phases, skills and modules the detailed card derives stats from.

// The index carries the whole operator list — the three the Doctor owns plus
// the ones they don't, which is what the "Unowned" filter reads.
/** Real `/api/operators/index` entries (fetched 2026-10-08), the shape `operatorsIndex` takes. */
const operatorsIndex = [
    {"id": "char_4064_mlynar", "name": "Młynar", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "librator", "position": "MELEE", "tagList": ["DPS", "Nuker"], "nationId": "kazimierz", "isNotObtainable": false, "groupId": null, "teamId": null, "professionName": "Guard", "subProfessionName": "Liberator", "nationName": "Kazimierz", "groupName": null, "teamName": null, "obtainChannel": "headhunting", "artists": ["竜崎いち"], "portrait": "/portraits/char_4064_mlynar_2.png", "gender": "Male", "race": "Kuranta", "placeOfBirth": "Kazimierz", "stats": {"hp": 3906, "atk": 355, "def": 502, "res": 15.0, "cost": 12, "block": 3}, "hasOffensiveRecovery": false, "hasDefensiveRecovery": false, "allSkillsManual": true, "voiceActors": ["Yuhang Wang", "Anthony Howell", "Rikiya Koyama", "Choi Hyun-su"], "phaseMaxLevels": [50, 80, 90], "skillCount": 3, "potentialRankCount": 5, "modules": [{"uniEquipId": "uniequip_001_mlynar", "typeName1": "ORIGINAL", "typeName2": null, "type": "INITIAL"}, {"uniEquipId": "uniequip_002_mlynar", "typeName1": "LIB", "typeName2": "X", "type": "ADVANCED"}], "dateOfBirth": "Dec 3"},
    {"id": "char_263_skadi", "name": "Skadi", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "fearless", "position": "MELEE", "tagList": ["DPS", "Survival"], "nationId": "egir", "isNotObtainable": false, "groupId": "abyssal", "teamId": null, "professionName": "Guard", "subProfessionName": "Dreadnought", "nationName": "Ægir", "groupName": "Abyssal Hunters", "teamName": null, "obtainChannel": "headhunting", "artists": ["alchemaniac"], "portrait": "/portraits/char_263_skadi_2.png", "gender": "Female", "race": "Unknown", "placeOfBirth": "Ægir", "stats": {"hp": 3866, "atk": 1015, "def": 263, "res": 0.0, "cost": 19, "block": 1}, "hasOffensiveRecovery": false, "hasDefensiveRecovery": false, "allSkillsManual": false, "voiceActors": ["Kiiki", "Cristina Vee", "Rina Sato", "Kim Na-yul"], "phaseMaxLevels": [50, 80, 90], "skillCount": 3, "potentialRankCount": 5, "modules": [{"uniEquipId": "uniequip_001_skadi", "typeName1": "ORIGINAL", "typeName2": null, "type": "INITIAL"}, {"uniEquipId": "uniequip_002_skadi", "typeName1": "DRE", "typeName2": "Y", "type": "ADVANCED"}, {"uniEquipId": "uniequip_003_skadi", "typeName1": "DRE", "typeName2": "X", "type": "ADVANCED"}], "dateOfBirth": "Mar. 7"},
    {"id": "char_140_whitew", "name": "Lappland", "appellation": " ", "rarity": 5, "profession": "WARRIOR", "subProfessionId": "lord", "position": "MELEE", "tagList": ["DPS", "Debuff"], "nationId": "siracusa", "isNotObtainable": false, "groupId": null, "teamId": null, "professionName": "Guard", "subProfessionName": "Lord", "nationName": "Siracusa", "groupName": null, "teamName": null, "obtainChannel": "headhunting", "artists": ["幻象黑兔"], "portrait": "/portraits/char_140_whitew_2.png", "gender": "Female", "race": "Lupo", "placeOfBirth": "Siracusa", "stats": {"hp": 2350, "atk": 685, "def": 365, "res": 15.0, "cost": 19, "block": 2}, "hasOffensiveRecovery": true, "hasDefensiveRecovery": false, "allSkillsManual": false, "voiceActors": ["Yee Chen", "Christina Kowalchuk", "Elisa Contestabile", "Asami Imai", "Seo Yu-ri"], "phaseMaxLevels": [50, 70, 80], "skillCount": 2, "potentialRankCount": 5, "modules": [{"uniEquipId": "uniequip_001_whitew", "typeName1": "ORIGINAL", "typeName2": null, "type": "INITIAL"}, {"uniEquipId": "uniequip_002_whitew", "typeName1": "LOR", "typeName2": "X", "type": "ADVANCED"}], "dateOfBirth": "Nov. 11"},
    {"id": "char_1028_texas2", "name": "Texas the Omertosa", "appellation": " ", "rarity": 6, "profession": "SPECIAL", "subProfessionId": "executor", "position": "MELEE", "tagList": ["Fast-Redeploy", "DPS"], "nationId": "lungmen", "isNotObtainable": false, "groupId": "penguin", "teamId": null, "professionName": "Specialist", "subProfessionName": "Executor", "nationName": "Yan-Lungmen", "groupName": "Penguin Logistics", "teamName": null, "obtainChannel": "headhunting", "artists": ["Studio Montagne", "幻象黑兔"], "portrait": "/portraits/char_1028_texas2_2.png", "gender": "Female", "race": "Lupo", "placeOfBirth": "Columbia", "stats": {"hp": 1598, "atk": 569, "def": 320, "res": 0.0, "cost": 10, "block": 1}, "hasOffensiveRecovery": false, "hasDefensiveRecovery": false, "allSkillsManual": false, "voiceActors": ["Menglu Yang", "Jessica Preddy", "Ilaria Silvestri", "Azusa Tadokoro", "Kim Chae-ha"], "phaseMaxLevels": [50, 80, 90], "skillCount": 3, "potentialRankCount": 5, "modules": [{"uniEquipId": "uniequip_001_texas2", "typeName1": "ORIGINAL", "typeName2": null, "type": "INITIAL"}, {"uniEquipId": "uniequip_002_texas2", "typeName1": "EXE", "typeName2": "Y", "type": "ADVANCED"}], "dateOfBirth": "Jun 1"},
    {"id": "char_2015_dusk", "name": "Dusk", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "splashcaster", "position": "RANGED", "tagList": ["AoE", "DPS", "Crowd-Control"], "nationId": "yan", "isNotObtainable": false, "groupId": "sui", "teamId": null, "professionName": "Caster", "subProfessionName": "Splash Caster", "nationName": "Yan", "groupName": "Yan-Sui", "teamName": null, "obtainChannel": "headhunting", "artists": ["幻象黑兔", "Studio Montagne"], "portrait": "/portraits/char_2015_dusk_2.png", "gender": "Female", "race": "Undisclosed", "placeOfBirth": "Yan", "stats": {"hp": 1801, "atk": 918, "def": 127, "res": 20.0, "cost": 34, "block": 1}, "hasOffensiveRecovery": false, "hasDefensiveRecovery": false, "allSkillsManual": false, "voiceActors": ["Blank", "Aileen Mythen", "yukana", "Bae Jeong-mi"], "phaseMaxLevels": [50, 80, 90], "skillCount": 3, "potentialRankCount": 5, "modules": [{"uniEquipId": "uniequip_001_dusk", "typeName1": "ORIGINAL", "typeName2": null, "type": "INITIAL"}, {"uniEquipId": "uniequip_002_dusk", "typeName1": "SPC", "typeName2": "X", "type": "ADVANCED"}, {"uniEquipId": "uniequip_003_dusk", "typeName1": "SPC", "typeName2": "Y", "type": "ADVANCED"}, {"uniEquipId": "uniequip_004_dusk", "typeName1": "ISW", "typeName2": "A", "type": "ADVANCED"}], "dateOfBirth": "November 11"},
    {"id": "char_4045_heidi", "name": "Heidi", "appellation": " ", "rarity": 5, "profession": "SUPPORT", "subProfessionId": "bard", "position": "RANGED", "tagList": ["Support", "Healing"], "nationId": "victoria", "isNotObtainable": false, "groupId": null, "teamId": null, "professionName": "Supporter", "subProfessionName": "Bard", "nationName": "Victoria", "groupName": null, "teamName": null, "obtainChannel": "event", "artists": ["熊太"], "portrait": "/portraits/char_4045_heidi_2.png", "gender": "Female", "race": "Feline", "placeOfBirth": "Victoria", "stats": {"hp": 1260, "atk": 320, "def": 268, "res": 0.0, "cost": 9, "block": 1}, "hasOffensiveRecovery": false, "hasDefensiveRecovery": false, "allSkillsManual": true, "voiceActors": ["Ying Xie", "Sophie Shad", "Akira Sekine", "Sung Ye-won"], "phaseMaxLevels": [50, 70, 80], "skillCount": 2, "potentialRankCount": 5, "modules": [{"uniEquipId": "uniequip_001_heidi", "typeName1": "ORIGINAL", "typeName2": null, "type": "INITIAL"}, {"uniEquipId": "uniequip_002_heidi", "typeName1": "BAR", "typeName2": "X", "type": "ADVANCED"}], "dateOfBirth": "Mar 26"},
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
