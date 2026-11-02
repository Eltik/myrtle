import type { IRecruitableOperator, PotentialByOperator } from "./types";

/** One rarity a combination can roll: how many of its operators the roster does not hold, out of all of them. */
export interface IRarityNewOdds {
    rarity: number;
    unowned: number;
    total: number;
}

/**
 * The chance a recruit from this pool is an operator the roster does not hold, one entry per rarity, highest first.
 *
 * The game rolls the rarity first, at rates it does not publish, then picks uniformly among the matching operators of
 * that rarity. So `unowned / total` inside one rarity is exact, and the entries must never be folded into one number:
 * that would need the unpublished rarity weights. Null when there is no roster to compare against.
 */
export function newOperatorOdds(operators: readonly IRecruitableOperator[], potentialByOperator: PotentialByOperator | null | undefined): IRarityNewOdds[] | null {
    if (!potentialByOperator) return null;
    const byRarity = new Map<number, IRarityNewOdds>();
    for (const op of operators) {
        let entry = byRarity.get(op.rarity);
        if (!entry) {
            entry = { rarity: op.rarity, unowned: 0, total: 0 };
            byRarity.set(op.rarity, entry);
        }
        entry.total += 1;
        if (!potentialByOperator.has(op.id)) entry.unowned += 1;
    }
    return [...byRarity.values()].sort((a, b) => b.rarity - a.rarity);
}

/** Whole percent, kept off 0 and 100 unless the odds really are none or all, so a rounded figure never reads as a certainty. */
export function newOddsPercent({ unowned, total }: IRarityNewOdds): number {
    if (unowned === 0) return 0;
    if (unowned === total) return 100;
    return Math.min(99, Math.max(1, Math.round((unowned / total) * 100)));
}
