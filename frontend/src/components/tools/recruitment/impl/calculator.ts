import { rarityToNumber } from "#/lib/utils";
import { SENIOR_OPERATOR_TAG_ID, STARTER_TAG_ID, TOP_OPERATOR_TAG_ID } from "./constants";
import type { ICalculatorOptions, IRecruitableOperator, IRecruitableOperatorWithTags, ITagCombinationResult, OperatorSortMode } from "./types";

function getCommonFirstPriority(rarity: number): number {
    const priorityMap: Record<number, number> = {
        4: 0,
        3: 1,
        2: 2,
        5: 3,
        1: 4,
        6: 5,
    };
    return priorityMap[rarity] ?? 99;
}

function sortOperators(operators: IRecruitableOperator[], mode: OperatorSortMode): IRecruitableOperator[] {
    return [...operators].sort((a, b) => {
        if (mode === "common-first") {
            const priorityDiff = getCommonFirstPriority(a.rarity) - getCommonFirstPriority(b.rarity);
            if (priorityDiff !== 0) return priorityDiff;
            return b.rarity - a.rarity;
        }
        return b.rarity - a.rarity;
    });
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
    const { showLowRarity = false, includeRobots = true, operatorSortMode = "rarity-desc" } = options;

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
        }));

        const tagIds = combo.map((t) => t.id);

        if (!includeRobots) {
            filteredOps = filteredOps.filter((op) => op.rarity !== 1);
        }

        if (!showLowRarity) {
            const wantsStarter = tagIds.includes(STARTER_TAG_ID);
            filteredOps = filteredOps.filter((op) => op.rarity >= 3 || op.rarity === 1 || (op.rarity === 2 && wantsStarter));
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
        if (b.guaranteedRarity !== a.guaranteedRarity) {
            return b.guaranteedRarity - a.guaranteedRarity;
        }
        if (a.operators.length !== b.operators.length) {
            return a.operators.length - b.operators.length;
        }
        return b.maxRarity - a.maxRarity;
    });
}
