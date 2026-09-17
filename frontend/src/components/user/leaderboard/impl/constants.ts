import type { messages as constantsMessages } from "./constants.messages";

export { DEFAULT_AVATAR_ID } from "#/lib/utils";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type LeaderboardMessageKey = keyof typeof constantsMessages & string;

export const PAGE_SIZE = 25;

export type LeaderboardScope = "global" | "friends";

export const SERVERS = ["EN", "JP", "CN", "KR", "TW"] as const;
export type ServerCode = (typeof SERVERS)[number];

/** `value` is the API's sort parameter and stays English; `labelKey` is the message. */
export const LEADERBOARD_SORTS = [
    { value: "total_score", labelKey: "leaderboard.sort.total" },
    { value: "operator_score", labelKey: "leaderboard.sort.operators" },
    { value: "stage_score", labelKey: "leaderboard.sort.stages" },
    { value: "roguelike_score", labelKey: "leaderboard.sort.roguelike" },
    { value: "sandbox_score", labelKey: "leaderboard.sort.sandbox" },
    { value: "medal_score", labelKey: "leaderboard.sort.medals" },
    { value: "base_score", labelKey: "leaderboard.sort.base" },
    // "Skins" is NOT listed, and the omission is the fix, not an oversight.
    // `user_scores.skin_score` is written as the literal 0.0 at both sites that
    // persist a score (`backend/src/core/regrade_job.rs` `regrade_one` and
    // `backend/src/bin/regrade_users.rs` `regrade_one`); there is no
    // `grade_skins` module and `UserGrade` has no skin field to source one
    // from. The column, the view, the generated type and this option all
    // existed end to end, so the sort ran and ranked every single player at
    // 0.0%. Offering a sort that cannot discriminate is worse than not
    // offering it.
    //
    // To restore it: add a skin section to `core::grade`, give `UserGrade` the
    // field, write it at both `regrade_one` sites, regrade, then re-add the
    // line below.
    //   { value: "skin_score", labelKey: "leaderboard.sort.skins" },
    //
    // The real owned-skin numbers already exist, but on `v_user_profile`
    // (`skin_count`, `non_default_skin_count`), not on `v_leaderboard` - a
    // count-based sort would need those columns added to that view and the
    // bindings regenerated.
] as const satisfies ReadonlyArray<{ value: string; labelKey: LeaderboardMessageKey }>;
export type LeaderboardSort = (typeof LEADERBOARD_SORTS)[number]["value"];

export function toPct(score01: number | null | undefined): number {
    if (score01 == null || Number.isNaN(score01)) return 0;
    return Math.max(0, Math.min(100, score01 * 100));
}

export function formatPct(score01: number | null | undefined, digits = 1): string {
    return `${toPct(score01).toFixed(digits)}%`;
}

/**
 * `value` is the API's interval parameter. The other three are message keys for
 * the same window worded three ways: on the interval button (`shortKey`), in
 * the movers-card heading (`subtitleKey`) and at the end of a sentence
 * (`sinceKey`).
 */
export const INTERVALS = [
    { value: "1 day", shortKey: "leaderboard.interval.day.short", labelKey: "leaderboard.interval.day.label", subtitleKey: "leaderboard.interval.day.subtitle", sinceKey: "leaderboard.interval.day.since" },
    { value: "7 days", shortKey: "leaderboard.interval.week.short", labelKey: "leaderboard.interval.week.label", subtitleKey: "leaderboard.interval.week.subtitle", sinceKey: "leaderboard.interval.week.since" },
    { value: "30 days", shortKey: "leaderboard.interval.month.short", labelKey: "leaderboard.interval.month.label", subtitleKey: "leaderboard.interval.month.subtitle", sinceKey: "leaderboard.interval.month.since" },
] as const satisfies ReadonlyArray<{ value: string; shortKey: LeaderboardMessageKey; labelKey: LeaderboardMessageKey; subtitleKey: LeaderboardMessageKey; sinceKey: LeaderboardMessageKey }>;
export type LeaderboardInterval = (typeof INTERVALS)[number]["value"];

export const SERVER_TINTS: Record<string, { fg: string; bg: string }> = {
    EN: { fg: "#4f74e0", bg: "color-mix(in srgb, #4f74e0 15%, transparent)" },
    JP: { fg: "#e04f74", bg: "color-mix(in srgb, #e04f74 15%, transparent)" },
    CN: { fg: "#b87a2c", bg: "color-mix(in srgb, #e0a04f 15%, transparent)" },
    KR: { fg: "var(--lagoon-deep)", bg: "color-mix(in srgb, #4fb8b2 18%, transparent)" },
    TW: { fg: "#7a5cb0", bg: "color-mix(in srgb, #9e7ad4 18%, transparent)" },
};
