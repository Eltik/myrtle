import { RosterCompactCard } from "frontend";

// Real gamedata slices for the two operators these cards show. The card only
// reads `static.skills` (how many mastery pips to draw) and `static.modules`
// (which module badges are unlocked), so the fixtures carry exactly that.
const MLYNAR_STATIC = {
    id: "char_4064_mlynar",
    name: "Młynar",
    rarity: "TIER_6",
    profession: "WARRIOR",
    subProfessionId: "librator",
    skills: [{ skillId: "skchr_mlynar_1" }, { skillId: "skchr_mlynar_2" }, { skillId: "skchr_mlynar_3" }],
    modules: [
        { uniEquipId: "uniequip_001_mlynar", uniEquipName: "Młynar's Badge", uniEquipIcon: "uniequip_001_mlynar", image: null, typeName1: "ORIGINAL", typeName2: null, type: "INITIAL" },
        { uniEquipId: "uniequip_002_mlynar", uniEquipName: "'Man in Scabbard'", uniEquipIcon: "uniequip_002_mlynar", image: "/textures/spritepack/ui_equip_big_img_hub_19/uniequip_002_mlynar.png", typeName1: "LIB", typeName2: "X", type: "ADVANCED" },
    ],
};

const SKADI_STATIC = {
    id: "char_263_skadi",
    name: "Skadi",
    rarity: "TIER_6",
    profession: "WARRIOR",
    subProfessionId: "fearless",
    skills: [{ skillId: "skcom_quickattack[3]" }, { skillId: "skchr_skadi_2" }, { skillId: "skchr_skadi_3" }],
    modules: [
        { uniEquipId: "uniequip_001_skadi", uniEquipName: "Skadi's Badge", uniEquipIcon: "uniequip_001_skadi", image: null, typeName1: "ORIGINAL", typeName2: null, type: "INITIAL" },
        { uniEquipId: "uniequip_002_skadi", uniEquipName: "Moist Sword Bag", uniEquipIcon: "uniequip_002_skadi", image: "/textures/spritepack/ui_equip_big_img_hub_8/uniequip_002_skadi.png", typeName1: "DRE", typeName2: "Y", type: "ADVANCED" },
        { uniEquipId: "uniequip_003_skadi", uniEquipName: "No Ending to This Dream", uniEquipIcon: "uniequip_003_skadi", image: "/textures/spritepack/ui_equip_big_img_hub_12/uniequip_003_skadi.png", typeName1: "DRE", typeName2: "X", type: "ADVANCED" },
    ],
};

const LAPPLAND_STATIC = {
    id: "char_140_whitew",
    name: "Lappland",
    rarity: "TIER_5",
    profession: "WARRIOR",
    subProfessionId: "lord",
    skills: [{ skillId: "skchr_whitew_1" }, { skillId: "skchr_whitew_2" }],
    modules: [{ uniEquipId: "uniequip_001_whitew", uniEquipName: "Lappland's Badge", uniEquipIcon: "uniequip_001_whitew", image: null, typeName1: "ORIGINAL", typeName2: null, type: "INITIAL" }],
};

// Fully built: E2 90, all three skills at M3, module at stage 3, P6. `isMaxed`
// turns on the rarity glow and swaps every badge for the "Maxed" ribbon.
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

// Mid-build: E2 82 with mixed masteries and two modules at different stages, so
// every badge cluster on the card is populated.
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

// Early game: E1 55, no masteries, no modules unlocked — only the level ring
// and the elite pip show.
const lappland = {
    user_id: "1000048871",
    operator_id: "char_140_whitew",
    elite: 1,
    level: 55,
    exp: 0,
    potential: 1,
    skill_level: 4,
    favor_point: 4200,
    skin_id: null,
    default_skill: 1,
    voice_lan: "JP",
    current_equip: null,
    current_tmpl: null,
    obtained_at: 1698796800,
    masteries: [],
    modules: [],
    isOwned: true,
    meta: null,
    static: LAPPLAND_STATIC,
    name: "Lappland",
    rarity: 5,
};

export const MaxedOperator = () => (
    <div className="pt-2 pl-2">
        <RosterCompactCard entry={mlynar} />
    </div>
);

export const PartiallyBuilt = () => (
    <div className="pt-2 pl-2">
        <RosterCompactCard entry={skadi} />
    </div>
);

export const EarlyElite = () => (
    <div className="pt-2 pl-2">
        <RosterCompactCard entry={lappland} />
    </div>
);

// The grid the Roster tab lays these out in when "compact" view is selected.
export const CompactGrid = () => (
    <div className="grid grid-cols-[repeat(auto-fill,minmax(6.5rem,1fr))] gap-3 pt-2 sm:grid-cols-[repeat(auto-fill,minmax(9rem,1fr))]">
        <RosterCompactCard entry={mlynar} />
        <RosterCompactCard entry={skadi} />
        <RosterCompactCard entry={lappland} />
    </div>
);
