import type { Operator } from "../operators";

export enum Position {
    MELEE = "MELEE",
    RANGED = "RANGED",
}

export enum RarityTier {
    TIER_5 = "TIER_5",
    // Add other tiers if they exist
}

export enum Profession {
    WARRIOR = "WARRIOR",
    MEDIC = "MEDIC",
    CASTER = "CASTER",
    // Add other professions if they exist
}

export enum UnlockPhase {
    PHASE_0 = "PHASE_0",
    PHASE_1 = "PHASE_1",
    PHASE_2 = "PHASE_2",
}

export enum CostType {
    MATERIAL = "MATERIAL",
    // Add other cost types if they exist
}

export enum PotentialRankType {
    BUFF = "BUFF",
    // Add other potential rank types if they exist
}

export enum AttributeType {
    MAX_HP = "MAX_HP",
    ATK = "ATK",
    DEF = "DEF",
    MAGIC_RESISTANCE = "MAGIC_RESISTANCE",
    COST = "COST",
    // Add other attribute types if they exist
}

export enum FormulaItem {
    ADDITION = "ADDITION",
    // Add other formula items if they exist
}

export enum StageCompleteState {
    PASS = "PASS",
}

export interface CharInfo {
    tmplIds: string[];
    default: string;
}

export interface AttributeData {
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
    fearedImmune: boolean;
}

export interface AttributesKeyFrame {
    level: number;
    data: AttributeData;
}

export interface EvolveCost {
    id: string;
    count: number;
    type: CostType;
}

export interface Phase {
    characterPrefabKey: string;
    rangeId: string | null;
    maxLevel: number;
    attributesKeyFrames: AttributesKeyFrame[];
    evolveCost: EvolveCost[] | null;
}

export interface UnlockCondition {
    phase: UnlockPhase;
    level: number;
}

export interface LevelUpCost {
    id: string;
    count: number;
    type: CostType;
}

export interface LevelUpCostCond {
    unlockCond: UnlockCondition;
    lvlUpTime: number;
    levelUpCost: LevelUpCost[];
}

export interface Skill {
    skillId: string;
    overridePrefabKey: string | null;
    overrideTokenKey: string | null;
    levelUpCostCond: LevelUpCostCond[];
    unlockCond: UnlockCondition;
}

export interface BlackboardEntry {
    key: string;
    value: number;
    valueStr: string | null;
}

export interface TalentCandidate {
    unlockCondition: UnlockCondition;
    requiredPotentialRank: number;
    prefabKey: string | null;
    name: string;
    description: string | null;
    rangeId: string | null;
    blackboard: BlackboardEntry[];
    tokenKey: string | null;
    isHideTalent?: boolean; // Marked optional as seen in some entries
    overrideDescripton?: string; // Optional, seen in Trait candidates
}

export interface Trait {
    candidates: TalentCandidate[];
}

export interface TalentEntry {
    candidates: TalentCandidate[];
}

export interface AttributeModifier {
    attributeType: AttributeType;
    formulaItem: FormulaItem;
    value: number;
    loadFromBlackboard: boolean;
    fetchBaseValueFromSourceEntity: boolean;
}

export interface BuffAttributes {
    abnormalFlags: any | null;
    abnormalImmunes: any | null;
    abnormalAntis: any | null;
    abnormalCombos: any | null;
    abnormalComboImmunes: any | null;
    attributeModifiers: AttributeModifier[];
}

export interface PotentialBuff {
    attributes: BuffAttributes;
}

export interface PotentialRank {
    type: PotentialRankType;
    description: string;
    buff: PotentialBuff;
    equivalentCost: any | null;
}

export interface FavorKeyFrame {
    level: number;
    data: Partial<AttributeData>; // Favor often only modifies a subset of attributes
}

export interface AllSkillLvlupEntry {
    unlockCond: UnlockCondition;
    lvlUpCost: LevelUpCost[];
}

export interface UnlockCondEntry {
    stageId: string;
    completeState: StageCompleteState;
    unlockTs: number;
}

export interface UnlockConds {
    conds: UnlockCondEntry[];
}

export interface PatchDetailInfo {
    patchId: string;
    sortId: number;
    infoParam: string; // e.g., "Caster", "Guard", "Medic"
    transSortId: number;
}

export interface CharPatchTable {
    infos: Record<string, CharInfo>;
    patchChars: Record<string, Operator>;
    unlockConds: Record<string, UnlockConds>;
    patchDetailInfoList: Record<string, PatchDetailInfo>;
}
