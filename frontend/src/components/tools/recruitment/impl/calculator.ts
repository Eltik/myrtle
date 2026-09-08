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
export function guaranteedFloorRarity(result: ITagCombinationResult): number {
    if (result.guaranteedRarity >= 5) return result.guaranteedRarity;

    // The worst rarity on the PRIORITY scale, which is not the numeric minimum: a
    // pool holding a Robot and a 3★ floors at 3★, because a robot is the better
    // outcome of the two. `guaranteedRarity` is a plain `Math.min` and would say
    // Robot. The card renders whatever this returns, so what a user reads is the
    // same value the ranking used.
    let worst = result.guaranteedRarity;
    for (const op of result.operators) {
        if ((RARITY_DESC_PRIORITY[op.rarity] ?? 99) > (RARITY_DESC_PRIORITY[worst] ?? 99)) worst = op.rarity;
    }
    return worst;
}

function floorPriority(result: ITagCombinationResult): number {
    return RARITY_DESC_PRIORITY[guaranteedFloorRarity(result)] ?? 99;
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

// ============================================================================
// Ranking precedence
// ============================================================================
//
// THE ORDER OF `rankingFor` BELOW IS THE DECISION. Read this before changing it.
//
//   1. PRIMARY  - the guaranteed floor. What is the WORST this combination can
//                 hand you?
//   2. TIEBREAK - pool composition (5★ share, breadth, ceiling). Only ever
//                 consulted when two combinations have the same floor.
//
// Pool composition must never outrank the floor. The tool exists to decide
// where to spend a nine-hour recruit, and for that decision the floor is the
// whole question: a pool padded with 5★s that can still hand you a Popukar is a
// worse bet than one that cannot go below a Robot. Concretely, a Nuker combo
// floored at Robot outranks a Survival combo that can still drop a 3★, however
// many 5★s Survival's pool happens to contain.
//
// The 5★ share is also not a real drop probability - it is pool share, which
// this project has already concluded is not the in-game rate, which is why no
// percentage is displayed anywhere in this tool. A number we deliberately will
// not show a user because it is wrong must not be deciding the order either.
//
// This precedence has been inverted three times, each pass silently undoing the
// last, because it is easy to reorder two lines inside an anonymous comparator
// without noticing there was an argument attached to them. Hence the shape
// below: the precedence is an ordered list of named comparators rather than a
// sequence of statements, and `assertFloorOutranksPoolShare` fails loudly in dev
// if the first two are ever swapped. If you are here to put pool composition
// back on top, you need to delete that check too - please take that as the
// argument it is meant to be, and change the reasoning above rather than only
// the code.

type ResultComparator = (a: ITagCombinationResult, b: ITagCombinationResult) => number;

/** PRIMARY: better guaranteed worst case first. */
const byGuaranteedFloor: ResultComparator = (a, b) => floorPriority(a) - floorPriority(b);

/** TIEBREAK: among equal floors, prefer the denser 5★ pool. */
const byFiveStarShare: ResultComparator = (a, b) => fiveStarChance(b) - fiveStarChance(a);

/** TIEBREAK: prefer the broader pool, then the higher ceiling. */
const byBreadthThenCeiling: ResultComparator = (a, b) => (a.operators.length !== b.operators.length ? b.operators.length - a.operators.length : b.maxRarity - a.maxRarity);

/** TIEBREAK: prefer the higher ceiling, then the tighter pool. */
const byCeilingThenPrecision: ResultComparator = (a, b) => (b.maxRarity !== a.maxRarity ? b.maxRarity - a.maxRarity : a.operators.length - b.operators.length);

function rankingFor(mode: OperatorSortMode): readonly ResultComparator[] {
    // The operator sort mode only ever reorders the TIEBREAKS. Both modes keep
    // the floor first; a display preference does not change which recruit is the
    // better bet.
    return [byGuaranteedFloor, byFiveStarShare, mode === "common-first" ? byBreadthThenCeiling : byCeilingThenPrecision];
}

function rankCombinations(mode: OperatorSortMode): ResultComparator {
    const ranking = rankingFor(mode);
    return (a, b) => {
        for (const compare of ranking) {
            const diff = compare(a, b);
            if (diff !== 0) return diff;
        }
        return 0;
    };
}

/**
 * The regression guard. There are deliberately no test files in this frontend,
 * so the invariant lives here and runs in dev instead: two combinations where
 * the floor and the 5★ share disagree - a Robot-floored pool with no 5★s
 * against a 3★-floored pool that is half 5★ - must rank the better floor first.
 * This is the exact comparison that has regressed three times.
 */
function assertFloorOutranksPoolShare(): void {
    const make = (rarities: number[]): ITagCombinationResult => ({
        tags: [],
        tagNames: [],
        operators: rarities.map((rarity, i) => ({ id: `op${i}`, name: `op${i}`, rarity, profession: "", position: "", tagList: [] })),
        guaranteedRarity: Math.min(...rarities),
        maxRarity: Math.max(...rarities),
        fiveStarCount: rarities.filter((r) => r >= 5).length,
    });

    // Robot floor, no 5★s. Worse pool, better guarantee - must come first.
    const robotFloor = make([1, 4]);
    // 3★ floor, half the pool is 5★. Better pool, worse guarantee.
    const threeStarFloor = make([3, 5]);

    for (const mode of ["rarity-desc", "common-first"] as const) {
        if (rankCombinations(mode)(robotFloor, threeStarFloor) >= 0) {
            throw new Error(`recruitment ranking: pool composition is outranking the guaranteed floor in "${mode}" mode. See the precedence note in calculator.ts - the floor is the primary key and pool share is a tiebreak beneath it.`);
        }
    }
}

if (import.meta.env.DEV) {
    assertFloorOutranksPoolShare();
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

    return [...results].sort(rankCombinations(operatorSortMode));
}
