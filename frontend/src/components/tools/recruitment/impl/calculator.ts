import { rarityToNumber } from "#/lib/utils";
import { SENIOR_OPERATOR_TAG_ID, TOP_OPERATOR_TAG_ID } from "./constants";
import type { ICalculatorOptions, IRecruitableOperator, IRecruitableOperatorWithTags, ITagCombinationResult, OperatorSortMode } from "./types";

// "Highest rarity first": 6 > 5 > 4 > Robot(1) > 3 > 2.
// Robots sit just above 3-stars because they are valuable guaranteed pulls.
const RARITY_DESC_PRIORITY: Record<number, number> = { 6: 0, 5: 1, 4: 2, 1: 3, 3: 4, 2: 5 };
// "Most common first": Robot(1) > 3 > 4 > 2 > 5 > 6.
// 3-stars are the most common recruit, with robots elevated above them.
const COMMON_FIRST_PRIORITY: Record<number, number> = { 1: 0, 3: 1, 4: 2, 2: 3, 5: 4, 6: 5 };

function sortOperators(operators: IRecruitableOperator[], mode: OperatorSortMode): IRecruitableOperator[] {
    const priority = mode === "common-first" ? COMMON_FIRST_PRIORITY : RARITY_DESC_PRIORITY;
    return [...operators].sort((a, b) => {
        const diff = (priority[a.rarity] ?? 99) - (priority[b.rarity] ?? 99);
        if (diff !== 0) return diff;
        return a.name.localeCompare(b.name);
    });
}

function resultHasRarity(result: ITagCombinationResult, rarity: number): boolean {
    return result.operators.some((op) => op.rarity === rarity);
}

function isRobotValuable(result: ITagCombinationResult): boolean {
    return resultHasRarity(result, 1) && !resultHasRarity(result, 3);
}

function operatorMatchesTag(op: IRecruitableOperatorWithTags, tagId: number, tagName: string): boolean {
    const rarity = rarityToNumber(op.rarity);

    switch (tagId) {
        case 9:
            return op.position === "MELEE";
        case 10:
            return op.position === "RANGED";
        case 1:
            return op.profession === "WARRIOR";
        case 2:
            return op.profession === "SNIPER";
        case 3:
            return op.profession === "TANK";
        case 4:
            return op.profession === "MEDIC";
        case 5:
            return op.profession === "SUPPORT";
        case 6:
            return op.profession === "CASTER";
        case 7:
            return op.profession === "SPECIAL";
        case 8:
            return op.profession === "PIONEER";
        case 11:
            return rarity === 6;
        case 14:
            return rarity === 5;
        case 17:
            return rarity === 2;
        case 28:
            return rarity === 1;
        default:
            return op.tagList.includes(tagName);
    }
}

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

export function calculateResults(selectedTags: { id: number; name: string }[], allOperators: IRecruitableOperatorWithTags[], options: ICalculatorOptions = {}): ITagCombinationResult[] {
    const { includeRobots = true, prioritizeFiveStarChance = true, operatorSortMode = "rarity-desc" } = options;

    if (selectedTags.length === 0) return [];

    const hasTopOperator = selectedTags.some((t) => t.id === TOP_OPERATOR_TAG_ID);

    const combinations = getCombinations(selectedTags, selectedTags.length);
    const results: ITagCombinationResult[] = [];

    for (const combo of combinations) {
        const matching = allOperators.filter((op) => {
            const rarity = rarityToNumber(op.rarity);
            if (!hasTopOperator && rarity === 6) return false;

            return combo.every((tag) => operatorMatchesTag(op, tag.id, tag.name));
        });

        if (matching.length === 0) continue;

        let filteredOps: IRecruitableOperator[] = matching.map((op) => ({
            id: op.id,
            name: op.name,
            rarity: rarityToNumber(op.rarity),
            profession: op.profession,
            position: op.position,
            tagList: op.tagList,
        }));

        const tagIds = combo.map((t) => t.id);

        if (!includeRobots) {
            filteredOps = filteredOps.filter((op) => op.rarity !== 1);
        }

        if (filteredOps.length === 0) continue;

        const minRarity = Math.min(...filteredOps.map((op) => op.rarity));
        const maxRarity = Math.max(...filteredOps.map((op) => op.rarity));

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
        const aGuaranteedHigh = a.guaranteedRarity >= 5 ? a.guaranteedRarity : 0;
        const bGuaranteedHigh = b.guaranteedRarity >= 5 ? b.guaranteedRarity : 0;
        if (aGuaranteedHigh !== bGuaranteedHigh) return bGuaranteedHigh - aGuaranteedHigh;

        if (prioritizeFiveStarChance) {
            const aFiveChance = a.maxRarity === 5 && a.guaranteedRarity < 5 ? 1 : 0;
            const bFiveChance = b.maxRarity === 5 && b.guaranteedRarity < 5 ? 1 : 0;
            if (aFiveChance !== bFiveChance) return bFiveChance - aFiveChance;
        }

        const aRobot = isRobotValuable(a) ? 1 : 0;
        const bRobot = isRobotValuable(b) ? 1 : 0;
        if (aRobot !== bRobot) return bRobot - aRobot;

        if (b.guaranteedRarity !== a.guaranteedRarity) return b.guaranteedRarity - a.guaranteedRarity;

        if (operatorSortMode === "common-first") {
            if (a.operators.length !== b.operators.length) return b.operators.length - a.operators.length;
            return b.maxRarity - a.maxRarity;
        }
        if (b.maxRarity !== a.maxRarity) return b.maxRarity - a.maxRarity;
        return a.operators.length - b.operators.length;
    });
}
