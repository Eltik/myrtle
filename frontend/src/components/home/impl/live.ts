import { gamedataPath } from "#/lib/api/gamedata";
import type { ActivityBasicInfo } from "#/types/generated/ActivityBasicInfo";
import type { GachaPoolClient } from "#/types/generated/GachaPoolClient";

const DAY_SECS = 86_400;

/** How many running events and banners the home page shows. */
export const LIVE_LIMIT = 3;

/**
 * One card in "Happening now", in a shape that no longer cares which source or
 * server it came from. `start` and `end` are unix seconds; `type` is the raw
 * gamedata tag (an activity's `type` or a pool's `gachaRuleType`); `artPaths`
 * are backend asset paths to try in order.
 */
export interface LiveItem {
    kind: "event" | "banner";
    key: string;
    name: string;
    type: string;
    start: number;
    end: number;
    featured6: readonly string[];
    featured5: readonly string[];
    artPaths: readonly string[];
}

function isRunning(startSec: number, endSec: number, nowSec: number): boolean {
    return startSec <= nowSec && endSec >= nowSec;
}

/**
 * An activity's art on `server`. A rerun (`act…sre`) often ships no art of its
 * own, so the original side story's (`act…side`) is the second try, the same
 * retry the release planner's backend context makes.
 */
function eventArtPaths(actId: string, server: string): string[] {
    const paths = [gamedataPath(server, `/event-image/${actId}`)];
    if (actId.endsWith("sre")) paths.push(gamedataPath(server, `/event-image/${actId.slice(0, -3)}side`));
    return paths;
}

export function eventItem(activity: ActivityBasicInfo, server: string): LiveItem {
    return { kind: "event", key: `event:${activity.id}`, name: activity.name, type: activity.type, start: activity.startTime, end: activity.endTime, featured6: [], featured5: [], artPaths: eventArtPaths(activity.id, server) };
}

export function bannerItem(pool: GachaPoolClient, server: string): LiveItem {
    return {
        kind: "banner",
        key: `banner:${pool.gachaPoolId}`,
        name: pool.gachaPoolName,
        type: pool.gachaRuleType,
        start: pool.openTime,
        end: pool.endTime,
        featured6: pool.featured6,
        featured5: pool.featured5,
        artPaths: [gamedataPath(server, `/banner-image/${pool.gachaPoolId}`)],
    };
}

/**
 * Every event and banner on `server` that has not ended by `nowSec`, ordered
 * by the one ending soonest. This is what the server function ships: small
 * (a handful of items), yet enough for {@link selectLive} to keep the list
 * right while the page stays open, dropping a window as it closes and taking
 * in one as it opens. An event counts only when it has stages (a sign-in or a
 * minigame is not something to plan around), and a returning-players-only
 * pool (`returning`, from the backend's `open_server_table` read) is never
 * "happening": the card means running now for everyone, and the game times
 * that pool per player behind a years-long placeholder window.
 */
export function liveCandidates(activities: readonly ActivityBasicInfo[], pools: readonly GachaPoolClient[], server: string, nowSec: number): LiveItem[] {
    const items: LiveItem[] = [];
    for (const activity of activities) {
        if (activity.hasStage && activity.endTime >= nowSec) items.push(eventItem(activity, server));
    }
    for (const pool of pools) {
        if (!pool.returning && pool.endTime >= nowSec) items.push(bannerItem(pool, server));
    }
    return items.sort((a, b) => a.end - b.end || a.key.localeCompare(b.key));
}

/** The candidates running at `nowSec`, soonest end first, capped at `limit`. */
export function selectLive(candidates: readonly LiveItem[], nowSec: number, limit = LIVE_LIMIT): LiveItem[] {
    return candidates.filter((item) => isRunning(item.start, item.end, nowSec)).slice(0, limit);
}

/** The events and banners running on `server` at `nowSec`, merged and ordered by the one ending soonest. */
export function liveItems(activities: readonly ActivityBasicInfo[], pools: readonly GachaPoolClient[], server: string, nowSec: number, limit = LIVE_LIMIT): LiveItem[] {
    return selectLive(liveCandidates(activities, pools, server, nowSec), nowSec, limit);
}

/** The featured operators a banner card names: its 6★ rate-ups, or its 5★ ones when it has none. */
export function namedFeatured(item: Pick<LiveItem, "featured6" | "featured5">): readonly string[] {
    return item.featured6.length > 0 ? item.featured6 : item.featured5;
}

/**
 * Whether a card shows the permit placeholder in its art slot: a banner with no
 * art of its own AND no rate-up to show as faces (DOUBLE_JP_41_0_9 on JP names
 * neither). A banner with art keeps it, rate-ups or not.
 */
export function showsBannerPlaceholder(item: Pick<LiveItem, "kind" | "featured6" | "featured5">, hasArt: boolean): boolean {
    return item.kind === "banner" && !hasArt && namedFeatured(item).length === 0;
}

/** Elapsed share of a window, 0..100. A zero-length window reads as done. */
export function elapsedPercent(startSec: number, endSec: number, nowSec: number): number {
    if (endSec <= startSec) return 100;
    return Math.min(100, Math.max(0, ((nowSec - startSec) / (endSec - startSec)) * 100));
}

/** Time left as whole days plus a zero-padded `HH:MM:SS`, or `null` once it has ended. */
export function countdownParts(msLeft: number): { days: number; clock: string } | null {
    if (msLeft <= 0) return null;
    const pad = (n: number): string => String(n).padStart(2, "0");
    const totalSec = Math.floor(msLeft / 1000);
    const days = Math.floor(totalSec / DAY_SECS);
    const h = Math.floor(totalSec / 3600) % 24;
    const m = Math.floor(totalSec / 60) % 60;
    const s = totalSec % 60;
    return { days, clock: `${pad(h)}:${pad(m)}:${pad(s)}` };
}
