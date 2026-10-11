import { displayArtists, type IAlbum, type ITrack } from "../impl/model";

/** The Albums view's orders. `newest` is the label's own list order. */
export const ALBUM_SORTS = ["newest", "title", "tracks"] as const;
export type AlbumSort = (typeof ALBUM_SORTS)[number];

/** The song views' orders. `catalog` keeps the list as the view builds it. */
export const SONG_SORTS = ["catalog", "title", "album", "artist"] as const;
export type SongSort = (typeof SONG_SORTS)[number];

/**
 * Albums in the asked order; `reversed` flips any of them. Newest is list
 * order ascending (0 is the newest release), title is alphabetical, track
 * count is the biggest first. Ties keep list order, so the result is stable.
 */
export function sortAlbums(albums: readonly IAlbum[], sort: AlbumSort, reversed: boolean, collator: Intl.Collator): IAlbum[] {
    const out = [...albums];
    const byList = (a: IAlbum, b: IAlbum) => a.listOrder - b.listOrder;
    if (sort === "newest") out.sort(byList);
    else if (sort === "title") out.sort((a, b) => collator.compare(a.name, b.name) || byList(a, b));
    else out.sort((a, b) => b.trackCount - a.trackCount || byList(a, b));
    if (reversed) out.reverse();
    return out;
}

/**
 * Songs in the asked order; `reversed` flips any of them. Title ignores the
 * `(Instrumental)` marker, album keeps track order inside an album, artist
 * sorts by the names a row shows (the guests, else the house name). Ties keep
 * the input order.
 */
export function sortSongs(tracks: readonly ITrack[], sort: SongSort, reversed: boolean, collator: Intl.Collator, houseArtist: string | null): ITrack[] {
    const at = new Map<ITrack, number>();
    tracks.forEach((t, i) => {
        at.set(t, i);
    });
    const pos = (t: ITrack) => at.get(t) ?? 0;
    const out = [...tracks];
    if (sort === "title") out.sort((a, b) => collator.compare(a.title, b.title) || pos(a) - pos(b));
    else if (sort === "album") out.sort((a, b) => collator.compare(a.albumName ?? "", b.albumName ?? "") || a.trackOrder - b.trackOrder || pos(a) - pos(b));
    else if (sort === "artist") {
        const key = new Map<ITrack, string>();
        for (const t of tracks) key.set(t, displayArtists(t, houseArtist).join(", "));
        out.sort((a, b) => collator.compare(key.get(a) ?? "", key.get(b) ?? "") || pos(a) - pos(b));
    }
    if (reversed) out.reverse();
    return out;
}
