import { Store } from "@tanstack/store";
import { searchAndRank } from "#/lib/search/fuzzy";
import type { AnnouncementSummary } from "#/types/generated/AnnouncementSummary";

/**
 * Which in-game notices this visitor has opened, per server, as `{ [server]: id[] }`.
 * Per-viewer state, so localStorage: nothing about it needs to follow an account.
 */
export const SEEN_STORAGE_KEY = "myrtle-announcements-seen";

/** Ids kept per server. A board lists 5 to 22 notices, so this never drops a live one. */
const SEEN_CAP = 200;

/**
 * `pending` until the one read on mount, `unavailable` when storage throws. With no storage the
 * unread count is hidden: a count that can never go down is worse than none.
 */
type SeenStatus = "pending" | "ready" | "unavailable";

interface ISeenState {
    status: SeenStatus;
    seen: Record<string, readonly string[]>;
}

export const seenStore = new Store<ISeenState>({ status: "pending", seen: {} });

function parseSeen(raw: string | null): Record<string, readonly string[]> {
    if (!raw) return {};
    try {
        const parsed: unknown = JSON.parse(raw);
        if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return {};
        const out: Record<string, readonly string[]> = {};
        for (const [server, ids] of Object.entries(parsed)) {
            if (Array.isArray(ids)) out[server] = ids.filter((id): id is string => typeof id === "string");
        }
        return out;
    } catch {
        return {};
    }
}

/** Reads storage once. Safe to call from every consumer's effect. */
export function initSeen(): void {
    if (seenStore.state.status !== "pending" || typeof window === "undefined") return;
    try {
        seenStore.setState(() => ({ status: "ready", seen: parseSeen(window.localStorage.getItem(SEEN_STORAGE_KEY)) }));
    } catch {
        seenStore.setState(() => ({ status: "unavailable", seen: {} }));
    }
}

/** Adds `ids` to `server`'s opened notices, newest last, capped. */
export function mergeSeen(seen: Record<string, readonly string[]>, server: string, ids: readonly string[]): Record<string, readonly string[]> {
    const prior = seen[server] ?? [];
    const fresh = ids.filter((id) => !prior.includes(id));
    if (fresh.length === 0) return seen;
    return { ...seen, [server]: [...prior, ...fresh].slice(-SEEN_CAP) };
}

export function markSeen(server: string, ids: readonly string[]): void {
    const state = seenStore.state;
    if (state.status !== "ready") return;
    const next = mergeSeen(state.seen, server, ids);
    if (next === state.seen) return;
    seenStore.setState(() => ({ status: "ready", seen: next }));
    try {
        window.localStorage.setItem(SEEN_STORAGE_KEY, JSON.stringify(next));
    } catch {
        // Quota or a blocked store: the marks hold for this page load only.
    }
}

/** The notices on `list` this visitor has not opened. Empty until storage has been read. */
export function unreadOf(list: readonly AnnouncementSummary[], server: string, state: ISeenState): AnnouncementSummary[] {
    if (state.status !== "ready") return [];
    const seen = state.seen[server] ?? [];
    return list.filter((a) => !seen.includes(a.id));
}

export interface IGroupCount {
    group: string;
    count: number;
}

/** The categories the board actually uses, in the order they first appear, with counts. */
export function groupsOf(list: readonly AnnouncementSummary[]): IGroupCount[] {
    const counts = new Map<string, number>();
    for (const a of list) {
        if (a.group) counts.set(a.group, (counts.get(a.group) ?? 0) + 1);
    }
    return [...counts.entries()].map(([group, count]) => ({ group, count }));
}

/** `group` (null = every group), then a fuzzy match on the title; board order when there is no query. */
export function filterAnnouncements(list: readonly AnnouncementSummary[], group: string | null, query: string): AnnouncementSummary[] {
    const inGroup = group ? list.filter((a) => a.group === group) : [...list];
    if (!query.trim()) return inGroup;
    return searchAndRank(query, inGroup, (a) => ({ name: a.title, extra: `${a.group} ${a.published_on ?? ""}` })).map((s) => s.item);
}

/** The first server a reader is known to play or read on; the API falls back from one with no board. */
export function preferredServer(...candidates: readonly (string | null | undefined)[]): string | undefined {
    return candidates.find((c): c is string => typeof c === "string" && c.length > 0);
}

/** A `YYYY-MM-DD` board date as a local calendar date, so formatting never shifts it a day. */
export function boardDate(publishedOn: string | null): Date | null {
    if (!publishedOn) return null;
    const [y, m, d] = publishedOn.split("-").map(Number);
    if (!y || !m || !d) return null;
    return new Date(y, m - 1, d);
}
