// Backend API types
export interface GachaTag {
    tagId: number;
    tagName: string;
    tagGroup: number;
}

export interface RecruitmentData {
    tags: GachaTag[];
    tagMap: Record<number, GachaTag>;
    tagNameMap: Record<string, GachaTag>;
    recruitDetail: string;
    recruitPool: {
        recruitTimeTable: { recruitPrice: number }[];
    };
}

// Backend operator from calculate response
export interface BackendOperator {
    id: string;
    name: string;
    rarity: string; // "TIER_6", "TIER_5", etc.
    profession: string;
    position: string;
    tagList: string[];
}

// Backend calculate response
export interface CalculateResponse {
    recruitment: Array<{
        label: string[];
        operators: BackendOperator[];
    }>;
}

// Frontend types
export type TagType = "qualification" | "position" | "class" | "affix";

export interface RecruitmentTag {
    id: number;
    name: string;
    type: TagType;
}

export interface RecruitableOperator {
    id: string;
    name: string;
    rarity: number;
    profession: string;
    position: string;
}

export interface TagCombinationResult {
    tags: number[];
    tagNames: string[];
    operators: RecruitableOperator[];
    guaranteedRarity: number;
    minRarity: number;
    maxRarity: number;
}
