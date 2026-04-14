import type { CalculatorOptions, OperatorSortMode, RecruitableOperator, RecruitableOperatorWithTags, TagCombinationResult } from "~/types/frontend/impl/tools/recruitment";
import { SENIOR_OPERATOR_TAG_ID, TOP_OPERATOR_TAG_ID } from "./constants";

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
 * Get sort priority for "common-first" mode
 * Common operators (4★, 3★, 2★) appear before uncommon (5★, 1★)
 * Order: 4★ → 3★ → 2★ → 5★ → 1★
 */
function getCommonFirstPriority(rarity: number): number {
    const priorityMap: Record<number, number> = {
        4: 0, // Most common, shown first
        3: 1,
        2: 2,
        5: 3, // Uncommon
        1: 4, // Robots, shown last
        6: 5, // 6★ rarely in recruitment pool anyway
    };
    return priorityMap[rarity] ?? 99;
}

/**
 * Sort operators based on the selected sort mode
 */
function sortOperators(operators: RecruitableOperator[], mode: OperatorSortMode): RecruitableOperator[] {
    return [...operators].sort((a, b) => {
        if (mode === "common-first") {
            const priorityDiff = getCommonFirstPriority(a.rarity) - getCommonFirstPriority(b.rarity);
            if (priorityDiff !== 0) return priorityDiff;
            return b.rarity - a.rarity;
        }
        return b.rarity - a.rarity;
    });
}

/**
 * Check if an operator matches a specific tag
 */
function operatorMatchesTag(op: RecruitableOperatorWithTags, tagId: number, tagName: string): boolean {
    const rarity = rarityToNumber(op.rarity);

    switch (tagId) {
        case 9:
            return op.position === "MELEE";
        case 10:
            return op.position === "RANGED";
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
        case 11:
            return rarity === 6; // Top Operator
        case 14:
            return rarity === 5; // Senior Operator
        case 17:
            return rarity === 2; // Starter
        case 28:
            return rarity === 1; // Robot
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
export function calculateResults(selectedTags: { id: number; name: string }[], allOperators: RecruitableOperatorWithTags[], options: CalculatorOptions = {}): TagCombinationResult[] {
    const { showLowRarity = false, includeRobots = true, operatorSortMode = "rarity-desc" } = options;

    if (selectedTags.length === 0) return [];

    const hasTopOperator = selectedTags.some((t) => t.id === TOP_OPERATOR_TAG_ID);

    const combinations = getCombinations(selectedTags, selectedTags.length);
    const results: TagCombinationResult[] = [];

    for (const combo of combinations) {
        const matching = allOperators.filter((op) => {
            const rarity = rarityToNumber(op.rarity);

            if (!hasTopOperator && rarity === 6) return false;

            return combo.every((tag) => operatorMatchesTag(op, tag.id, tag.name));
        });

        if (matching.length === 0) continue;

        let filteredOps: RecruitableOperator[] = matching.map((op) => ({
            id: op.id,
            name: op.name,
            rarity: rarityToNumber(op.rarity),
            profession: op.profession,
            position: op.position,
        }));

        if (!includeRobots) {
            filteredOps = filteredOps.filter((op) => op.rarity !== 1);
        }

        if (!showLowRarity) {
            filteredOps = filteredOps.filter((op) => op.rarity >= 3 || op.rarity === 1);
        }

        if (filteredOps.length === 0) continue;

        const minRarity = Math.min(...filteredOps.map((op) => op.rarity));
        const maxRarity = Math.max(...filteredOps.map((op) => op.rarity));

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
            operators: sortOperators(filteredOps, operatorSortMode),
            guaranteedRarity,
            minRarity,
            maxRarity,
        });
    }

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
