import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import type { ReviewOutfit } from "#/types/generated/ReviewOutfit";
import type { ReviewWindow } from "#/types/generated/ReviewWindow";
import type { messages as reviewMessages } from "./reviews.messages";

/** The `t` this label helper needs, narrowed to the keys it can render. */
export type ReviewT = TypedT<typeof reviewMessages>;

/** Default `t` for a caller outside an `I18nProvider`; resolves against the bundled source catalog. */
const sourceT: ReviewT = (key, values) => formatMessage(sourceMessage(fullMessageKey("tools", key)) ?? key, DEFAULT_LOCALE, values);

export const REVIEW_NAME_CN = "罗德岛风尚回顾";
export const REVIEW_NAME_EN = "Rhodes Fashion Review";
const FIRST_GAME_YEAR = 2019;

/** The edition's stock: every eligible outfit released on CN up to its cutoff (about two years before it). */
export function reviewOutfits(review: ReviewWindow, pool: ReviewOutfit[]): ReviewOutfit[] {
    return pool.filter((o) => o.cnGetTime <= review.poolCutoff);
}

export function reviewYearGroup(o: ReviewOutfit, t: ReviewT = sourceT): string {
    const year = new Date((o.enGetTime ?? o.cnGetTime) * 1000).getUTCFullYear();
    return t("release.review.yearGroup", { gameYear: year - FIRST_GAME_YEAR + 1, calendarYear: year });
}
