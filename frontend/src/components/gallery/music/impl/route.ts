/**
 * The music page's URL. `?tab=library` (absent is Discovery), the Library
 * view (`?view=`, `?playlist=`, `?artist=`; any of them implies Library) and
 * the open album (`?album=<cid>`). The URL IS the navigation state, so every
 * view and album has a link.
 */
export const MUSIC_TABS = ["discovery", "library"] as const;
export type MusicTab = (typeof MUSIC_TABS)[number];

export const LIBRARY_VIEWS = ["albums", "songs", "artists", "liked", "history", "stats", "charts", "game"] as const;
export type LibraryView = (typeof LIBRARY_VIEWS)[number];

export interface IMusicSearch {
    tab?: MusicTab;
    view?: LibraryView;
    playlist?: string;
    artist?: string;
    album?: string;
}

function isOneOf<T extends string>(list: readonly T[], value: unknown): value is T {
    return typeof value === "string" && (list as readonly string[]).includes(value);
}

function text(value: unknown): string | undefined {
    if (typeof value === "number") return String(value);
    if (typeof value !== "string") return undefined;
    const v = value.trim();
    return v === "" ? undefined : v;
}

export function validateMusicSearch(search: Record<string, unknown>): IMusicSearch {
    const out: IMusicSearch = {};
    if (isOneOf(MUSIC_TABS, search.tab) && search.tab !== "discovery") out.tab = search.tab;
    if (isOneOf(LIBRARY_VIEWS, search.view)) out.view = search.view;
    const playlist = text(search.playlist);
    if (playlist) out.playlist = playlist;
    const artist = text(search.artist);
    if (artist) out.artist = artist;
    const album = text(search.album);
    if (album) out.album = album;
    return out;
}

/** What the Library shows: a playlist, an artist, else a named view. */
export type LibrarySelection = { kind: "view"; view: LibraryView } | { kind: "playlist"; id: string } | { kind: "artist"; name: string };

export function tabOf(search: IMusicSearch): MusicTab {
    if (search.tab) return search.tab;
    return search.view || search.playlist || search.artist ? "library" : "discovery";
}

export function selectionOf(search: IMusicSearch, fallback: LibraryView): LibrarySelection {
    if (search.playlist) return { kind: "playlist", id: search.playlist };
    if (search.artist) return { kind: "artist", name: search.artist };
    return { kind: "view", view: search.view ?? fallback };
}

/** The search params that show `selection` in the Library, keeping an open album. */
export function searchFor(selection: LibrarySelection, prev: IMusicSearch): IMusicSearch {
    const base: IMusicSearch = { tab: "library", album: prev.album };
    if (selection.kind === "playlist") return { ...base, playlist: selection.id };
    if (selection.kind === "artist") return { ...base, artist: selection.name };
    return { ...base, view: selection.view };
}
