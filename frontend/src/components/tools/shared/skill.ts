/**
 * Skill-rank model for the operator-build calculators.
 *
 * A single 1-10 `skillRank` encodes both pre-mastery skill levels (1-7) and
 * masteries (8/9/10 = M1/M2/M3). Promotion gates the reachable range, mirroring
 * the backend's `MAX_SKILL_LEVELS = [4, 7, 10]`:
 *   - E0 -> ranks 1-4
 *   - E1 -> ranks 1-7
 *   - E2 -> ranks 1-10 (masteries unlocked)
 */

import { DEFAULT_LOCALE, formatMessage, sourceMessage } from "#/lib/i18n";
import { fullMessageKey, type TypedT } from "#/lib/i18n/messages";
import type { messages as skillMessages } from "./skill.messages";

/** The `t` these label helpers need, narrowed to the keys they can render. */
export type SkillT = TypedT<typeof skillMessages>;

/**
 * Default `t` for a caller outside an `I18nProvider`. It resolves against the
 * bundled source catalog, so the English is the same one the components
 * render and this file carries no second copy of the text.
 */
const sourceT: SkillT = (key, values) => formatMessage(sourceMessage(fullMessageKey("tools", key)) ?? key, DEFAULT_LOCALE, values);

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
export function skillRankLabel(rank: number, t: SkillT = sourceT): string {
    return isMasteryRank(rank) ? t("calc.skillRank.mastery", { mastery: rank - 7 }) : t("calc.skillRank.level", { rank });
}

/** Compact label for summaries, e.g. "L5" or "M2". */
export function skillRankShort(rank: number, t: SkillT = sourceT): string {
    return isMasteryRank(rank) ? t("calc.skillRank.masteryShort", { mastery: rank - 7 }) : t("calc.skillRank.levelShort", { rank });
}

/** Clamp a rank into the range a given elite allows. */
export function clampRankToElite(rank: number, elite: number): number {
    return Math.min(Math.max(1, rank), maxSkillRankForElite(elite));
}

/** Why a rank is unavailable at the current elite, or null if it's allowed. */
export function rankLockReason(rank: number, elite: number, t: SkillT = sourceT): string | null {
    if (rank <= maxSkillRankForElite(elite)) return null;
    if (isMasteryRank(rank)) return t("calc.skillRank.lock.mastery");
    return rank > 4 ? t("calc.skillRank.lock.level") : null;
}

/**
 * Map a unified rank to backend request fields: masteries (8-10) -> `masteryLevel`
 * (M1/M2/M3), skill levels (1-7) -> the explicit `skillLevel`.
 */
export function rankToRequest(rank: number): { skillLevel?: number; masteryLevel?: number } {
    return isMasteryRank(rank) ? { masteryLevel: rank - 7 } : { skillLevel: rank, masteryLevel: 0 };
}
