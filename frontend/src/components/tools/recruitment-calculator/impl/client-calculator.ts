import { SENIOR_OPERATOR_TAG_ID, TOP_OPERATOR_TAG_ID } from "./constants";
import type { RecruitableOperator, RecruitableOperatorWithTags, TagCombinationResult } from "./types";

/**
 * Convert rarity string to number
 */
function rarityToNumber(rarity: string): number {
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
 * Check if an operator matches a specific tag
 */
function operatorMatchesTag(op: RecruitableOperatorWithTags, tagId: number, tagName: string): boolean {
    const rarity = rarityToNumber(op.rarity);

    switch (tagId) {
        // Position tags
        case 9:
            return op.position === "MELEE";
        case 10:
            return op.position === "RANGED";
        // Class tags - match backend profession enum values
        case 1:
            return op.profession === "WARRIOR"; // Guard
        case 2:
            return op.profession === "SNIPER";
        case 3:
            return op.profession === "TANK"; // Defender
        case 4:
            return op.profession === "MEDIC";
        case 5:
            return op.profession === "SUPPORT"; // Supporter
        case 6:
            return op.profession === "CASTER";
        case 7:
            return op.profession === "SPECIAL"; // Specialist
        case 8:
            return op.profession === "PIONEER"; // Vanguard
        // Rarity tags
        case 11:
            return rarity === 6; // Top Operator
        case 14:
            return rarity === 5; // Senior Operator
        case 17:
            return rarity === 2; // Starter
        case 28:
            return rarity === 1; // Robot
        // Affix tags - check tagList
        default:
            return op.tagList.includes(tagName);
    }
}

/**
 * Get all combinations of items from an array up to a certain size
 */
function getCombinations<T>(arr: T[], maxSize: number): T[][] {
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

/**
 * Calculate recruitment results
 */
export function calculateResults(
    selectedTags: { id: number; name: string }[],
    allOperators: RecruitableOperatorWithTags[],
    options: {
        showLowRarity?: boolean;
        includeRobots?: boolean;
    } = {},
): TagCombinationResult[] {
    const { showLowRarity = false, includeRobots = true } = options;

    if (selectedTags.length === 0) return [];

    // Check if Top Operator tag is selected (allows 6-star operators)
    const hasTopOperator = selectedTags.some((t) => t.id === TOP_OPERATOR_TAG_ID);

    // Generate all combinations of selected tags
    const combinations = getCombinations(selectedTags, selectedTags.length);
    const results: TagCombinationResult[] = [];

    for (const combo of combinations) {
        // Filter operators that match ALL tags in this combination
        const matching = allOperators.filter((op) => {
            const rarity = rarityToNumber(op.rarity);

            // Unless Top Operator tag is selected, exclude 6-star operators
            if (!hasTopOperator && rarity === 6) return false;

            // Must match ALL tags in this combination
            return combo.every((tag) => operatorMatchesTag(op, tag.id, tag.name));
        });

        if (matching.length === 0) continue;

        // Convert to frontend format with numeric rarity
        let filteredOps: RecruitableOperator[] = matching.map((op) => ({
            id: op.id,
            name: op.name,
            rarity: rarityToNumber(op.rarity),
            profession: op.profession,
            position: op.position,
        }));

        // Filter out robots if requested (robots are 1-star)
        if (!includeRobots) {
            filteredOps = filteredOps.filter((op) => op.rarity !== 1);
        }

        // Filter out low rarity if requested (keep 3+ star or robots if included)
        if (!showLowRarity) {
            filteredOps = filteredOps.filter((op) => op.rarity >= 3 || op.rarity === 1);
        }

        if (filteredOps.length === 0) continue;

        const minRarity = Math.min(...filteredOps.map((op) => op.rarity));
        const maxRarity = Math.max(...filteredOps.map((op) => op.rarity));

        // Calculate guaranteed rarity based on special tags
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

    // Sort results by guaranteed rarity (desc), then by number of operators (asc), then by max rarity (desc)
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
