import { BaseOptimizer } from "frontend";

// BaseOptimizer is the data-coupled container of the RIIC planner: it owns
// `useOptimizer(uid)`, which fans out to the base catalog, the profile's synced
// layout, the evaluation and the rotation plan through server functions. In
// the design bundle those are stubbed and reject at once, so after one frame
// of skeleton the container settles into the state it shows a profile whose
// base has never synced - which is its honest preview. The roster and static
// operator props are real-shaped; nothing downstream reads them before a
// layout exists.

const PHASES_6 = [{ maxLevel: 50 }, { maxLevel: 80 }, { maxLevel: 90 }];

const op = (id: string, name: string, profession: string, subProfessionId: string, baseSkills: Record<string, unknown>[] = []) => ({
    id,
    name,
    rarity: "TIER_6",
    profession,
    subProfessionId,
    isNotObtainable: false,
    phases: PHASES_6,
    skills: [{ skillId: `${id}_s1` }, { skillId: `${id}_s2` }, { skillId: `${id}_s3` }],
    modules: [],
    potentialRanks: [{}, {}, {}, {}, {}],
    baseSkills,
});

const entry = (operatorId: string, elite: number, level: number) => ({
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
    masteries: [0, 1, 2].map((index) => ({ index, mastery: 0 })),
    modules: [],
});

const OPERATORS_STATIC = [
    op("char_272_strong", "Jaye", "SPECIAL", "merchant", [
        { buffId: "trade_ord_limit_diff[000]", buffName: "Street Economics", description: "When this Operator is assigned to a Trading Post, increases order acquisition efficiency by <@cc.vup>+4%</> for every difference of <@cc.vup>1</> order(s) between the current number of orders and the maximum number of orders", roomType: "TRADING", efficiency: 0, targets: [], skillIcon: "bskill_tra_limit_diff", unlockElite: 0, unlockLevel: 1, slot: 0 },
    ]),
    op("char_254_vodfox", "Shamare", "SUPPORT", "bard"),
    op("char_190_clour", "Vermeil", "SNIPER", "fastshot"),
    op("char_002_amiya", "Amiya", "CASTER", "corecaster"),
];

const ROSTER = [entry("char_272_strong", 2, 60), entry("char_254_vodfox", 1, 55), entry("char_190_clour", 2, 40), entry("char_002_amiya", 2, 70)];

/** The state the container settles into when the profile has no synced base. */
export const NoBaseData = () => <BaseOptimizer operatorsStatic={OPERATORS_STATIC} roster={ROSTER} uid="1000123456" />;
