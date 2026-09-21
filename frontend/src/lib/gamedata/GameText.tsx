import { createElement, Fragment, type ReactNode } from "react";
import { colorForTag, foldRichText, parseRichText, plainText, type RichVisitor } from "./richtext";

/**
 * How a game `<@scope.name>` span is coloured. `game` is the in-game palette
 * for stat-sheet text (traits, skills, base skills); `emphasis` is plain bold
 * for muted body copy; `plain` drops the span and keeps its text, for surfaces
 * whose source never carries a game span (voice lines, lore, module text).
 */
export type GameTextPalette = "game" | "emphasis" | "plain";

interface IGameTextProps {
    text: string;
    palette?: GameTextPalette;
    /** Case-insensitive substring to wrap in `<mark>`; matched on the tag-free text, so it may straddle a tag. */
    highlight?: string;
}

const MARK_CLASS = "rounded-sm bg-primary/30 px-0.5 text-foreground";

/**
 * Arknights rich text as React nodes. The HTML string from `renderTags` is
 * right for a `dangerouslySetInnerHTML` sink, but a surface that composes on
 * top of the text (search highlighting here) cannot use a string, and the
 * fallback of rendering `{text}` shows the tags literally: `A <i>clear</i>
 * battle plan.` on 187 EN voice lines. This is a visitor over the same parse
 * tree the HTML path uses, so the two renderers cannot drift.
 */
export function GameText({ text, palette = "plain", highlight = "" }: IGameTextProps) {
    const marks = new Highlighter(plainText(text), highlight);
    const visitor: RichVisitor<ReactNode> = {
        text: (value) => marks.leaf(value),
        inline: (tag, children) => createElement(tag, null, keyed(children)),
        style: (tag, children) => styleSpan(tag, keyed(children), palette),
    };
    return <>{keyed(foldRichText(parseRichText(text), visitor))}</>;
}

function styleSpan(tag: string, children: ReactNode[], palette: GameTextPalette): ReactNode {
    if (palette === "game") return <span style={{ color: colorForTag(tag) }}>{children}</span>;
    if (palette === "emphasis") return <strong style={{ color: "var(--foreground)" }}>{children}</strong>;
    return children;
}

function keyed(children: ReactNode[]): ReactNode[] {
    return children.map((child, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: positions in one parsed string are stable
        <Fragment key={i}>{child}</Fragment>
    ));
}

/**
 * Projects highlight matches onto text leaves. Matching runs once over the
 * tag-free text so a query can straddle a tag (`clear battle` across
 * `<i>clear</i> battle`); `leaf` is then fed each text run in document order
 * and wraps whichever slices of it fall inside a match.
 */
class Highlighter {
    private readonly ranges: [number, number][];
    private next = 0;
    private offset = 0;

    constructor(plain: string, query: string) {
        this.ranges = matchRanges(plain.toLowerCase(), query.toLowerCase());
    }

    leaf(value: string): ReactNode {
        const start = this.offset;
        const end = start + value.length;
        this.offset = end;
        if (this.next >= this.ranges.length) return value;

        const out: ReactNode[] = [];
        let at = start;
        for (; this.next < this.ranges.length; this.next++) {
            const [from, to] = this.ranges[this.next];
            if (from >= end) break;
            const markStart = Math.max(from, at);
            const markEnd = Math.min(to, end);
            if (markStart > at) out.push(value.slice(at - start, markStart - start));
            out.push(
                <mark key={markStart} className={MARK_CLASS}>
                    {value.slice(markStart - start, markEnd - start)}
                </mark>,
            );
            at = markEnd;
            // A match running past this leaf continues in the next one.
            if (to > end) break;
        }
        if (at < end) out.push(value.slice(at - start));
        return out;
    }
}

function matchRanges(haystack: string, needle: string): [number, number][] {
    if (!needle) return [];
    const out: [number, number][] = [];
    for (let at = haystack.indexOf(needle); at !== -1; at = haystack.indexOf(needle, at + needle.length)) out.push([at, at + needle.length]);
    return out;
}
