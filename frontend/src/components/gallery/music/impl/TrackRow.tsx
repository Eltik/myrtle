import { Link } from "@tanstack/react-router";
import { ChevronDown, DownloadIcon, HeartIcon, PauseIcon, PlayIcon, PlusIcon } from "lucide-react";
import type React from "react";
import { memo, useCallback, useId, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { useAuth } from "#/hooks/use-auth";
import { isGamedataServer } from "#/lib/api/gamedata";
import { GameText } from "#/lib/gamedata/GameText";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { MusicPlace } from "#/types/generated/MusicPlace";
import type { MusicPlaceItem } from "#/types/generated/MusicPlaceItem";
import type { MusicTrack } from "#/types/generated/MusicTrack";
import { AddMenu } from "../library/AddMenu";
import { TrackCount } from "../library/TrackCount";
import type { messages } from "../MusicGallery.messages";
import { useIsCurrent, usePlayer } from "../player/engine";
import { countDownload } from "./download";
import { toggleLike, useIsLiked } from "./library/store";
import type { ITrack } from "./model";
import { clipFileName, displayName } from "./tracks";

type MusicT = TypedT<typeof messages>;

/** Stages or stories a place shows before its "+N more". */
const ITEMS_SHOWN = 8;

/**
 * ONE IN-GAME TRACK: a play button, the name, where it comes from and what it
 * is for, a heart and the Add menu, and a fold with where it plays and the two
 * clips to download. Playback goes through the music engine, which plays the
 * intro once and then loops, like the game. Memoised on the track object,
 * which the catalogue keeps stable, so typing in the search box re-renders
 * only the rows that enter or leave the list.
 */
export const TrackRow = memo(function TrackRow({ track, record, onPlay }: { track: ITrack; record: MusicTrack; onPlay: (track: ITrack) => void }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const [open, setOpen] = useState(false);
    const panelId = useId();
    const name = displayName(record);
    const origin = record.origin ?? t("music.row.shared");
    const current = useIsCurrent(track.id);
    const sounding = usePlayer((s) => s.status === "playing" || s.status === "loading") && current;
    const liked = useIsLiked(track.id);
    const playable = Boolean(record.loopUrl);

    return (
        <li className={cn("rounded-xl border bg-card/55 transition-colors", current ? "border-primary/60" : "border-border")}>
            <div className="flex items-center gap-2.5 px-2.5 py-2 sm:gap-3 sm:px-3">
                <Button size="icon-xl" variant={current ? "default" : "outline"} className="shrink-0 rounded-full" disabled={!playable} onClick={() => onPlay(track)} aria-label={sounding ? t("music.row.pause", { name }) : t("music.row.play", { name })}>
                    {sounding ? <PauseIcon aria-hidden="true" /> : <PlayIcon aria-hidden="true" />}
                </Button>
                <button type="button" className="flex min-h-11 min-w-0 flex-1 cursor-pointer items-center gap-2 text-left" aria-expanded={open} aria-controls={panelId} aria-label={t("music.row.details", { name })} onClick={() => setOpen((o) => !o)}>
                    <span className="flex min-w-0 flex-1 flex-col leading-tight">
                        <span className={cn("truncate", current ? "text-primary" : "text-foreground", record.title ? "font-sans font-semibold text-[13.5px]" : "font-mono text-[12.5px]")}>{record.title ? <GameText text={record.title} /> : record.id}</span>
                        <span className="mt-0.5 truncate font-sans text-[11.5px] text-muted-foreground">
                            <GameText text={origin} />
                            <span aria-hidden="true"> · </span>
                            {record.kinds.map((k) => t(`music.kind.${k}`)).join(", ")}
                        </span>
                    </span>
                    {playable ? null : (
                        <Badge variant="outline" className="shrink-0 text-muted-foreground">
                            {t("music.row.unavailable")}
                        </Badge>
                    )}
                    <ChevronDown aria-hidden="true" className={cn("size-4 shrink-0 text-muted-foreground transition-transform", open && "rotate-180")} />
                </button>
                <TrackCount trackId={track.id} className="max-sm:hidden" />
                <Button size="icon-sm" variant="ghost" onClick={() => toggleLike(track.id)} aria-pressed={liked} aria-label={liked ? t("music.list.unlike", { name }) : t("music.list.likeTrack", { name })} className={cn("max-sm:min-h-11 max-sm:min-w-11", liked && "text-primary")}>
                    <HeartIcon className={cn(liked && "fill-current")} aria-hidden="true" />
                </Button>
                <AddMenu
                    tracks={[track]}
                    trigger={
                        <Button size="icon-sm" variant="ghost" className="max-sm:min-h-11 max-sm:min-w-11" aria-label={t("music.list.addTrack", { name })} title={t("music.list.addTitle")}>
                            <PlusIcon aria-hidden="true" />
                        </Button>
                    }
                />
            </div>
            {open ? <TrackDetails id={panelId} trackId={track.id} track={record} name={name} /> : null}
        </li>
    );
});

function TrackDetails({ id, trackId, track, name }: { id: string; trackId: string; track: MusicTrack; name: string }): React.ReactElement {
    const t: MusicT = useT("gallery");
    return (
        <div id={id} className="border-border border-t px-3 pt-2.5 pb-3 sm:pl-16">
            {track.loopUrl ? (
                <div className="mb-3 flex flex-wrap items-center gap-2">
                    <span className="font-sans text-[11.5px] text-muted-foreground">{t("music.download.label")}</span>
                    {track.introUrl ? <DownloadButton trackId={trackId} url={track.introUrl} server={track.server} label={t("music.download.intro")} aria={t("music.download.introAria", { name })} /> : null}
                    <DownloadButton trackId={trackId} url={track.loopUrl} server={track.server} label={t("music.download.loop")} aria={t("music.download.loopAria", { name })} />
                    {track.server ? <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.06em]">{t("music.row.otherServer", { server: track.server.toUpperCase() })}</span> : null}
                </div>
            ) : (
                <p className="mt-0 mb-3 font-sans text-[12.5px] text-muted-foreground">{t("music.row.unavailableNote")}</p>
            )}

            {track.places.length > 0 ? (
                <section aria-label={t("music.row.places")}>
                    <h3 className="m-0 mb-1.5 font-mono font-normal text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">{t("music.row.places")}</h3>
                    <ul className="m-0 flex list-none flex-col gap-2 p-0">
                        {track.places.map((place, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: places are a fixed list per track, never reordered
                            <PlaceRow key={i} place={place} />
                        ))}
                    </ul>
                </section>
            ) : (
                <p className="mt-0 mb-1.5 font-sans text-[12.5px] text-muted-foreground">{t("music.row.noPlace")}</p>
            )}

            {track.banks.length > 0 ? (
                <details className="mt-2.5">
                    <summary className="cursor-pointer font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.08em]">{t("music.row.banks")}</summary>
                    <p className="mt-1 mb-0 break-all font-mono text-[11px] text-muted-foreground">{track.banks.join(", ")}</p>
                </details>
            ) : null}
        </div>
    );
}

function PlaceRow({ place }: { place: MusicPlace }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const [all, setAll] = useState(false);
    const items = all ? place.items : place.items.slice(0, ITEMS_SHOWN);
    const hidden = place.items.length - items.length;
    return (
        <li className="font-sans text-[12.5px] leading-snug">
            <span className="text-muted-foreground">{t(`music.place.${place.kind}`)}</span>
            {place.name ? (
                <>
                    <span aria-hidden="true" className="text-muted-foreground">
                        {" · "}
                    </span>
                    <span className="text-foreground">
                        <GameText text={place.name} />
                    </span>
                </>
            ) : null}
            {place.role ? <span className="text-muted-foreground">{` (${t(`music.role.${place.role}`)})`}</span> : null}
            {items.length > 0 ? (
                <span className="mt-1 flex flex-wrap gap-1.5">
                    {items.map((item) => (
                        <PlaceItem key={`${item.id ?? ""}\u0000${item.code ?? ""}\u0000${item.name ?? ""}`} kind={place.kind} item={item} />
                    ))}
                    {hidden > 0 ? (
                        <button type="button" className="min-h-7 cursor-pointer rounded-md px-1.5 font-sans text-[11.5px] text-primary hover:underline" onClick={() => setAll(true)}>
                            {t("music.row.more", { count: hidden })}
                        </button>
                    ) : null}
                </span>
            ) : null}
        </li>
    );
}

const CHIP = "inline-flex min-h-7 max-w-full items-center gap-1 rounded-md border border-border bg-secondary/45 px-1.5 text-[11.5px]";

function PlaceItem({ kind, item }: { kind: MusicPlace["kind"]; item: MusicPlaceItem }): React.ReactElement {
    const label = (
        <>
            {item.code ? <span className="font-mono text-[10.5px] text-muted-foreground">{item.code}</span> : null}
            {item.name ? (
                <span className="truncate">
                    <GameText text={item.name} />
                </span>
            ) : null}
        </>
    );
    if (item.id && kind === "stage") {
        return (
            <Link to="/stages/$stageId" params={{ stageId: item.id }} className={cn(CHIP, "text-foreground hover:border-primary/60")}>
                {label}
            </Link>
        );
    }
    if (item.id && kind === "story") {
        return (
            <Link to="/stories/$storyId" params={{ storyId: item.id }} className={cn(CHIP, "text-foreground hover:border-primary/60")}>
                {label}
            </Link>
        );
    }
    return <span className={cn(CHIP, "text-muted-foreground")}>{label}</span>;
}

/**
 * A clip saved under its own file name. The audio sits on the backend's
 * origin, where a plain `download` attribute is ignored, so the clip is
 * fetched and handed over as a blob. It counts as the track's download once
 * the clip answered (the server keeps one a day per listener, so the intro
 * and the loop count once together).
 */
function DownloadButton({ trackId, url, server, label, aria }: { trackId: string; url: string; server?: string; label: string; aria: string }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const signedIn = useAuth().user != null;
    const [state, setState] = useState<"idle" | "busy" | "failed">("idle");
    const onClick = useCallback(async () => {
        setState("busy");
        try {
            const res = await fetch(asset(url, server && isGamedataServer(server) ? server : undefined));
            if (!res.ok) throw new Error(String(res.status));
            countDownload(trackId, signedIn);
            const blob = await res.blob();
            const href = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = href;
            a.download = clipFileName(url);
            document.body.append(a);
            a.click();
            a.remove();
            setTimeout(() => URL.revokeObjectURL(href), 1000);
            setState("idle");
        } catch {
            setState("failed");
        }
    }, [trackId, url, server, signedIn]);
    return (
        <Button size="sm" variant="outline" className="max-sm:min-h-11" disabled={state === "busy"} onClick={onClick} aria-label={aria}>
            <DownloadIcon aria-hidden="true" />
            {state === "failed" ? t("music.download.failed") : label}
        </Button>
    );
}
