import type React from "react";
import { useCallback, useMemo } from "react";
import { Skeleton } from "#/components/ui/skeleton";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { gameTrackId, type ICatalog, type ITrack } from "../impl/model";
import { playFrom } from "../impl/playback";
import { TrackRow } from "../impl/TrackRow";
import { filterTracks, type MusicKindFilter, prepareMusicSearch } from "../impl/tracks";
import type { messages } from "../MusicGallery.messages";
import { NOTICE_CLASS, ShowMore, usePagedList } from "./paging";

type MusicT = TypedT<typeof messages>;

/** Placeholder rows while the index loads. */
const SKELETON_ROWS = 8;

export interface IGameIndexState {
    loading: boolean;
    failed: boolean;
    /** The backend predates the route. */
    unavailable: boolean;
}

/**
 * "From the game": every track the game names, searched by title, event or
 * place and filtered by what it plays over. Each row plays through the music
 * engine with the visible list as the queue, and can be liked or put in a
 * playlist like any label track.
 */
export function GameTracks({ catalog, query, kind, collator, state }: { catalog: ICatalog; query: string; kind: MusicKindFilter; collator: Intl.Collator; state: IGameIndexState }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const records = catalog.game?.tracks;
    const search = useMemo(() => prepareMusicSearch(records ?? []), [records]);
    const visible = useMemo(() => {
        const out: ITrack[] = [];
        for (const r of filterTracks(search, query, kind, collator)) {
            const tr = catalog.byId.get(gameTrackId(r.id));
            if (tr) out.push(tr);
        }
        return out;
    }, [search, query, kind, collator, catalog]);

    const sourceName = t("music.rail.game");
    const onPlay = useCallback((track: ITrack) => playFrom(visible, track, { kind: "game", name: sourceName }), [visible, sourceName]);

    // The page limit is remembered against the controls it was raised under,
    // so a new search starts from one page without an effect.
    const pageKey = `${query}\u0000${kind}`;
    const page = usePagedList(visible, pageKey);

    if (state.unavailable) return <div className={NOTICE_CLASS}>{t("music.unavailableRoute")}</div>;
    if (state.failed) return <p className="py-10 text-center font-sans text-[13px] text-muted-foreground">{t("music.failed")}</p>;
    if (state.loading || !records) {
        return (
            <output className="flex flex-col gap-2" aria-label={t("music.loading")}>
                {Array.from({ length: SKELETON_ROWS }, (_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: a placeholder has no identity beyond its position
                    <Skeleton key={i} className="h-15 w-full rounded-xl" />
                ))}
            </output>
        );
    }

    return (
        <div>
            <p className="mt-0 mb-3 font-sans text-[12px] text-muted-foreground">
                {t("music.results", { count: visible.length })}
                <span aria-hidden="true"> · </span>
                {t("music.loopNote")}
            </p>
            {visible.length === 0 ? (
                <div className="rounded-[14px] border border-border border-dashed p-14 text-center font-sans text-[14px] text-muted-foreground">{t("music.empty")}</div>
            ) : (
                <ul className="m-0 flex list-none flex-col gap-2 p-0">{page.shown.map((track) => (track.game ? <TrackRow key={track.id} track={track} record={track.game} onPlay={onPlay} /> : null))}</ul>
            )}
            <ShowMore page={page} />
        </div>
    );
}
