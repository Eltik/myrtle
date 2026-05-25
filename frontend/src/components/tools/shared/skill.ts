/**
 * Skill-rank model for the operator-build calculators.
 *
 * A single 1-10 `skillRank` encodes both pre-mastery skill levels (1-7) and
 * masteries (8/9/10 = M1/M2/M3). Promotion gates the reachable range, mirroring
 * the backend's `MAX_SKILL_LEVELS = [4, 7, 10]`:
 *   - E0 → ranks 1-4
 *   - E1 → ranks 1-7
 *   - E2 → ranks 1-10 (masteries unlocked)
 */

export const MAX_SKILL_RANK = 10;
/** Ranks 8/9/10 represent masteries M1/M2/M3. */
export const FIRST_MASTERY_RANK = 8;

/** Highest selectable skill rank at a given elite (promotion) phase. */
export function maxSkillRankForElite(elite: number): number {
    if (elite >= 2) return 10;
    if (elite >= 1) return 7;
    return 4;
}

export function isMasteryRank(rank: number): boolean {
    return rank >= FIRST_MASTERY_RANK;
}

/** Full label, e.g. "Lv 5" or "M2". */
export function skillRankLabel(rank: number): string {
    return isMasteryRank(rank) ? `M${rank - 7}` : `Lv ${rank}`;
}

/** Compact label for summaries, e.g. "L5" or "M2". */
export function skillRankShort(rank: number): string {
    return isMasteryRank(rank) ? `M${rank - 7}` : `L${rank}`;
}

/** Clamp a rank into the range a given elite allows. */
export function clampRankToElite(rank: number, elite: number): number {
    return Math.min(Math.max(1, rank), maxSkillRankForElite(elite));
}

/** Why a rank is unavailable at the current elite, or null if it's allowed. */
export function rankLockReason(rank: number, elite: number): string | null {
    if (rank <= maxSkillRankForElite(elite)) return null;
    if (isMasteryRank(rank)) return "Masteries require E2";
    return rank > 4 ? "Skill levels above 4 require E1" : null;
}

/**
 * Map a unified rank to backend request fields: masteries (8-10) → `masteryLevel`
 * (M1/M2/M3), skill levels (1-7) → the explicit `skillLevel`.
 */
export function rankToRequest(rank: number): { skillLevel?: number; masteryLevel?: number } {
    return isMasteryRank(rank) ? { masteryLevel: rank - 7 } : { skillLevel: rank, masteryLevel: 0 };
}
