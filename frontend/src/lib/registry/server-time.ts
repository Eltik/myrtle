/** UTC offsets (hours) of each game server's local time. Fixed - no DST. */
const SERVER_UTC_OFFSET_HOURS: Record<string, number> = { EN: -7, JP: 9, KR: 9, CN: 8, TW: 8 };

/** Game days roll over at 04:00 server time, not midnight. */
const DAILY_RESET_HOUR = 4;

/**
 * Distinct game days spanned by [startTs, endTs] (Unix seconds), inclusive -
 * the maximum possible `cumulative_signin` over that span. The registration
 * day itself is claimable (verified: an account registered minutes after the
 * reset has cumulative_signin equal to this count exactly), but players who
 * register shortly before a reset usually lose that first truncated day, so
 * one-below-max is common and legitimate. Naive elapsed/24h division is NOT
 * an upper bound and can show sign-ins exceeding the total.
 */
export function countGameDays(startTs: number, endTs: number, server: string): number {
    const offsetSec = ((SERVER_UTC_OFFSET_HOURS[server.toUpperCase()] ?? 0) - DAILY_RESET_HOUR) * 3600;
    const dayIndex = (ts: number) => Math.floor((ts + offsetSec) / 86400);
    return Math.max(1, dayIndex(endTs) - dayIndex(startTs) + 1);
}

/**
 * Calendar date (in server time, respecting the 04:00 reset) of the game day
 * containing `ts` (Unix seconds). E.g. 02:00 server time on the 8th is still
 * the game day of the 7th. `month` is 0-based, matching `Date`.
 */
export function gameDate(ts: number, server: string): { year: number; month: number; day: number } {
    const offsetSec = ((SERVER_UTC_OFFSET_HOURS[server.toUpperCase()] ?? 0) - DAILY_RESET_HOUR) * 3600;
    const d = new Date((ts + offsetSec) * 1000);
    return { year: d.getUTCFullYear(), month: d.getUTCMonth(), day: d.getUTCDate() };
}
