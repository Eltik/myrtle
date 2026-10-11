import { type IPreparedTarget, prepareQuery, prepareTarget, scorePrepared } from "#/lib/search/fuzzy";
import type { IAlbum, IArtist, ITrack } from "../impl/model";

/**
 * THE LIBRARY'S SEARCH, through the site's fuzzy scorer. A song is found by
 * its title (the name tier), its artists and its album (aliases); an album by
 * its name; an artist by their name. A blank query keeps everything in the
 * order given.
 */
export interface IPreparedList<T> {
    items: readonly T[];
    prepared: readonly IPreparedTarget[];
}

export function prepareSongs(tracks: readonly ITrack[]): IPreparedList<ITrack> {
    return { items: tracks, prepared: tracks.map((t) => prepareTarget({ name: t.title, aliases: [...t.artists, ...(t.albumName ? [t.albumName] : [])] })) };
}

export function prepareAlbums(albums: readonly IAlbum[]): IPreparedList<IAlbum> {
    return { items: albums, prepared: albums.map((a) => prepareTarget({ name: a.name })) };
}

export function prepareArtists(artists: readonly IArtist[]): IPreparedList<IArtist> {
    return { items: artists, prepared: artists.map((a) => prepareTarget({ name: a.name })) };
}

export interface ISongFilters {
    hideInstrumentals: boolean;
}

/**
 * The items matching `query`, with `keep` applied first. With a query, the
 * result is best match first (ties keep the input order) when `rank` is set,
 * else the input order.
 */
export function filterPrepared<T>(list: IPreparedList<T>, query: string, rank: boolean, keep?: (item: T) => boolean): T[] {
    const q = query.trim() === "" ? null : prepareQuery(query);
    const kept: { item: T; score: number; at: number }[] = [];
    list.items.forEach((item, at) => {
        if (keep && !keep(item)) return;
        if (q === null) {
            kept.push({ item, score: 0, at });
            return;
        }
        const target = list.prepared[at];
        const score = target ? scorePrepared(q, target) : 0;
        if (score > 0) kept.push({ item, score, at });
    });
    if (q !== null && rank) kept.sort((a, b) => b.score - a.score || a.at - b.at);
    return kept.map((k) => k.item);
}

export function filterSongs(list: IPreparedList<ITrack>, query: string, filters: ISongFilters, rank: boolean): ITrack[] {
    return filterPrepared(list, query, rank, filters.hideInstrumentals ? (t) => !t.instrumental : undefined);
}
