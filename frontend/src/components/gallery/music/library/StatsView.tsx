import type React from "react";
import { useMemo } from "react";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { sizedCover } from "../impl/cover";
import { listeningTime } from "../impl/format";
import { useLibrary } from "../impl/library/store";
import { displayArtists, type ICatalog } from "../impl/model";
import { useMusicNav } from "../impl/nav";
import { playFrom } from "../impl/playback";
import type { messages } from "../MusicGallery.messages";
import { statsOf } from "./stats";

type MusicT = TypedT<typeof messages>;

export function StatsView({ catalog }: { catalog: ICatalog }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const f = useFormatters();
    const nav = useMusicNav();
    const totals = useLibrary((s) => s.totals);
    const firstPlayedAt = useLibrary((s) => s.firstPlayedAt);
    const mode = useLibrary((s) => s.mode);
    const stats = useMemo(() => statsOf(totals, catalog), [totals, catalog]);

    const time = listeningTime(stats.seconds);
    const timeText = time.unit === "seconds" ? t("music.stats.seconds", { seconds: time.seconds }) : time.unit === "minutes" ? t("music.stats.minutes", { minutes: time.minutes }) : t("music.stats.hours", { hours: time.hours, minutes: time.minutes });
    const topTrackList = stats.topTracks.map((r) => r.track);

    return (
        <div className="flex flex-col gap-5">
            {stats.plays === 0 ? (
                <p className="rounded-xl border border-border border-dashed p-10 text-center font-sans text-[13px] text-muted-foreground">{t("music.stats.empty")}</p>
            ) : (
                <>
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
                        <Tile value={f.number(stats.plays)} label={t("music.stats.played")} />
                        <Tile value={timeText} label={t("music.stats.time")} />
                        <Tile value={firstPlayedAt ? f.date(firstPlayedAt) : t("music.stats.never")} label={t("music.stats.since")} />
                    </div>
                    <div className="grid grid-cols-1 gap-5 md:grid-cols-3">
                        <Column title={t("music.stats.topTracks")} empty={stats.topTracks.length === 0}>
                            {stats.topTracks.map(({ track, plays }, i) => (
                                <li key={track.id}>
                                    <button type="button" onClick={() => playFrom(topTrackList, track, { kind: "list", name: t("music.stats.topTracks") })} className="flex min-h-11 w-full cursor-pointer items-center gap-2.5 rounded-md px-1.5 text-left hover:bg-primary/6">
                                        <Rank n={i + 1} />
                                        <span className="size-8 shrink-0 overflow-hidden rounded-[4px] bg-muted">{track.coverUrl ? <img referrerPolicy="no-referrer" src={sizedCover(track.coverUrl, 32)} alt="" loading="lazy" decoding="async" className="size-full object-cover" /> : null}</span>
                                        <span className="flex min-w-0 flex-1 flex-col leading-tight">
                                            <span className="truncate font-medium font-sans text-[13px] text-foreground">{track.title}</span>
                                            <span className="truncate font-sans text-[11.5px] text-muted-foreground">{displayArtists(track, catalog.houseArtist).join(", ") || track.albumName}</span>
                                        </span>
                                        <Count n={plays} />
                                    </button>
                                </li>
                            ))}
                        </Column>
                        <Column title={t("music.stats.topArtists")} empty={stats.topArtists.length === 0} emptyText={t("music.stats.noArtists")}>
                            {stats.topArtists.map(({ name, plays }, i) => (
                                <li key={name}>
                                    <button type="button" onClick={() => nav.openLibrary({ kind: "artist", name })} className="flex min-h-11 w-full cursor-pointer items-center gap-2.5 rounded-md px-1.5 text-left hover:bg-primary/6">
                                        <Rank n={i + 1} />
                                        <span className="min-w-0 flex-1 truncate font-medium font-sans text-[13px] text-foreground">{name}</span>
                                        <Count n={plays} />
                                    </button>
                                </li>
                            ))}
                        </Column>
                        <Column title={t("music.stats.topAlbums")} empty={stats.topAlbums.length === 0}>
                            {stats.topAlbums.map(({ album, plays }, i) => (
                                <li key={album.id}>
                                    <button type="button" onClick={() => nav.openAlbum(album.id)} className="flex min-h-11 w-full cursor-pointer items-center gap-2.5 rounded-md px-1.5 text-left hover:bg-primary/6">
                                        <Rank n={i + 1} />
                                        <img referrerPolicy="no-referrer" src={sizedCover(album.coverUrl, 32)} alt="" loading="lazy" decoding="async" className="size-8 shrink-0 rounded-[4px] bg-muted object-cover" />
                                        <span className="min-w-0 flex-1 truncate font-medium font-sans text-[13px] text-foreground">{album.name}</span>
                                        <Count n={plays} />
                                    </button>
                                </li>
                            ))}
                        </Column>
                    </div>
                </>
            )}
            {mode === "local" ? <p className="m-0 font-sans text-[12px] text-muted-foreground">{t("music.stats.localNote")}</p> : null}
        </div>
    );
}

function Tile({ value, label }: { value: string; label: string }): React.ReactElement {
    return (
        <div className="rounded-xl border border-border bg-card/55 px-4 py-3.5">
            <div className="font-light font-sans text-[28px] text-primary tabular-nums leading-none tracking-[-0.02em]">{value}</div>
            <div className="mt-1.5 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{label}</div>
        </div>
    );
}

function Column({ title, empty, emptyText, children }: { title: string; empty: boolean; emptyText?: string; children: React.ReactNode }): React.ReactElement {
    const t: MusicT = useT("gallery");
    return (
        <section aria-label={title}>
            <h3 className="m-0 mb-2 font-mono font-normal text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{title}</h3>
            {empty ? <p className="m-0 font-sans text-[12.5px] text-muted-foreground">{emptyText ?? t("music.stats.nothing")}</p> : <ol className="m-0 flex list-none flex-col p-0">{children}</ol>}
        </section>
    );
}

function Rank({ n }: { n: number }): React.ReactElement {
    return <span className="w-5 shrink-0 text-right font-mono text-[11px] text-muted-foreground tabular-nums">{n}</span>;
}

function Count({ n }: { n: number }): React.ReactElement {
    const t: MusicT = useT("gallery");
    return <span className="shrink-0 font-mono text-[11.5px] text-primary tabular-nums">{t("music.stats.times", { count: n })}</span>;
}
