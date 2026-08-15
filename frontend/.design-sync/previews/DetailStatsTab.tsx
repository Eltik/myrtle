import { DetailStatsTab } from "frontend";

// `stats.levels` is Hypergryph's per-difficulty stat table. When the handbook's
// TITLE entries line up 1:1 with the levels the tab labels the blocks with the
// narrative form names; otherwise it falls back to "Level N".
const base = {
    attackType: null,
    ability: null,
    isInvalidKilled: false,
    overrideKillCntInfos: null,
    hideInHandbook: false,
    hideInStage: true,
    linkEnemies: [],
    invisibleDetail: false,
    portrait: null,
    abilityList: [],
    enemyTags: null as string[] | null,
    immunities: { stun: false, silence: false, sleep: false, frozen: false, levitate: false },
    flatStats: { maxHp: 0, atk: 0, def: 0, res: 0, aspd: 100, ms: 0, weight: 0, baseAttackTime: 0, hpRecoveryPerSec: 0 },
    applyWay: "MELEE",
    race: null,
};

const level = (n: number, a: Record<string, number | boolean>) => ({
    level: n,
    attributes: a,
    applyWay: "ALL",
    motion: "WALK",
    rangeRadius: null,
    lifePointReduce: 2,
    skills: [],
});

const skulsr = {
    ...base,
    enemyId: "enemy_1500_skulsr",
    enemyIndex: "SS",
    sortId: 1297,
    name: "Skullshatterer",
    enemyLevel: "BOSS",
    damageType: ["PHYSIC"],
    description: "One of Reunion's squad leaders who serves in the Assault Squad.",
    abilityList: [
        { text: "Launches grenades when not being blocked, significantly reducing the DEF of the target and surrounding allies for a short period of time", textFormat: "NORMAL" },
        { text: "Gains significantly increased ATK when HP is under 50%", textFormat: "NORMAL" },
    ],
    stats: {
        levels: [
            level(0, { maxHp: 10500, atk: 1000, def: 150, magicResistance: 30, moveSpeed: 0.6, attackSpeed: 100, baseAttackTime: 3, massLevel: 5, hpRecoveryPerSec: 0, stunImmune: false, silenceImmune: true, sleepImmune: false, frozenImmune: false, levitateImmune: false }),
            level(1, { maxHp: 30000, atk: 1300, def: 240, magicResistance: 30, moveSpeed: 0.6, attackSpeed: 100, baseAttackTime: 3, massLevel: 5, hpRecoveryPerSec: 0, stunImmune: false, silenceImmune: true, sleepImmune: false, frozenImmune: false, levitateImmune: false }),
            level(2, { maxHp: 80000, atk: 1950, def: 400, magicResistance: 30, moveSpeed: 0.6, attackSpeed: 100, baseAttackTime: 3, massLevel: 5, hpRecoveryPerSec: 0, stunImmune: false, silenceImmune: true, sleepImmune: false, frozenImmune: false, levitateImmune: false }),
        ],
    },
};

const signora = {
    ...base,
    enemyId: "enemy_1560_cnvlap",
    enemyIndex: "LOM",
    enemyTags: [],
    sortId: 1356,
    name: "'La Signora del Carnevale'",
    enemyLevel: "BOSS",
    damageType: ["PHYSIC", "MAGIC"],
    description: "La Signora del Carnevale finally takes the stage.",
    abilityList: [
        { text: "Unblockable; is treated as an aerial unit when standing atop the landship", textFormat: "NORMAL" },
        { text: "First Form", textFormat: "TITLE" },
        { text: "Immobile; attacks deal Arts damage", textFormat: "NORMAL" },
        { text: "Second Form", textFormat: "TITLE" },
        { text: "Attacks deal Physical damage to 2 targets simultaneously", textFormat: "NORMAL" },
    ],
    stats: {
        levels: [
            level(0, { maxHp: 60000, atk: 425, def: 500, magicResistance: 30, moveSpeed: 1, attackSpeed: 100, baseAttackTime: 5, massLevel: 5, hpRecoveryPerSec: 0, stunImmune: true, silenceImmune: true, sleepImmune: false, frozenImmune: false, levitateImmune: true }),
            level(1, { maxHp: 90000, atk: 600, def: 500, magicResistance: 30, moveSpeed: 1, attackSpeed: 100, baseAttackTime: 5, massLevel: 5, hpRecoveryPerSec: 0, stunImmune: true, silenceImmune: true, sleepImmune: false, frozenImmune: false, levitateImmune: true }),
        ],
    },
};

const lancer = {
    ...base,
    enemyId: "enemy_1072_dlancer_2",
    enemyIndex: "S10",
    enemyTags: ["sarkaz"],
    sortId: 137,
    name: "Sarkaz Lancer Leader",
    enemyLevel: "ELITE",
    damageType: ["PHYSIC"],
    race: "Sarkaz",
    description: "A Sarkaz, more threatening than a normal Sarkaz Lancer.",
    abilityList: [{ text: "Gradually accelerates when moving, and deals additional damage on the first attack after being blocked based upon Movement Speed.", textFormat: "NORMAL" }],
    stats: {
        levels: [level(0, { maxHp: 8000, atk: 500, def: 230, magicResistance: 40, moveSpeed: 0.3, attackSpeed: 100, baseAttackTime: 4, massLevel: 3, hpRecoveryPerSec: 0, stunImmune: false, silenceImmune: false, sleepImmune: false, frozenImmune: false, levitateImmune: false })],
    },
};

const saint = {
    ...base,
    enemyId: "enemy_1567_pope",
    enemyIndex: "ST",
    enemyTags: [],
    sortId: 1363,
    name: "'Saint'",
    enemyLevel: "BOSS",
    damageType: ["PHYSIC", "MAGIC"],
    description: "The First Saint of Laterano.",
    abilityList: [
        { text: "First Form", textFormat: "TITLE" },
        { text: "High DEF", textFormat: "NORMAL" },
        { text: "Second Form", textFormat: "TITLE" },
        { text: "Attacks deal Arts damage 3 times, prioritizing different targets", textFormat: "NORMAL" },
    ],
    stats: {
        levels: [level(0, { maxHp: 20000, atk: 600, def: 1000, magicResistance: 60, moveSpeed: 0.6, attackSpeed: 100, baseAttackTime: 6, massLevel: 10, hpRecoveryPerSec: 0, stunImmune: true, silenceImmune: true, sleepImmune: true, frozenImmune: true, levitateImmune: true })],
    },
};

export const MultiLevelBoss = () => (
    <div className="w-full max-w-2xl">
        <DetailStatsTab enemy={skulsr} />
    </div>
);

export const NarrativeForms = () => (
    <div className="w-full max-w-2xl">
        <DetailStatsTab enemy={signora} />
    </div>
);

export const SinglePhase = () => (
    <div className="w-full max-w-2xl">
        <DetailStatsTab enemy={lancer} />
    </div>
);

export const FormsWithoutStatBlocks = () => (
    <div className="w-full max-w-2xl">
        <DetailStatsTab enemy={saint} />
    </div>
);
