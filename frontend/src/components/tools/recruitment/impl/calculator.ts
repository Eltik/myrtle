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

// A combination is only as good as its worst possible outcome, ranked on the same
// scale as "highest rarity first" (6 > 5 > 4 > Robot > 3 > 2): a pool that is all
// 4★+ must always outrank one that can still drop a 1★/3★, and a robot lock stays
// above 3★ floors because robots are valuable guaranteed pulls.
function floorPriority(result: ITagCombinationResult): number {
    if (result.guaranteedRarity >= 5) {
        return RARITY_DESC_PRIORITY[result.guaranteedRarity] ?? 99;
    }
    return Math.max(...result.operators.map((op) => RARITY_DESC_PRIORITY[op.rarity] ?? 99));
}

function fiveStarChance(result: ITagCombinationResult): number {
    if (result.guaranteedRarity >= 5) return 1;
    return result.fiveStarCount / result.operators.length;
}

// In-game ids for the position, class, and rarity-qualification tags. Affix tags
// (Nuker, Summon, ...) carry no special semantics and match by name instead.
const POSITION_BY_TAG_ID: Record<number, string> = { 9: "MELEE", 10: "RANGED" };
const PROFESSION_BY_TAG_ID: Record<number, string> = { 1: "WARRIOR", 2: "SNIPER", 3: "TANK", 4: "MEDIC", 5: "SUPPORT", 6: "CASTER", 7: "SPECIAL", 8: "PIONEER" };
const RARITY_BY_TAG_ID: Record<number, number> = {
    [TOP_OPERATOR_TAG_ID]: 6,
    [SENIOR_OPERATOR_TAG_ID]: 5,
    17: 2, // Starter
    28: 1, // Robot
};

function operatorMatchesTag(op: IRecruitableOperatorWithTags, tagId: number, tagName: string): boolean {
    const position = POSITION_BY_TAG_ID[tagId];
    if (position) return op.position === position;

    const profession = PROFESSION_BY_TAG_ID[tagId];
    if (profession) return op.profession === profession;

    const rarity = RARITY_BY_TAG_ID[tagId];
    if (rarity) return rarityToNumber(op.rarity) === rarity;

    return op.tagList.includes(tagName);
}

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

export function calculateResults(selectedTags: { id: number; name: string }[], allOperators: IRecruitableOperatorWithTags[], options: ICalculatorOptions = {}): ITagCombinationResult[] {
    const { includeRobots = true, includeTwoStars = true, includeThreeStars = true, operatorSortMode = "rarity-desc" } = options;

    if (selectedTags.length === 0) return [];

    const combinations = getCombinations(selectedTags, selectedTags.length);
    const results: ITagCombinationResult[] = [];

    for (const combo of combinations) {
        const comboHasTopOperator = combo.some((t) => t.id === TOP_OPERATOR_TAG_ID);

        const matching = allOperators.filter((op) => {
            const rarity = rarityToNumber(op.rarity);
            if (!comboHasTopOperator && rarity === 6) return false;

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
        if (!includeTwoStars) {
            filteredOps = filteredOps.filter((op) => op.rarity !== 2);
        }
        if (!includeThreeStars) {
            filteredOps = filteredOps.filter((op) => op.rarity !== 3);
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
            maxRarity,
            fiveStarCount: filteredOps.filter((op) => op.rarity >= 5).length,
        });
    }

    return results.sort((a, b) => {
        const chanceDiff = fiveStarChance(b) - fiveStarChance(a);
        if (chanceDiff !== 0) return chanceDiff;

        const floorDiff = floorPriority(a) - floorPriority(b);
        if (floorDiff !== 0) return floorDiff;

        if (operatorSortMode === "common-first") {
            if (a.operators.length !== b.operators.length) return b.operators.length - a.operators.length;
            return b.maxRarity - a.maxRarity;
        }
        if (b.maxRarity !== a.maxRarity) return b.maxRarity - a.maxRarity;
        return a.operators.length - b.operators.length;
    });
}
