import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useStore } from "@tanstack/react-store";
import { Store } from "@tanstack/store";
import { Disc3Icon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogDescription, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { musicListenersQueryOptions } from "#/lib/api/music";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { MusicListenerSort } from "#/types/generated/MusicListenerSort";
import type { MusicStatWindow } from "#/types/generated/MusicStatWindow";
import { sizedCover } from "../impl/cover";
import { displayArtists, type ITrack } from "../impl/model";
import type { messages } from "../MusicGallery.messages";
import { ListenerRows, ListenerSortTabs, RankingBody, StatWindowTabs } from "./ListenerList";
import { useTrackCounts } from "./TrackCount";

type MusicT = TypedT<typeof messages>;

/** Rows a page of listeners adds, and the most the backend hands out at once. */
export const LISTENER_PAGE = 20;
export const LISTENER_MAX = 100;

const openTrack = new Store<{ open: boolean; track: ITrack | null; houseArtist: string | null }>({ open: false, track: null, houseArtist: null });

/** Open one track's top listeners: from a row's menu, or from a top-tracks row. */
export function openTrackListeners(track: ITrack, houseArtist: string | null): void {
    openTrack.setState(() => ({ open: true, track, houseArtist }));
}

/**
 * ONE TRACK'S TOP LISTENERS, a dialog the music page mounts once. Closing
 * keeps the track, so its title does not blank while the dialog animates out.
 */
export function TrackListenersDialog(): React.ReactElement {
    const { open, track, houseArtist } = useStore(openTrack, (s) => s);
    return (
        <Dialog open={open} onOpenChange={(next) => openTrack.setState((s) => ({ ...s, open: next }))}>
            <DialogPopup className="sm:max-w-md">{track ? <Body key={track.id} track={track} houseArtist={houseArtist} /> : null}</DialogPopup>
        </Dialog>
    );
}

function Body({ track, houseArtist }: { track: ITrack; houseArtist: string | null }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const [span, setSpan] = useState<MusicStatWindow>("all");
    const [sort, setSort] = useState<MusicListenerSort>("listens");
    const [limit, setLimit] = useState(LISTENER_PAGE);
    const q = useQuery({ ...musicListenersQueryOptions(track.id, span, sort, limit), placeholderData: keepPreviousData });
    const counts = useTrackCounts(track.id);
    const sub = displayArtists(track, houseArtist).join(", ") || track.albumName || "";

    return (
        <>
            <DialogHeader>
                <div className="flex min-w-0 items-center gap-3 pr-8">
                    <span className="relative size-12 shrink-0 overflow-hidden rounded-md border border-border bg-muted">
                        {track.coverUrl ? <img referrerPolicy="no-referrer" src={sizedCover(track.coverUrl, 48)} alt="" decoding="async" className="size-full object-cover" /> : <Disc3Icon aria-hidden="true" className="absolute inset-0 m-auto size-5 text-muted-foreground" />}
                    </span>
                    <span className="flex min-w-0 flex-col gap-0.5">
                        <span className="font-mono text-[10px] text-primary/80 uppercase tracking-[0.14em]">{t("music.charts.trackKicker")}</span>
                        <DialogTitle className="truncate text-base" title={track.title}>
                            {track.title}
                        </DialogTitle>
                        {sub ? (
                            <span className="truncate font-sans text-[12px] text-muted-foreground" title={sub}>
                                {sub}
                            </span>
                        ) : null}
                    </span>
                </div>
                {counts.listens + counts.downloads > 0 ? <DialogDescription>{t("music.charts.trackTotals", { listens: counts.listens, downloads: counts.downloads })}</DialogDescription> : null}
            </DialogHeader>
            <DialogPanel>
                <div className="mb-3 flex flex-wrap items-center gap-2">
                    <StatWindowTabs
                        size="sm"
                        value={span}
                        onChange={(w) => {
                            setSpan(w);
                            setLimit(LISTENER_PAGE);
                        }}
                    />
                    <ListenerSortTabs
                        value={sort}
                        onChange={(s) => {
                            setSort(s);
                            setLimit(LISTENER_PAGE);
                        }}
                    />
                </div>
                <RankingBody query={q} isEmpty={(page) => page.entries.length === 0} emptyText={t("music.charts.trackNobody")} skeletonRows={5}>
                    {(page) => (
                        <>
                            <ListenerRows entries={page.entries} sort={sort} />
                            {page.entries.length < page.total && limit < LISTENER_MAX ? (
                                <div className="mt-3 flex justify-center">
                                    <Button variant="outline" size="sm" className="max-sm:min-h-11" disabled={q.isFetching} onClick={() => setLimit((l) => Math.min(LISTENER_MAX, l + LISTENER_PAGE))}>
                                        {t("music.charts.more")}
                                    </Button>
                                </div>
                            ) : null}
                        </>
                    )}
                </RankingBody>
                <p className="mt-3 mb-0 font-sans text-[11.5px] text-muted-foreground">{t("music.charts.publicNote")}</p>
            </DialogPanel>
        </>
    );
}
