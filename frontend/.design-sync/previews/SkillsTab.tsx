import { SkillsTab } from "frontend";

// Enemy "skills" are engine prefab triggers, not operator skills: a prefab key,
// a priority, and the cooldown/SP economy that drives it. When every level ships
// the same kit the tab collapses to one list; when they differ it splits per form.
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

const skill = (prefabKey: string, priority: number, cooldown: number, initCooldown: number, spCost: number) => ({ prefabKey, priority, cooldown, initCooldown, spCost, blackboard: [] });

const level = (n: number, maxHp: number, atk: number, def: number, skills: ReturnType<typeof skill>[]) => ({
    level: n,
    attributes: { maxHp, atk, def, magicResistance: 30, moveSpeed: 0.6, attackSpeed: 100, baseAttackTime: 3, massLevel: 5, hpRecoveryPerSec: 0, stunImmune: true, silenceImmune: true, sleepImmune: false, frozenImmune: false, levitateImmune: true },
    applyWay: "ALL",
    motion: "WALK",
    rangeRadius: null,
    lifePointReduce: 2,
    skills,
});

const DEGEN_KIT = [skill("CircleAttack", 0, 15, 15, 0), skill("Blink", 0, 30, 0, 0), skill("CircleAttack2", 0, 15, 15, 0), skill("Blink2", 0, 30, 0, 0)];

const degen = {
    ...base,
    enemyId: "enemy_1525_blkswb",
    enemyIndex: "BJ",
    sortId: 1321,
    name: "Degenbrecher",
    enemyLevel: "BOSS",
    damageType: ["PHYSIC"],
    description: "Three-time champion of the Kazimierz Major.",
    abilityList: [
        { text: "First Form", textFormat: "TITLE" },
        { text: "Attacks against the blocker ignore a certain amount of DEF", textFormat: "NORMAL" },
        { text: "Second Form", textFormat: "TITLE" },
        { text: "Attacks ignore more of the target's DEF and strike twice", textFormat: "NORMAL" },
    ],
    stats: { levels: [level(0, 28000, 900, 600, DEGEN_KIT), level(1, 45000, 1000, 800, DEGEN_KIT)] },
};

const antagonizer = {
    ...base,
    enemyId: "enemy_1532_minima",
    enemyIndex: "CST",
    enemyTags: ["drone"],
    sortId: 1328,
    name: "'Materialist Antagonizer'",
    enemyLevel: "BOSS",
    damageType: ["MAGIC"],
    race: "Drone",
    description: "Construction machinery attempting to protect Stitch Canvas.",
    abilityList: [
        { text: "Initial Form", textFormat: "TITLE" },
        { text: "Aerial unit; unblockable", textFormat: "NORMAL" },
        { text: "Stitch Form", textFormat: "TITLE" },
        { text: "Will not attack, and cannot be blocked", textFormat: "NORMAL" },
    ],
    stats: {
        levels: [level(0, 55000, 500, 600, [skill("RefreshShield", 2, 80, 80, 0), skill("S2", 0, 31, 20, 0), skill("S3", 1, 31, 35, 0)]), level(1, 75000, 700, 800, [skill("RefreshShield", 2, 40, 40, 0)])],
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
    stats: { levels: [level(0, 20000, 600, 1000, [skill("M1Attack", 0, 0, 0, 0), skill("M1RangeProjectile", 1, 20, 13, 0), skill("M2Attack", 0, 0, 0, 0), skill("M2RangeProjectile", 1, 20, 13, 0)])] },
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
    stats: { levels: [level(0, 8000, 500, 230, [])] },
};

export const SingleFormKit = () => (
    <div className="w-full max-w-2xl">
        <SkillsTab enemy={saint} />
    </div>
);

export const IdenticalAcrossForms = () => (
    <div className="w-full max-w-2xl">
        <SkillsTab enemy={degen} />
    </div>
);

export const PerFormKits = () => (
    <div className="w-full max-w-2xl">
        <SkillsTab enemy={antagonizer} />
    </div>
);

export const NoSkillData = () => (
    <div className="w-full max-w-2xl">
        <SkillsTab enemy={lancer} />
    </div>
);
