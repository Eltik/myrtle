import type { MusicPlayTotal } from "#/types/generated/MusicPlayTotal";
import type { IAlbum, ICatalog, ITrack } from "../impl/model";

/** Rows each "Your Stats" column lists. */
export const STATS_TOP = 10;

export interface IListeningStats {
    plays: number;
    seconds: number;
    topTracks: { track: ITrack; plays: number }[];
    topArtists: { name: string; plays: number }[];
    topAlbums: { album: IAlbum; plays: number }[];
}

/**
 * The stats view's numbers from the per-track totals. Plays and time count
 * every total, known to the catalogue or not; the top lists only name what
 * the catalogue knows. Artists leave the house credit out, so the column
 * shows the guests a listener actually picked.
 */
export function statsOf(totals: readonly MusicPlayTotal[], catalog: Pick<ICatalog, "byId" | "albumById" | "houseArtist">, top = STATS_TOP): IListeningStats {
    let plays = 0;
    let seconds = 0;
    const topTracks: { track: ITrack; plays: number }[] = [];
    const artists = new Map<string, number>();
    const albums = new Map<string, number>();
    for (const total of totals) {
        plays += total.plays;
        seconds += total.seconds;
        const track = catalog.byId.get(total.trackId);
        if (!track) continue;
        topTracks.push({ track, plays: total.plays });
        for (const a of new Set(track.artists)) if (a !== catalog.houseArtist) artists.set(a, (artists.get(a) ?? 0) + total.plays);
        if (track.albumId) albums.set(track.albumId, (albums.get(track.albumId) ?? 0) + total.plays);
    }
    const byPlays = <T extends { plays: number }>(a: T, b: T) => b.plays - a.plays;
    topTracks.sort(byPlays);
    const topAlbums: { album: IAlbum; plays: number }[] = [];
    for (const [id, n] of albums) {
        const album = catalog.albumById.get(id);
        if (album) topAlbums.push({ album, plays: n });
    }
    return {
        plays,
        seconds,
        topTracks: topTracks.slice(0, top),
        topArtists: [...artists]
            .map(([name, n]) => ({ name, plays: n }))
            .sort((a, b) => b.plays - a.plays || a.name.localeCompare(b.name))
            .slice(0, top),
        topAlbums: topAlbums.sort((a, b) => b.plays - a.plays || a.album.listOrder - b.album.listOrder).slice(0, top),
    };
}
