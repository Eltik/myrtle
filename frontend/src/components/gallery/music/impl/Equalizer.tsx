import type React from "react";
import { cn } from "#/lib/utils";

/**
 * The music page's one perpetual animation, defined once: the vinyl spin,
 * which runs only while the open album plays. React 19 hoists a `<style>`
 * with `href` and `precedence` into the head and dedupes it, so every mount
 * shares one sheet. The equaliser bars used to bounce forever in every
 * playing row; they are a still glyph now.
 */
const CSS = `
@keyframes music-spin { to { transform: rotate(360deg); } }
.music-spin { animation: music-spin 6s linear infinite; }
@media (prefers-reduced-motion: reduce) { .music-spin { animation: none; } }
`;

export function MusicKeyframes(): React.ReactElement {
    return (
        <style href="myrtle-music-keyframes" precedence="default">
            {CSS}
        </style>
    );
}

/** Three still bars at mixed heights: the "now playing" glyph of a row. */
export function Equalizer({ className }: { className?: string }): React.ReactElement {
    return (
        <span aria-hidden="true" className={cn("inline-flex h-3 items-end gap-0.5", className)}>
            {[0.55, 1, 0.75].map((h) => (
                <span key={h} className="w-0.75 rounded-[1px] bg-primary" style={{ height: `${h * 100}%` }} />
            ))}
        </span>
    );
}
