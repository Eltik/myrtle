import { asset } from "#/components/operators/detail/impl/assets";
import { isGamedataServer } from "#/lib/api/gamedata";
import { sirenStreamUrl } from "#/lib/api/music";
import type { MusicIndex } from "#/types/generated/MusicIndex";
import type { MusicTrack } from "#/types/generated/MusicTrack";
import type { SirenAlbum } from "#/types/generated/SirenAlbum";
import type { SirenCatalog } from "#/types/generated/SirenCatalog";
import type { SirenSong } from "#/types/generated/SirenSong";
import { rememberSirenCoverHosts } from "./cover";
import { displayName } from "./tracks";

/**
 * ONE MODEL for both collections the music page holds: Monster Siren Records
 * releases (`siren`) and the game's own soundtrack as the client ships it
 * (`game`). A track id is prefixed with its kind, the same ids the backend
 * stores for likes, plays and playlists, so a playlist can mix both.
 *
 * Nothing here links a label track to an in-game track. The label's API
 * carries no game key (its fields are cid, name, intro, belong, coverUrl,
 * coverDeUrl, artists, lyricUrl, mvUrl) and the game's music table carries no
 * label id (`audio_data.Musics[]` has id, name, bank), so the two collections
 * share no key, and a title match would be a guess. They stay separate.
 */
export type TrackKind = "siren" | "game";

export interface ITrack {
    /** `siren:<cid>` or `game:<clip id>`. */
    id: string;
    kind: TrackKind;
    /** The label's cid or the clip id, without the prefix. */
    key: string;
    /** The title to show: a Siren title without its trailing instrumental marker (the backend's `SirenSong.title`; `instrumental` carries the fact). */
    title: string;
    /** Credited artists. Empty for an in-game track. */
    artists: string[];
    /** `siren:<album cid>`; `null` for an in-game track. */
    albumId: string | null;
    /** The album name, or the in-game track's origin (event, chapter, season). */
    albumName: string | null;
    /**
     * Square cover on the label's CDN; `null` for an in-game track. Every
     * `<img>` of it sets `referrerPolicy="no-referrer"`: web.hycdn.cn answers a
     * localhost or myrtle Referer with 403 "denied by Referer ACL" and an empty
     * one with 200 (curl, 2026-10-11).
     */
    coverUrl: string | null;
    instrumental: boolean;
    hasLyrics: boolean;
    /** Position in the album; 0 for an in-game track. */
    trackOrder: number;
    /** The in-game record: clips, kinds, where it plays. Only on `game`. */
    game?: MusicTrack;
}

export interface IAlbum {
    /** `siren:<cid>`. */
    id: string;
    cid: string;
    name: string;
    coverUrl: string;
    artists: string[];
    /** 0 = newest. */
    listOrder: number;
    trackCount: number;
    lyricsCount: number;
    instrumentalCount: number;
    /** ISO date (`YYYY-MM-DD`) or `null`. */
    addedOn: string | null;
    /**
     * False when `addedOn` is the catalogue's bulk import day, which says when
     * the label's site launched, not when the release came out.
     */
    showAddedOn: boolean;
}

export interface IArtist {
    name: string;
    trackCount: number;
}

export interface ICatalog {
    /** Siren tracks in album order (newest release first), then game tracks. */
    tracks: ITrack[];
    byId: Map<string, ITrack>;
    sirenTracks: ITrack[];
    gameTracks: ITrack[];
    /** Newest first. */
    albums: IAlbum[];
    albumById: Map<string, IAlbum>;
    /** Tracks per album id, in track order. */
    tracksByAlbum: Map<string, ITrack[]>;
    /**
     * The artist credited on the most label tracks: the label's house name
     * (`塞壬唱片-MSR`, 896 of 911 tracks on 2026-10-11). Derived, never
     * spelled out, so a renamed house credit still works.
     */
    houseArtist: string | null;
    /** Guest artists (every credited name except the house one), alphabetical. */
    artists: IArtist[];
    /** The label catalogue has loaded. */
    sirenReady: boolean;
    /** The in-game index has loaded. */
    gameReady: boolean;
    /** The label's catalogue as synced; `null` until it loads. */
    siren: SirenCatalog | null;
    game: MusicIndex | null;
}

export const SIREN_PREFIX = "siren:";
export const GAME_PREFIX = "game:";

export function sirenTrackId(cid: string): string {
    return `${SIREN_PREFIX}${cid}`;
}

export function gameTrackId(clipId: string): string {
    return `${GAME_PREFIX}${clipId}`;
}

export function sirenAlbumId(cid: string): string {
    return `${SIREN_PREFIX}${cid}`;
}

/** The kind and the bare key of a prefixed id; `null` for an id of neither kind. */
export function parseTrackId(id: string): { kind: TrackKind; key: string } | null {
    if (id.startsWith(SIREN_PREFIX) && id.length > SIREN_PREFIX.length) return { kind: "siren", key: id.slice(SIREN_PREFIX.length) };
    if (id.startsWith(GAME_PREFIX) && id.length > GAME_PREFIX.length) return { kind: "game", key: id.slice(GAME_PREFIX.length) };
    return null;
}

function sirenTrack(song: SirenSong, album: IAlbum | undefined): ITrack {
    return {
        id: sirenTrackId(song.cid),
        kind: "siren",
        key: song.cid,
        title: song.title,
        artists: song.artists,
        albumId: sirenAlbumId(song.albumCid),
        albumName: album?.name ?? null,
        coverUrl: album?.coverUrl ?? null,
        instrumental: song.instrumental,
        hasLyrics: song.hasLyrics,
        trackOrder: song.trackOrder,
    };
}

function gameTrack(track: MusicTrack): ITrack {
    return {
        id: gameTrackId(track.id),
        kind: "game",
        key: track.id,
        title: displayName(track),
        artists: [],
        albumId: null,
        albumName: track.origin ?? null,
        coverUrl: null,
        instrumental: false,
        hasLyrics: false,
        trackOrder: 0,
        game: track,
    };
}

function album(a: SirenAlbum, bulkImportDay: string | null): IAlbum {
    return {
        id: sirenAlbumId(a.cid),
        cid: a.cid,
        name: a.name,
        coverUrl: a.coverUrl,
        artists: a.artists,
        listOrder: a.listOrder,
        trackCount: a.trackCount,
        lyricsCount: a.lyricsCount,
        instrumentalCount: a.instrumentalCount,
        addedOn: a.addedOn,
        showAddedOn: a.addedOn !== null && a.addedOn !== bulkImportDay,
    };
}

/** The artist credited on the most tracks; `null` when nothing is credited. */
export function houseArtistOf(songs: readonly { artists: string[] }[]): string | null {
    const counts = new Map<string, number>();
    for (const s of songs) for (const a of s.artists) counts.set(a, (counts.get(a) ?? 0) + 1);
    let best: string | null = null;
    let bestN = 0;
    for (const [name, n] of counts) {
        if (n > bestN) {
            best = name;
            bestN = n;
        }
    }
    return best;
}

export function buildCatalog(siren: SirenCatalog | null, game: MusicIndex | null, collator?: Intl.Collator): ICatalog {
    const albums: IAlbum[] = [];
    const albumById = new Map<string, IAlbum>();
    const tracksByAlbum = new Map<string, ITrack[]>();
    const sirenTracks: ITrack[] = [];
    if (siren) {
        const sorted = [...siren.albums].sort((a, b) => a.listOrder - b.listOrder);
        rememberSirenCoverHosts(sorted.map((a) => a.coverUrl));
        for (const a of sorted) {
            const al = album(a, siren.bulkImportDay);
            albums.push(al);
            albumById.set(al.id, al);
            tracksByAlbum.set(al.id, []);
        }
        const byAlbum = new Map<string, ITrack[]>();
        for (const song of siren.songs) {
            const al = albumById.get(sirenAlbumId(song.albumCid));
            const tr = sirenTrack(song, al);
            const list = byAlbum.get(tr.albumId ?? "") ?? [];
            list.push(tr);
            byAlbum.set(tr.albumId ?? "", list);
        }
        for (const al of albums) {
            const list = (byAlbum.get(al.id) ?? []).sort((a, b) => a.trackOrder - b.trackOrder);
            tracksByAlbum.set(al.id, list);
            sirenTracks.push(...list);
            byAlbum.delete(al.id);
        }
        // A track whose album is not listed still plays; it goes last.
        for (const rest of byAlbum.values()) sirenTracks.push(...rest.sort((a, b) => a.trackOrder - b.trackOrder));
    }
    const gameTracks: ITrack[] = game ? game.tracks.map(gameTrack) : [];
    const tracks = [...sirenTracks, ...gameTracks];
    const byId = new Map<string, ITrack>();
    for (const t of tracks) byId.set(t.id, t);

    const houseArtist = houseArtistOf(sirenTracks);
    const artistCounts = new Map<string, number>();
    for (const t of sirenTracks) for (const a of t.artists) if (a !== houseArtist) artistCounts.set(a, (artistCounts.get(a) ?? 0) + 1);
    const cmp = collator ?? new Intl.Collator(undefined, { sensitivity: "base", numeric: true });
    const artists: IArtist[] = [...artistCounts].map(([name, trackCount]) => ({ name, trackCount })).sort((a, b) => cmp.compare(a.name, b.name));

    return { tracks, byId, sirenTracks, gameTracks, albums, albumById, tracksByAlbum, houseArtist, artists, sirenReady: siren !== null, gameReady: game !== null, siren, game };
}

/** The artists a track shows: the guests, else the house name. */
export function displayArtists(track: ITrack, houseArtist: string | null): string[] {
    const guests = track.artists.filter((a) => a !== houseArtist);
    return guests.length > 0 ? guests : track.artists;
}

/** What the engine loads for a track. */
export type TrackSource = { kind: "stream"; src: string } | { kind: "loop"; intro: string | undefined; loop: string } | { kind: "none" };

export function trackSource(track: ITrack): TrackSource {
    if (track.kind === "siren") return { kind: "stream", src: sirenStreamUrl(track.key) };
    const g = track.game;
    if (!g?.loopUrl) return { kind: "none" };
    const server = g.server && isGamedataServer(g.server) ? g.server : undefined;
    return { kind: "loop", intro: g.introUrl ? asset(g.introUrl, server) : undefined, loop: asset(g.loopUrl, server) };
}

export function isPlayable(track: ITrack): boolean {
    return track.kind === "siren" || Boolean(track.game?.loopUrl);
}

/**
 * RADIO: what plays once a queue runs out. The rest of `track`'s album in
 * order, then the albums released around it, alternating newer and older by
 * list order, until `limit` tracks. In-game tracks have no album, so radio
 * from one is the game tracks that share its origin, then the rest of the
 * game collection in index order.
 */
export function radioFor(catalog: ICatalog, track: ITrack, exclude: ReadonlySet<string>, limit = 60): ITrack[] {
    const out: ITrack[] = [];
    const push = (t: ITrack) => {
        if (out.length < limit && !exclude.has(t.id) && t.id !== track.id && isPlayable(t) && !out.includes(t)) out.push(t);
    };
    if (track.kind === "siren" && track.albumId) {
        const own = catalog.tracksByAlbum.get(track.albumId) ?? [];
        const at = own.findIndex((t) => t.id === track.id);
        for (const t of own.slice(at + 1)) push(t);
        const al = catalog.albumById.get(track.albumId);
        if (al) {
            const idx = catalog.albums.indexOf(al);
            for (let step = 1; out.length < limit && (idx - step >= 0 || idx + step < catalog.albums.length); step += 1) {
                const newer = catalog.albums[idx - step];
                const older = catalog.albums[idx + step];
                for (const a of [newer, older]) if (a) for (const t of catalog.tracksByAlbum.get(a.id) ?? []) push(t);
            }
        }
        return out;
    }
    const origin = track.albumName;
    if (origin) for (const t of catalog.gameTracks) if (t.albumName === origin) push(t);
    for (const t of catalog.gameTracks) push(t);
    return out;
}
