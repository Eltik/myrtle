import { RosterDetailedCard } from "frontend";

// Real gamedata for Młynar, trimmed to the fields the card reads: the three
// elite phases (level→stat interpolation), skills, the advanced module's
// attribute bonuses, potential ranks and the trust keyframes.
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
        {
            uniEquipId: "uniequip_002_mlynar",
            uniEquipName: "'Man in Scabbard'",
            uniEquipIcon: "uniequip_002_mlynar",
            image: "/textures/spritepack/ui_equip_big_img_hub_19/uniequip_002_mlynar.png",
            typeName1: "LIB",
            typeName2: "X",
            type: "ADVANCED",
            data: { phases: [{ attributeBlackboard: [{ key: "max_hp", value: 150 }, { key: "atk", value: 14 }] }, { attributeBlackboard: [{ key: "max_hp", value: 250 }, { key: "atk", value: 24 }] }, { attributeBlackboard: [{ key: "max_hp", value: 360 }, { key: "atk", value: 35 }, { key: "attack_speed", value: 7 }] }] },
        },
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
        { uniEquipId: "uniequip_002_skadi", uniEquipName: "Moist Sword Bag", uniEquipIcon: "uniequip_002_skadi", image: "/textures/spritepack/ui_equip_big_img_hub_8/uniequip_002_skadi.png", typeName1: "DRE", typeName2: "Y", type: "ADVANCED", data: { phases: [{ attributeBlackboard: [{ key: "max_hp", value: 200 }, { key: "atk", value: 35 }] }, { attributeBlackboard: [{ key: "max_hp", value: 320 }, { key: "atk", value: 58 }] }, { attributeBlackboard: [{ key: "max_hp", value: 450 }, { key: "atk", value: 85 }] }] } },
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

const mlynar = {
    user_id: "1000048871",
    operator_id: "char_4064_mlynar",
    elite: 2,
    level: 90,
    exp: 0,
    potential: 5,
    skill_level: 7,
    favor_point: 25570,
    skin_id: null,
    default_skill: 2,
    voice_lan: "JP",
    current_equip: "uniequip_002_mlynar",
    current_tmpl: null,
    obtained_at: 1671840000,
    masteries: [{ index: 0, mastery: 3 }, { index: 1, mastery: 3 }, { index: 2, mastery: 3 }],
    modules: [{ id: "uniequip_002_mlynar", level: 3, locked: false }],
    isOwned: true,
    meta: null,
    static: MLYNAR_STATIC,
    name: "Młynar",
    rarity: 6,
};

const skadi = {
    user_id: "1000048871",
    operator_id: "char_263_skadi",
    elite: 2,
    level: 82,
    exp: 0,
    potential: 2,
    skill_level: 7,
    favor_point: 18400,
    skin_id: null,
    default_skill: 2,
    voice_lan: "CN_MANDARIN",
    current_equip: "uniequip_003_skadi",
    current_tmpl: null,
    obtained_at: 1580515200,
    masteries: [{ index: 0, mastery: 0 }, { index: 1, mastery: 3 }, { index: 2, mastery: 1 }],
    modules: [{ id: "uniequip_002_skadi", level: 2, locked: false }, { id: "uniequip_003_skadi", level: 3, locked: false }],
    isOwned: true,
    meta: null,
    static: SKADI_STATIC,
    name: "Skadi",
    rarity: 6,
};

// The roster loads before the operator catalog does, so `static` is briefly
// null: stats fall back to "--" and the accordions say "Loading …".
const catalogPending = { ...skadi, static: null, name: "Skadi", operator_id: "char_263_skadi" };

export const MaxedOperator = () => (
    <div className="mx-auto max-w-sm">
        <RosterDetailedCard entry={mlynar} />
    </div>
);

export const PartiallyBuilt = () => (
    <div className="mx-auto max-w-sm">
        <RosterDetailedCard entry={skadi} />
    </div>
);

export const CatalogPending = () => (
    <div className="mx-auto max-w-sm">
        <RosterDetailedCard entry={catalogPending} />
    </div>
);
