import { useQuery } from "@tanstack/react-query";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { ListMusicIcon, MicIcon } from "lucide-react";
import type React from "react";
import { lazy, Suspense, useCallback, useEffect, useMemo } from "react";
import { Button } from "#/components/ui/button";
import { PageHeader } from "#/components/ui/page-header";
import { Skeleton } from "#/components/ui/skeleton";
import { musicIndexQueryOptions, sirenCatalogQueryOptions } from "#/lib/api/music";
import { useGamedataServer, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { AlbumModal } from "./album/AlbumModal";
import { Coverflow } from "./discovery/Coverflow";
import { MusicKeyframes } from "./impl/Equalizer";
import { useLibrarySync } from "./impl/library/store";
import { buildCatalog, type IAlbum, parseTrackId } from "./impl/model";
import { type IMusicNav, MusicNavContext } from "./impl/nav";
import { type IMusicSearch, type LibrarySelection, type MusicTab, searchFor, selectionOf, tabOf } from "./impl/route";
import type { IGameIndexState } from "./library/GameTracks";
import { Library, SirenNotice, type SirenStatus } from "./library/Library";
import type { messages } from "./MusicGallery.messages";
import { setCatalog, togglePanel, useMusicPlayerLifetime, usePlayer } from "./player/engine";

type MusicT = TypedT<typeof messages>;

const MusicPlayerDock = lazy(() => import("./player/MusicPlayerDock").then((m) => ({ default: m.MusicPlayerDock })));

/**
 * THE MUSIC PAGE at `/gallery/music`: Monster Siren Records' releases and the
 * game's own soundtrack, in one player.
 *
 * Discovery is a coverflow of every release; Library is the browsable
 * collection (albums, songs, artists, likes, history, stats, playlists, the
 * in-game index). Both catalogues are fetched on the client, not in the route
 * loader: the in-game index alone is about 1.2 MB of JSON, and a loader would
 * serialise it into the page's HTML.
 *
 * The URL is the navigation state (see `impl/route.ts`), so every view and
 * album has a link. Playback lives in `player/engine.ts` and outlives the
 * views; the dock mounts last.
 */
export function MusicGallery(): React.ReactElement {
    const t: MusicT = useT("gallery");
    const locale = useLocale();
    const search = useSearch({ from: "/gallery/music" });
    const navigate = useNavigate({ from: "/gallery/music" });
    const sirenQ = useQuery(sirenCatalogQueryOptions());
    const gameQ = useQuery(musicIndexQueryOptions(useGamedataServer()));
    useLibrarySync();
    useMusicPlayerLifetime();

    const collator = useMemo(() => new Intl.Collator(locale, { sensitivity: "base", numeric: true }), [locale]);
    const siren = sirenQ.data ?? null;
    const game = gameQ.data ?? null;
    const catalog = useMemo(() => buildCatalog(siren, game, collator), [siren, game, collator]);
    useEffect(() => {
        setCatalog(catalog);
    }, [catalog]);

    const syncing = sirenQ.isPending && (sirenQ.failureReason as { status?: number } | null)?.status === 503;
    const sirenStatus: SirenStatus = sirenQ.isSuccess ? (siren ? "ready" : "missing") : sirenQ.isError ? "failed" : syncing ? "syncing" : "loading";
    const gameState: IGameIndexState = { loading: gameQ.isLoading, failed: gameQ.isError, unavailable: gameQ.isSuccess && game === null };

    const tab = tabOf(search);
    const selection = selectionOf(search, "albums");

    const go = useCallback(
        (patch: (prev: IMusicSearch) => IMusicSearch, replace = false) => {
            void navigate({ search: patch, replace, resetScroll: false });
        },
        [navigate],
    );
    const nav = useMemo<IMusicNav>(
        () => ({
            openAlbum: (albumId) => {
                const parsed = parseTrackId(albumId);
                if (parsed) go((prev) => ({ ...prev, album: parsed.key }), true);
            },
            openLibrary: (sel: LibrarySelection) => go((prev) => searchFor(sel, prev)),
        }),
        [go],
    );
    const setTab = (next: MusicTab) => go((prev) => (next === "discovery" ? { album: prev.album } : { ...prev, tab: "library" }));
    const openAlbum = useCallback((album: IAlbum) => nav.openAlbum(album.id), [nav]);

    return (
        <MusicNavContext.Provider value={nav}>
            <MusicKeyframes />
            <div className="page-shell pb-28 [--page-max:1200px]">
                <PageHeader
                    className="mb-4 border-primary/70 border-b-2 pb-3"
                    breadcrumbLabel={t("music.breadcrumb")}
                    breadcrumb={[t("music.breadcrumb.gallery"), t("music.title")]}
                    title={t("music.title")}
                    description={<HeaderCounts status={sirenStatus} albums={siren?.albums.length ?? 0} tracks={game?.tracks.length ?? null} />}
                />

                <Toolbar tab={tab} onTab={setTab} />

                {tab === "discovery" ? (
                    sirenStatus === "ready" && catalog.albums.length > 0 ? (
                        <Coverflow albums={catalog.albums} onOpen={openAlbum} />
                    ) : sirenStatus === "loading" ? (
                        <Skeleton className="h-75 w-full rounded-xl" />
                    ) : (
                        <div className="flex flex-col items-center gap-3">
                            <div className="w-full">
                                <SirenNotice status={sirenStatus === "ready" ? "missing" : sirenStatus} />
                            </div>
                            <Button variant="outline" className="max-sm:min-h-11" onClick={() => nav.openLibrary({ kind: "view", view: "game" })}>
                                {t("music.discovery.toGame")}
                            </Button>
                        </div>
                    )
                ) : (
                    <Library catalog={catalog} selection={selection} sirenStatus={sirenStatus} gameState={gameState} collator={collator} />
                )}
            </div>
            <AlbumModal cid={search.album ?? null} catalog={catalog} sirenStatus={sirenStatus} onClose={() => go((prev) => ({ ...prev, album: undefined }), true)} />
            <Suspense fallback={null}>
                <MusicPlayerDock />
            </Suspense>
        </MusicNavContext.Provider>
    );
}

/** The line under the title: what is in the collection, or where the label catalogue stands. */
function HeaderCounts({ status, albums, tracks }: { status: SirenStatus; albums: number; tracks: number | null }): React.ReactElement {
    const t: MusicT = useT("gallery");
    if (status === "loading") return <>{t("music.counts.loading")}</>;
    if (status === "syncing") return <>{t("music.counts.syncing")}</>;
    if (status === "failed") return <>{t("music.siren.failed")}</>;
    if (status === "missing") return <>{tracks !== null ? t("music.counts.noSiren", { tracks }) : t("music.siren.missing")}</>;
    return <>{tracks !== null ? t("music.counts", { albums, tracks }) : t("music.counts.sirenOnly", { albums })}</>;
}

/** Discovery | Library on the left; the Lyrics and Queue panel toggles on the right. */
function Toolbar({ tab, onTab }: { tab: MusicTab; onTab: (tab: MusicTab) => void }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const panel = usePlayer((s) => s.panel);
    const tabs: readonly MusicTab[] = ["discovery", "library"];
    return (
        <div className="mb-4 flex flex-wrap items-center justify-between gap-2">
            <fieldset className="flex gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75" aria-label={t("music.tab.aria")}>
                {tabs.map((v) => (
                    <button
                        key={v}
                        type="button"
                        onClick={() => onTab(v)}
                        aria-pressed={tab === v}
                        className={cn("h-11 cursor-pointer rounded-md px-3.5 font-sans font-semibold text-[12.5px] transition-colors sm:h-8", tab === v ? "bg-primary text-primary-foreground shadow-sm/5" : "text-muted-foreground hover:text-foreground")}
                    >
                        {t(`music.tab.${v}`)}
                    </button>
                ))}
            </fieldset>
            <div className="flex gap-1.5">
                <Button variant={panel === "lyrics" ? "default" : "outline"} className="max-sm:min-h-11" aria-pressed={panel === "lyrics"} title={t("music.tab.lyricsTitle")} onClick={() => togglePanel("lyrics")}>
                    <MicIcon aria-hidden="true" />
                    {t("music.tab.lyrics")}
                </Button>
                <Button variant={panel === "queue" ? "default" : "outline"} className="max-sm:min-h-11" aria-pressed={panel === "queue"} title={t("music.tab.queueTitle")} onClick={() => togglePanel("queue")}>
                    <ListMusicIcon aria-hidden="true" />
                    {t("music.tab.queue")}
                </Button>
            </div>
        </div>
    );
}
