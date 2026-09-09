/**
 * Operator types - derived from the ts-rs bindings generated out of
 * `backend/src/core/gamedata/types/`.
 *
 * The operator endpoints serve the `character_table`-mirroring structs with
 * PascalCase and trailing-underscore keys, and `deepCamelize` in
 * `#/lib/api/operators` normalizes them at the fetch boundary. `Camelize<T>`
 * applies that same transform in the type system, so these types describe what
 * components actually receive while the wire format stays PascalCase.
 *
 * Do not add fields here - change the Rust struct and re-run:
 *   cd backend && cargo test export_bindings
 */
import type { Camelize } from "./camelize";
import type { AddModuleCandidates } from "./generated/AddModuleCandidates";
import type { AddOrOverrideTalentDataBundle } from "./generated/AddOrOverrideTalentDataBundle";
import type { AllSkillLevelUp } from "./generated/AllSkillLevelUp";
import type { AttributeData } from "./generated/AttributeData";
import type { AttributeKeyFrame } from "./generated/AttributeKeyFrame";
import type { AttributeModifier } from "./generated/AttributeModifier";
import type { AudioCategory as AudioCategoryGenerated } from "./generated/AudioCategory";
import type { AudioSound } from "./generated/AudioSound";
import type { BasicInfo } from "./generated/BasicInfo";
import type { Blackboard } from "./generated/Blackboard";
import type { Drone } from "./generated/Drone";
import type { EnrichedSkill } from "./generated/EnrichedSkill";
import type { EvolveCost } from "./generated/EvolveCost";
import type { HandbookAvgEntry } from "./generated/HandbookAvgEntry";
import type { HandbookAvgList } from "./generated/HandbookAvgList";
import type { HandbookItem } from "./generated/HandbookItem";
import type { HandbookRewardItem } from "./generated/HandbookRewardItem";
import type { HandbookStory } from "./generated/HandbookStory";
import type { HandbookStoryTextAudio } from "./generated/HandbookStoryTextAudio";
import type { HandbookUnlockParam } from "./generated/HandbookUnlockParam";
import type { ItemClass as ItemClassGenerated } from "./generated/ItemClass";
import type { ItemOccPer as ItemOccPerGenerated } from "./generated/ItemOccPer";
import type { ItemRarity as ItemRarityGenerated } from "./generated/ItemRarity";
import type { LevelUpCostCond } from "./generated/LevelUpCostCond";
import type { LevelUpCostItem } from "./generated/LevelUpCostItem";
import type { Module } from "./generated/Module";
import type { ModuleBlackboard } from "./generated/ModuleBlackboard";
import type { ModuleCandidates } from "./generated/ModuleCandidates";
import type { ModuleData } from "./generated/ModuleData";
import type { ModuleItemCost } from "./generated/ModuleItemCost";
import type { ModulePart } from "./generated/ModulePart";
import type { ModulePhase } from "./generated/ModulePhase";
import type { ModuleTarget as ModuleTargetGenerated } from "./generated/ModuleTarget";
import type { ModuleType as ModuleTypeGenerated } from "./generated/ModuleType";
import type { ModuleUnlockCondition } from "./generated/ModuleUnlockCondition";
import type { Operator } from "./generated/Operator";
import type { OperatorAudio } from "./generated/OperatorAudio";
import type { OperatorBaseSkill } from "./generated/OperatorBaseSkill";
import type { OperatorModule } from "./generated/OperatorModule";
import type { OperatorPhase as OperatorPhaseGenerated } from "./generated/OperatorPhase";
import type { OperatorPosition as OperatorPositionGenerated } from "./generated/OperatorPosition";
import type { OperatorProfession as OperatorProfessionGenerated } from "./generated/OperatorProfession";
import type { OperatorProfile } from "./generated/OperatorProfile";
import type { OperatorRarity as OperatorRarityGenerated } from "./generated/OperatorRarity";
import type { OperatorSkillRef } from "./generated/OperatorSkillRef";
import type { OverrideTraitDataBundle } from "./generated/OverrideTraitDataBundle";
import type { Phase } from "./generated/Phase";
import type { PhysicalExam } from "./generated/PhysicalExam";
import type { PotentialBuff } from "./generated/PotentialBuff";
import type { PotentialBuffAttributes } from "./generated/PotentialBuffAttributes";
import type { PotentialRank } from "./generated/PotentialRank";
import type { SkillLevel } from "./generated/SkillLevel";
import type { SkillSpData } from "./generated/SkillSpData";
import type { SkillStatic } from "./generated/SkillStatic";
import type { Talent } from "./generated/Talent";
import type { TalentCandidate } from "./generated/TalentCandidate";
import type { Trait } from "./generated/Trait";
import type { TraitCandidate } from "./generated/TraitCandidate";
import type { UnlockCondition } from "./generated/UnlockCondition";

// Enums cross the wire as string values, so they need no key transform.
export type OperatorProfession = OperatorProfessionGenerated;
export type OperatorPosition = OperatorPositionGenerated;
export type OperatorRarityTier = OperatorRarityGenerated;
export type OperatorPhase = OperatorPhaseGenerated;
export type ItemRarityTier = ItemRarityGenerated;
export type ItemClass = ItemClassGenerated;
export type ItemOccPer = ItemOccPerGenerated;
export type ModuleType = ModuleTypeGenerated;
export type ModuleTarget = ModuleTargetGenerated;
export type AudioCategory = AudioCategoryGenerated;

/** Rarity as a number (1-6), computed frontend-side from the TIER_n tier. */
export type OperatorRarity = 1 | 2 | 3 | 4 | 5 | 6;
export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

export type IAudioSound = Camelize<AudioSound>;
export type IOperatorAudio = Camelize<OperatorAudio>;
export type IBlackboard = Camelize<Blackboard>;
export type IUnlockCondition = Camelize<UnlockCondition>;
export type ITraitCandidate = Camelize<TraitCandidate>;
export type ITrait = Camelize<Trait>;
export type IAttributeData = Camelize<AttributeData>;
export type IAttributeKeyFrame = Camelize<AttributeKeyFrame>;
export type IEvolveCost = Camelize<EvolveCost>;
export type IOperatorPhase = Camelize<Phase>;
export type ILevelUpCostItem = Camelize<LevelUpCostItem>;
export type ILevelUpCostCond = Camelize<LevelUpCostCond>;
export type ISkillSpData = Camelize<SkillSpData>;
export type ISkillLevel = Camelize<SkillLevel>;
export type ISkillStatic = Camelize<SkillStatic>;
export type IEnrichedSkill = Camelize<EnrichedSkill>;
export type ITalentCandidate = Camelize<TalentCandidate>;
export type ITalent = Camelize<Talent>;
export type IAttributeModifier = Camelize<AttributeModifier>;
export type IPotentialBuffAttributes = Camelize<PotentialBuffAttributes>;
export type IPotentialBuff = Camelize<PotentialBuff>;
export type IPotentialRank = Camelize<PotentialRank>;
export type IAllSkillLevelUp = Camelize<AllSkillLevelUp>;
export type IModuleItemCost = Camelize<ModuleItemCost>;
export type IModule = Camelize<Module>;
export type IModuleBlackboard = Camelize<ModuleBlackboard>;
export type IModuleUnlockCondition = Camelize<ModuleUnlockCondition>;
export type IAddModuleCandidates = Camelize<AddModuleCandidates>;
export type IModuleCandidates = Camelize<ModuleCandidates>;
export type IAddOrOverrideTalentDataBundle = Camelize<AddOrOverrideTalentDataBundle>;
export type IOverrideTraitDataBundle = Camelize<OverrideTraitDataBundle>;
export type IModulePart = Camelize<ModulePart>;
export type IModulePhase = Camelize<ModulePhase>;
export type IModuleData = Camelize<ModuleData>;
export type IOperatorModule = Camelize<OperatorModule>;
export type IHandbookStory = Camelize<HandbookStory>;
export type IHandbookStoryTextAudio = Camelize<HandbookStoryTextAudio>;
export type IHandbookUnlockParam = Camelize<HandbookUnlockParam>;
export type IHandbookAvgEntry = Camelize<HandbookAvgEntry>;
export type IHandbookRewardItem = Camelize<HandbookRewardItem>;
export type IHandbookAvgList = Camelize<HandbookAvgList>;
export type IHandbookItem = Camelize<HandbookItem>;
export type IBasicInfo = Camelize<BasicInfo>;
export type IPhysicalExam = Camelize<PhysicalExam>;
export type IOperatorProfile = Camelize<OperatorProfile>;
export type IOperatorBaseSkill = Camelize<OperatorBaseSkill>;
export type IOperatorSkillRef = Camelize<OperatorSkillRef>;
export type IDrone = Camelize<Drone>;

// `/operators/index` is a purpose-built response type in app/services, not a
// gamedata struct, and it is served camelCase with no normalization - so it
// has no generated counterpart and stays hand-written.
export interface IOperatorIndexStats {
    hp: number;
    atk: number;
    def: number;
    res: number;
    cost: number;
    block: number;
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
    groupId: string | null;
    teamId: string | null;
    artists: string[];
    /** Small portrait image (headshot) - /upk/arts/charportraits/{pack}/{id}_{1|2}.png */
    portrait: string | null;
    /** "Male" / "Female" / etc.; empty string when unknown. */
    gender: string;
    /** e.g. "Feline"; empty string when unknown. */
    race: string;
    /** e.g. "Kazimierz"; empty string when unknown. */
    placeOfBirth: string;
    stats: IOperatorIndexStats;
    hasOffensiveRecovery: boolean;
    hasDefensiveRecovery: boolean;
    allSkillsManual: boolean;
}

/**
 * One operator as the detail endpoints serve it: the Rust `Operator` plus the
 * fields the API layer adds around it (`server`) and the enrichment attaches
 * (`audio`, artwork paths, alternate-form template ids).
 */
export type IOperatorListItem = Camelize<Operator> & {
    server?: "en" | "cn";
    audio: IOperatorAudio[];
    /** Small portrait image (headshot). */
    portrait: string | null;
    /** Full character art; null → use portrait as fallback. */
    skin: string | null;
    /** Template group ids for operators with alternate forms (Amiya). */
    tmplIds?: string[];
    /** Canonical base id for the template group (e.g. "char_002_amiya"). */
    tmplDefault?: string;
};

export type IOperatorsStaticMap = Record<string, IOperatorListItem>;
