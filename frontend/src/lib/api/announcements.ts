import { queryOptions } from "@tanstack/react-query";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/core/announcements/mod.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
import type { Announcement } from "#/types/generated/Announcement";
import type { AnnouncementList } from "#/types/generated/AnnouncementList";

/**
 * The backend re-reads Yostar's boards every 5 minutes and answers from memory with an `ETag`,
 * so a minute of staleness costs nothing and a 5-minute refetch keeps the bell honest.
 * Read straight from the backend, by the browser as by SSR, like the Archives.
 */
const STALE = 60 * 1000;
const REFETCH = 5 * 60 * 1000;

/**
 * `server`'s board, or the backend's default board when `server` has none (a CN reader gets a 404
 * for `cn` and lands on the default instead of an error). `server` undefined asks for the default.
 */
async function getBoard(server: string | undefined): Promise<AnnouncementList> {
    const path = server ? `/announcements?server=${encodeURIComponent(server)}` : "/announcements";
    const res = await backendFetch(path);
    if (res.status === 404 && server) return getBoard(undefined);
    if (!res.ok) throw new Error(`Failed to load announcements: ${res.status}`);
    return (await res.json()) as AnnouncementList;
}

/** One notice, or `null` when the board no longer lists it (404). */
async function getAnnouncement(id: string, server: string): Promise<Announcement | null> {
    const res = await backendFetch(`/announcements/${encodeURIComponent(id)}?server=${encodeURIComponent(server)}`);
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`Failed to load announcement: ${res.status}`);
    return (await res.json()) as Announcement;
}

export function announcementsQueryOptions(server: string | undefined) {
    return queryOptions({
        queryKey: ["announcements", "list", server ?? null],
        queryFn: () => getBoard(server),
        staleTime: STALE,
        refetchInterval: REFETCH,
        // The default three retries held the page on a skeleton for 7 s before its error state.
        retry: 1,
    });
}

export function announcementQueryOptions(id: string, server: string) {
    return queryOptions({
        queryKey: ["announcements", "detail", server, id],
        queryFn: () => getAnnouncement(id, server),
        staleTime: STALE,
    });
}
