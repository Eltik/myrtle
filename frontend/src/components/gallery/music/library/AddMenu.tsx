import { CheckIcon, DownloadIcon, ListEndIcon, ListPlusIcon, ListStartIcon, PlusIcon, UsersIcon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Menu, MenuItem, MenuPopup, MenuSeparator, MenuTrigger } from "#/components/ui/menu";
import { toastManager } from "#/components/ui/toast";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { isDownloadable, useDownload, useTrackDownload } from "../impl/download";
import { addToPlaylist, createPlaylist, useLibrary } from "../impl/library/store";
import type { ITrack } from "../impl/model";
import type { messages } from "../MusicGallery.messages";
import { enqueueEnd, enqueueNext, usePlayer } from "../player/engine";
import { PlaylistNameDialog } from "./PlaylistNameDialog";
import { openTrackListeners } from "./TrackListenersDialog";

type MusicT = TypedT<typeof messages>;

/**
 * THE ADD MENU every row, the bulk bar and the album modal share: play next,
 * add to the queue, add to a playlist (checked when every track is already
 * in it), or a new playlist with the tracks in it. With several tracks each
 * item says `(all N)`. For one track it also saves the file and opens the
 * track's top listeners.
 */
export function AddMenu({ tracks, trigger, align = "end" }: { tracks: readonly ITrack[]; trigger: React.ReactElement; align?: "start" | "center" | "end" }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const playlists = useLibrary((s) => s.playlists);
    const playlistLimit = useLibrary((s) => s.playlistLimit);
    const [naming, setNaming] = useState(false);
    const ids = tracks.map((tr) => tr.id);
    const many = tracks.length > 1;
    const all = (label: string) => (many ? t("music.menu.all", { label, count: tracks.length }) : label);

    const addTo = (id: string, name: string) => {
        const added = addToPlaylist(id, ids);
        toastManager.add({ title: added > 0 ? t("music.menu.added", { count: added, name }) : t("music.menu.alreadyIn", { name }), type: added > 0 ? "success" : "info" });
    };

    return (
        <>
            <Menu>
                <MenuTrigger render={trigger} />
                <MenuPopup align={align} className="max-w-80">
                    <MenuItem disabled={tracks.length === 0} onClick={() => enqueueNext(tracks)}>
                        <ListStartIcon />
                        {all(t("music.menu.playNext"))}
                    </MenuItem>
                    <MenuItem disabled={tracks.length === 0} onClick={() => enqueueEnd(tracks)}>
                        <ListEndIcon />
                        {all(t("music.menu.addToQueue"))}
                    </MenuItem>
                    <MenuSeparator />
                    {playlists.map((pl) => {
                        const inIt = ids.length > 0 && ids.every((id) => pl.trackIds.includes(id));
                        return (
                            <MenuItem key={pl.id} disabled={inIt || ids.length === 0} onClick={() => addTo(pl.id, pl.name)}>
                                {inIt ? <CheckIcon /> : <ListPlusIcon />}
                                <span className="min-w-0 truncate">{inIt ? t("music.menu.addToAdded", { name: pl.name }) : all(t("music.menu.addTo", { name: pl.name }))}</span>
                            </MenuItem>
                        );
                    })}
                    <MenuItem disabled={playlists.length >= playlistLimit} onClick={() => setNaming(true)}>
                        <PlusIcon />
                        {t("music.menu.newPlaylist")}
                    </MenuItem>
                    {tracks.length === 1 && tracks[0] ? <OneTrackItems track={tracks[0]} /> : null}
                </MenuPopup>
            </Menu>
            <PlaylistNameDialog
                open={naming}
                onOpenChange={setNaming}
                mode="create"
                onSubmit={(name) => {
                    void createPlaylist(name, ids).then((pl) => {
                        if (pl) toastManager.add({ title: t("music.menu.created", { name: pl.name, count: pl.trackIds.length }), type: "success" });
                        else toastManager.add({ title: t("music.playlist.createFailed"), type: "error" });
                    });
                }}
            />
        </>
    );
}

/** Download and Top listeners, the items that only make sense for one track. */
function OneTrackItems({ track }: { track: ITrack }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const f = useFormatters();
    const download = useTrackDownload();
    const running = useDownload(track.id);
    const houseArtist = usePlayer((s) => s.houseArtist);
    return (
        <>
            <MenuSeparator />
            <MenuItem disabled={!isDownloadable(track) || running !== null} onClick={() => download(track)}>
                <DownloadIcon />
                {running ? (running.total ? t("music.menu.downloadingPercent", { percent: f.percent(running.received / running.total) }) : t("music.menu.downloading")) : t("music.menu.download")}
            </MenuItem>
            <MenuItem onClick={() => openTrackListeners(track, houseArtist)}>
                <UsersIcon />
                {t("music.menu.listeners")}
            </MenuItem>
        </>
    );
}
