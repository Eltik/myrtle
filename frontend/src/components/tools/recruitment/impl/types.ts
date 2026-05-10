// Recruitment calculator types

// Tag types for categorization
export type TagType = "qualification" | "position" | "class" | "affix";

// A recruitment tag from the game
export interface IRecruitmentTag {
    id: number;
    name: string;
    type: TagType;
}

// An operator that can be recruited
export interface IRecruitableOperator {
    id: string;
    name: string;
    rarity: number;
    profession: string;
    position: string;
}

// Operator with raw tag data for client-side calculation
export interface IRecruitableOperatorWithTags {
    id: string;
    name: string;
    rarity: string; // "TIER_6", "TIER_5", etc.
    profession: string; // "WARRIOR", "SNIPER", etc.
    position: string; // "MELEE", "RANGED"
    tagList: string[]; // Affix tags like "Nuker", "DPS", etc.
}

// Result of a tag combination calculation
export interface ITagCombinationResult {
    tags: number[];
    tagNames: string[];
    operators: IRecruitableOperator[];
    guaranteedRarity: number;
    minRarity: number;
    maxRarity: number;
}

// Sorting mode for operators within results
export type OperatorSortMode = "rarity-desc" | "common-first";

// Calculator options
export interface ICalculatorOptions {
    showLowRarity?: boolean;
    includeRobots?: boolean;
    operatorSortMode?: OperatorSortMode;
}
