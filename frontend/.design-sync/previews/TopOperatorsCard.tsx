import { TopOperatorsCard } from "frontend";

const PHASES_6 = [{ maxLevel: 50 }, { maxLevel: 80 }, { maxLevel: 90 }];
const PHASES_5 = [{ maxLevel: 50 }, { maxLevel: 70 }, { maxLevel: 80 }];
const POTENTIAL_RANKS = [{}, {}, {}, {}, {}];

/** Static operator record — only the fields the completeness grader reads. */
const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, moduleIds: string[]) => ({
    id,
    name,
    rarity: `TIER_${rarity}`,
    profession,
    subProfessionId,
    isNotObtainable: false,
    phases: rarity === 6 ? PHASES_6 : PHASES_5,
    skills: [{ skillId: `${id}_s1` }, { skillId: `${id}_s2` }, { skillId: `${id}_s3` }],
    modules: [{ uniEquipId: `uniequip_001_${id.slice(-4)}`, type: "INITIAL" }, ...moduleIds.map((m) => ({ uniEquipId: m, type: "ADVANCED" }))],
    potentialRanks: POTENTIAL_RANKS,
});

/** One row of the imported roster table. */
const entry = (operatorId: string, elite: number, level: number, masteries: number[], modules: { id: string; level: number }[], potential: number, favorPoint: number, skinId: string | null = null) => ({
    user_id: "1000123456",
    operator_id: operatorId,
    elite,
    level,
    exp: 0,
    potential,
    skill_level: 7,
    favor_point: favorPoint,
    skin_id: skinId,
    default_skill: 0,
    voice_lan: "JP",
    current_equip: modules[0]?.id ?? null,
    current_tmpl: null,
    obtained_at: 1_690_000_000,
    masteries: masteries.map((mastery, index) => ({ index, mastery })),
    modules: modules.map((m) => ({ ...m, locked: false })),
});

const OPERATORS = [
    op("char_4064_mlynar", "Mlynar", 6, "WARRIOR", "sword", ["uniequip_002_mlynar"]),
    op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", ["uniequip_002_texas2"]),
    op("char_2015_dusk", "Dusk", 6, "CASTER", "splashcaster", ["uniequip_002_dusk"]),
    op("char_293_thorns", "Thorns", 6, "WARRIOR", "artsfghter", ["uniequip_002_thorns"]),
    op("char_350_surtr", "Surtr", 6, "WARRIOR", "musha", ["uniequip_002_surtr"]),
    op("char_4087_ines", "Ines", 6, "PIONEER", "bearer", ["uniequip_002_ines"]),
    op("char_003_kalts", "Kal'tsit", 6, "MEDIC", "physician", ["uniequip_002_kalts"]),
    op("char_1013_chen2", "Ch'en the Holungday", 6, "SNIPER", "fastshot", ["uniequip_002_chen2"]),
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "centurion", ["uniequip_002_skadi"]),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", ["uniequip_002_texas"]),
    op("char_017_huang", "Blaze", 5, "WARRIOR", "centurion", ["uniequip_002_huang"]),
    op("char_4045_heidi", "Heidi", 5, "SUPPORT", "bard", ["uniequip_002_heidi"]),
];

const ROSTER = [
    entry("char_4064_mlynar", 2, 90, [3, 3, 3], [{ id: "uniequip_002_mlynar", level: 3 }], 5, 25_570, "char_4064_mlynar@epoque#28"),
    entry("char_1028_texas2", 2, 90, [3, 3, 3], [{ id: "uniequip_002_texas2", level: 3 }], 5, 25_570, "char_1028_texas2@iteration#1"),
    entry("char_2015_dusk", 2, 90, [3, 3, 2], [{ id: "uniequip_002_dusk", level: 3 }], 4, 25_570),
    entry("char_293_thorns", 2, 90, [3, 3, 3], [{ id: "uniequip_002_thorns", level: 2 }], 3, 24_100),
    entry("char_350_surtr", 2, 88, [3, 2, 3], [{ id: "uniequip_002_surtr", level: 3 }], 2, 22_800),
    entry("char_4087_ines", 2, 90, [3, 3, 1], [{ id: "uniequip_002_ines", level: 2 }], 1, 19_400),
    entry("char_003_kalts", 2, 80, [3, 1, 0], [{ id: "uniequip_002_kalts", level: 1 }], 2, 17_200, "char_003_kalts@boc#6"),
    entry("char_1013_chen2", 2, 76, [3, 0, 0], [{ id: "uniequip_002_chen2", level: 1 }], 1, 12_600),
    entry("char_263_skadi", 1, 62, [0, 0, 0], [], 1, 8_400),
    entry("char_102_texas", 2, 80, [3, 3, 3], [{ id: "uniequip_002_texas", level: 3 }], 5, 25_570, "char_102_texas@winter#1"),
    entry("char_017_huang", 2, 70, [2, 1, 0], [{ id: "uniequip_002_huang", level: 1 }], 3, 14_300),
    entry("char_4045_heidi", 1, 55, [0, 0, 0], [], 1, 6_100),
];

const EARLY_ROSTER = [entry("char_102_texas", 1, 55, [0, 0, 0], [], 2, 7_400), entry("char_017_huang", 1, 48, [0, 0, 0], [], 1, 5_200), entry("char_4045_heidi", 0, 42, [0, 0, 0], [], 1, 2_900), entry("char_263_skadi", 0, 36, [0, 0, 0], [], 1, 1_800)];

const MAXED_ROSTER = ROSTER.slice(0, 4).map((e) => ({ ...e, elite: 2, level: 90, potential: 5, favor_point: 25_570, masteries: [0, 1, 2].map((index) => ({ index, mastery: 3 })), modules: e.modules.map((m) => ({ ...m, level: 3 })) }));

export const DeepRoster = () => (
    <div className="w-full max-w-3xl">
        <TopOperatorsCard operatorsStatic={OPERATORS} roster={ROSTER} />
    </div>
);

export const EarlyAccount = () => (
    <div className="w-full max-w-3xl">
        <TopOperatorsCard operatorsStatic={OPERATORS} roster={EARLY_ROSTER} />
    </div>
);

export const FullyBuilt = () => (
    <div className="w-full max-w-3xl">
        <TopOperatorsCard operatorsStatic={OPERATORS} roster={MAXED_ROSTER} />
    </div>
);
