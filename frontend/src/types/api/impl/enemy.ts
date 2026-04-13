// Enemy types

// ============================================================================
// Enums
// ============================================================================

export type EnemyLevel = "NORMAL" | "ELITE" | "BOSS";

export type DamageType = "PHYSIC" | "MAGIC" | "NO_DAMAGE" | "HEAL";

// ============================================================================
// Base Types
// ============================================================================

export interface StatRange {
    min: number;
    max: number;
}

export interface EnemyInfoList {
    classLevel: string;
    attack: StatRange;
    def: StatRange;
    magicRes: StatRange;
    maxHP: StatRange;
    moveSpeed: StatRange;
    attackSpeed: StatRange;
    enemyDamageRes: StatRange;
    enemyRes: StatRange;
}

export interface RaceData {
    id: string;
    raceName: string;
    sortId: number;
}

export interface AbilityInfo {
    text: string;
    textFormat: string;
}

// ============================================================================
// Enriched Stats Types (from enemy_database.json)
// ============================================================================

export interface SkillBlackboardEntry {
    key: string;
    value: number;
    valueStr: string | null;
}

export interface EnemySkill {
    prefabKey: string;
    priority: number;
    cooldown: number;
    initCooldown: number;
    spCost: number;
    blackboard: SkillBlackboardEntry[];
}

export interface EnemyAttributes {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
    moveSpeed: number;
    attackSpeed: number;
    baseAttackTime: number;
    massLevel: number;
    hpRecoveryPerSec: number;
    stunImmune: boolean;
    silenceImmune: boolean;
    sleepImmune: boolean;
    frozenImmune: boolean;
    levitateImmune: boolean;
}

export type ApplyWay = "MELEE" | "RANGED" | "NONE" | null;

export interface EnemyLevelStats {
    level: number;
    attributes: EnemyAttributes;
    applyWay: ApplyWay;
    motion: string | null;
    rangeRadius: number | null;
    lifePointReduce: number;
    skills: EnemySkill[];
}

export interface EnemyStats {
    levels: EnemyLevelStats[];
}

// ============================================================================
// Main Enemy Type
// ============================================================================

export interface Enemy {
    enemyId: string;
    enemyIndex: string;
    enemyTags: string[] | null;
    sortId: number;
    name: string;
    enemyLevel: EnemyLevel;
    description: string;
    attackType: string | null;
    ability: string | null;
    isInvalidKilled: boolean;
    overrideKillCntInfos: unknown | null;
    hideInHandbook: boolean;
    hideInStage: boolean;
    abilityList: AbilityInfo[];
    linkEnemies: string[];
    damageType: DamageType[];
    invisibleDetail: boolean;
    // Enriched fields from enemy_database.json
    stats: EnemyStats | null;
    // Enemy portrait/icon path
    portrait: string | null;
}

// ============================================================================
// Container Types
// ============================================================================

export interface EnemyHandbook {
    levelInfoList: EnemyInfoList[];
    enemyData: Record<string, Enemy>;
    raceData: Record<string, RaceData>;
}

// ============================================================================
// API Response Types
// ============================================================================

export interface EnemiesResponse {
    enemies: Enemy[];
    hasMore: boolean;
    nextCursor: string | null;
    total: number;
}

export interface EnemyResponse {
    enemy: Enemy;
}

export interface RacesResponse {
    races: Record<string, RaceData>;
}

export interface LevelInfoResponse {
    levels: EnemyInfoList[];
}
