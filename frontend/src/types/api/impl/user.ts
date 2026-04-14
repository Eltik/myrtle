// ==================== v3 Backend Types ====================

/** v3 backend user profile (from v_user_profile database view) */
export interface UserProfile {
    id: string;
    uid: string;
    nickname: string | null;
    level: number | null;
    avatar_id: string | null;
    secretary: string | null;
    secretary_skin_id: string | null;
    resume_id: string | null;
    role: string;
    server: string;
    // Score
    total_score: number | null;
    grade: string | null;
    // Settings
    public_profile: boolean | null;
    store_gacha: boolean | null;
    share_stats: boolean | null;
    // Status
    exp: number | null;
    orundum: number | null;
    lmd: number | null;
    sanity: number | null;
    max_sanity: number | null;
    gacha_tickets: number | null;
    ten_pull_tickets: number | null;
    monthly_sub_end: number | null;
    register_ts: number | null;
    last_online_ts: number | null;
    resume: string | null;
    friend_num_limit: number | null;
    // Counts
    operator_count: number | null;
    item_count: number | null;
    skin_count: number | null;
}

/** v3 roster entry (from v_user_roster database view) */
export interface RosterEntry {
    user_id: string;
    operator_id: string;
    elite: number;
    level: number;
    exp: number;
    potential: number;
    skill_level: number;
    favor_point: number;
    skin_id: string | null;
    default_skill: number | null;
    voice_lan: string | null;
    current_equip: string | null;
    current_tmpl: string | null;
    obtained_at: number | null;
    masteries: RosterMastery[];
    modules: RosterModule[];
}

/** Mastery data from roster jsonb */
export interface RosterMastery {
    skill_id: string;
    specialize_level: number;
}

/** Module data from roster jsonb */
export interface RosterModule {
    equip_id: string;
    level: number;
}

// ==================== Server Types ====================

export type ApiServer = "en" | "jp" | "cn" | "kr";
export type YostarServer = "en" | "jp" | "kr";
export type AKServer = "en" | "jp" | "kr" | "cn" | "bili" | "tw";
export type ArknightsServer = AKServer; // Alias for compatibility

// ==================== Troop / Character Types ====================

export interface Troop {
    curCharInstId: number;
    curSquadCount: number;
    squads: Record<string, unknown>;
    chars: Record<string, CharacterData>;
    charGroup: Record<string, CharGroup>;
    charMission: Record<string, Record<string, number>>;
    addon?: unknown;
}

/**
 * Character data from game sync.
 * @deprecated Use RosterEntry for v3 backend data.
 */
export interface CharacterData {
    instId: number;
    charId: string;
    favorPoint: number;
    potentialRank: number;
    mainSkillLvl: number;
    skin: string;
    level: number;
    exp: number;
    evolvePhase: number;
    defaultSkillIndex: number;
    gainTime: number;
    currentTmpl: string | null;
    tmpl: Record<string, CharacterTemplate> | null;
    skills: CharacterSkill[];
    voiceLan: string;
    currentEquip: string | null;
    equip: Record<string, EquipData>;
    starMark?: number;
    static?: CharacterStatic | null;
}

export interface CharacterTemplate {
    equip: Record<string, EquipData>;
    skills: CharacterSkill[];
    skinId: string;
    currentEquip: string;
    defaultSkillIndex: number;
}

export interface CharacterSkill {
    skillId: string;
    unlock: number;
    state: number;
    specializeLevel: number;
    completeUpgradeTime: number;
    static?: UserSkillStatic | null;
}

export interface EquipData {
    hide: number;
    locked: number;
    level: number;
}

export interface CharGroup {
    favorPoint: number;
}

export interface Avatar {
    type: string | null;
    id: string | null;
}

// ==================== Inventory Types ====================

export interface InventoryItem {
    amount: number;
    buildingProductList?: BuildingProductEntry[];
    classifyType?: string;
    description?: string;
    hideInItemGet?: boolean;
    iconId?: string;
    itemId: string;
    itemType?: string;
    name?: string;
    obtainApproach?: string | null;
    overrideBkg?: string | null;
    rarity?: string;
    sortId?: number;
    stackIconId?: string;
    stageDropList?: StageDropEntry[];
    usage?: string;
    voucherRelateList?: unknown[] | null;
    image?: string;
}

export interface BuildingProductEntry {
    formulaId: string;
    roomType: string;
}

export interface StageDropEntry {
    occPer: string;
    stageId: string;
}

// ==================== Character Static Types ====================
// Prefixed with "User" to avoid conflicts with game data table types in operator.ts, skill.ts, etc.

/** Static operator data added by backend formatUser */
export interface CharacterStatic {
    id: string;
    name: string;
    appellation?: string;
    description?: string;
    rarity: string; // "TIER_1" through "TIER_6"
    profession: string; // "CASTER", "GUARD", etc.
    subProfessionId?: string;
    position: string; // "RANGED" or "MELEE"
    nationId?: string | null;
    groupId?: string | null;
    teamId?: string | null;
    displayNumber?: string;
    portrait?: string;
    skin?: string;
    tagList?: string[];
    isSpChar?: boolean;
    isNotObtainable?: boolean;
    itemUsage?: string;
    itemDesc?: string;
    itemObtainApproach?: string;
    maxPotentialLevel?: number;
    canUseGeneralPotentialItem?: boolean;
    canUseActivityPotentialItem?: boolean;
    potentialItemId?: string | null;
    classicPotentialItemId?: string | null;
    activityPotentialItemId?: string | null;
    displayTokenDict?: Record<string, unknown> | null;
    trait?: UserCharacterTrait | null;
    phases?: UserCharacterPhase[];
    skills?: UserCharacterStaticSkill[];
    talents?: UserCharacterTalent[];
    potentialRanks?: UserPotentialRank[];
    favorKeyFrames?: UserFavorKeyFrame[];
    allSkillLevelUp?: UserSkillLevelUpCost[];
    modules?: UserCharacterModule[];
    handbook?: UserCharacterHandbook;
    profile?: UserCharacterProfile;
    trust?: number;
    artists?: string[];
}

export interface UserCharacterTrait {
    candidates?: UserTraitCandidate[];
}

export interface UserTraitCandidate {
    additionalDescription?: string | null;
    blackboard?: UserBlackboardEntry[];
    overrideDescripton?: string | null; // Note: typo in game data
    prefabKey?: string | null;
    rangeId?: string | null;
    requiredPotentialRank?: number;
    unlockCondition?: UserUnlockCondition;
}

export interface UserBlackboardEntry {
    key: string;
    value: number;
    valueStr?: string | null;
}

export interface UserUnlockCondition {
    Level: number;
    Phase: string; // "PHASE_0", "PHASE_1", "PHASE_2"
}

export interface UserCharacterPhase {
    AttributesKeyFrames?: UserAttributeKeyFrame[];
    CharacterPrefabKey?: string;
    EvolveCost?: UserItemCost[] | null;
    MaxLevel: number;
    RangeId?: string;
}

export interface UserAttributeKeyFrame {
    Data: UserCharacterAttributes;
    Level: number;
}

export interface UserCharacterAttributes {
    Atk: number;
    AttackSpeed: number;
    BaseAttackTime: number;
    BlockCnt: number;
    Cost: number;
    Def: number;
    HpRecoveryPerSec: number;
    MagicResistance: number;
    MassLevel: number;
    MaxDeckStackCnt: number;
    MaxDeployCount: number;
    MaxHp: number;
    MoveSpeed: number;
    RespawnTime: number;
    SpRecoveryPerSec: number;
    StunImmune: boolean;
    SilenceImmune: boolean;
    SleepImmune: boolean;
    FrozenImmune: boolean;
    LevitateImmune: boolean;
    DisarmedCombatImmune: boolean;
    TauntLevel: number;
}

export interface UserItemCost {
    Count: number;
    IconId?: string;
    Id: string;
    Image?: string;
    Type_: string;
}

export interface UserCharacterStaticSkill {
    skillId: string;
    overridePrefabKey?: string | null;
    overrideTokenKey?: string | null;
    levelUpCostCond?: UserSkillLevelUpCondition[];
    static?: UserSkillStatic;
}

export interface UserSkillLevelUpCondition {
    LevelUpCost?: UserItemCost[];
    LevelUpTime?: number;
    UnlockCond?: UserUnlockCondition;
}

export interface UserCharacterTalent {
    Candidates?: UserTalentCandidate[];
}

export interface UserTalentCandidate {
    BlackBoard?: UserBlackboardEntry[];
    Description?: string;
    Name?: string;
    PrefabKey?: string | null;
    RangeId?: string | null;
    RequiredPotentialRank?: number;
    TalentIndex?: number;
    UnlockCondition?: UserUnlockCondition;
}

export interface UserPotentialRank {
    Buff?: UserPotentialBuff | null;
    Description?: string;
    Type_?: number;
}

export interface UserPotentialBuff {
    Attributes?: UserPotentialBuffAttributes;
}

export interface UserPotentialBuffAttributes {
    AbnormalFlags?: unknown | null;
    AbnormalImmunes?: unknown | null;
    AbnormalAntis?: unknown | null;
    AbnormalCombos?: unknown | null;
    AbnormalComboImmunes?: unknown | null;
    AttributeModifiers?: UserAttributeModifier[];
}

export interface UserAttributeModifier {
    AttributeType: string; // "MAX_HP", "ATK", "DEF", "MAGIC_RESISTANCE", "COST", "ATTACK_SPEED", "RESPAWN_TIME", etc.
    FormulaItem: number;
    Value: number;
    FrameIndex: number;
}

export interface UserFavorKeyFrame {
    Data: UserFavorData;
    Level: number;
}

export interface UserFavorData {
    Atk: number;
    Def: number;
    MaxHp: number;
}

export interface UserSkillLevelUpCost {
    LvlUpCost?: UserItemCost[];
    UnlockCond?: UserUnlockCondition;
}

export interface UserCharacterModule {
    id: string;
    charId: string;
    uniEquipId: string;
    uniEquipName?: string;
    uniEquipIcon?: string;
    uniEquipDesc?: string;
    uniEquipGetTime?: number;
    charEquipOrder?: number;
    tmplId?: string | null;
    type: string;
    typeName1?: string;
    typeName2?: string;
    typeIcon?: string;
    equipShiningColor?: string;
    showEvolvePhase?: string;
    unlockEvolvePhase?: string;
    showLevel?: number;
    unlockLevel?: number;
    unlockFavorPoint?: number;
    image?: string;
    data?: UserModuleData;
    missionList?: UserModuleMission[];
    itemCost?: Record<string, UserItemCost[]>;
}

export interface UserModuleData {
    phases?: UserModulePhase[];
}

export interface UserModulePhase {
    attributeBlackboard?: UserBlackboardEntry[];
    equipLevel?: number;
    parts?: UserModulePart[];
    tokenAttributeBlackboard?: Record<string, UserBlackboardEntry[]>;
}

export interface UserModulePart {
    isToken?: boolean;
    overrideTraitDataBundle?: UserModuleTraitBundle;
    addOrOverrideTalentDataBundle?: UserModuleTalentBundle;
    target?: string;
}

export interface UserModuleTraitBundle {
    candidates?: UserTraitCandidate[] | null;
}

export interface UserModuleTalentBundle {
    candidates?: UserTalentCandidate[] | null;
}

export interface UserModuleMission {
    desc?: string;
    jumpStageId?: string | null;
    template?: string;
    uniEquipId?: string;
    uniEquipMissionId?: string;
    uniEquipMissionSort?: number;
}

export interface UserCharacterHandbook {
    charID: string;
    infoName?: string;
    isLimited?: boolean;
    storyTextAudio?: UserStoryTextAudio[];
    handbookAvgList?: UserHandbookAvg[];
}

export interface UserStoryTextAudio {
    stories?: UserHandbookStory[];
    storyTitle?: string;
    unLockorNot?: boolean;
}

export interface UserHandbookStory {
    storyText?: string;
    unLockParam?: string;
    unLockType?: number;
}

export interface UserHandbookAvg {
    avgList?: UserAvgEntry[];
    storySetId?: string;
    storySetName?: string;
    sortId?: number;
}

export interface UserAvgEntry {
    avgTag?: string;
    storyId?: string;
    storyIntro?: string;
    storyName?: string;
    storyTxt?: string;
}

export interface UserCharacterProfile {
    basicInfo?: UserBasicInfo;
    physicalExam?: UserPhysicalExam;
}

export interface UserBasicInfo {
    codeName?: string;
    combatExperience?: string;
    dateOfBirth?: string;
    gender?: string;
    height?: string;
    infectionStatus?: string;
    placeOfBirth?: string;
    race?: string;
}

export interface UserPhysicalExam {
    cellOriginiumAssimilation?: string;
    originiumArtsAssimilation?: string;
    physicalStrength?: string;
    tacticalAcumen?: string;
    combatSkill?: string;
    mobility?: string;
    bloodOriginiumCrystalDensity?: string;
}

// ==================== Skill Static Types ====================

/** Static skill data added by backend formatUser */
export interface UserSkillStatic {
    skillId: string;
    name?: string;
    description?: string;
    duration?: number;
    hidden?: boolean;
    iconId?: string | null;
    image?: string;
    spData?: UserSkillSpData;
    levels?: UserSkillLevel[]; // Full skill level data when available
}

export interface UserSkillLevel {
    name: string;
    rangeId?: string | null;
    description: string;
    skillType: string | number;
    durationType?: string | null;
    duration: number;
    spData: UserSkillSpData;
    prefabId?: string;
    blackboard: UserSkillBlackboard[];
}

export interface UserSkillBlackboard {
    key: string;
    value: number;
    valueStr?: string | null;
}

export interface UserSkillSpData {
    increment?: number;
    initSp?: number;
    levelUpCost?: unknown[] | null;
    maxChargeTime?: number;
    spCost?: number;
    spType?: string | number; // "INCREASE_WITH_TIME", "INCREASE_WHEN_ATTACK", etc.
}

// ==================== Enriched Roster Type ====================

/** RosterEntry enriched with static game data for display purposes */
export interface EnrichedRosterEntry extends RosterEntry {
    static?: CharacterStatic | null;
}

// ==================== Unowned / Display Types ====================

export interface UnownedOperator {
    charId: string;
    name: string;
    rarity: string;
    profession: string;
    subProfessionId: string;
    portrait: string;
    position: string;
    isOwned: false;
}

export type DisplayCharacter = (EnrichedRosterEntry & { isOwned: true }) | UnownedOperator;
