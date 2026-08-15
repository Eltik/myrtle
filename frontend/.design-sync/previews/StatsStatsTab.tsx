import { StatsStatsTab } from "frontend";

// StatsTab is data-coupled: it reads the skins index, the user's owned skins
// and the check-in record through TanStack Query, and renders StatsTabSkeleton
// until the skins index resolves. Preview server functions are stubbed, so the
// honest card for this container is its loading state — the roster and static
// operator props below are real-shaped so the memoized stats still compute.
const PHASES_6 = [{ maxLevel: 50 }, { maxLevel: 80 }, { maxLevel: 90 }];

const op = (id: string, name: string, profession: string, subProfessionId: string) => ({
    id,
    name,
    rarity: "TIER_6",
    profession,
    subProfessionId,
    isNotObtainable: false,
    phases: PHASES_6,
    skills: [{ skillId: `${id}_s1` }, { skillId: `${id}_s2` }, { skillId: `${id}_s3` }],
    modules: [{ uniEquipId: `uniequip_002_${id.slice(-5)}`, type: "ADVANCED" }],
    potentialRanks: [{}, {}, {}, {}, {}],
});

const entry = (operatorId: string, elite: number, level: number, mastery: number, moduleLevel: number) => ({
    user_id: "1000123456",
    operator_id: operatorId,
    elite,
    level,
    exp: 0,
    potential: 1,
    skill_level: 7,
    favor_point: 25_570,
    skin_id: null,
    default_skill: 0,
    voice_lan: "JP",
    current_equip: null,
    current_tmpl: null,
    obtained_at: 1_690_000_000,
    masteries: [0, 1, 2].map((index) => ({ index, mastery })),
    modules: [{ id: `uniequip_002_${operatorId.slice(-5)}`, level: moduleLevel, locked: false }],
});

const OPERATORS = [op("char_4064_mlynar", "Mlynar", "WARRIOR", "sword"), op("char_293_thorns", "Thorns", "WARRIOR", "artsfghter"), op("char_350_surtr", "Surtr", "WARRIOR", "musha"), op("char_4087_ines", "Ines", "PIONEER", "bearer"), op("char_003_kalts", "Kal'tsit", "MEDIC", "physician"), op("char_2015_dusk", "Dusk", "CASTER", "splashcaster")];

const ROSTER = [entry("char_4064_mlynar", 2, 90, 3, 3), entry("char_293_thorns", 2, 90, 3, 2), entry("char_350_surtr", 2, 88, 2, 3), entry("char_4087_ines", 2, 84, 1, 1), entry("char_003_kalts", 1, 70, 0, 0), entry("char_2015_dusk", 2, 90, 3, 3)];

export const LoadingSkinsIndex = () => <StatsStatsTab nonDefaultSkinCount={63} operatorsStatic={OPERATORS} roster={ROSTER} server="en" uid="1000123456" />;
