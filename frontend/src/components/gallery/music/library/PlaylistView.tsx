import { ListMusicIcon, PencilIcon, PlayIcon, ShuffleIcon, Trash2Icon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { deletePlaylist, renamePlaylist } from "../impl/library/store";
import { type ITrack, isPlayable } from "../impl/model";
import { useMusicNav } from "../impl/nav";
import { randomIndex } from "../impl/playback";
import type { messages } from "../MusicGallery.messages";
import { playerStore, playQueue, toggleShuffle } from "../player/engine";
import type { IQueueSource } from "../player/types";
import { PlaylistNameDialog } from "./PlaylistNameDialog";

type MusicT = TypedT<typeof messages>;

/** Covers the collage shows. */
const COLLAGE = 4;

/**
 * A playlist's header: a collage of its first distinct covers, the name
 * (click to rename), the count, Play, Shuffle and Delete.
 */
export function PlaylistHeader({ id, name, tracks, totalIds, source }: { id: string; name: string; tracks: readonly ITrack[]; totalIds: number; source: IQueueSource }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const nav = useMusicNav();
    const [renaming, setRenaming] = useState(false);
    const [deleting, setDeleting] = useState(false);
    const covers = [...new Set(tracks.map((tr) => tr.coverUrl).filter((c): c is string => c !== null))].slice(0, COLLAGE);
    const playable = tracks.filter(isPlayable);

    const play = (shuffle: boolean) => {
        if (playable.length === 0) return;
        if (shuffle !== playerStore.state.shuffle) toggleShuffle();
        playQueue(playable, shuffle ? randomIndex(playable.length) : 0, source);
    };

    return (
        <div className="mb-4 flex flex-wrap items-center gap-4">
            <div className="grid size-24 shrink-0 grid-cols-2 grid-rows-2 overflow-hidden rounded-lg bg-muted">
                {covers.length === 0 ? (
                    <ListMusicIcon aria-hidden="true" className="col-span-2 row-span-2 m-auto size-9 text-muted-foreground" />
                ) : covers.length < COLLAGE ? (
                    <img referrerPolicy="no-referrer" src={covers[0]} alt="" className="col-span-2 row-span-2 size-full object-cover" />
                ) : (
                    covers.map((c) => <img referrerPolicy="no-referrer" key={c} src={c} alt="" className="size-full object-cover" />)
                )}
            </div>
            <div className="flex min-w-0 flex-1 flex-col gap-1">
                <Tooltip>
                    <TooltipTrigger render={<button type="button" onClick={() => setRenaming(true)} className="group/name flex min-w-0 cursor-pointer items-center gap-2 text-left max-sm:min-h-11" />}>
                        <h2 className="m-0 truncate font-sans font-semibold text-[22px] text-foreground">{name}</h2>
                        <PencilIcon aria-hidden="true" className="size-3.5 shrink-0 text-muted-foreground opacity-0 transition-opacity group-hover/name:opacity-100" />
                    </TooltipTrigger>
                    <TooltipPopup>{t("music.playlist.rename")}</TooltipPopup>
                </Tooltip>
                <span className="font-sans text-[12.5px] text-muted-foreground">{t("music.list.trackCount", { count: totalIds })}</span>
                <div className="mt-1 flex flex-wrap gap-2">
                    <Button size="sm" className="max-sm:min-h-11" disabled={playable.length === 0} onClick={() => play(false)}>
                        <PlayIcon aria-hidden="true" />
                        {t("music.album.play")}
                    </Button>
                    <Button size="sm" variant="outline" className="max-sm:min-h-11" disabled={playable.length === 0} onClick={() => play(true)}>
                        <ShuffleIcon aria-hidden="true" />
                        {t("music.album.shuffle")}
                    </Button>
                    <Button size="sm" variant="destructive-outline" className="max-sm:min-h-11" onClick={() => setDeleting(true)}>
                        <Trash2Icon aria-hidden="true" />
                        {t("music.playlist.delete")}
                    </Button>
                </div>
            </div>
            <PlaylistNameDialog open={renaming} onOpenChange={setRenaming} mode="rename" initial={name} onSubmit={(next) => renamePlaylist(id, next)} />
            <AlertDialog open={deleting} onOpenChange={setDeleting}>
                <AlertDialogPopup>
                    <AlertDialogHeader>
                        <AlertDialogTitle>{t("music.playlist.deleteTitle")}</AlertDialogTitle>
                        <AlertDialogDescription>{t("music.playlist.deleteConfirm", { name })}</AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogClose render={<Button variant="outline" />}>{t("music.playlist.cancel")}</AlertDialogClose>
                        <Button
                            variant="destructive"
                            onClick={() => {
                                setDeleting(false);
                                deletePlaylist(id);
                                nav.openLibrary({ kind: "view", view: "albums" });
                            }}
                        >
                            {t("music.playlist.deleteAction")}
                        </Button>
                    </AlertDialogFooter>
                </AlertDialogPopup>
            </AlertDialog>
        </div>
    );
}
