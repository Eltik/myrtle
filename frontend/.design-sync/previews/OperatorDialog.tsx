import { Dialog, OperatorDialog } from "frontend";

// Trimmed real gamedata: the dialog reads phases (for the derived stat grid),
// skills, advanced-module bonuses, potential ranks and trust keyframes.
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

const maxed = {
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

// E1 with no module unlocked and mastery still untouched — the Modules section
// falls back to its "nothing unlocked" line while Skills stays populated.
const inProgress = {
    ...maxed,
    elite: 1,
    level: 70,
    potential: 1,
    skill_level: 5,
    favor_point: 9200,
    current_equip: null,
    voice_lan: "CN_MANDARIN",
    masteries: [],
    modules: [],
    obtained_at: 1705276800,
};

// The dialog is portalled, so the story needs a tall stage for the backdrop.
export const FullyBuilt = () => (
    <div className="min-h-[520px]">
        <Dialog open>
            <OperatorDialog entry={maxed} />
        </Dialog>
    </div>
);

export const StillInvesting = () => (
    <div className="min-h-[520px]">
        <Dialog open>
            <OperatorDialog entry={inProgress} />
        </Dialog>
    </div>
);
