import { TAG_ID_TO_TYPE_MAP } from "./constants";
import type { IRecruitmentTag, TagType } from "./types";

// Backend tag format
export interface IGachaTag {
    tagId: number;
    tagName: string;
    tagGroup: number;
}

/**
 * Transform backend tags to frontend format with type information
 */
export function transformTags(tags: IGachaTag[]): IRecruitmentTag[] {
    return tags.map((tag) => ({
        id: tag.tagId,
        name: tag.tagName,
        type: TAG_ID_TO_TYPE_MAP[tag.tagId] ?? ("affix" as TagType),
    }));
}

/**
 * Group tags by their type
 */
export function groupTagsByType(tags: IRecruitmentTag[]): Record<TagType, IRecruitmentTag[]> {
    const groups: Record<TagType, IRecruitmentTag[]> = {
        qualification: [],
        position: [],
        class: [],
        affix: [],
    };

    for (const tag of tags) {
        groups[tag.type].push(tag);
    }

    return groups;
}

/**
 * Generate stars string based on rarity
 */
export function getStarsDisplay(rarity: number): string {
    return "★".repeat(rarity);
}
