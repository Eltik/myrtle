import { useQuery } from "@tanstack/react-query";
import { useStore } from "@tanstack/react-store";
import { Store } from "@tanstack/store";
import { useEffect } from "react";
import { useAuth } from "#/hooks/use-auth";
import { readStoredValue, writeStoredValue } from "#/hooks/use-local-storage-state";
import { addPlaylistTracksFn, createPlaylistFn, deletePlaylistFn, getMusicLibraryFn, likeTrackFn, logPlayFn, musicLibraryQueryOptions, patchPlaylistFn, removePlaylistTrackFn, reportMusicStat, updatePlayFn } from "#/lib/api/music";
import type { MusicLibrary } from "#/types/generated/MusicLibrary";
import type { MusicPlay } from "#/types/generated/MusicPlay";
import type { MusicPlaylist } from "#/types/generated/MusicPlaylist";
import type { MusicPlayTotal } from "#/types/generated/MusicPlayTotal";

/**
 * THE ACCOUNT'S MUSIC: liked tracks, plays and playlists, one store for the
 * whole page. Signed in, it mirrors the backend (`/music/library`) and writes
 * through optimistically, reloading the server's truth when a write fails.
 * Signed out, it lives in `localStorage` under `LOCAL_KEY` with the same
 * shape, so every view works without an account. The two are not merged:
 * signing in shows the account's library, signing out the browser's.
 */

export const LOCAL_KEY = "myrtle.music.library";
/** Plays the browser keeps when signed out; the totals are computed from them. */
const LOCAL_HISTORY_MAX = 2000;
/** The same caps the backend enforces, for the local mode. */
const LOCAL_PLAYLIST_LIMIT = 100;
const LOCAL_PLAYLIST_TRACKS_MAX = 1000;

export type LibraryMode = "local" | "server";

export interface ILibraryState {
    mode: LibraryMode;
    viewerId: string | null;
    /** The current mode's data has been read at least once. */
    loaded: boolean;
    /** Track id to the ISO time it was liked. */
    likes: Record<string, string>;
    /** Newest first. */
    history: MusicPlay[];
    /** Most played first. */
    totals: MusicPlayTotal[];
    firstPlayedAt: string | null;
    /** Oldest first. */
    playlists: MusicPlaylist[];
    playlistLimit: number;
    playlistTracksMax: number;
}

interface ILocalDocument {
    likes: Record<string, string>;
    history: MusicPlay[];
    playlists: MusicPlaylist[];
}

const EMPTY: ILibraryState = {
    mode: "local",
    viewerId: null,
    loaded: false,
    likes: {},
    history: [],
    totals: [],
    firstPlayedAt: null,
    playlists: [],
    playlistLimit: LOCAL_PLAYLIST_LIMIT,
    playlistTracksMax: LOCAL_PLAYLIST_TRACKS_MAX,
};

export const libraryStore = new Store<ILibraryState>(EMPTY);

function readLocal(): ILocalDocument {
    return readStoredValue<ILocalDocument>(LOCAL_KEY, { likes: {}, history: [], playlists: [] }, (raw) => {
        const v = JSON.parse(raw) as Partial<ILocalDocument>;
        return { likes: v.likes && typeof v.likes === "object" ? v.likes : {}, history: Array.isArray(v.history) ? v.history : [], playlists: Array.isArray(v.playlists) ? v.playlists : [] };
    });
}

function writeLocal(): void {
    const s = libraryStore.state;
    if (s.mode !== "local") return;
    writeStoredValue<ILocalDocument>(LOCAL_KEY, { likes: s.likes, history: s.history.slice(0, LOCAL_HISTORY_MAX), playlists: s.playlists });
}

/** Per-track totals from a list of plays, most played first. */
export function totalsOf(history: readonly MusicPlay[]): MusicPlayTotal[] {
    const map = new Map<string, MusicPlayTotal>();
    for (const p of history) {
        const t = map.get(p.trackId) ?? { trackId: p.trackId, plays: 0, seconds: 0, lastPlayedAt: null };
        t.plays += 1;
        t.seconds += p.listenedSeconds;
        if (t.lastPlayedAt === null || p.playedAt > t.lastPlayedAt) t.lastPlayedAt = p.playedAt;
        map.set(p.trackId, t);
    }
    return [...map.values()].sort((a, b) => b.plays - a.plays || b.seconds - a.seconds || a.trackId.localeCompare(b.trackId));
}

function firstOf(history: readonly MusicPlay[]): string | null {
    let first: string | null = null;
    for (const p of history) if (first === null || p.playedAt < first) first = p.playedAt;
    return first;
}

function loadLocal(): void {
    const doc = readLocal();
    libraryStore.setState(() => ({
        ...EMPTY,
        loaded: true,
        likes: doc.likes,
        history: doc.history,
        totals: totalsOf(doc.history),
        firstPlayedAt: firstOf(doc.history),
        playlists: doc.playlists,
    }));
}

function applyServer(viewerId: string, lib: MusicLibrary): void {
    const likes: Record<string, string> = {};
    for (const l of lib.likes) likes[l.trackId] = l.likedAt;
    libraryStore.setState(() => ({
        mode: "server",
        viewerId,
        loaded: true,
        likes,
        history: lib.history,
        totals: lib.totals,
        firstPlayedAt: lib.firstPlayedAt,
        playlists: lib.playlists,
        playlistLimit: Number(lib.playlistLimit),
        playlistTracksMax: Number(lib.playlistTracksMax),
    }));
}

/** After a failed write, the server's truth replaces the optimistic state. */
async function resync(): Promise<void> {
    const s = libraryStore.state;
    if (s.mode !== "server" || !s.viewerId) return;
    try {
        const lib = await getMusicLibraryFn();
        if (lib) applyServer(s.viewerId, lib);
    } catch {
        // The next page load reads it again.
    }
}

function serverWrite(call: () => Promise<unknown>): void {
    call().catch(() => {
        void resync();
    });
}

let localPlayId = -1;
let localPlaylistSeq = 0;

function nowIso(): string {
    return new Date().toISOString();
}

export function isLiked(trackId: string): boolean {
    return trackId in libraryStore.state.likes;
}

/** Returns the new liked state. */
export function toggleLike(trackId: string): boolean {
    const liked = !isLiked(trackId);
    libraryStore.setState((s) => {
        const likes = { ...s.likes };
        if (liked) likes[trackId] = nowIso();
        else delete likes[trackId];
        return { ...s, likes };
    });
    if (libraryStore.state.mode === "server") serverWrite(() => likeTrackFn({ data: { trackId, liked } }));
    else writeLocal();
    return liked;
}

/**
 * Log a play once the listener has heard enough of it; the returned id names
 * it for `updatePlay`. Local plays get a negative id.
 *
 * The same moment counts the community listen. A server play already does
 * (`POST /music/history` writes the account's listen), so only a play kept
 * in this browser reports one, from the browser, counted by address. A play
 * goes one way or the other, never both, so no listen counts twice; a server
 * log that fails reports nothing, since it may have landed.
 */
export async function logPlay(trackId: string, listenedSeconds: number): Promise<number> {
    const s = libraryStore.state;
    if (s.mode === "server") {
        try {
            const play = await logPlayFn({ data: { trackId, listenedSeconds } });
            libraryStore.setState((st) => ({ ...st, history: [play, ...st.history], totals: bump(st.totals, play), firstPlayedAt: st.firstPlayedAt ?? play.playedAt }));
            return play.id;
        } catch {
            void resync();
            return Number.NaN;
        }
    }
    void reportMusicStat("listen", trackId);
    const play: MusicPlay = { id: localPlayId--, trackId, listenedSeconds, playedAt: nowIso() };
    libraryStore.setState((st) => {
        const history = [play, ...st.history].slice(0, LOCAL_HISTORY_MAX);
        return { ...st, history, totals: totalsOf(history), firstPlayedAt: firstOf(history) };
    });
    writeLocal();
    return play.id;
}

function bump(totals: MusicPlayTotal[], play: MusicPlay): MusicPlayTotal[] {
    const next = totals.map((t) => (t.trackId === play.trackId ? { ...t, plays: t.plays + 1, seconds: t.seconds + play.listenedSeconds, lastPlayedAt: play.playedAt } : t));
    if (!next.some((t) => t.trackId === play.trackId)) next.push({ trackId: play.trackId, plays: 1, seconds: play.listenedSeconds, lastPlayedAt: play.playedAt });
    return next.sort((a, b) => b.plays - a.plays || b.seconds - a.seconds);
}

/** Raise a play's listened time. The time never goes down. */
export function updatePlay(id: number, listenedSeconds: number): void {
    if (!Number.isFinite(id)) return;
    let delta = 0;
    libraryStore.setState((st) => {
        const history = st.history.map((p) => {
            if (p.id !== id || listenedSeconds <= p.listenedSeconds) return p;
            delta = listenedSeconds - p.listenedSeconds;
            return { ...p, listenedSeconds };
        });
        if (delta === 0) return st;
        const play = history.find((p) => p.id === id);
        const totals = play ? st.totals.map((t) => (t.trackId === play.trackId ? { ...t, seconds: t.seconds + delta } : t)) : st.totals;
        return { ...st, history, totals };
    });
    if (delta === 0) return;
    if (libraryStore.state.mode === "server") serverWrite(() => updatePlayFn({ data: { id, listenedSeconds } }));
    else writeLocal();
}

export async function createPlaylist(name: string, trackIds: string[] = []): Promise<MusicPlaylist | null> {
    const s = libraryStore.state;
    const clean = name.trim();
    if (!clean || s.playlists.length >= s.playlistLimit) return null;
    const ids = [...new Set(trackIds)].slice(0, s.playlistTracksMax);
    if (s.mode === "server") {
        try {
            const pl = await createPlaylistFn({ data: { name: clean, trackIds: ids } });
            libraryStore.setState((st) => ({ ...st, playlists: [...st.playlists, pl] }));
            return pl;
        } catch {
            void resync();
            return null;
        }
    }
    const now = nowIso();
    const pl: MusicPlaylist = { id: `local-${Date.now().toString(36)}-${(localPlaylistSeq++).toString(36)}`, name: clean, trackIds: ids, createdAt: now, updatedAt: now };
    libraryStore.setState((st) => ({ ...st, playlists: [...st.playlists, pl] }));
    writeLocal();
    return pl;
}

function patchLocal(id: string, patch: Partial<Pick<MusicPlaylist, "name" | "trackIds">>): MusicPlaylist | null {
    let out: MusicPlaylist | null = null;
    libraryStore.setState((st) => ({
        ...st,
        playlists: st.playlists.map((p) => {
            if (p.id !== id) return p;
            out = { ...p, ...patch, updatedAt: nowIso() };
            return out;
        }),
    }));
    return out;
}

export function renamePlaylist(id: string, name: string): void {
    const clean = name.trim();
    if (!clean) return;
    patchLocal(id, { name: clean });
    if (libraryStore.state.mode === "server") serverWrite(() => patchPlaylistFn({ data: { id, name: clean } }));
    else writeLocal();
}

/** The full new order; a track left out is removed. */
export function setPlaylistTracks(id: string, trackIds: string[]): void {
    const ids = [...new Set(trackIds)].slice(0, libraryStore.state.playlistTracksMax);
    patchLocal(id, { trackIds: ids });
    if (libraryStore.state.mode === "server") serverWrite(() => patchPlaylistFn({ data: { id, trackIds: ids } }));
    else writeLocal();
}

/** Append the tracks not already in the playlist. Returns how many were new. */
export function addToPlaylist(id: string, trackIds: string[]): number {
    const pl = libraryStore.state.playlists.find((p) => p.id === id);
    if (!pl) return 0;
    const fresh = [...new Set(trackIds)].filter((t) => !pl.trackIds.includes(t));
    const room = Math.max(0, libraryStore.state.playlistTracksMax - pl.trackIds.length);
    const add = fresh.slice(0, room);
    if (add.length === 0) return 0;
    patchLocal(id, { trackIds: [...pl.trackIds, ...add] });
    if (libraryStore.state.mode === "server") serverWrite(() => addPlaylistTracksFn({ data: { id, trackIds: add } }));
    else writeLocal();
    return add.length;
}

export function removeFromPlaylist(id: string, trackId: string): void {
    const pl = libraryStore.state.playlists.find((p) => p.id === id);
    if (!pl || !pl.trackIds.includes(trackId)) return;
    patchLocal(id, { trackIds: pl.trackIds.filter((t) => t !== trackId) });
    if (libraryStore.state.mode === "server") serverWrite(() => removePlaylistTrackFn({ data: { id, trackId } }));
    else writeLocal();
}

export function deletePlaylist(id: string): void {
    libraryStore.setState((st) => ({ ...st, playlists: st.playlists.filter((p) => p.id !== id) }));
    if (libraryStore.state.mode === "server") serverWrite(() => deletePlaylistFn({ data: id }));
    else writeLocal();
}

export function useLibrary<T>(selector: (s: ILibraryState) => T): T {
    return useStore(libraryStore, selector);
}

export function useIsLiked(trackId: string | null): boolean {
    return useStore(libraryStore, (s) => (trackId ? trackId in s.likes : false));
}

/**
 * Mount ONCE on the music page. Signed in, it loads the account's library and
 * keeps the store on it; signed out, it loads the browser's.
 */
export function useLibrarySync(): void {
    const { user } = useAuth();
    const viewerId = user?.id ?? null;
    const q = useQuery(musicLibraryQueryOptions(viewerId));
    const lib = q.data ?? null;
    useEffect(() => {
        if (viewerId === null) {
            loadLocal();
            return;
        }
        if (lib) applyServer(viewerId, lib);
        else if (q.isError || (q.isSuccess && lib === null)) {
            // The backend predates the route or the session lapsed: the
            // browser's library is better than nothing, and nothing is written
            // to the server in local mode.
            loadLocal();
        }
    }, [viewerId, lib, q.isError, q.isSuccess]);
}
