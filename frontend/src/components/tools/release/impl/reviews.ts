import type { ReviewOutfit } from "#/types/generated/ReviewOutfit";
import type { ReviewWindow } from "#/types/generated/ReviewWindow";

export const REVIEW_NAME_CN = "罗德岛风尚回顾";
export const REVIEW_NAME_EN = "Rhodes Fashion Review";
const FIRST_GAME_YEAR = 2019;

/** The edition's stock: every eligible outfit released on CN up to its cutoff (about two years before it). */
export function reviewOutfits(review: ReviewWindow, pool: ReviewOutfit[]): ReviewOutfit[] {
    return pool.filter((o) => o.cnGetTime <= review.poolCutoff);
}

export function reviewYearGroup(o: ReviewOutfit): string {
    const year = new Date((o.enGetTime ?? o.cnGetTime) * 1000).getUTCFullYear();
    return `Year ${year - FIRST_GAME_YEAR + 1} (${year})`;
}
