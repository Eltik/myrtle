/**
 * A Monster Siren album's social card, as data. Built from the one
 * `/music/siren/albums/{cid}` record by both the music route's `head()` and
 * the `music` OG handler, through {@link buildMusicOgData}.
 *
 * No track duration exists anywhere in the API (the album, the song record and
 * the upstream `song_detail` carry none; only the audio file itself does), so
 * the card counts tracks and draws no total time.
 *
 * Only albums get a card. Playlists are private to their owner (every
 * playlist route needs the owner's token, and a signed-out library lives in
 * localStorage), so an anonymous crawler cannot read one; and an artist is a
 * bare name with no record or picture of its own. Both keep the music preset.
 */

import type { SirenAlbumDetail } from "#/types/generated/SirenAlbumDetail";

/** Track titles the card lists. */
export const MUSIC_OG_TRACKS = 4;

export interface IMusicOgData {
    cid: string;
    name: string;
    artists: string[];
    trackCount: number;
    /** The first {@link MUSIC_OG_TRACKS} titles, in track order. */
    tracks: string[];
    /** The cover's source URL, hashed: a re-uploaded cover moves to a new URL. */
    coverSource: string;
    /** The cover inlined as a data URI; filled by the OG handler only. */
    coverURL?: string;
    /** The cover shrunk to 8 px, inlined; drawn card-sized it smears into the cover's colour field (see `MusicTemplate`). Filled by the OG handler only. */
    backdropURL?: string;
}

/**
 * The cover at `px` wide from Monster Siren's CDN, which resizes on
 * `x-oss-process` (an 8 px JPEG is 3,608 bytes); `null` for any other host.
 */
export function sirenCoverAt(url: string, px: number): string | null {
    try {
        const u = new URL(url);
        if (!u.hostname.endsWith(".hycdn.cn")) return null;
        u.searchParams.set("x-oss-process", `image/resize,w_${px}`);
        return u.toString();
    } catch {
        return null;
    }
}

/** An album cid is digits on Monster Siren (`8913`, `0242`); anything else is not an album. */
export function isMusicAlbumId(id: string): boolean {
    return /^\d{1,12}$/.test(id);
}

export function buildMusicOgData(album: SirenAlbumDetail): IMusicOgData {
    const songs = [...album.songs].sort((a, b) => a.trackOrder - b.trackOrder);
    return {
        cid: album.cid,
        name: album.name.trim(),
        artists: album.artists.map((a) => a.trim()).filter(Boolean),
        trackCount: songs.length,
        tracks: songs.slice(0, MUSIC_OG_TRACKS).map((s) => s.name.trim()),
        coverSource: album.coverUrl,
    };
}

export function musicOgHashParts(data: IMusicOgData): unknown[] {
    return [data.cid, data.name, data.artists.join("|"), data.trackCount, data.tracks.join("|"), data.coverSource];
}
