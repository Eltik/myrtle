import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { env } from "#/env";
import { backendFetch } from "#/lib/fetch";
import type { MusicIndex } from "#/types/generated/MusicIndex";
import type { MusicLibrary } from "#/types/generated/MusicLibrary";
import type { MusicListenerPage } from "#/types/generated/MusicListenerPage";
import type { MusicListenerSort } from "#/types/generated/MusicListenerSort";
import type { MusicPlay } from "#/types/generated/MusicPlay";
import type { MusicPlaylist } from "#/types/generated/MusicPlaylist";
import type { MusicStatRecorded } from "#/types/generated/MusicStatRecorded";
import type { MusicStatsCatalog } from "#/types/generated/MusicStatsCatalog";
import type { MusicStatWindow } from "#/types/generated/MusicStatWindow";
import type { MusicTopTracks } from "#/types/generated/MusicTopTracks";
import type { SirenAlbumDetail } from "#/types/generated/SirenAlbumDetail";
import type { SirenCatalog } from "#/types/generated/SirenCatalog";
import { parseError } from "./_shared";
import { optionalSiteToken, requireSiteToken } from "./_shared.server";
import { DEFAULT_GAMEDATA_SERVER, gamedataKey, gamedataPath, resolveGamedataServer } from "./gamedata";

/**
 * Every piece of music the game names, with its clips and where it plays.
 * `null` when the running backend predates the route (404), so the page can
 * say so instead of failing.
 *
 * Read straight from the backend by the browser (`backendFetch`), not through a
 * server function: the SSR server used to fetch the 1,161,730-byte JSON
 * uncompressed and re-encode it as 2,450,827 bytes of seroval, sent to the
 * browser uncompressed. The backend answers 109,638 bytes of brotli with an
 * `ETag`, and a revalidation that matches is a 304.
 */
async function fetchMusicIndex(server: string): Promise<MusicIndex | null> {
    const res = await backendFetch(gamedataPath(server, "/gallery/music"));
    if (res.status === 404) return null;
    if (!res.ok) throw new Error(`Failed to load the music index: ${res.status}`);
    return (await res.json()) as MusicIndex;
}

export function musicIndexQueryOptions(server: string = DEFAULT_GAMEDATA_SERVER) {
    return queryOptions({
        queryKey: ["gallery", "music", ...gamedataKey(server)],
        queryFn: () => fetchMusicIndex(resolveGamedataServer(server)),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}

/** The response body of a successful request; the backend's error otherwise. */
async function readJSON<T>(res: Response): Promise<T> {
    if (!res.ok) throw await parseError(res);
    return (await res.json()) as T;
}

/** `null` for a 404, a backend that predates the route; `readJSON` otherwise. */
async function readJSONOrNull<T>(res: Response): Promise<T | null> {
    if (res.status === 404) return null;
    return readJSON<T>(res);
}

/**
 * The label's whole catalogue. `null` when the running backend predates the
 * route (404). A 503 means the backend's first sync is still running: the
 * query retries on its own schedule, so the page can say "syncing" and wait.
 * Read by the browser straight from the backend, as the music index is: the
 * server-function relay sent 410,019 bytes of seroval for 27,842 of brotli.
 */
async function fetchSirenCatalog(): Promise<SirenCatalog | null> {
    const res = await backendFetch("/music/siren");
    return readJSONOrNull<SirenCatalog>(res);
}

export function sirenCatalogQueryOptions() {
    return queryOptions({
        queryKey: ["music", "siren", "catalog"],
        queryFn: () => fetchSirenCatalog(),
        staleTime: 15 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
        retry: (count, err) => count < 6 && (err as { status?: number }).status === 503,
        retryDelay: 10_000,
    });
}

async function fetchSirenAlbum(cid: string): Promise<SirenAlbumDetail | null> {
    const res = await backendFetch(`/music/siren/albums/${encodeURIComponent(cid)}`);
    return readJSONOrNull<SirenAlbumDetail>(res);
}

export function sirenAlbumQueryOptions(cid: string) {
    return queryOptions({
        queryKey: ["music", "siren", "album", cid],
        queryFn: () => fetchSirenAlbum(cid),
        staleTime: 15 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

/**
 * Client-side URLs on the backend's public origin. The stream answers with a
 * redirect to the label's CDN, so an `<audio>` element loads it directly; the
 * lyrics are a plain text LRC file.
 */
function sirenSongUrl(cid: string, leaf: "stream" | "lyrics"): string {
    return `${env.VITE_BACKEND_URL ?? ""}/api/music/siren/songs/${encodeURIComponent(cid)}/${leaf}`;
}

export function sirenStreamUrl(cid: string): string {
    return sirenSongUrl(cid, "stream");
}

export function sirenLyricsUrl(cid: string): string {
    return sirenSongUrl(cid, "lyrics");
}

/** `null` when signed out or when the running backend predates the route. */
export const getMusicLibraryFn = createServerFn({ method: "GET" }).handler(async (): Promise<MusicLibrary | null> => {
    const token = getCookie("site_token");
    if (!token) return null;
    const res = await backendFetch("/music/library", { bearerToken: token });
    if (res.status === 401 || res.status === 404) return null;
    return readJSON<MusicLibrary>(res);
});

/** `viewerId` keys the cache so one account's library never shows to another. */
export function musicLibraryQueryOptions(viewerId: string | null) {
    return queryOptions({
        queryKey: ["music", "library", viewerId ?? "anon"],
        queryFn: () => (viewerId ? getMusicLibraryFn() : Promise.resolve(null)),
        enabled: viewerId !== null,
        staleTime: 60 * 1000,
        gcTime: 10 * 60 * 1000,
    });
}

export const likeTrackFn = createServerFn({ method: "POST" })
    .inputValidator((data: { trackId: string; liked: boolean }) => data)
    .handler(async ({ data: { trackId, liked } }): Promise<null> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/music/likes/${encodeURIComponent(trackId)}`, { method: liked ? "PUT" : "DELETE", bearerToken: token });
        // Unliking a track that was never liked is a 404 here and a no-op for the caller.
        if (!res.ok && !(res.status === 404 && !liked)) throw await parseError(res);
        return null;
    });

export const logPlayFn = createServerFn({ method: "POST" })
    .inputValidator((data: { trackId: string; listenedSeconds: number }) => data)
    .handler(async ({ data }): Promise<MusicPlay> => {
        const token = requireSiteToken();
        const res = await backendFetch("/music/history", { method: "POST", bearerToken: token, body: JSON.stringify({ trackId: data.trackId, listenedSeconds: data.listenedSeconds }) });
        return readJSON<MusicPlay>(res);
    });

export const updatePlayFn = createServerFn({ method: "POST" })
    .inputValidator((data: { id: number; listenedSeconds: number }) => data)
    .handler(async ({ data }): Promise<null> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/music/history/${data.id}`, { method: "PUT", bearerToken: token, body: JSON.stringify({ listenedSeconds: data.listenedSeconds }) });
        if (!res.ok) throw await parseError(res);
        return null;
    });

export const createPlaylistFn = createServerFn({ method: "POST" })
    .inputValidator((data: { name: string; trackIds: string[] }) => data)
    .handler(async ({ data }): Promise<MusicPlaylist> => {
        const token = requireSiteToken();
        const res = await backendFetch("/music/playlists", { method: "POST", bearerToken: token, body: JSON.stringify(data) });
        return readJSON<MusicPlaylist>(res);
    });

export const patchPlaylistFn = createServerFn({ method: "POST" })
    .inputValidator((data: { id: string; name?: string; trackIds?: string[] }) => data)
    .handler(async ({ data: { id, ...patch } }): Promise<MusicPlaylist> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/music/playlists/${encodeURIComponent(id)}`, { method: "PATCH", bearerToken: token, body: JSON.stringify(patch) });
        return readJSON<MusicPlaylist>(res);
    });

export const deletePlaylistFn = createServerFn({ method: "POST" })
    .inputValidator((id: string) => id)
    .handler(async ({ data: id }): Promise<null> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/music/playlists/${encodeURIComponent(id)}`, { method: "DELETE", bearerToken: token });
        if (!res.ok && res.status !== 404) throw await parseError(res);
        return null;
    });

export const addPlaylistTracksFn = createServerFn({ method: "POST" })
    .inputValidator((data: { id: string; trackIds: string[] }) => data)
    .handler(async ({ data: { id, trackIds } }): Promise<MusicPlaylist> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/music/playlists/${encodeURIComponent(id)}/tracks`, { method: "POST", bearerToken: token, body: JSON.stringify({ trackIds }) });
        return readJSON<MusicPlaylist>(res);
    });

export const removePlaylistTrackFn = createServerFn({ method: "POST" })
    .inputValidator((data: { id: string; trackId: string }) => data)
    .handler(async ({ data: { id, trackId } }): Promise<MusicPlaylist> => {
        const token = requireSiteToken();
        const res = await backendFetch(`/music/playlists/${encodeURIComponent(id)}/tracks/${encodeURIComponent(trackId)}`, { method: "DELETE", bearerToken: token });
        return readJSON<MusicPlaylist>(res);
    });

// ---------------------------------------------------------------------------
// Community stats
// ---------------------------------------------------------------------------

/**
 * Every track's all-time listens and downloads, read by the browser straight
 * from the backend (one small JSON for the whole catalogue, cached 60 s
 * there). `null` when the running backend predates the route.
 */
async function fetchMusicStats(): Promise<MusicStatsCatalog | null> {
    const res = await backendFetch("/music/stats");
    return readJSONOrNull<MusicStatsCatalog>(res);
}

export function musicStatsQueryOptions() {
    return queryOptions({
        queryKey: ["music", "stats", "catalog"],
        queryFn: () => fetchMusicStats(),
        staleTime: 60 * 1000,
        gcTime: 30 * 60 * 1000,
        retry: 1,
    });
}

async function fetchTopTracks(window: MusicStatWindow, limit: number): Promise<MusicTopTracks | null> {
    const res = await backendFetch(`/music/stats/top-tracks?window=${window}&limit=${limit}`);
    return readJSONOrNull<MusicTopTracks>(res);
}

export function musicTopTracksQueryOptions(window: MusicStatWindow, limit = 50) {
    return queryOptions({
        queryKey: ["music", "stats", "top-tracks", window, limit],
        queryFn: () => fetchTopTracks(window, limit),
        staleTime: 60 * 1000,
        retry: 1,
    });
}

/**
 * Signed-in listeners ranked over a window, site-wide or for one track.
 * `null` when the running backend predates the route (or, for a track, does
 * not know it).
 */
async function fetchListeners(trackId: string | null, window: MusicStatWindow, sort: MusicListenerSort, limit: number, offset: number): Promise<MusicListenerPage | null> {
    const base = trackId ? `/music/stats/tracks/${encodeURIComponent(trackId)}/listeners` : "/music/stats/top-listeners";
    const res = await backendFetch(`${base}?window=${window}&sort=${sort}&limit=${limit}&offset=${offset}`);
    return readJSONOrNull<MusicListenerPage>(res);
}

export function musicListenersQueryOptions(trackId: string | null, window: MusicStatWindow, sort: MusicListenerSort, limit = 20, offset = 0) {
    return queryOptions({
        queryKey: ["music", "stats", "listeners", trackId ?? "all", window, sort, limit, offset],
        queryFn: () => fetchListeners(trackId, window, sort, limit, offset),
        staleTime: 60 * 1000,
        retry: 1,
    });
}

export type MusicStatKind = "listen" | "download";

/**
 * Count a listen or a download as THIS BROWSER, straight to the backend and
 * never through a server function: the backend dedupes an anonymous caller by
 * its address, and through the SSR server every caller would share the
 * server's. No cookie travels (the session is an httpOnly cookie on the
 * site's origin), so the backend counts it against the address, not an
 * account. `keepalive` lets a report made just before the tab closes land.
 * Never throws; `null` when nothing answered.
 */
export async function reportMusicStat(kind: MusicStatKind, trackId: string): Promise<MusicStatRecorded | null> {
    const init: RequestInit = { method: "POST", body: JSON.stringify({ trackId }) };
    try {
        let res: Response;
        try {
            res = await backendFetch(`/music/stats/${kind}`, { ...init, keepalive: true });
        } catch {
            // Older engines refuse a keepalive request that needs a CORS preflight; the server
            // dedupes, so a second try can only count once.
            res = await backendFetch(`/music/stats/${kind}`, init);
        }
        if (!res.ok) return null;
        return (await res.json()) as MusicStatRecorded;
    } catch {
        return null;
    }
}

/**
 * Count a download against the signed-in account (the session cookie becomes
 * the bearer here, so the server's address never identifies anyone). `null`
 * when there is no session, so the caller can report it from the browser.
 */
export const recordMusicDownloadFn = createServerFn({ method: "POST" })
    .inputValidator((trackId: string) => trackId)
    .handler(async ({ data: trackId }): Promise<MusicStatRecorded | null> => {
        const token = optionalSiteToken();
        if (!token) return null;
        const res = await backendFetch("/music/stats/download", { method: "POST", bearerToken: token, body: JSON.stringify({ trackId }) });
        // A lapsed session is a 401 here; the caller then counts it as the browser.
        if (res.status === 401) return null;
        return readJSON<MusicStatRecorded>(res);
    });
