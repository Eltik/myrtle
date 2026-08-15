import { EnemyHero } from "frontend";

// The masthead of /enemies/$id: portrait tile tinted by threat tier, name,
// index · race · attack range, and one pill per damage type.
const base = {
    enemyTags: null as string[] | null,
    attackType: null,
    ability: null,
    isInvalidKilled: false,
    overrideKillCntInfos: null,
    hideInHandbook: false,
    hideInStage: true,
    abilityList: [],
    linkEnemies: [],
    invisibleDetail: false,
    stats: null,
    immunities: { stun: false, silence: false, sleep: false, frozen: false, levitate: false },
    flatStats: { maxHp: 0, atk: 0, def: 0, res: 0, aspd: 100, ms: 0, weight: 0, baseAttackTime: 0, hpRecoveryPerSec: 0 },
};

const slug = {
    ...base,
    enemyId: "enemy_1007_slime",
    enemyIndex: "B1",
    enemyTags: ["infection"],
    sortId: 1,
    name: "Originium Slug",
    enemyLevel: "NORMAL",
    description: "An infected wild creature.",
    damageType: ["PHYSIC"],
    portrait: "/textures/spritepack/icon_enemies_0/enemy_1007_slime.png",
    applyWay: "MELEE",
    race: "Infected Creature",
};

const crossbow = {
    ...base,
    enemyId: "enemy_1012_dcross",
    enemyIndex: "S3",
    enemyTags: ["sarkaz"],
    sortId: 45,
    name: "Sarkaz Crossbowman",
    enemyLevel: "ELITE",
    description: "A mercenary from Sarkaz.",
    damageType: ["PHYSIC"],
    portrait: "/textures/spritepack/icon_enemies_0/enemy_1012_dcross.png",
    applyWay: "RANGED",
    race: "Sarkaz",
};

const degen = {
    ...base,
    enemyId: "enemy_1525_blkswb",
    enemyIndex: "BJ",
    sortId: 1321,
    name: "Degenbrecher",
    enemyLevel: "BOSS",
    description: "Three-time champion of the Kazimierz Major.",
    damageType: ["PHYSIC"],
    portrait: "/textures/spritepack/icon_enemies_2/enemy_1525_blkswb.png",
    applyWay: "MELEE",
    race: null,
};

const mandra = {
    ...base,
    enemyId: "enemy_1523_mandra",
    enemyIndex: "MD",
    sortId: 1320,
    name: "Mandragora",
    enemyLevel: "BOSS",
    description: "One of the leaders of the Dublinn forces.",
    damageType: ["PHYSIC", "MAGIC"],
    portrait: "/textures/spritepack/icon_enemies_2/enemy_1523_mandra.png",
    applyWay: "MELEE",
    race: null,
};

export const BossHero = () => (
    <div className="w-full max-w-3xl">
        <EnemyHero enemy={degen} />
    </div>
);

export const EliteHero = () => (
    <div className="w-full max-w-3xl">
        <EnemyHero enemy={crossbow} />
    </div>
);

export const NormalHero = () => (
    <div className="w-full max-w-3xl">
        <EnemyHero enemy={slug} />
    </div>
);

export const DualDamageType = () => (
    <div className="w-full max-w-3xl">
        <EnemyHero enemy={mandra} />
    </div>
);
