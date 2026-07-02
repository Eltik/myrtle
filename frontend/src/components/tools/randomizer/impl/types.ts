import type { OperatorPosition, OperatorProfession, OperatorRarity } from "#/types/operators";
import type { IStage } from "#/types/stages";

export interface IRandomizerOperator {
    id: string;
    name: string;
    rarity: OperatorRarity;
    profession: OperatorProfession;
    subProfessionId: string;
    position: OperatorPosition;
    /** Operator race (e.g. "Feline"); empty string when unknown. Used by challenge filters. */
    race: string;
    /** Has at least one skill with an offensive (attack) SP-recovery type. */
    hasOffensiveRecovery: boolean;
    /** Has at least one skill with a defensive (damage-taken) SP-recovery type. */
    hasDefensiveRecovery: boolean;
    /** Every skill is manually activated (no auto-trigger). */
    allSkillsManual: boolean;
}

export interface IRandomizerSettings {
    allowedClasses: OperatorProfession[];
    allowedRarities: OperatorRarity[];
    allowedZoneTypes: string[];
    squadSize: number;
    allowDuplicates: boolean;
    hideUnplayableOperators: boolean;
    onlyOwnedOperators: boolean;
    onlyCompletedStages: boolean;
    onlyAvailableStages: boolean;
    onlyE2Operators: boolean;
    /** Stage IDs the user has explicitly opted out of. Empty = all stages allowed. */
    deselectedStageIds: string[];
}

export type ChallengeKind = "restriction" | "modifier" | "objective";

export type ChallengeType = "PLAIN" | "SQUAD_FILTER" | "STAGE";

export interface IChallengeBase {
    /** Stable identifier for the challenge (used for keys / debugging). */
    id: string;
    kind: ChallengeKind;
    title: string;
    description: string;
    /** Relative weight in the weighted pick (defaults to 1). Use 0 to disable temporarily. */
    weight?: number;
}

/** A challenge with no automatic effect */
export interface IPlainChallenge extends IChallengeBase {
    type: "PLAIN";
}

/** A challenge that filters the squad pool before the squad is rolled. */
export interface ISquadFilterChallenge extends IChallengeBase {
    type: "SQUAD_FILTER";
    /**
     * Predicate applied to the slim randomizer operator. Return true to KEEP the
     * operator. Filter runs after settings/roster filtering.
     */
    filter: (op: IRandomizerOperator) => boolean;
}

/** A challenge that only applies when the rolled stage matches a predicate. */
export interface IStageChallenge extends IChallengeBase {
    type: "STAGE";
    /** Return true if this challenge should be eligible for the given rolled stage. */
    match: (stage: IStage) => boolean;
}

export type IChallenge = IPlainChallenge | ISquadFilterChallenge | IStageChallenge;

export interface IRosterIndex {
    /** Set of operator IDs the user owns. */
    owned: Set<string>;
    /** Subset of owned operators with elite ≥ 2. */
    e2: Set<string>;
}
