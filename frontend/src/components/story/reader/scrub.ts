/**
 * The top progress bar as a SCRUBBER: the halt list behind it, and the pure
 * lookups the pointer handlers use.
 *
 * The list is one walk of a probe engine, the same walk `replayTo` already
 * does to rebuild the backlog, so hovering costs nothing: the walk happens
 * once per script and the tooltip is an array index.
 *
 * Nothing here touches the engine; the walk lives in `useStoryPlayer` and
 * hands the summaries down, so every function in this file is testable without
 * a script.
 */

import { clamp, clamp01 } from "#/lib/story/num";

export interface HaltSummary {
    haltIndex: number;
    /** A `video` summary is a cutscene, whose preview is the label, not a line. */
    kind: "line" | "decision" | "video";
    speaker?: string;
    /** The first words of the line, already plain and already substituted. */
    preview: string;
}

/** The first `max` characters of a line, cut on a word boundary where one is near the end. */
export function firstWords(text: string, max = 56): string {
    const flat = text.replace(/\s+/g, " ").trim();
    if (flat.length <= max) return flat;
    const cut = flat.slice(0, max);
    const space = cut.lastIndexOf(" ");
    return `${(space > max * 0.6 ? cut.slice(0, space) : cut).trimEnd()}…`;
}

/** Clamp a pointer position along the bar to 0..1. A zero-width bar reads as 0. */
export function fractionAt(clientX: number, rect: { left: number; width: number }): number {
    if (!(rect.width > 0)) return 0;
    return clamp01((clientX - rect.left) / rect.width);
}

/**
 * The halt a fraction of the bar points at. The bar's filled width is
 * `(haltIndex + 1) / totalHalts`, so fraction f maps to ordinal
 * `ceil(f * total) - 1`, which makes the far left halt 0 and the far right the
 * last one. An empty list has no answer, and says so with null rather than
 * with halt 0.
 */
export function haltAtFraction(list: readonly HaltSummary[], fraction: number): HaltSummary | null {
    if (list.length === 0) return null;
    const f = clamp01(fraction);
    const at = clamp(Math.ceil(f * list.length) - 1, 0, list.length - 1);
    return list[at];
}

/** Where a halt sits along the bar, 0..1, matching the filled width the bar draws. */
export function fractionForHalt(list: readonly HaltSummary[], haltIndex: number): number {
    if (list.length === 0) return 0;
    const at = list.findIndex((h) => h.haltIndex === haltIndex);
    if (at < 0) return 0;
    return (at + 1) / list.length;
}

/**
 * Where a backlog row jumps to. Null means "you are already there", which is
 * what keeps a click on the current row from replaying the whole story to the
 * line it is already showing. A CHOICE row carries the decision halt's own
 * index, so jumping to it puts the decision back on screen.
 */
export function backlogJumpTarget(entry: { haltIndex: number }, currentHaltIndex: number): number | null {
    if (entry.haltIndex < 0) return null;
    return entry.haltIndex === currentHaltIndex ? null : entry.haltIndex;
}

/** One backlog beat, structurally: the copy needs a speaker and a line and nothing else about it. */
export interface BacklogLine {
    kind: "line" | "choice" | "cutscene";
    speaker?: string;
    text: string;
}

/**
 * The backlog as PLAIN TEXT, one line per beat, which is what goes on the
 * clipboard: `Speaker: line` for dialogue, the line alone for narration (it
 * has no speaker, and inventing one would put a voice on it), and `> option`
 * for a decision the reader took.
 *
 * The markup goes through the reader's own parser rather than a regex, so
 * `<color>`, `<@tu.kw>` and `<p=N>` come out as the words they wrap and a
 * paragraph break stays a break; the nickname is already substituted upstream,
 * because the entries carry the text the box drew.
 */
export function backlogPlainText(entries: readonly BacklogLine[], plain: (text: string) => string, title?: string): string {
    const body = entries.map((e) => (e.kind === "choice" ? `> ${plain(e.text)}` : e.kind === "cutscene" ? `[${plain(e.text)}]` : e.speaker ? `${e.speaker}: ${plain(e.text)}` : plain(e.text)));
    return (title ? [title, "", ...body] : body).join("\n");
}
