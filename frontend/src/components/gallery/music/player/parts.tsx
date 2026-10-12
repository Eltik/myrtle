import { DownloadIcon, Repeat1Icon, RepeatIcon, ShuffleIcon } from "lucide-react";
import type React from "react";
import { Spinner } from "#/components/ui/spinner";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { isDownloadable, useDownload, useTrackDownload } from "../impl/download";
import { displayArtists, type ITrack } from "../impl/model";
import { cycleRepeat, toggleShuffle, usePlayer } from "./engine";
import type { messages } from "./Player.messages";
import type { IQueueSource } from "./types";

export type PlayerT = TypedT<typeof messages>;

/** The "Playing from ..." line for a queue source; a source without a name reads as "Now playing". */
export function useKicker(source: IQueueSource): string {
    const t: PlayerT = useT("gallery");
    const name = source.name?.trim();
    if (!name || source.kind === "none") return t("music.player.kicker.none");
    switch (source.kind) {
        case "album":
            return t("music.player.kicker.album", { name });
        case "playlist":
            return t("music.player.kicker.playlist", { name });
        case "artist":
            return t("music.player.kicker.artist", { name });
        case "search":
            return t("music.player.kicker.search", { name });
        case "radio":
            return t("music.player.kicker.radio", { name });
        case "game":
            // The source name is already "From the game"; a second "from" would read twice.
            return t("music.player.kicker.game");
        default:
            return t("music.player.kicker.other", { name });
    }
}

/** The line under a title: the credited guests (else the house name), or the album or origin when nothing is credited. */
export function subLine(track: ITrack, houseArtist: string | null): string {
    const artists = displayArtists(track, houseArtist);
    if (artists.length > 0) return artists.join(", ");
    return track.albumName ?? "";
}

/** One line that clips with an ellipsis and carries its full text as a hover hint. */
export function ClippedText({ text, className }: { text: string; className: string }): React.ReactElement {
    return (
        <span className={cn("truncate", className)} title={text}>
            {text}
        </span>
    );
}

/**
 * A round icon button with a tooltip naming it. The name is also the
 * accessible name, so the tooltip and a screen reader say the same thing.
 */
export function IconButton({ label, onClick, children, pressed, className, disabled, tooltip = true }: { label: string; onClick: () => void; children: React.ReactNode; pressed?: boolean; className?: string; disabled?: boolean; tooltip?: boolean }): React.ReactElement {
    const button = (props: React.ComponentProps<"button">) => (
        <button
            {...props}
            aria-label={label}
            aria-pressed={pressed}
            className={cn(
                "relative inline-flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-full text-muted-foreground outline-none transition-colors pointer-coarse:after:absolute pointer-coarse:after:size-11 hover:bg-secondary/60 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/40 disabled:pointer-events-none disabled:opacity-50 max-sm:min-h-11 max-sm:min-w-11 [&_svg]:size-4",
                pressed && "text-primary hover:text-primary",
                className,
            )}
            disabled={disabled}
            onClick={onClick}
            type="button"
        >
            {children}
        </button>
    );
    if (!tooltip) return button({});
    return (
        <Tooltip>
            <TooltipTrigger render={(p) => button(p as React.ComponentProps<"button">)} />
            <TooltipPopup>{label}</TooltipPopup>
        </Tooltip>
    );
}

/** Save the playing track; while it downloads, a spinner and the progress in its name. */
export function DownloadCurrent({ track, t, className, tooltip }: { track: ITrack; t: PlayerT; className?: string; tooltip?: boolean }): React.ReactElement {
    const f = useFormatters();
    const download = useTrackDownload();
    const running = useDownload(track.id);
    const label = running ? (running.total ? t("music.player.downloadingPercent", { percent: f.percent(running.received / running.total) }) : t("music.player.downloading")) : t("music.player.download");
    return (
        <IconButton className={className} label={label} onClick={() => download(track)} disabled={running !== null || !isDownloadable(track)} tooltip={tooltip}>
            {running ? <Spinner aria-hidden="true" /> : <DownloadIcon aria-hidden="true" />}
        </IconButton>
    );
}

/** Shuffle on and off; the bar and the queue sheet both carry it. */
export function ShuffleButton({ t, className, tooltip }: { t: PlayerT; className?: string; tooltip?: boolean }): React.ReactElement {
    const shuffle = usePlayer((s) => s.shuffle);
    return (
        <IconButton className={className} label={shuffle ? t("music.player.shuffle.on") : t("music.player.shuffle.off")} onClick={toggleShuffle} pressed={shuffle} tooltip={tooltip}>
            <ShuffleIcon aria-hidden="true" />
        </IconButton>
    );
}

/** Repeat off, all, one; the bar and the queue sheet both carry it. */
export function RepeatButton({ t, className, tooltip }: { t: PlayerT; className?: string; tooltip?: boolean }): React.ReactElement {
    const repeat = usePlayer((s) => s.repeat);
    const label = repeat === "one" ? t("music.player.repeat.one") : repeat === "all" ? t("music.player.repeat.all") : t("music.player.repeat.off");
    return (
        <IconButton className={className} label={label} onClick={cycleRepeat} pressed={repeat !== "off"} tooltip={tooltip}>
            {repeat === "one" ? <Repeat1Icon aria-hidden="true" /> : <RepeatIcon aria-hidden="true" />}
        </IconButton>
    );
}
