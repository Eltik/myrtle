/**
 * Items listed in the "Rank by" picker before any search, in this order. The
 * first nine are `user_status` currencies on the backend and the rest are
 * ordinary `user_items` rows; the API takes the game item id for both, so this
 * list is purely a matter of what deserves a place up front.
 */
export const FEATURED_ITEMS = [
    "4002", // Originite Prime
    "4003", // Orundum
    "4001", // LMD
    "7003", // Headhunting Permit
    "7004", // Ten-roll Headhunting Permit
    "4004", // Distinction Certificate
    "4005", // Commendation Certificate
    "4006", // Purchase Certificate
    "SOCIAL_PT", // Credit
    "mod_unlock_token", // Module Data Block
    "3003", // Pure Gold
    "3401", // Furniture Part
] as const;

/** Item ids are `[A-Za-z0-9_]{1,50}`, the same rule the API enforces. */
export function isValidItemId(value: unknown): value is string {
    return typeof value === "string" && /^[A-Za-z0-9_]{1,50}$/.test(value);
}
