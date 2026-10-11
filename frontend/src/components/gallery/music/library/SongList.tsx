import { GripVerticalIcon, HeartIcon, MicIcon, MinusIcon, PauseIcon, PlayIcon, PlusIcon } from "lucide-react";
import type React from "react";
import { memo, useMemo, useRef, useState } from "react";
import { pageOf, RECORD_PAGE } from "#/components/story/library/impl/paging";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { sizedCover } from "../impl/cover";
import { Equalizer } from "../impl/Equalizer";
import { removeFromPlaylist, setPlaylistTracks, toggleLike, useIsLiked } from "../impl/library/store";
import { displayArtists, type ITrack, isPlayable } from "../impl/model";
import { useMusicNav } from "../impl/nav";
import { playFrom } from "../impl/playback";
import type { messages } from "../MusicGallery.messages";
import { useIsCurrent, usePlayer } from "../player/engine";
import type { IQueueSource } from "../player/types";
import { AddMenu } from "./AddMenu";
import { moveTrack } from "./reorder";

type MusicT = TypedT<typeof messages>;

export interface ISongListPlaylist {
    id: string;
    /** The playlist's full order, including ids the catalogue does not know right now. */
    trackIds: readonly string[];
    /** Drag and Alt+Arrow reorder; only in catalog order with no search or filter. */
    reorderable: boolean;
}

interface ISongListProps {
    tracks: readonly ITrack[];
    source: IQueueSource;
    houseArtist: string | null;
    /** Set in a playlist view: rows get a remove button and, when allowed, a grip. */
    playlist?: ISongListPlaylist;
    /** Changes whenever the list's controls change, so paging starts over. */
    pageKey: string;
}

/** An 18 px box takes a 44 px touch target on a coarse pointer, centred on the box. */
const CHECK_HIT = "pointer-coarse:after:absolute pointer-coarse:after:-inset-[15px]";

const GRID = "grid grid-cols-[28px_minmax(0,1fr)_32px_32px] items-center gap-x-2 sm:grid-cols-[28px_minmax(0,1fr)_minmax(0,22%)_minmax(0,18%)_32px_32px]";

/**
 * THE SONG LIST All Songs, Liked, Recently Played, an artist and a playlist
 * share. Playback starts only from a row's number, cover or title, so a row
 * can be selected or its album opened without playing anything. A page of
 * rows mounts at a time behind "Show more".
 *
 * A long list stays cheap to scroll: off-screen rows skip style, layout and
 * paint (`content-visibility: auto` with a 48 px estimate, a row's desktop
 * height; `auto` remembers the real one once drawn), row covers decode off
 * the main thread, and rows are memoised with stable callbacks, so selecting
 * or dragging re-renders only the rows that changed.
 */
export function SongList({ tracks, source, houseArtist, playlist, pageKey }: ISongListProps): React.ReactElement {
    const t: MusicT = useT("gallery");
    const [selected, setSelected] = useState<ReadonlySet<string>>(() => new Set());
    const anchor = useRef<number | null>(null);
    const [raised, setRaised] = useState<{ key: string; limit: number }>({ key: pageKey, limit: RECORD_PAGE });
    const limit = raised.key === pageKey ? raised.limit : RECORD_PAGE;
    const { shown, hidden } = pageOf(tracks, limit);
    const more = Math.min(RECORD_PAGE, hidden);
    const [drag, setDrag] = useState<{ from: number; over: number } | null>(null);
    // Rows are memoised, so they get callbacks that never change identity and call the latest closures.
    const latest = useRef<{ select: (index: number, range: boolean) => void; reorder: IReorder | undefined }>({ select: () => {}, reorder: undefined });
    const stable = useMemo(
        () => ({
            select: (index: number, range: boolean) => latest.current.select(index, range),
            reorder: {
                onGripDown: (index: number, e: React.PointerEvent<HTMLElement>) => latest.current.reorder?.onGripDown(index, e),
                onGripMove: (e: React.PointerEvent<HTMLElement>) => latest.current.reorder?.onGripMove(e),
                onGripUp: () => latest.current.reorder?.onGripUp(),
                onMoveKey: (index: number, dir: -1 | 1) => latest.current.reorder?.onMoveKey(index, dir),
            } satisfies IReorder,
        }),
        [],
    );

    const selectedTracks = useMemo(() => tracks.filter((tr) => selected.has(tr.id)), [tracks, selected]);
    const allShownSelected = shown.length > 0 && shown.every((tr) => selected.has(tr.id));

    const onSelect = (index: number, range: boolean) => {
        const track = tracks[index];
        if (!track) return;
        setSelected((prev) => {
            const next = new Set(prev);
            if (range && anchor.current !== null) {
                const [a, b] = anchor.current < index ? [anchor.current, index] : [index, anchor.current];
                for (let i = a; i <= b; i += 1) {
                    const tr = tracks[i];
                    if (tr) next.add(tr.id);
                }
            } else if (next.has(track.id)) next.delete(track.id);
            else next.add(track.id);
            return next;
        });
        anchor.current = index;
    };

    const move = (from: number, to: number) => {
        const a = tracks[from];
        const b = tracks[to];
        if (!playlist || !a || !b || from === to) return;
        setPlaylistTracks(playlist.id, moveTrack(playlist.trackIds, a.id, b.id));
    };

    const reorder = playlist?.reorderable
        ? {
              onGripDown: (index: number, e: React.PointerEvent<HTMLElement>) => {
                  if (e.button !== 0) return;
                  e.currentTarget.setPointerCapture(e.pointerId);
                  setDrag({ from: index, over: index });
              },
              onGripMove: (e: React.PointerEvent<HTMLElement>) => {
                  if (!drag) return;
                  const row = document.elementFromPoint(e.clientX, e.clientY)?.closest<HTMLElement>("[data-row-index]");
                  const over = row ? Number(row.dataset.rowIndex) : Number.NaN;
                  if (Number.isFinite(over) && over !== drag.over) setDrag({ from: drag.from, over });
              },
              onGripUp: () => {
                  if (drag) move(drag.from, drag.over);
                  setDrag(null);
              },
              onMoveKey: (index: number, dir: -1 | 1) => move(index, index + dir),
          }
        : undefined;

    latest.current = { select: onSelect, reorder };

    if (tracks.length === 0) return <p className="rounded-xl border border-border border-dashed p-10 text-center font-sans text-[13px] text-muted-foreground">{t("music.list.empty")}</p>;

    return (
        <div>
            <div className={cn(GRID, "border-border border-b px-2 pb-1.5 font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]")}>
                <Checkbox checked={allShownSelected} indeterminate={!allShownSelected && selectedTracks.length > 0} onCheckedChange={(checked) => setSelected(checked ? new Set(shown.map((tr) => tr.id)) : new Set())} aria-label={t("music.list.selectAll")} className={cn("justify-self-center", CHECK_HIT)} />
                <span className="flex items-center gap-2">
                    <span className="w-7 text-center">#</span>
                    <span className="w-7" aria-hidden="true" />
                    {t("music.list.title")}
                </span>
                <span className="max-sm:hidden">{t("music.list.album")}</span>
                <span className="max-sm:hidden">{t("music.list.artist")}</span>
                <span className="sr-only">{t("music.list.like")}</span>
                <span className="sr-only">{playlist ? t("music.list.remove") : t("music.list.add")}</span>
            </div>
            <ul className="m-0 list-none p-0">
                {shown.map((track, i) => (
                    <SongRow
                        key={track.id}
                        index={i}
                        track={track}
                        tracks={tracks}
                        source={source}
                        houseArtist={houseArtist}
                        selected={selected.has(track.id)}
                        onSelect={stable.select}
                        playlistId={playlist?.id}
                        reorder={reorder ? stable.reorder : undefined}
                        dropTarget={drag !== null && drag.over === i && drag.from !== i ? (drag.from < i ? "below" : "above") : null}
                        dragging={drag?.from === i}
                    />
                ))}
            </ul>
            {hidden > 0 ? (
                <div className="mt-4 flex justify-center">
                    <Button variant="outline" className="max-sm:min-h-11" onClick={() => setRaised({ key: pageKey, limit: limit + more })}>
                        {t("music.showMore", { count: more, remaining: hidden })}
                    </Button>
                </div>
            ) : null}
            {selectedTracks.length > 0 ? (
                <div className="sticky bottom-24 z-20 mx-auto mt-4 flex w-fit items-center gap-2 rounded-xl border border-border bg-popover px-3 py-2 shadow-lg/10">
                    <span className="font-sans font-semibold text-[13px] tabular-nums">{t("music.list.selected", { count: selectedTracks.length })}</span>
                    <AddMenu
                        tracks={selectedTracks}
                        align="center"
                        trigger={
                            <Button size="sm" variant="default" className="max-sm:min-h-11" title={t("music.list.actionsTitle")}>
                                {t("music.list.actions")}
                            </Button>
                        }
                    />
                    <Button size="sm" variant="ghost" className="max-sm:min-h-11" onClick={() => setSelected(new Set())}>
                        {t("music.list.clear")}
                    </Button>
                </div>
            ) : null}
        </div>
    );
}

interface IReorder {
    onGripDown: (index: number, e: React.PointerEvent<HTMLElement>) => void;
    onGripMove: (e: React.PointerEvent<HTMLElement>) => void;
    onGripUp: () => void;
    onMoveKey: (index: number, dir: -1 | 1) => void;
}

interface ISongRowProps {
    index: number;
    track: ITrack;
    tracks: readonly ITrack[];
    source: IQueueSource;
    houseArtist: string | null;
    selected: boolean;
    onSelect: (index: number, range: boolean) => void;
    playlistId?: string;
    reorder?: IReorder;
    dropTarget: "above" | "below" | null;
    dragging: boolean;
}

const SongRow = memo(function SongRow({ index, track, tracks, source, houseArtist, selected, onSelect, playlistId, reorder, dropTarget, dragging }: ISongRowProps): React.ReactElement {
    const t: MusicT = useT("gallery");
    const nav = useMusicNav();
    const current = useIsCurrent(track.id);
    const playing = usePlayer((s) => s.status === "playing" || s.status === "loading") && current;
    const liked = useIsLiked(track.id);
    const title = track.title;
    const artists = displayArtists(track, houseArtist);
    const playable = isPlayable(track);
    const albumId = track.albumId;

    return (
        <li
            data-row-index={index}
            onKeyDown={
                reorder
                    ? (e) => {
                          if (!e.altKey || (e.key !== "ArrowUp" && e.key !== "ArrowDown")) return;
                          e.preventDefault();
                          reorder.onMoveKey(index, e.key === "ArrowUp" ? -1 : 1);
                      }
                    : undefined
            }
            className={cn(
                GRID,
                "group/row relative rounded-lg px-2 py-1 transition-colors hover:bg-primary/6",
                // Paint containment would clip the reorder grip, which hangs outside the row.
                !reorder && "[contain-intrinsic-size:auto_48px] [content-visibility:auto]",
                current && "bg-primary/10 shadow-[inset_2px_0_0_var(--color-primary)]",
                selected && "bg-secondary/60",
                dragging && "opacity-50",
                dropTarget === "above" && "shadow-[inset_0_2px_0_var(--color-primary)]",
                dropTarget === "below" && "shadow-[inset_0_-2px_0_var(--color-primary)]",
            )}
        >
            <span className="flex items-center justify-center gap-0.5">
                {reorder ? (
                    <button
                        type="button"
                        aria-label={t("music.list.reorder", { name: title })}
                        title={t("music.list.reorderHint")}
                        onPointerDown={(e) => reorder.onGripDown(index, e)}
                        onPointerMove={reorder.onGripMove}
                        onPointerUp={reorder.onGripUp}
                        onPointerCancel={reorder.onGripUp}
                        className="absolute top-1/2 -left-4 hidden h-7 w-4 -translate-y-1/2 cursor-grab touch-none items-center justify-center text-muted-foreground hover:text-foreground sm:flex"
                    >
                        <GripVerticalIcon className="size-3.5" aria-hidden="true" />
                    </button>
                ) : null}
                <Checkbox checked={selected} onClick={(e) => onSelect(index, e.shiftKey)} aria-label={t("music.list.select", { name: title })} className={CHECK_HIT} />
            </span>
            <button
                type="button"
                disabled={!playable}
                onClick={() => playFrom(tracks, track, source)}
                aria-label={current && playing ? t("music.row.pause", { name: title }) : t("music.row.play", { name: title })}
                className="group/play flex min-h-10 pointer-coarse:min-h-11 min-w-0 cursor-pointer items-center gap-2 text-left disabled:cursor-default max-sm:min-h-11"
            >
                <span className="flex w-7 shrink-0 items-center justify-center font-mono text-[11.5px] text-muted-foreground tabular-nums">
                    {current ? (
                        playing ? (
                            <>
                                <Equalizer className="group-hover/play:hidden" />
                                <PauseIcon className="hidden size-3.5 text-primary group-hover/play:block" aria-hidden="true" />
                            </>
                        ) : (
                            <PlayIcon className="size-3.5 text-primary" aria-hidden="true" />
                        )
                    ) : (
                        <>
                            <span className={cn(playable && "group-hover/play:hidden")}>{index + 1}</span>
                            {playable ? <PlayIcon className="hidden size-3.5 text-foreground group-hover/play:block" aria-hidden="true" /> : null}
                        </>
                    )}
                </span>
                <span className="relative size-7 shrink-0 overflow-hidden rounded-[4px] bg-muted">{track.coverUrl ? <img referrerPolicy="no-referrer" src={sizedCover(track.coverUrl, 28)} alt="" loading="lazy" decoding="async" className="size-full object-cover" /> : null}</span>
                <span className="flex min-w-0 flex-1 flex-col leading-tight">
                    <span className="flex min-w-0 items-center gap-1.5">
                        <span className={cn("truncate font-medium font-sans text-[13px]", current ? "text-primary" : "text-foreground")}>{title}</span>
                        {track.instrumental ? (
                            <Badge variant="outline" size="sm" className="shrink-0 font-mono tracking-[0.06em]">
                                {t("music.list.inst")}
                            </Badge>
                        ) : null}
                        {track.hasLyrics ? (
                            <span title={t("music.list.hasLyrics")} className="inline-flex shrink-0">
                                <MicIcon className="size-3 text-muted-foreground" aria-label={t("music.list.hasLyrics")} />
                            </span>
                        ) : null}
                    </span>
                    <span className="truncate font-sans text-[11.5px] text-muted-foreground sm:hidden">{artists.join(", ") || track.albumName}</span>
                </span>
            </button>
            <span className="min-w-0 max-sm:hidden">
                {albumId ? (
                    <button type="button" onClick={() => nav.openAlbum(albumId)} title={track.albumName ?? undefined} className="pointer-coarse:-my-[13px] block max-w-full cursor-pointer truncate pointer-coarse:py-[13px] text-left font-sans text-[12.5px] text-muted-foreground hover:text-foreground hover:underline">
                        {track.albumName}
                    </button>
                ) : (
                    <span className="block truncate font-sans text-[12.5px] text-muted-foreground" title={track.albumName ?? undefined}>
                        {track.albumName}
                    </span>
                )}
            </span>
            <span className="min-w-0 truncate font-sans text-[12.5px] text-muted-foreground max-sm:hidden" title={artists.join(", ")}>
                {artists.map((name, i) => (
                    <span key={name}>
                        {i > 0 ? ", " : null}
                        {name === houseArtist ? (
                            name
                        ) : (
                            <button type="button" onClick={() => nav.openLibrary({ kind: "artist", name })} className="cursor-pointer hover:text-foreground hover:underline">
                                {name}
                            </button>
                        )}
                    </span>
                ))}
            </span>
            <Button
                size="icon-sm"
                variant="ghost"
                onClick={() => toggleLike(track.id)}
                aria-pressed={liked}
                aria-label={liked ? t("music.list.unlike", { name: title }) : t("music.list.likeTrack", { name: title })}
                className={cn("transition-opacity", liked ? "text-primary" : "sm:opacity-0 sm:group-hover/row:opacity-100 sm:focus-visible:opacity-100")}
            >
                <HeartIcon className={cn(liked && "fill-current")} aria-hidden="true" />
            </Button>
            {playlistId ? (
                <Button size="icon-sm" variant="ghost" onClick={() => removeFromPlaylist(playlistId, track.id)} aria-label={t("music.list.removeTrack", { name: title })} title={t("music.list.remove")}>
                    <MinusIcon aria-hidden="true" />
                </Button>
            ) : (
                <AddMenu
                    tracks={[track]}
                    trigger={
                        <Button size="icon-sm" variant="ghost" aria-label={t("music.list.addTrack", { name: title })} title={t("music.list.addTitle")}>
                            <PlusIcon aria-hidden="true" />
                        </Button>
                    }
                />
            )}
        </li>
    );
});
