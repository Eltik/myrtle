import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { Gamepad2Icon, PlayIcon, UsersIcon } from "lucide-react";
import type React from "react";
import { memo, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { musicListenersQueryOptions, musicTopTracksQueryOptions } from "#/lib/api/music";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { MusicListenerSort } from "#/types/generated/MusicListenerSort";
import type { MusicStatWindow } from "#/types/generated/MusicStatWindow";
import { sizedCover } from "../impl/cover";
import { Equalizer } from "../impl/Equalizer";
import { displayArtists, type ICatalog, type ITrack, parseTrackId } from "../impl/model";
import { playFrom } from "../impl/playback";
import type { messages } from "../MusicGallery.messages";
import { useIsCurrent, usePlayer } from "../player/engine";
import type { IQueueSource } from "../player/types";
import { ListenerRows, ListenerSortTabs, RankingBody, StatWindowTabs } from "./ListenerList";
import { LISTENER_MAX, LISTENER_PAGE, openTrackListeners } from "./TrackListenersDialog";

type MusicT = TypedT<typeof messages>;

/** How many top tracks the backend is asked for, and how many show before "Show more". */
const TOP_TRACKS = 50;
const TOP_TRACKS_SHOWN = 20;

/**
 * THE CHARTS VIEW: what the whole community plays. Top tracks over a span
 * (every counted listen, signed in or not; a row plays the chart from there)
 * and the top signed-in listeners over the same span, by plays or by time
 * heard. Each top track opens its own listeners.
 */
export function ChartsView({ catalog, waiting }: { catalog: ICatalog; waiting: IChartsWaiting }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const [span, setSpan] = useState<MusicStatWindow>("week");
    return (
        <div className="flex flex-col gap-4">
            <StatWindowTabs value={span} onChange={setSpan} />
            <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
                <TopTracks span={span} catalog={catalog} waiting={waiting} />
                <TopListeners span={span} />
            </div>
            <p className="m-0 font-sans text-[12px] text-muted-foreground">{t("music.charts.rules")}</p>
        </div>
    );
}

function SectionTitle({ children, aside }: { children: React.ReactNode; aside?: React.ReactNode }): React.ReactElement {
    return (
        <div className="mb-2 flex min-h-8 flex-wrap items-center justify-between gap-2">
            <h3 className="m-0 font-mono font-normal text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{children}</h3>
            {aside}
        </div>
    );
}

/** Which collections are still on their way; a top track from one of them shows as a placeholder until it lands. */
export interface IChartsWaiting {
    siren: boolean;
    game: boolean;
}

interface IChartRow {
    rank: number;
    trackId: string;
    listens: number;
    track: ITrack | null;
}

function TopTracks({ span, catalog, waiting }: { span: MusicStatWindow; catalog: ICatalog; waiting: IChartsWaiting }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const q = useQuery({ ...musicTopTracksQueryOptions(span, TOP_TRACKS), placeholderData: keepPreviousData });
    const [shown, setShown] = useState(TOP_TRACKS_SHOWN);
    const title = t("music.charts.topTracks");

    // A track the catalogue does not hold is a placeholder while its collection loads, and is left
    // out once it loaded without it (or failed to load: there is nothing to name or play it by).
    const rows = useMemo((): IChartRow[] => {
        const out: IChartRow[] = [];
        for (const e of q.data?.entries ?? []) {
            const track = catalog.byId.get(e.trackId) ?? null;
            const kind = parseTrackId(e.trackId)?.kind;
            const pending = (kind === "siren" && waiting.siren) || (kind === "game" && waiting.game);
            if (track || pending) out.push({ rank: e.rank, trackId: e.trackId, listens: e.listens, track });
        }
        return out;
    }, [q.data, catalog, waiting.siren, waiting.game]);
    const tracks = useMemo(() => rows.flatMap((r) => (r.track ? [r.track] : [])), [rows]);
    const source: IQueueSource = { kind: "list", name: title };

    return (
        <section aria-label={title}>
            <SectionTitle>{title}</SectionTitle>
            <RankingBody query={q} isEmpty={() => rows.length === 0} emptyText={t("music.charts.noTracks")}>
                {() => (
                    <>
                        <ol className="m-0 flex list-none flex-col p-0">
                            {rows.slice(0, shown).map((row) => (
                                <ChartTrack key={row.trackId} row={row} tracks={tracks} source={source} houseArtist={catalog.houseArtist} />
                            ))}
                        </ol>
                        {rows.length > shown ? (
                            <div className="mt-3 flex justify-center">
                                <Button variant="outline" size="sm" className="max-sm:min-h-11" onClick={() => setShown(rows.length)}>
                                    {t("music.charts.more")}
                                </Button>
                            </div>
                        ) : null}
                    </>
                )}
            </RankingBody>
        </section>
    );
}

const ChartTrack = memo(function ChartTrack({ row, tracks, source, houseArtist }: { row: IChartRow; tracks: readonly ITrack[]; source: IQueueSource; houseArtist: string | null }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const f = useFormatters();
    const { track } = row;
    const current = useIsCurrent(row.trackId);
    const playing = usePlayer((s) => s.status === "playing" || s.status === "loading") && current;
    const listens = t("music.charts.listens", { count: row.listens });
    if (!track) {
        return (
            <li className="flex min-h-12 items-center gap-2.5 px-1.5" aria-hidden="true">
                <span className="w-6 shrink-0 text-right font-mono text-[11px] text-muted-foreground tabular-nums">{row.rank}</span>
                <span className="size-8 shrink-0 animate-pulse rounded-[4px] bg-muted" />
                <span className="h-3 w-40 animate-pulse rounded bg-muted" />
            </li>
        );
    }
    const name = track.title;
    const sub = displayArtists(track, houseArtist).join(", ") || track.albumName || "";
    return (
        <li className={cn("group/chart flex items-center gap-1 rounded-md pr-1 transition-colors hover:bg-primary/6", current && "bg-primary/10 shadow-[inset_2px_0_0_var(--color-primary)]")}>
            <button
                type="button"
                onClick={() => playFrom(tracks, track, source)}
                aria-label={t("music.charts.trackAria", { rank: row.rank, name, listens })}
                className="flex min-h-12 min-w-0 flex-1 cursor-pointer items-center gap-2.5 rounded-md px-1.5 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
            >
                <span className="flex w-6 shrink-0 items-center justify-end font-mono text-[11px] text-muted-foreground tabular-nums">
                    {current ? (
                        playing ? (
                            <Equalizer />
                        ) : (
                            <PlayIcon className="size-3.5 text-primary" aria-hidden="true" />
                        )
                    ) : (
                        <>
                            <span className="group-hover/chart:hidden">{row.rank}</span>
                            <PlayIcon className="hidden size-3.5 text-foreground group-hover/chart:block" aria-hidden="true" />
                        </>
                    )}
                </span>
                <span className="relative size-8 shrink-0 overflow-hidden rounded-[4px] bg-muted">
                    {track.coverUrl ? <img referrerPolicy="no-referrer" src={sizedCover(track.coverUrl, 32)} alt="" loading="lazy" decoding="async" className="size-full object-cover" /> : <Gamepad2Icon className="absolute inset-0 m-auto size-4 text-muted-foreground" aria-hidden="true" />}
                </span>
                <span className="flex min-w-0 flex-1 flex-col leading-tight">
                    <span className={cn("truncate font-medium font-sans text-[13px]", current ? "text-primary" : "text-foreground")} title={name}>
                        {name}
                    </span>
                    {sub ? (
                        <span className="truncate font-sans text-[11.5px] text-muted-foreground" title={sub}>
                            {sub}
                        </span>
                    ) : null}
                </span>
                <span className="shrink-0 font-mono text-[11.5px] text-primary tabular-nums" title={listens}>
                    {f.compact(row.listens)}
                </span>
            </button>
            <Tooltip>
                <TooltipTrigger render={<Button size="icon-sm" variant="ghost" className="max-sm:min-h-11 max-sm:min-w-11" onClick={() => openTrackListeners(track, houseArtist)} aria-label={t("music.charts.trackListenersOf", { name })} />}>
                    <UsersIcon aria-hidden="true" />
                </TooltipTrigger>
                <TooltipPopup>{t("music.charts.trackListeners")}</TooltipPopup>
            </Tooltip>
        </li>
    );
});

function TopListeners({ span }: { span: MusicStatWindow }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const [sort, setSort] = useState<MusicListenerSort>("listens");
    const [limit, setLimit] = useState<{ key: string; n: number }>({ key: "", n: LISTENER_PAGE });
    // The page size starts over whenever the span or the sort changes.
    const key = `${span}:${sort}`;
    const n = limit.key === key ? limit.n : LISTENER_PAGE;
    const q = useQuery({ ...musicListenersQueryOptions(null, span, sort, n), placeholderData: keepPreviousData });
    const title = t("music.charts.topListeners");

    return (
        <section aria-label={title}>
            <SectionTitle aside={<ListenerSortTabs value={sort} onChange={setSort} />}>{title}</SectionTitle>
            <RankingBody query={q} isEmpty={(page) => page.entries.length === 0} emptyText={t("music.charts.nobody")}>
                {(page) => (
                    <>
                        <ListenerRows entries={page.entries} sort={sort} />
                        <div className="mt-3 flex flex-wrap items-center justify-between gap-2">
                            <span className="font-sans text-[11.5px] text-muted-foreground">{t("music.charts.ranked", { count: page.total })}</span>
                            {page.entries.length < page.total && n < LISTENER_MAX ? (
                                <Button variant="outline" size="sm" className="max-sm:min-h-11" disabled={q.isFetching} onClick={() => setLimit({ key, n: Math.min(LISTENER_MAX, n + LISTENER_PAGE) })}>
                                    {t("music.charts.more")}
                                </Button>
                            ) : null}
                        </div>
                    </>
                )}
            </RankingBody>
            <p className="mt-2 mb-0 font-sans text-[11.5px] text-muted-foreground">{t("music.charts.publicNote")}</p>
        </section>
    );
}
