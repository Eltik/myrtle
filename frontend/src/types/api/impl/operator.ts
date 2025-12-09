// Operator types

import type { HandbookItem, OperatorProfile } from "./handbook";
import type { Item, ItemType } from "./material";
import type { Module, ModuleData } from "./module";
import type { SkillLevel } from "./skill";

export type OperatorPosition = "RANGED" | "MELEE" | "ALL" | "NONE";

export type OperatorPhase = "PHASE_0" | "PHASE_1" | "PHASE_2";

export type OperatorRarity = "TIER_6" | "TIER_5" | "TIER_4" | "TIER_3" | "TIER_2" | "TIER_1";

export type OperatorProfession = "MEDIC" | "CASTER" | "WARRIOR" | "PIONEER" | "SNIPER" | "SPECIAL" | "SUPPORT" | "TANK" | "TOKEN" | "TRAP";

export interface UnlockCondition {
    Phase: OperatorPhase;
    Level: number;
}

export interface Blackboard {
    key: string;
    value: number;
    valueStr: string | null;
}

export interface TraitCandidate {
    UnlockCondition: UnlockCondition;
    RequiredPotentialRank: number;
    Blackboard: Blackboard[];
    OverrideDescription: string | null;
    PrefabKey: string | null;
    RangeId: string | null;
}

export interface Trait {
    Candidates: TraitCandidate[];
}

export interface AttributeData {
    MaxHp: number;
    Atk: number;
    Def: number;
    MagicResistance: number;
    Cost: number;
    BlockCnt: number;
    MoveSpeed: number;
    AttackSpeed: number;
    BaseAttackTime: number;
    RespawnTime: number;
    HpRecoveryPerSec: number;
    SpRecoveryPerSec: number;
    MaxDeployCount: number;
    MaxDeckStackCnt: number;
    TauntLevel: number;
    MassLevel: number;
    BaseForceLevel: number;
    StunImmune: boolean;
    SilenceImmune: boolean;
    SleepImmune: boolean;
    FrozenImmune: boolean;
    LevitateImmune: boolean;
    DisarmedCombatImmune: boolean;
}

export interface AttributeKeyFrame {
    Level: number;
    Data: AttributeData;
}

export interface EvolveCost {
    Id: string;
    Count: number;
    Type_: ItemType;
}

export interface Phase {
    CharacterPrefabKey: string;
    RangeId: string | null;
    MaxLevel: number;
    AttributesKeyFrames: AttributeKeyFrame[];
    EvolveCost: EvolveCost[] | null;
}

export interface LevelUpCostItem {
    Id: string;
    Count: number;
    Type_: string;
}

export interface LevelUpCostCond {
    UnlockCond: UnlockCondition;
    LvlUpTime: number;
    LevelUpCost: LevelUpCostItem[];
}

export interface SpData {
    SpType: string;
    LevelUpCost: unknown[];
    MaxChargeTime: number;
    SpCost: number;
    InitSp: number;
    Increment: number;
}

export interface SkillStatic {
    Levels: SkillLevel[];
    SkillId: string;
    IconId: string | null;
    Hidden: boolean;
    Image: string | null;
}

export interface OperatorSkillRef {
    SkillId: string | null;
    OverridePrefabKey: string | null;
    OverrideTokenKey: string | null;
    LevelUpCostCond: LevelUpCostCond[];
    UnlockCond: UnlockCondition | null;
}

export interface TalentCandidate {
    UnlockCondition: UnlockCondition;
    RequiredPotentialRank: number;
    PrefabKey: string | null;
    Name: string | null;
    Description: string | null;
    RangeId: string | null;
    Blackboard: Blackboard[];
    TokenKey: string | null;
    IsHideTalent: boolean | null;
}

export interface Talent {
    Candidates: TalentCandidate[];
}

export interface AttributeModifier {
    AttributeType: string;
    FormulaItem: string;
    Value: number;
    LoadFromBlackboard: boolean;
    FetchBaseValueFromSourceEntity: boolean;
}

export interface PotentialBuffAttributes {
    AbnormalFlags: null;
    AbnormalImmunes: null;
    AbnormalAntis: null;
    AbnormalCombos: null;
    AbnormalComboImmunes: null;
    AttributeModifiers: AttributeModifier[] | null;
}

export interface PotentialBuff {
    Attributes: PotentialBuffAttributes;
}

export interface PotentialRank {
    Type_: string;
    Description: string;
    Buff: PotentialBuff | null;
}

export interface AllSkillLevelUp {
    UnlockCond: UnlockCondition;
    LvlUpCost: LevelUpCostItem[];
}

export interface CharacterTable {
    Characters: Record<string, RawOperator>;
}

export interface RawOperator {
    Name: string;
    Description: string | null;
    CanUseGeneralPotentialItem: boolean;
    CanUseActivityPotentialItem: boolean;
    PotentialItemId: string | null;
    ActivityPotentialItemId: string | null;
    ClassicPotentialItemId: string | null;
    NationId: string | null;
    GroupId: string | null;
    TeamId: string | null;
    DisplayNumber: string | null;
    Appellation: string;
    Position: OperatorPosition;
    TagList: string[] | null;
    ItemUsage: string | null;
    ItemDesc: string | null;
    ItemObtainApproach: string | null;
    IsNotObtainable: boolean;
    IsSpChar: boolean;
    MaxPotentialLevel: number;
    Rarity: OperatorRarity;
    Profession: OperatorProfession;
    SubProfessionId: string | null;
    Trait_: Trait | null;
    Phases: Phase[];
    Skills: OperatorSkillRef[];
    DisplayTokenDict: Record<string, boolean> | null;
    Talents: Talent[] | null;
    PotentialRanks: PotentialRank[];
    FavorKeyFrames: AttributeKeyFrame[] | null;
    AllSkillLvlup: AllSkillLevelUp[];
    SortIndex: number | null;
    SpTargetType: string | null;
}

export interface OperatorModule {
    module: Module;
    data: ModuleData;
}

export interface EnrichedSkill {
    skillId: string;
    overridePrefabKey: string | null;
    overrideTokenKey: string | null;
    levelUpCostCond: LevelUpCostCond[];
    static: SkillStatic | null;
}

export interface Operator {
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
    rarity: OperatorRarity;
    profession: OperatorProfession;
    subProfessionId: string;
    trait: Trait | null;
    phases: Phase[];
    skills: EnrichedSkill[];
    displayTokenDict: Record<string, boolean> | null;
    talents: Talent[];
    potentialRanks: PotentialRank[];
    favorKeyFrames: AttributeKeyFrame[];
    allSkillLevelUp: AllSkillLevelUp[];
    modules: OperatorModule[];
    handbook: HandbookItem;
    profile: OperatorProfile | null;
    artists: string[];
}

export type Elite = "E0" | "E1" | "E2";

export interface EliteCostItem {
    quantity: number;
    material: Item;
}

export interface LevelCost {
    level: number;
    eliteCost: EliteCostItem[];
    elite: Elite;
}

export type LevelUpCost = LevelCost[];

export type MasteryLevel = "M1" | "M2" | "M3";

export interface SkillCostItem {
    quantity: number;
    material: Item;
}

export interface SkillCost {
    unlockCondition: UnlockCondition;
    lvlUpTime: number;
    skillCost: SkillCostItem[];
    level: MasteryLevel;
}

export interface SkillLevelCost {
    skillId: string;
    cost: SkillCost[];
}

export type SkillLevelUpCost = SkillLevelCost[];

export interface Drone {
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
    rarity: OperatorRarity;
    profession: string;
    subProfessionId: string;
    trait: Trait | null;
    phases: Phase[];
    skills: OperatorSkillRef[];
    displayTokenDict: Record<string, boolean> | null;
    talents: Talent[];
    potentialRanks: PotentialRank[];
    favorKeyFrames: null;
    allSkillLvlup: AllSkillLevelUp[];
    modules: OperatorModule[];
}
