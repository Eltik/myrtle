import type { ILyricLine } from "./types";

/** One `[mm:ss]`, `[mm:ss.x]`, `[mm:ss.xx]` or `[mm:ss.xxx]` tag. */
const TIME_TAG = /\[(\d{1,3}):(\d{1,2})(?:[.:](\d{1,3}))?\]/g;

/**
 * An LRC file as timed lines, sorted by time.
 *
 * A line may carry several time tags (`[00:12.00][01:30.50]chorus`), and each
 * becomes its own entry. Metadata tags (`[ar:...]`, `[offset:...]`) carry no
 * time and are dropped with every other line that has no time tag, and so is
 * a timed line whose text is blank, which LRC files use to clear the screen.
 * Credit lines (`[00:00.00]作曲 Composer: ...`) are kept: they are real text
 * the label ships at a real time.
 */
export function parseLrc(text: string): ILyricLine[] {
    const out: ILyricLine[] = [];
    for (const raw of text.split(/\r?\n|\r/)) {
        TIME_TAG.lastIndex = 0;
        const times: number[] = [];
        let end = 0;
        let match = TIME_TAG.exec(raw);
        // Tags only count at the start of the line, back to back.
        while (match !== null && match.index === end) {
            const minutes = Number(match[1]);
            const seconds = Number(match[2]);
            const fraction = match[3] ? Number(`0.${match[3]}`) : 0;
            times.push(minutes * 60 + seconds + fraction);
            end = TIME_TAG.lastIndex;
            match = TIME_TAG.exec(raw);
        }
        if (times.length === 0) continue;
        const body = raw.slice(end).trim();
        if (body === "") continue;
        for (const time of times) out.push({ time, text: body });
    }
    // A stable sort keeps lines that share a time in file order.
    return out.sort((a, b) => a.time - b.time);
}

/** The index of the line sounding at `seconds`: the last one at or before it, or -1 before the first. */
export function activeLineIndex(lines: readonly ILyricLine[], seconds: number): number {
    let lo = 0;
    let hi = lines.length - 1;
    let found = -1;
    while (lo <= hi) {
        const mid = (lo + hi) >> 1;
        if ((lines[mid]?.time ?? 0) <= seconds) {
            found = mid;
            lo = mid + 1;
        } else hi = mid - 1;
    }
    return found;
}
