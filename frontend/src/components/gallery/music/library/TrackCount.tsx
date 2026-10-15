import { useQuery } from "@tanstack/react-query";
import { HeadphonesIcon } from "lucide-react";
import type React from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { musicStatsQueryOptions } from "#/lib/api/music";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { MusicStatsCatalog } from "#/types/generated/MusicStatsCatalog";
import type { MusicTrackCounts } from "#/types/generated/MusicTrackCounts";
import type { messages } from "../MusicGallery.messages";

type MusicT = TypedT<typeof messages>;

const NONE: MusicTrackCounts = { listens: 0, downloads: 0 };

/**
 * One track's community counts from the shared `/music/stats` query: every
 * row reads the same cached catalogue, fetched once by the browser. Zero
 * while it loads, when it failed, and for a track nobody played.
 */
export function useTrackCounts(trackId: string): MusicTrackCounts {
    const q = useQuery({ ...musicStatsQueryOptions(), select: (data: MusicStatsCatalog | null) => data?.tracks[trackId] ?? NONE });
    return q.data ?? NONE;
}

/**
 * A track's listens as a compact number (`1.2K`), the exact listens and
 * downloads in its tooltip. The box keeps its width when there is nothing to
 * show, so rows never move when the counts arrive.
 */
export function TrackCount({ trackId, className }: { trackId: string; className?: string }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const f = useFormatters();
    const counts = useTrackCounts(trackId);
    const box = cn("inline-flex w-14 shrink-0 items-center justify-end gap-1 whitespace-nowrap font-mono text-[11px] text-muted-foreground tabular-nums", className);
    if (counts.listens === 0 && counts.downloads === 0) return <span className={box} aria-hidden="true" />;
    const exact = t("music.counts.exact", { listens: counts.listens, downloads: counts.downloads });
    return (
        <Tooltip>
            <TooltipTrigger render={<span className={box} />}>
                <HeadphonesIcon className="size-3 opacity-70" aria-hidden="true" />
                <span aria-hidden="true">{f.compact(counts.listens)}</span>
                <span className="sr-only">{exact}</span>
            </TooltipTrigger>
            <TooltipPopup>{exact}</TooltipPopup>
        </Tooltip>
    );
}
