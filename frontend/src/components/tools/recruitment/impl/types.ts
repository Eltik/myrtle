export type TagType = "qualification" | "position" | "class" | "affix";

export interface IRecruitmentTag {
    id: number;
    name: string;
    type: TagType;
}

/**
 * What one potential rank buys, read off the structured game data rather than
 * the localised description string so the label renders the same on every
 * server. `stat` is a BUFF rank's first attribute modifier, `talent` is a
 * CUSTOM rank resolved through the talent candidates' `requiredPotentialRank`
 * (0-based talent index), `text` is the description verbatim for any shape
 * the resolver does not know, so nothing is silently dropped.
 */
export type IRecruitPotential = { kind: "stat"; attribute: string; value: number } | { kind: "talent"; index: number; of: number } | { kind: "text"; text: string };

/** One recruitable operator with everything the calculator and the cards read, precomputed once by the recruitment server fn. */
export interface IRecruitableOperator {
    id: string;
    name: string;
    /** 1 (Robot) to 6. */
    rarity: number;
    profession: string;
    position: string;
    /** Recruitment tag names: position, class, rarity qualification, then the game's own affix tags. */
    tagList: string[];
    /** Index i is the gain from potential i+1 -> i+2 (0-based, as the roster stores it). */
    potentials: IRecruitPotential[];
}

export interface ITagCombinationResult {
    tags: number[];
    tagNames: string[];
    operators: IRecruitableOperator[];
    guaranteedRarity: number;
    maxRarity: number;
    fiveStarCount: number;
}

export type OperatorSortMode = "rarity-desc" | "common-first" | "potential-asc";

/** Operator id -> 0-based potential (0 = P1, 5 = P6), reduced from the signed-in roster. */
export type PotentialByOperator = ReadonlyMap<string, number>;

export interface ICalculatorOptions {
    includeRobots?: boolean;
    includeTwoStars?: boolean;
    includeThreeStars?: boolean;
    operatorSortMode?: OperatorSortMode;
    /** Absent when nobody is signed in; only the "potential-asc" sort reads it, the combination ranking never does. */
    potentialByOperator?: PotentialByOperator;
}

/** The options the panel edits: every calculator input except the roster, which comes from the session. */
export type ICalculatorSettings = Required<Omit<ICalculatorOptions, "potentialByOperator">>;

/** The two roster overlays, kept apart from the calculator inputs: flipping one re-renders cards, never re-runs the search. */
export interface IRosterViewOptions {
    showPotentials: boolean;
    showNextUpgrade: boolean;
}

/** What the result cards need to draw the roster over the operators; null when there is no signed-in roster to draw. */
export interface IRosterOverlay extends IRosterViewOptions {
    potentialByOperator: PotentialByOperator;
}
