import type { OperatorPosition, OperatorProfession, OperatorRarity } from "#/types/operators";

export interface IRandomizerOperator {
    id: string;
    name: string;
    rarity: OperatorRarity;
    profession: OperatorProfession;
    subProfessionId: string;
    position: OperatorPosition;
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

export interface IChallenge {
    kind: ChallengeKind;
    title: string;
    description: string;
}

export interface IRosterIndex {
    /** Set of operator IDs the user owns. */
    owned: Set<string>;
    /** Subset of owned operators with elite ≥ 2. */
    e2: Set<string>;
}
