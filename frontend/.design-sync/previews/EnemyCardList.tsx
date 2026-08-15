import { EnemyCardList } from "frontend";

// Real handbook rows (api.myrtle.moe /static/enemies), enriched the way
// `enrichEnemy` does before the list renders them.
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
    flatStats: { maxHp: 550, atk: 130, def: 0, res: 0, aspd: 100, ms: 1, weight: 0, baseAttackTime: 1.7, hpRecoveryPerSec: 0 },
    applyWay: "MELEE",
    race: "Infected Creature",
};

const sentinel = {
    ...base,
    enemyId: "enemy_1073_dscout",
    enemyIndex: "S11",
    enemyTags: ["sarkaz"],
    sortId: 138,
    name: "Sarkaz Sentinel",
    enemyLevel: "NORMAL",
    description: "A Sarkaz mercenary responsible for scouting.",
    damageType: ["NO_DAMAGE"],
    portrait: "/textures/spritepack/icon_enemies_0/enemy_1073_dscout.png",
    flatStats: { maxHp: 4000, atk: 0, def: 100, res: 30, aspd: 100, ms: 0.5, weight: 1, baseAttackTime: 1, hpRecoveryPerSec: 0 },
    applyWay: "NONE",
    race: "Sarkaz",
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
    flatStats: { maxHp: 6000, atk: 450, def: 200, res: 50, aspd: 100, ms: 0.8, weight: 2, baseAttackTime: 3, hpRecoveryPerSec: 0 },
    applyWay: "RANGED",
    race: "Sarkaz",
};

const lancer = {
    ...base,
    enemyId: "enemy_1072_dlancer_2",
    enemyIndex: "S10",
    enemyTags: ["sarkaz"],
    sortId: 44,
    name: "Sarkaz Lancer Leader",
    enemyLevel: "ELITE",
    description: "A veteran Sarkaz lancer.",
    damageType: ["PHYSIC"],
    portrait: "/textures/spritepack/icon_enemies_0/enemy_1072_dlancer_2.png",
    flatStats: { maxHp: 8000, atk: 500, def: 230, res: 40, aspd: 100, ms: 0.3, weight: 3, baseAttackTime: 4, hpRecoveryPerSec: 0 },
    applyWay: "MELEE",
    race: "Sarkaz",
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
    flatStats: { maxHp: 50000, atk: 640, def: 520, res: 35, aspd: 100, ms: 0.4, weight: 7, baseAttackTime: 1.8, hpRecoveryPerSec: 0 },
    applyWay: "MELEE",
    race: null,
};

const skulsr = {
    ...base,
    enemyId: "enemy_1500_skulsr",
    enemyIndex: "SS",
    sortId: 1297,
    name: "Skullshatterer",
    enemyLevel: "BOSS",
    description: "One of Reunion's squad leaders.",
    damageType: ["PHYSIC"],
    portrait: "/textures/spritepack/icon_enemies_2/enemy_1500_skulsr.png",
    flatStats: { maxHp: 10500, atk: 1000, def: 150, res: 30, aspd: 100, ms: 0.6, weight: 5, baseAttackTime: 3, hpRecoveryPerSec: 0 },
    applyWay: "MELEE",
    race: null,
};

// Scoped the way `computeStatMaxByLevel` does — the tier maxima of the cohort
// currently on screen, so the HP meter reads against its own threat tier.
const statMax = {
    NORMAL: { hp: 4500, atk: 800, def: 400 },
    ELITE: { hp: 12000, atk: 1200, def: 700 },
    BOSS: { hp: 60000, atk: 3000, def: 1000 },
};

// The column rail the list view renders above the rows.
const HeaderRail = () => (
    <div className="grid items-center gap-3.5 border-border/60 border-b px-3.5 pb-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em]" style={{ gridTemplateColumns: "56px 1fr 90px 144px 110px" }}>
        <span />
        <span>Name</span>
        <span>Threat</span>
        <span>Damage</span>
        <span className="text-right">HP</span>
    </div>
);

export const ThreatSpread = () => (
    <div className="flex w-full flex-col gap-1">
        <EnemyCardList enemy={slug} statMax={statMax} />
        <EnemyCardList enemy={crossbow} statMax={statMax} />
        <EnemyCardList enemy={mandra} statMax={statMax} />
    </div>
);

export const WithColumnHeader = () => (
    <div className="flex w-full flex-col gap-1">
        <HeaderRail />
        <EnemyCardList enemy={sentinel} statMax={statMax} />
        <EnemyCardList enemy={lancer} statMax={statMax} />
        <EnemyCardList enemy={skulsr} statMax={statMax} />
    </div>
);

export const NormalTier = () => (
    <div className="flex w-full flex-col gap-1">
        <EnemyCardList enemy={slug} statMax={statMax} />
        <EnemyCardList enemy={sentinel} statMax={statMax} />
    </div>
);
