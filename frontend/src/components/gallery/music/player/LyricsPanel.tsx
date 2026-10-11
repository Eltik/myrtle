import type React from "react";
import { memo, useEffect, useRef } from "react";
import { Sheet, SheetHeader, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { useT } from "#/lib/i18n";
import { clockTime } from "#/lib/story/reading";
import { cn } from "#/lib/utils";
import { openPanel, seek, usePlayer, usePlayhead } from "./engine";
import { activeLineIndex } from "./lrc";
import type { PlayerT } from "./parts";

/** After the listener scrolls, wheels, touches or keys in the panel, follow stays off this long. */
const FOLLOW_PAUSE_MS = 4000;

/**
 * Distance from the active line to its look: a soft spotlight that fades with
 * distance. Opacity, and a 6% scale on the active line, are all that change
 * and all that transition (compositor-only; the scale does not relayout). The
 * spotlight used to grow the active line to 1.28em and shrink the far ones
 * with a font-size transition, which relaid out the whole list on every frame
 * of every line change and shifted the scroll target under the smooth scroll.
 */
function lineClass(distance: number): string {
    const d = Math.min(4, Math.abs(distance));
    if (d === 0) return "scale-106 font-semibold opacity-100";
    if (d === 1) return "opacity-55";
    if (d === 2) return "opacity-30";
    if (d === 3) return "opacity-20";
    return "opacity-[.14]";
}

export function LyricsPanel(): React.ReactElement {
    const t: PlayerT = useT("gallery");
    const open = usePlayer((s) => s.panel === "lyrics");
    return (
        <Sheet onOpenChange={(o) => openPanel(o ? "lyrics" : null)} open={open}>
            <SheetPopup className="max-w-100" side="right">
                <SheetHeader className="pb-2">
                    <SheetTitle>{t("music.lyrics.title")}</SheetTitle>
                </SheetHeader>
                <LyricsBody t={t} />
            </SheetPopup>
        </Sheet>
    );
}

function LyricsBody({ t }: { t: PlayerT }): React.ReactElement {
    const current = usePlayer((s) => s.current);
    const lyrics = usePlayer((s) => s.lyrics);
    const status = usePlayer((s) => s.status);
    const { position } = usePlayhead();
    const scroller = useRef<HTMLDivElement>(null);
    const lastUserAt = useRef(0);
    const lines = lyrics.trackId === current?.id ? lyrics.lines : [];
    const active = activeLineIndex(lines, position);
    const playing = status === "playing";

    useEffect(() => {
        if (!playing || active < 0 || Date.now() - lastUserAt.current < FOLLOW_PAUSE_MS) return;
        const el = scroller.current?.querySelector<HTMLElement>(`[data-line="${active}"]`);
        const reduce = typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
        el?.scrollIntoView?.({ block: "center", behavior: reduce ? "auto" : "smooth" });
    }, [active, playing]);

    const touched = () => {
        lastUserAt.current = Date.now();
    };

    let message: string | null = null;
    if (!current) message = t("music.lyrics.nothing");
    else if (lyrics.trackId !== current.id || lyrics.status === "none") message = lyrics.status === "loading" ? t("music.lyrics.loading") : t("music.lyrics.none");
    else if (lyrics.status === "loading") message = t("music.lyrics.loading");
    else if (lyrics.status === "empty") message = t("music.lyrics.empty");
    else if (lyrics.status === "failed") message = t("music.lyrics.failed");

    if (message !== null) return <p className="px-6 pb-6 text-muted-foreground text-sm">{message}</p>;

    return (
        // biome-ignore lint/a11y/noStaticElementInteractions: passive listeners that only pause auto-follow; they activate nothing.
        <div className="min-h-0 flex-1 overflow-y-auto px-6 pb-[40vh]" onKeyDown={touched} onPointerDown={touched} onTouchStart={touched} onWheel={touched} ref={scroller}>
            <ol className="flex flex-col pt-[20vh] text-sm leading-loose">
                {lines.map((line, i) => (
                    // Lines can share a time and a text (a repeated chorus), so the index is part of the key.
                    // biome-ignore lint/suspicious/noArrayIndexKey: lines are static per track and never reorder.
                    <LyricLine key={`${line.time}-${i}`} index={i} time={line.time} text={line.text} distance={Math.min(4, Math.abs(i - active))} t={t} />
                ))}
            </ol>
        </div>
    );
}

/** One line. Memoised on its clamped distance, so a playhead tick re-renders nothing and a line change re-renders the few lines whose look changes. */
const LyricLine = memo(function LyricLine({ index, time, text, distance, t }: { index: number; time: number; text: string; distance: number; t: PlayerT }): React.ReactElement {
    return (
        <li>
            <button
                aria-current={distance === 0 ? "true" : undefined}
                aria-label={t("music.lyrics.jump", { time: clockTime(time), line: text })}
                className={cn("pointer-coarse:min-h-11 w-full origin-left cursor-pointer rounded-md text-left text-foreground outline-none transition-[opacity,scale] duration-200 ease-out hover:opacity-90 focus-visible:ring-2 focus-visible:ring-ring/40 motion-reduce:transition-none", lineClass(distance))}
                data-line={index}
                onClick={() => seek(time)}
                type="button"
            >
                {text}
            </button>
        </li>
    );
});
