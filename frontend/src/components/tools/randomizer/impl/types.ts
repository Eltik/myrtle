import type { OperatorPosition, OperatorProfession, OperatorRarity } from "#/types/operators";
import type { IStage } from "#/types/stages";
import type { ChallengeMessageKey } from "./challenges";

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
    /** Narrows the E2 rule to 6★; an owned operator of a lower rarity then passes it at any promotion. */
    e2SixStarOnly: boolean;
    /** Official tier list the pool is filtered by; `null` = no tier filter. */
    tierListSlug: string | null;
    /** Tier ids (or `UNRANKED_TIER_ID`) left out of the pool. Stored as exclusions so a tier added later is included. */
    deselectedTierIds: string[];
    /** Stage IDs the user has explicitly opted out of. Empty = all stages allowed. */
    deselectedStageIds: string[];
}

export type ChallengeKind = "restriction" | "modifier" | "objective";

export type ChallengeType = "PLAIN" | "SQUAD_FILTER" | "STAGE";

export interface IChallengeBase {
    /** Stable identifier for the challenge (used for keys / debugging). */
    id: string;
    kind: ChallengeKind;
    /** Key in `challenges.messages.ts`; the rendering component resolves it. */
    titleKey: ChallengeMessageKey;
    /** Key in `challenges.messages.ts`; the rendering component resolves it. */
    descKey: ChallengeMessageKey;
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
    match: (stage: IStage) => boolean;
}

export type IChallenge = IPlainChallenge | ISquadFilterChallenge | IStageChallenge;

export interface IRosterIndex {
    owned: Set<string>;
    /** Subset of owned operators with elite ≥ 2. */
    e2: Set<string>;
}
