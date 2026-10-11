import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie } from "@tanstack/react-start/server";
import { env } from "#/env";
import { backendFetch } from "#/lib/fetch";
import type { MusicIndex } from "#/types/generated/MusicIndex";
import type { MusicLibrary } from "#/types/generated/MusicLibrary";
import type { MusicPlay } from "#/types/generated/MusicPlay";
import type { MusicPlaylist } from "#/types/generated/MusicPlaylist";
import type { SirenAlbumDetail } from "#/types/generated/SirenAlbumDetail";
import type { SirenCatalog } from "#/types/generated/SirenCatalog";
import { parseError } from "./_shared";
import { requireSiteToken } from "./_shared.server";
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

/**
 * The label's whole catalogue. `null` when the running backend predates the
 * route (404). A 503 means the backend's first sync is still running: the
 * query retries on its own schedule, so the page can say "syncing" and wait.
 * Read by the browser straight from the backend, as the music index is: the
 * server-function relay sent 410,019 bytes of seroval for 27,842 of brotli.
 */
async function fetchSirenCatalog(): Promise<SirenCatalog | null> {
    const res = await backendFetch("/music/siren");
    if (res.status === 404) return null;
    return readJSON<SirenCatalog>(res);
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
    if (res.status === 404) return null;
    return readJSON<SirenAlbumDetail>(res);
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
