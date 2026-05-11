export type TagType = "qualification" | "position" | "class" | "affix";

export interface IRecruitmentTag {
    id: number;
    name: string;
    type: TagType;
}

export interface IRecruitableOperator {
    id: string;
    name: string;
    rarity: number;
    profession: string;
    position: string;
    tagList: string[];
}

export interface IRecruitableOperatorWithTags {
    id: string;
    name: string;
    rarity: string;
    profession: string;
    position: string;
    tagList: string[];
}

export interface ITagCombinationResult {
    tags: number[];
    tagNames: string[];
    operators: IRecruitableOperator[];
    guaranteedRarity: number;
    minRarity: number;
    maxRarity: number;
}

export type OperatorSortMode = "rarity-desc" | "common-first";

export interface ICalculatorOptions {
    showLowRarity?: boolean;
    includeRobots?: boolean;
    operatorSortMode?: OperatorSortMode;
}
