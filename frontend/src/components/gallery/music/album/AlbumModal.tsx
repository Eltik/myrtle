import { useQuery } from "@tanstack/react-query";
import { CheckIcon, HeartIcon, LinkIcon, ListPlusIcon, MicIcon, PauseIcon, PlayIcon, ShuffleIcon } from "lucide-react";
import type React from "react";
import { memo, useEffect, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Dialog, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Skeleton } from "#/components/ui/skeleton";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useLastDefined } from "#/hooks/use-last-defined";
import { useMediaQuery } from "#/hooks/use-media-query";
import { sirenAlbumQueryOptions } from "#/lib/api/music";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { sizedCover } from "../impl/cover";
import { Equalizer } from "../impl/Equalizer";
import { toggleLike, useIsLiked } from "../impl/library/store";
import { displayArtists, type IAlbum, type ICatalog, type ITrack, isPlayable, sirenAlbumId } from "../impl/model";
import { playFrom, randomIndex } from "../impl/playback";
import { AddMenu } from "../library/AddMenu";
import type { SirenStatus } from "../library/Library";
import type { messages } from "../MusicGallery.messages";
import { playerStore, playQueue, togglePlay, toggleShuffle, useIsCurrent, usePlayer } from "../player/engine";
import type { IQueueSource } from "../player/types";
import { Sleeve } from "./Vinyl";

type MusicT = TypedT<typeof messages>;

/** How long "Link copied" shows, ms. */
const COPIED_MS = 1600;
/** An intro longer than this many characters (or 3 lines) is clamped behind More. */
const INTRO_CLAMP_CHARS = 90;
const INTRO_CLAMP_LINES = 3;

/**
 * THE ALBUM MODAL, opened by `?album=<cid>`. The sleeve and the details on the
 * left, the release, its controls, the label's intro and the tracks on the
 * right. Backdrop, the X and Escape close it.
 */
export function AlbumModal({ cid, catalog, sirenStatus, onClose }: { cid: string | null; catalog: ICatalog; sirenStatus: SirenStatus; onClose: () => void }): React.ReactElement {
    // The last album stays on screen while the dialog animates closed.
    const shown = useLastDefined(cid);
    const album = shown ? (catalog.albumById.get(sirenAlbumId(shown)) ?? null) : null;
    const t: MusicT = useT("gallery");
    return (
        <Dialog
            open={cid !== null}
            onOpenChange={(open) => {
                if (!open) onClose();
            }}
        >
            <DialogPopup className="overflow-hidden max-sm:h-[92dvh] sm:h-[min(640px,84vh)] sm:max-w-[min(900px,92vw)]">
                {album ? (
                    <AlbumBody album={album} catalog={catalog} />
                ) : (
                    <div className="flex flex-1 flex-col items-center justify-center gap-3 p-10 text-center">
                        <DialogTitle className="sr-only">{t("music.album.kicker")}</DialogTitle>
                        {sirenStatus === "loading" || sirenStatus === "syncing" ? <p className="m-0 font-sans text-[13px] text-muted-foreground">{t("music.album.loading")}</p> : <p className="m-0 font-sans text-[13px] text-muted-foreground">{t("music.album.failed")}</p>}
                    </div>
                )}
            </DialogPopup>
        </Dialog>
    );
}

function AlbumBody({ album, catalog }: { album: IAlbum; catalog: ICatalog }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const f = useFormatters();
    const detail = useQuery(sirenAlbumQueryOptions(album.cid));
    const tracks = catalog.tracksByAlbum.get(album.id) ?? [];
    const playable = tracks.filter(isPlayable);
    const isThis = usePlayer((s) => s.current?.albumId === album.id);
    const playing = usePlayer((s) => s.status === "playing" || s.status === "loading") && isThis;
    const shuffle = usePlayer((s) => s.shuffle);
    const source: IQueueSource = { kind: "album", id: album.id, name: album.name };
    const artists = album.artists.length > 0 ? album.artists : catalog.houseArtist ? [catalog.houseArtist] : [];
    const artistText = artists.join(", ") || t("music.album.kicker");
    const severalArtists = new Set(tracks.flatMap((tr) => displayArtists(tr, catalog.houseArtist))).size > 1;
    const addedOn = album.showAddedOn && album.addedOn ? f.date(`${album.addedOn}T00:00:00`) : null;

    const onPlay = () => {
        if (isThis) togglePlay();
        else if (playable.length > 0) playQueue(playable, 0, source);
    };
    const onShuffle = () => {
        const turningOn = !playerStore.state.shuffle;
        toggleShuffle();
        if (turningOn && !isThis && playable.length > 0) playQueue(playable, randomIndex(playable.length), source);
    };

    return (
        <div className="relative flex min-h-0 flex-1 flex-col sm:flex-row">
            <img referrerPolicy="no-referrer" src={album.coverUrl} alt="" aria-hidden="true" className="pointer-events-none absolute inset-y-0 right-0 hidden h-full w-[64%] object-cover opacity-5 sm:block" style={{ maskImage: "linear-gradient(to left, #000 40%, transparent)" }} />
            <aside className="relative hidden w-[300px] shrink-0 flex-col gap-5 border-border border-r p-6 sm:flex">
                <Sleeve coverUrl={album.coverUrl} alt={album.name} spinning={playing} />
                <dl className="m-0 flex flex-col font-sans text-[12px]">
                    <Detail label={t("music.album.artist")} value={artistText} />
                    <Detail label={t("music.album.tracks")} value={f.number(album.trackCount)} />
                    {album.lyricsCount > 0 ? <Detail label={t("music.album.withLyrics")} value={f.number(album.lyricsCount)} /> : null}
                    {album.instrumentalCount > 0 ? <Detail label={t("music.album.instrumentals")} value={f.number(album.instrumentalCount)} /> : null}
                    {addedOn ? <Detail label={t("music.album.since")} value={addedOn} /> : null}
                </dl>
            </aside>
            <div className="relative flex min-h-0 min-w-0 flex-1 flex-col overflow-y-auto">
                {detail.data?.coverDeUrl ? <img referrerPolicy="no-referrer" src={detail.data.coverDeUrl} alt="" className="aspect-16/10 w-full object-cover sm:hidden" /> : null}
                <div className="flex flex-col gap-3 p-5 sm:p-6 sm:pr-14">
                    <div className="flex items-start gap-3">
                        <img referrerPolicy="no-referrer" src={sizedCover(album.coverUrl, 64)} alt="" className="size-16 shrink-0 rounded-md bg-muted object-cover sm:hidden" />
                        <div className="min-w-0 flex-1">
                            <div className="font-mono text-[10px] text-primary uppercase tracking-[0.16em]">{t("music.album.kicker")}</div>
                            <DialogTitle className="mt-1 font-sans font-semibold text-[22px] leading-tight sm:text-[26px]">{album.name}</DialogTitle>
                            <div className="mt-1 font-sans text-[12.5px] text-muted-foreground">{t("music.album.subline", { artists: artistText, count: album.trackCount })}</div>
                            <div className="mt-0.5 font-sans text-[11.5px] text-muted-foreground sm:hidden">
                                {album.lyricsCount > 0 ? t("music.album.lyricsMeta", { count: album.lyricsCount }) : null}
                                {album.lyricsCount > 0 && addedOn ? " · " : null}
                                {addedOn ? t("music.album.sinceMeta", { date: addedOn }) : null}
                            </div>
                        </div>
                    </div>
                    <div className="flex flex-wrap items-center gap-2 border-border border-t pt-3">
                        <Button className="max-sm:min-h-11" disabled={playable.length === 0} onClick={onPlay}>
                            {playing ? <PauseIcon aria-hidden="true" /> : <PlayIcon aria-hidden="true" />}
                            {playing ? t("music.album.pause") : t("music.album.play")}
                        </Button>
                        <Button variant={shuffle ? "secondary" : "outline"} className={cn("max-sm:min-h-11", shuffle && "text-primary")} aria-pressed={shuffle} disabled={playable.length === 0} onClick={onShuffle}>
                            <ShuffleIcon aria-hidden="true" />
                            {t("music.album.shuffle")}
                        </Button>
                        <AddMenu
                            tracks={tracks}
                            align="start"
                            trigger={
                                <Button variant="outline" className="max-sm:min-h-11" title={t("music.album.addAllTitle")}>
                                    <ListPlusIcon aria-hidden="true" />
                                    {t("music.album.addAll")}
                                </Button>
                            }
                        />
                        <ShareButton cid={album.cid} name={album.name} />
                    </div>
                    <Intro text={detail.data?.intro ?? null} loading={detail.isLoading} />
                </div>
                <ol className="m-0 list-none px-3 pb-4 sm:px-4">
                    {tracks.map((track, i) => (
                        <AlbumTrack key={track.id} track={track} index={i} tracks={tracks} source={source} houseArtist={catalog.houseArtist} showArtist={severalArtists} />
                    ))}
                </ol>
            </div>
        </div>
    );
}

function Detail({ label, value }: { label: string; value: string }): React.ReactElement {
    return (
        <div className="flex items-baseline justify-between gap-3 border-border border-b py-2 last:border-b-0">
            <dt className="shrink-0 text-muted-foreground">{label}</dt>
            <dd className="m-0 min-w-0 text-right font-semibold text-foreground">{value}</dd>
        </div>
    );
}

function Intro({ text, loading }: { text: string | null; loading: boolean }): React.ReactElement | null {
    const t: MusicT = useT("gallery");
    const [open, setOpen] = useState(false);
    if (loading) return <Skeleton className="h-12 w-full rounded-md" />;
    if (!text || text.trim() === "") return null;
    const long = text.length > INTRO_CLAMP_CHARS || text.split("\n").length > INTRO_CLAMP_LINES;
    return (
        <div className="font-sans text-[13px] text-muted-foreground leading-relaxed">
            <p className={cn("m-0 whitespace-pre-line", long && !open && "line-clamp-3")}>{text}</p>
            {long ? (
                <button type="button" onClick={() => setOpen((o) => !o)} className="mt-1 cursor-pointer font-medium text-[12px] text-primary hover:underline max-sm:min-h-11">
                    {open ? t("music.album.less") : t("music.album.more")}
                </button>
            ) : null}
        </div>
    );
}

function ShareButton({ cid, name }: { cid: string; name: string }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const coarse = useMediaQuery({ pointer: "coarse" });
    const [copied, setCopied] = useState(false);
    useEffect(() => {
        if (!copied) return;
        const id = setTimeout(() => setCopied(false), COPIED_MS);
        return () => clearTimeout(id);
    }, [copied]);
    const onClick = async () => {
        const url = `${window.location.origin}/gallery/music?album=${encodeURIComponent(cid)}`;
        if (coarse && typeof navigator.share === "function") {
            try {
                await navigator.share({ url, title: name });
            } catch {
                // The share sheet was dismissed.
            }
            return;
        }
        try {
            await navigator.clipboard.writeText(url);
            setCopied(true);
        } catch {
            // Clipboard refused (an insecure origin); nothing to show.
        }
    };
    const label = copied ? t("music.album.copied") : t("music.album.copyLink");
    return (
        <Tooltip>
            <TooltipTrigger render={<Button size="icon" variant="outline" className="max-sm:size-11" onClick={onClick} aria-label={label} />}>{copied ? <CheckIcon aria-hidden="true" /> : <LinkIcon aria-hidden="true" />}</TooltipTrigger>
            <TooltipPopup>{label}</TooltipPopup>
        </Tooltip>
    );
}

const AlbumTrack = memo(function AlbumTrack({ track, index, tracks, source, houseArtist, showArtist }: { track: ITrack; index: number; tracks: readonly ITrack[]; source: IQueueSource; houseArtist: string | null; showArtist: boolean }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const current = useIsCurrent(track.id);
    const playing = usePlayer((s) => s.status === "playing" || s.status === "loading") && current;
    const liked = useIsLiked(track.id);
    const title = track.title;
    return (
        <li className={cn("group/track flex min-h-10.5 items-center gap-1 rounded-md pr-1 transition-colors hover:bg-primary/6", current && "bg-primary/10 shadow-[inset_2px_0_0_var(--color-primary)]")}>
            <button
                type="button"
                onClick={() => playFrom(tracks, track, source)}
                aria-label={playing ? t("music.row.pause", { name: title }) : t("music.row.play", { name: title })}
                className="flex min-h-10.5 min-w-0 flex-1 cursor-pointer items-center gap-3 rounded-md pl-2 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring/40 max-sm:min-h-11"
            >
                <span className="flex w-6 shrink-0 items-center justify-center font-mono text-[12px] text-muted-foreground tabular-nums">
                    {current ? (
                        playing ? (
                            <>
                                <Equalizer className="group-hover/track:hidden" />
                                <PauseIcon className="hidden size-3.5 text-primary group-hover/track:block" aria-hidden="true" />
                            </>
                        ) : (
                            <PlayIcon className="size-3.5 text-primary" aria-hidden="true" />
                        )
                    ) : (
                        <>
                            <span className="group-hover/track:hidden">{index + 1}</span>
                            <PlayIcon className="hidden size-3.5 text-foreground group-hover/track:block" aria-hidden="true" />
                        </>
                    )}
                </span>
                <span className={cn("min-w-0 truncate font-sans text-[13.5px]", current ? "font-medium text-primary" : "text-foreground")}>{title}</span>
                {track.instrumental ? (
                    <Badge variant="outline" size="sm" className="shrink-0 font-mono tracking-[0.06em]">
                        {t("music.list.inst")}
                    </Badge>
                ) : null}
                {track.hasLyrics ? (
                    <Tooltip>
                        <TooltipTrigger render={<span className="inline-flex shrink-0" />}>
                            <MicIcon className="size-3 text-muted-foreground" aria-label={t("music.list.hasLyrics")} />
                        </TooltipTrigger>
                        <TooltipPopup>{t("music.list.hasLyrics")}</TooltipPopup>
                    </Tooltip>
                ) : null}
            </button>
            <span className="flex shrink-0 items-center gap-0.5 transition-opacity sm:opacity-0 sm:group-hover/track:opacity-100 sm:group-focus-within/track:opacity-100">
                {showArtist ? <span className="max-w-40 truncate px-1 font-sans text-[11.5px] text-muted-foreground max-sm:hidden">{displayArtists(track, houseArtist).join(", ")}</span> : null}
                <Button size="icon-sm" variant="ghost" onClick={() => toggleLike(track.id)} aria-pressed={liked} aria-label={liked ? t("music.list.unlike", { name: title }) : t("music.list.likeTrack", { name: title })} className={cn(liked && "text-primary")}>
                    <HeartIcon className={cn(liked && "fill-current")} aria-hidden="true" />
                </Button>
                <AddMenu
                    tracks={[track]}
                    trigger={
                        <Button size="icon-sm" variant="ghost" aria-label={t("music.list.addTrack", { name: title })} title={t("music.list.addTitle")}>
                            <ListPlusIcon aria-hidden="true" />
                        </Button>
                    }
                />
            </span>
        </li>
    );
});
