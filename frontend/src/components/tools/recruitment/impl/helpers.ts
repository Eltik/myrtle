import { TAG_ID_TO_TYPE_MAP } from "./constants";
import type { IRecruitmentTag, TagType } from "./types";

export interface IGachaTag {
    tagId: number;
    tagName: string;
    tagGroup: number;
}

export function transformTags(tags: IGachaTag[]): IRecruitmentTag[] {
    return tags.map((tag) => ({
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

    groups.class.sort((a, b) => a.name.localeCompare(b.name));
    groups.affix.sort((a, b) => a.name.localeCompare(b.name));

    return groups;
}

export function getStarsDisplay(rarity: number): string {
    return "★".repeat(rarity);
}
