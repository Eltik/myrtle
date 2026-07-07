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
