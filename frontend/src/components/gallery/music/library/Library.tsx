import type React from "react";
import { useMemo, useState } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { useDebounce } from "#/hooks/use-debounce";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { useLibrary } from "../impl/library/store";
import { type ICatalog, type ITrack, parseTrackId } from "../impl/model";
import type { LibrarySelection } from "../impl/route";
import type { MusicKindFilter } from "../impl/tracks";
import type { messages } from "../MusicGallery.messages";
import type { IQueueSource } from "../player/types";
import { AlbumsGrid } from "./AlbumsGrid";
import { ArtistsView } from "./ArtistsView";
import { FilterStrip, type StripMode } from "./FilterStrip";
import { filterPrepared, filterSongs, prepareAlbums, prepareArtists, prepareSongs } from "./filters";
import { GameTracks, type IGameIndexState } from "./GameTracks";
import { PlaylistHeader } from "./PlaylistView";
import { Rail } from "./Rail";
import { SongList } from "./SongList";
import { StatsView } from "./StatsView";
import { type AlbumSort, type SongSort, sortAlbums, sortSongs } from "./sort";

type MusicT = TypedT<typeof messages>;

/** The search box waits this long after the last keystroke before filtering, ms. */
const SEARCH_DEBOUNCE_MS = 120;

/** Where the label catalogue stands, for the views that need it. */
export type SirenStatus = "loading" | "syncing" | "ready" | "missing" | "failed";

interface ILibraryProps {
    catalog: ICatalog;
    selection: LibrarySelection;
    sirenStatus: SirenStatus;
    gameState: IGameIndexState;
    collator: Intl.Collator;
}

/** Tracks for a list of ids, in order, skipping the ones the catalogue does not hold. */
function resolve(ids: readonly string[], catalog: ICatalog): { tracks: ITrack[]; missing: number } {
    const tracks: ITrack[] = [];
    let missing = 0;
    for (const id of ids) {
        const tr = catalog.byId.get(id);
        if (tr) {
            tracks.push(tr);
            continue;
        }
        // An id counts as missing only once its collection has loaded.
        const kind = parseTrackId(id)?.kind;
        if ((kind === "siren" && catalog.sirenReady) || (kind === "game" && catalog.gameReady) || kind === undefined) missing += 1;
    }
    return { tracks, missing };
}

/**
 * THE LIBRARY TAB: the rail on the left, the filter strip and the chosen view
 * on the right. Sort, search and filters are shared across views, like one
 * toolbar, and reset with Reset.
 */
export function Library({ catalog, selection, sirenStatus, gameState, collator }: ILibraryProps): React.ReactElement {
    const t: MusicT = useT("gallery");
    const likes = useLibrary((s) => s.likes);
    const history = useLibrary((s) => s.history);
    const playlists = useLibrary((s) => s.playlists);
    const libraryLoaded = useLibrary((s) => s.loaded);

    const [rawQuery, setRawQuery] = useState("");
    const query = useDebounce(rawQuery, SEARCH_DEBOUNCE_MS);
    const [albumSort, setAlbumSort] = useState<AlbumSort>("newest");
    const [songSort, setSongSort] = useState<SongSort>("catalog");
    const [reversed, setReversed] = useState(false);
    const [hideInstrumentals, setHideInstrumentals] = useState(false);
    const [kind, setKind] = useState<MusicKindFilter>("all");
    const reset = () => {
        setRawQuery("");
        setAlbumSort("newest");
        setSongSort("catalog");
        setReversed(false);
        setHideInstrumentals(false);
        setKind("all");
    };

    const view = selection.kind === "view" ? selection.view : null;
    const playlist = selection.kind === "playlist" ? (playlists.find((p) => p.id === selection.id) ?? null) : null;

    // The base list of a song view, before search, filters and sort.
    const base = useMemo((): { tracks: ITrack[]; missing: number } | null => {
        if (selection.kind === "playlist") return playlist ? resolve(playlist.trackIds, catalog) : null;
        if (selection.kind === "artist") return { tracks: catalog.sirenTracks.filter((tr) => tr.artists.includes(selection.name)), missing: 0 };
        if (view === "songs") return { tracks: catalog.sirenTracks, missing: 0 };
        if (view === "liked") {
            const ids = Object.entries(likes)
                .sort((a, b) => (a[1] < b[1] ? 1 : a[1] > b[1] ? -1 : 0))
                .map(([id]) => id);
            return resolve(ids, catalog);
        }
        if (view === "history") {
            const seen = new Set<string>();
            const ids: string[] = [];
            for (const p of history) {
                if (seen.has(p.trackId)) continue;
                seen.add(p.trackId);
                ids.push(p.trackId);
            }
            return resolve(ids, catalog);
        }
        return null;
    }, [selection, view, playlist, catalog, likes, history]);

    const preparedSongs = useMemo(() => (base ? prepareSongs(base.tracks) : null), [base]);
    const songs = useMemo(() => {
        if (!preparedSongs) return [];
        const filtered = filterSongs(preparedSongs, query, { hideInstrumentals }, songSort === "catalog");
        return sortSongs(filtered, songSort, reversed, collator, catalog.houseArtist);
    }, [preparedSongs, query, hideInstrumentals, songSort, reversed, collator, catalog.houseArtist]);

    const preparedAlbums = useMemo(() => prepareAlbums(catalog.albums), [catalog.albums]);
    const albums = useMemo(() => sortAlbums(filterPrepared(preparedAlbums, query, false), albumSort, reversed, collator), [preparedAlbums, query, albumSort, reversed, collator]);
    const preparedArtists = useMemo(() => prepareArtists(catalog.artists), [catalog.artists]);
    const artists = useMemo(() => filterPrepared(preparedArtists, query, true), [preparedArtists, query]);

    const mode: StripMode = view === "albums" ? "albums" : view === "artists" ? "search" : view === "game" ? "game" : view === "stats" ? "search" : "songs";
    const needsSiren = view === "albums" || view === "songs" || view === "artists" || selection.kind === "artist";

    const title = selection.kind === "artist" ? t("music.rail.artistView", { name: selection.name }) : view ? t(`music.rail.${view}`) : null;
    const source: IQueueSource = selection.kind === "playlist" ? { kind: "playlist", id: selection.id, name: playlist?.name } : selection.kind === "artist" ? { kind: "artist", id: selection.name, name: selection.name } : { kind: "list", name: title ?? undefined };
    const pageKey = `${JSON.stringify(selection)}\u0000${query}\u0000${songSort}\u0000${albumSort}\u0000${reversed}\u0000${hideInstrumentals}`;

    let body: React.ReactNode;
    if (needsSiren && sirenStatus !== "ready") body = <SirenNotice status={sirenStatus} />;
    else if (view === "albums") body = <AlbumsGrid albums={albums} pageKey={pageKey} />;
    else if (view === "artists") body = <ArtistsView artists={artists} />;
    else if (view === "stats") body = <StatsView catalog={catalog} />;
    else if (view === "game") body = <GameTracks catalog={catalog} query={query} kind={kind} collator={collator} state={gameState} />;
    else if (selection.kind === "playlist" && !playlist) body = libraryLoaded ? <Notice text={t("music.playlist.gone")} /> : <ListSkeleton />;
    else {
        const reorderable = selection.kind === "playlist" && songSort === "catalog" && !reversed && query.trim() === "" && !hideInstrumentals;
        body = (
            <>
                {base && base.missing > 0 ? <p className="mt-0 mb-3 font-sans text-[12px] text-muted-foreground">{view === "liked" ? t("music.list.missingLiked", { count: base.missing }) : t("music.list.missing", { count: base.missing })}</p> : null}
                <SongList tracks={songs} source={source} houseArtist={catalog.houseArtist} pageKey={pageKey} playlist={playlist ? { id: playlist.id, trackIds: playlist.trackIds, reorderable } : undefined} />
            </>
        );
    }

    return (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-[200px_minmax(0,1fr)] md:gap-5">
            <Rail selection={selection} />
            <div className="min-w-0">
                {playlist ? <PlaylistHeader id={playlist.id} name={playlist.name} tracks={base?.tracks ?? []} totalIds={playlist.trackIds.length} source={source} /> : title ? <h2 className="mt-0 mb-3 font-sans font-semibold text-[18px] text-foreground">{title}</h2> : null}
                {view !== "stats" && !(selection.kind === "playlist" && !playlist) ? (
                    <FilterStrip
                        mode={mode}
                        query={rawQuery}
                        onQuery={setRawQuery}
                        albumSort={albumSort}
                        onAlbumSort={setAlbumSort}
                        songSort={songSort}
                        onSongSort={setSongSort}
                        reversed={reversed}
                        onReversed={setReversed}
                        hideInstrumentals={hideInstrumentals}
                        onHideInstrumentals={setHideInstrumentals}
                        kind={kind}
                        onKind={setKind}
                        onReset={reset}
                    />
                ) : null}
                {body}
            </div>
        </div>
    );
}

function Notice({ text }: { text: string }): React.ReactElement {
    return <div className="rounded-xl border border-border border-dashed p-10 text-center font-sans text-[13px] text-muted-foreground">{text}</div>;
}

function ListSkeleton(): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5">
            {Array.from({ length: 8 }, (_, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: a placeholder has no identity beyond its position
                <Skeleton key={i} className="h-10 w-full rounded-lg" />
            ))}
        </div>
    );
}

/** What a label view shows while the catalogue is not there. */
export function SirenNotice({ status }: { status: SirenStatus }): React.ReactElement {
    const t: MusicT = useT("gallery");
    if (status === "loading") return <ListSkeleton />;
    if (status === "syncing") return <Notice text={t("music.counts.syncing")} />;
    if (status === "missing") return <Notice text={t("music.siren.missing")} />;
    return <Notice text={t("music.siren.failed")} />;
}
