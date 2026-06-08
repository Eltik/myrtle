import { HIDDEN_TAG_IDS, TAG_ID_TO_TYPE_MAP } from "./constants";
import type { IRecruitmentTag, TagType } from "./types";

export interface IGachaTag {
    tagId: number;
    tagName: string;
    tagGroup: number;
}

export function transformTags(tags: IGachaTag[]): IRecruitmentTag[] {
    return tags
        .filter((tag) => !HIDDEN_TAG_IDS.has(tag.tagId))
        .map((tag) => ({
            id: tag.tagId,
            name: tag.tagName,
            type: TAG_ID_TO_TYPE_MAP[tag.tagId] ?? ("affix" as TagType),
        }));
}

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

    for (const type of Object.keys(groups) as TagType[]) {
        groups[type].sort((a, b) => a.id - b.id);
    }

    return groups;
}

export function getStarsDisplay(rarity: number): string {
    return "★".repeat(rarity);
}
