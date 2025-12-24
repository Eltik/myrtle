import { SENIOR_OPERATOR_TAG_ID, TAG_GROUP_MAP, TOP_OPERATOR_TAG_ID } from "./constants";
import type { GachaTag, RecruitableOperator, RecruitmentTag, TagCombinationResult, TagType } from "./types";

/**
 * Convert rarity string to number
 */
export function rarityToNumber(rarity: string): number {
    const rarityMap: Record<string, number> = {
        TIER_6: 6,
        TIER_5: 5,
        TIER_4: 4,
        TIER_3: 3,
        TIER_2: 2,
        TIER_1: 1,
    };
    return rarityMap[rarity] ?? 1;
}

/**
 * Transform backend tags to frontend format with type information
 */
export function transformTags(tags: GachaTag[]): RecruitmentTag[] {
    return tags.map((tag) => ({
        id: tag.tagId,
        name: tag.tagName,
        type: TAG_GROUP_MAP[tag.tagGroup] ?? ("affix" as TagType),
    }));
}

/**
 * Group tags by their type
 */
export function groupTagsByType(tags: RecruitmentTag[]): Record<TagType, RecruitmentTag[]> {
    const groups: Record<TagType, RecruitmentTag[]> = {
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
 * Get all combinations of items from an array up to a certain size
 */
export function getCombinations<T>(arr: T[], maxSize: number): T[][] {
    const result: T[][] = [];

    function combine(start: number, current: T[]) {
        if (current.length > 0) {
            result.push([...current]);
        }
        if (current.length >= maxSize) return;

        for (let i = start; i < arr.length; i++) {
            const item = arr[i];
            if (item !== undefined) {
                current.push(item);
                combine(i + 1, current);
                current.pop();
            }
        }
    }

    combine(0, []);
    return result;
}

// API response type for calculate endpoint
interface CalculateApiResponse {
    data: Array<{
        id: string;
        name: string;
        rarity: number;
        profession: string;
        position: string;
        guaranteed: boolean;
        tags: string[];
    }>;
}

/**
 * Fetch operators for a specific tag combination from the API
 */
export async function fetchOperatorsForTags(tagIds: number[]): Promise<RecruitableOperator[]> {
    const recruitmentString = tagIds.join(",");

    const response = await fetch("/api/static", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            type: "gacha",
            method: "calculate",
            tags: tagIds.map(String),
        }),
    });

    if (!response.ok) {
        console.error("Failed to fetch operators for tags:", recruitmentString);
        return [];
    }

    const data = (await response.json()) as CalculateApiResponse;

    if (!data.data || !Array.isArray(data.data)) {
        return [];
    }

    // Deduplicate operators by ID
    const operatorMap = new Map<string, RecruitableOperator>();
    for (const op of data.data) {
        if (!operatorMap.has(op.id)) {
            operatorMap.set(op.id, {
                id: op.id,
                name: op.name,
                rarity: op.rarity,
                profession: op.profession,
                position: op.position,
            });
        }
    }

    return Array.from(operatorMap.values());
}

/**
 * Calculate recruitment results for all tag combinations
 * Makes API calls for each combination
 */
export async function calculateRecruitmentResults(
    selectedTags: { id: number; name: string }[],
    options: {
        showLowRarity?: boolean;
        includeRobots?: boolean;
    } = {},
): Promise<TagCombinationResult[]> {
    const { showLowRarity = false, includeRobots = true } = options;

    if (selectedTags.length === 0) return [];

    const combinations = getCombinations(selectedTags, selectedTags.length);
    const results: TagCombinationResult[] = [];

    // Fetch operators for all combinations in parallel
    const fetchPromises = combinations.map(async (combo) => {
        const tagIds = combo.map((t) => t.id);
        const operators = await fetchOperatorsForTags(tagIds);
        return { combo, operators };
    });

    const fetchResults = await Promise.all(fetchPromises);

    for (const { combo, operators } of fetchResults) {
        if (operators.length === 0) continue;

        // Filter based on options
        let filteredOps = operators;

        // Filter out robots if requested (robots are 1-star)
        if (!includeRobots) {
            filteredOps = filteredOps.filter((op) => op.rarity !== 1);
        }

        // Filter out low rarity if requested (keep 3+ star or robots)
        if (!showLowRarity) {
            filteredOps = filteredOps.filter((op) => op.rarity >= 3 || op.rarity === 1);
        }

        if (filteredOps.length === 0) continue;

        const minRarity = Math.min(...filteredOps.map((op) => op.rarity));
        const maxRarity = Math.max(...filteredOps.map((op) => op.rarity));

        // Calculate guaranteed rarity
        const tagIds = combo.map((t) => t.id);
        let guaranteedRarity = minRarity;
        if (tagIds.includes(TOP_OPERATOR_TAG_ID)) {
            guaranteedRarity = 6;
        } else if (tagIds.includes(SENIOR_OPERATOR_TAG_ID)) {
            guaranteedRarity = Math.max(5, minRarity);
        }

        results.push({
            tags: tagIds,
            tagNames: combo.map((t) => t.name),
            operators: filteredOps.sort((a, b) => b.rarity - a.rarity),
            guaranteedRarity,
            minRarity,
            maxRarity,
        });
    }

    // Sort results by guaranteed rarity (desc), then by number of operators (asc)
    return results.sort((a, b) => {
        if (b.guaranteedRarity !== a.guaranteedRarity) {
            return b.guaranteedRarity - a.guaranteedRarity;
        }
        if (a.operators.length !== b.operators.length) {
            return a.operators.length - b.operators.length;
        }
        return b.maxRarity - a.maxRarity;
    });
}
