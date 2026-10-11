import type React from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { displayArtists, type ITrack } from "../impl/model";
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
