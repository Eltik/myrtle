import type { LangType } from "#/types/voices";

export type OperatorProfession = "MEDIC" | "CASTER" | "WARRIOR" | "PIONEER" | "SNIPER" | "SPECIAL" | "SUPPORT" | "TANK" | "TOKEN" | "TRAP";
export type OperatorPosition = "MELEE" | "RANGED" | "ALL" | "NONE";
export type OperatorRarityTier = "TIER_1" | "TIER_2" | "TIER_3" | "TIER_4" | "TIER_5" | "TIER_6";
export type OperatorRarity = 1 | 2 | 3 | 4 | 5 | 6;
export type OperatorPhase = "PHASE_0" | "PHASE_1" | "PHASE_2";

export type ItemRarityTier = "TIER_1" | "TIER_2" | "TIER_3" | "TIER_4" | "TIER_5" | "TIER_6";
export type ItemClass = "MATERIAL" | "CONSUME" | "NORMAL" | "NONE";
export type ItemOccPer = "USUAL" | "ALMOST" | "ALWAYS" | "SOMETIMES" | "OFTEN";

export type ModuleType = "INITIAL" | "ADVANCED";
export type ModuleTarget = "TRAIT" | "TRAIT_DATA_ONLY" | "TALENT_DATA_ONLY" | "TALENT" | "DISPLAY" | "OVERWRITE_BATTLE_DATA" | "UNKNOWN";

export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

export type AudioCategory = "deploy" | "attack" | "skill" | "voice" | "other";

export interface IAudioSound {
    asset: string;
    urls: string[];
}

export interface IOperatorAudio {
    bankName: string;
    event: string;
    category: AudioCategory;
    skillId?: string;
    skillSlot?: number;
    language?: LangType;
    sounds: IAudioSound[];
}

export interface IOperatorIndexEntry {
    id: string;
    name: string;
    appellation: string;
    rarity: OperatorRarity;
    profession: OperatorProfession;
    subProfessionId: string;
    position: OperatorPosition;
    tagList: string[];
    nationId: string;
    isNotObtainable: boolean;
}

export interface IBlackboard {
    key: string;
    value: number;
    valueStr: string | null;
}

export interface IUnlockCondition {
    phase: OperatorPhase;
    level: number;
}

export interface ITraitCandidate {
    unlockCondition: IUnlockCondition;
    requiredPotentialRank: number;
    blackboard: IBlackboard[];
    overrideDescription: string | null;
    prefabKey: string | null;
    rangeId: string | null;
}

export interface ITrait {
    candidates: ITraitCandidate[];
}

export interface IAttributeData {
    maxHp: number;
    atk: number;
    def: number;
    magicResistance: number;
    cost: number;
    blockCnt: number;
    moveSpeed: number;
    attackSpeed: number;
    baseAttackTime: number;
    respawnTime: number;
    hpRecoveryPerSec: number;
    spRecoveryPerSec: number;
    maxDeployCount: number;
    maxDeckStackCnt: number;
    tauntLevel: number;
    massLevel: number;
    baseForceLevel: number;
    stunImmune: boolean;
    silenceImmune: boolean;
    sleepImmune: boolean;
    frozenImmune: boolean;
    levitateImmune: boolean;
    disarmedCombatImmune: boolean;
}

export interface IAttributeKeyFrame {
    level: number;
    data: IAttributeData;
}

export interface IEvolveCost {
    id: string;
    count: number;
    type: string;
    iconId: string | null;
    image: string | null;
}

export interface IOperatorPhase {
    characterPrefabKey: string;
    rangeId: string | null;
    maxLevel: number;
    attributesKeyFrames: IAttributeKeyFrame[];
    evolveCost: IEvolveCost[] | null;
    levelUpCost: ILevelUpCostItem[] | null;
}

export interface ILevelUpCostItem {
    id: string;
    count: number;
    type: string;
    iconId: string | null;
    image: string | null;
}

export interface ILevelUpCostCond {
    unlockCond: IUnlockCondition;
    lvlUpTime: number;
    levelUpCost: ILevelUpCostItem[];
}

export interface ISkillSpData {
    spType: string;
    levelUpCost: null;
    maxChargeTime: number;
    spCost: number;
    initSp: number;
    increment: number;
}

export interface ISkillLevel {
    name: string;
    rangeId: string | null;
    description: string;
    skillType: string;
    durationType: string | null;
    spData: ISkillSpData;
    prefabId: string;
    duration: number;
    blackboard: IBlackboard[];
}

export interface ISkillStatic {
    levels: ISkillLevel[];
    skillId: string;
    iconId: string | null;
    hidden: boolean;
    image: string | null;
}

export interface IEnrichedSkill {
    skillId: string;
    overridePrefabKey: string | null;
    overrideTokenKey: string | null;
    levelUpCostCond: ILevelUpCostCond[];
    static: ISkillStatic | null;
}

export interface ITalentCandidate {
    unlockCondition: IUnlockCondition;
    requiredPotentialRank: number;
    prefabKey: string | null;
    name: string | null;
    description: string | null;
    rangeId: string | null;
    blackboard: IBlackboard[];
    tokenKey: string | null;
    isHideTalent: boolean | null;
}

export interface ITalent {
    candidates: ITalentCandidate[];
}

export interface IAttributeModifier {
    attributeType: string;
    formulaItem: string;
    value: number;
    loadFromBlackboard: boolean;
    fetchBaseValueFromSourceEntity: boolean;
}

export interface IPotentialBuffAttributes {
    abnormalFlags: null;
    abnormalImmunes: null;
    abnormalAntis: null;
    abnormalCombos: null;
    abnormalComboImmunes: null;
    attributeModifiers: IAttributeModifier[] | null;
}

export interface IPotentialBuff {
    attributes: IPotentialBuffAttributes;
}

export interface IPotentialRank {
    type: string;
    description: string;
    buff: IPotentialBuff | null;
}

export interface IAllSkillLevelUp {
    unlockCond: IUnlockCondition;
    lvlUpCost: ILevelUpCostItem[];
}

export interface IModuleItemCost {
    id: string;
    count: number;
    type: string;
    iconId: string | null;
    image: string | null;
}

export interface IModule {
    id: string | null;
    uniEquipId: string;
    uniEquipName: string;
    uniEquipIcon: string;
    image: string | null;
    uniEquipDesc: string;
    typeIcon: string;
    typeName1: string;
    typeName2: string | null;
    equipShiningColor: string;
    showEvolvePhase: string;
    unlockEvolvePhase: string;
    charId: string;
    tmplId: string | null;
    showLevel: number;
    unlockLevel: number;
    unlockFavorPoint: number;
    missionList: string[];
    itemCost: Record<string, IModuleItemCost[]> | null;
    type: ModuleType;
    uniEquipGetTime: number;
    charEquipOrder: number;
}

export interface IModuleBlackboard {
    key: string;
    value: number;
}

export interface IModuleUnlockCondition {
    phase: string;
    level: number;
}

export interface IAddModuleCandidates {
    displayRangeId: boolean;
    upgradeDescription: string;
    talentIndex: number;
    unlockCondition: IModuleUnlockCondition;
    requiredPotentialRank: number;
    prefabKey: string | null;
    name: string;
    description: string | null;
    rangeId: string | null;
    blackboard: IModuleBlackboard[];
    tokenKey: string | null;
    isHideTalent: boolean | null;
}

export interface IModuleCandidates {
    additionalDescription: string;
    unlockCondition: IModuleUnlockCondition;
    requiredPotentialRank: number;
    blackboard: IModuleBlackboard[];
    overrideDescription: string | null;
    prefabKey: string | null;
    rangeId: string | null;
}

export interface IAddOrOverrideTalentDataBundle {
    candidates: IAddModuleCandidates[] | null;
}

export interface IOverrideTraitDataBundle {
    candidates: IModuleCandidates[] | null;
}

export interface IModulePart {
    resKey: string | null;
    target: ModuleTarget;
    isToken: boolean;
    addOrOverrideTalentDataBundle: IAddOrOverrideTalentDataBundle;
    overrideTraitDataBundle: IOverrideTraitDataBundle;
}

export interface IModulePhase {
    equipLevel: number;
    parts: IModulePart[];
    attributeBlackboard: IModuleBlackboard[];
    tokenAttributeBlackboard: JsonValue[];
}

export interface IModuleData {
    phases: IModulePhase[];
}

export interface IOperatorModule extends IModule {
    data: IModuleData;
}

export interface IHandbookStory {
    storyText: string;
    unlockType: string;
    unLockParam: string;
    unLockString: string;
    patchIdList: string[] | null;
}

export interface IHandbookStoryTextAudio {
    stories: IHandbookStory[];
    storyTitle: string;
    unLockorNot: boolean;
}

export interface IHandbookUnlockParam {
    unlockType: string;
    unlockParam1: string | null;
    unlockParam2: string | null;
    unlockParam3: string | null;
}

export interface IHandbookAvgEntry {
    storyId: string;
    storySetId: string;
    storySort: number;
    storyCanShow: boolean;
    storyIntro: string;
    storyInfo: string;
    storyTxt: string;
}

export interface IHandbookRewardItem {
    id: string;
    count: number;
    type: string;
}

export interface IHandbookAvgList {
    storySetId: string;
    storySetName: string;
    sortId: number;
    storyGetTime: number;
    rewardItem: IHandbookRewardItem[];
    unlockParam: IHandbookUnlockParam[];
    avgList: IHandbookAvgEntry[];
    charId: string;
}

export interface IHandbookItem {
    charID: string;
    infoName: string;
    isLimited: boolean;
    storyTextAudio: IHandbookStoryTextAudio[];
    handbookAvgList: IHandbookAvgList[];
}

export interface IBasicInfo {
    codeName: string;
    gender: string;
    combatExperience: string;
    placeOfBirth: string;
    dateOfBirth: string;
    race: string;
    height: string;
    infectionStatus: string;
}

export interface IPhysicalExam {
    physicalStrength: string;
    mobility: string;
    physicalResilience: string;
    tacticalAcumen: string;
    combatSkill: string;
    originiumArtsAssimilation: string;
}

export interface IOperatorProfile {
    basicInfo: IBasicInfo;
    physicalExam: IPhysicalExam;
}

export interface IOperatorBaseSkill {
    buffId: string;
    buffName: string;
    description: string;
    roomType: string;
    efficiency: number;
    targets: string[];
    /** Sprite stem in `building_ui_buff_skills_h1_0/`; empty if absent. */
    skillIcon: string;
    unlockElite: number;
    unlockLevel: number;
}

export interface IOperatorSkillRef {
    skillId: string | null;
    overridePrefabKey: string | null;
    overrideTokenKey: string | null;
    levelUpCostCond: ILevelUpCostCond[];
    unlockCond: IUnlockCondition | null;
}

export interface IDrone {
    id: string | null;
    name: string;
    description: string;
    canUseGeneralPotentialItem: boolean;
    canUseActivityPotentialItem: boolean;
    potentialItemId: string | null;
    activityPotentialItemId: string | null;
    classicPotentialItemId: string | null;
    nationId: string | null;
    groupId: string | null;
    teamId: string | null;
    displayNumber: string | null;
    appellation: string;
    position: OperatorPosition;
    tagList: string[];
    itemUsage: string | null;
    itemDesc: string | null;
    itemObtainApproach: string | null;
    isNotObtainable: boolean;
    isSpChar: boolean;
    maxPotentialLevel: number;
    rarity: OperatorRarityTier;
    profession: string;
    subProfessionId: string;
    trait: ITrait | null;
    phases: IOperatorPhase[];
    skills: IOperatorSkillRef[];
    displayTokenDict: Record<string, boolean> | null;
    talents: ITalent[];
    potentialRanks: IPotentialRank[];
    favorKeyFrames: null;
    allSkillLvlup: IAllSkillLevelUp[];
    modules: IOperatorModule[];
}

export interface IOperatorListItem {
    id: string | null;
    name: string;
    description: string;
    canUseGeneralPotentialItem: boolean;
    canUseActivityPotentialItem: boolean;
    potentialItemId: string;
    activityPotentialItemId: string | null;
    classicPotentialItemId: string | null;
    nationId: string;
    groupId: string | null;
    teamId: string | null;
    displayNumber: string;
    appellation: string;
    position: OperatorPosition;
    tagList: string[];
    itemUsage: string;
    itemDesc: string;
    itemObtainApproach: string;
    isNotObtainable: boolean;
    isSpChar: boolean;
    maxPotentialLevel: number;
    rarity: OperatorRarityTier;
    profession: OperatorProfession;
    subProfessionId: string;
    trait: ITrait | null;
    phases: IOperatorPhase[];
    skills: IEnrichedSkill[];
    displayTokenDict: Record<string, boolean> | null;
    drones: IDrone[];
    talents: ITalent[];
    potentialRanks: IPotentialRank[];
    favorKeyFrames: IAttributeKeyFrame[];
    allSkillLevelUp: IAllSkillLevelUp[];
    modules: IOperatorModule[];
    handbook: IHandbookItem;
    profile: IOperatorProfile | null;
    artists: string[];
    baseSkills: IOperatorBaseSkill[];
    audio: IOperatorAudio[];
    /** Small portrait image (headshot) - /upk/arts/charportraits/{pack}/{id}_{1|2}.png */
    portrait: string | null;
    /** Full character art - /upk/chararts/{id}/{id}_{1|2}.png; null → use portrait as fallback */
    skin: string | null;
    /** Template group ids for operators with alternate forms (Amiya).
     *  Present on every form in the group. */
    tmplIds?: string[];
    /** Canonical base id for the template group (e.g. "char_002_amiya"). */
    tmplDefault?: string;
}

export type IOperatorsStaticMap = Record<string, IOperatorListItem>;
