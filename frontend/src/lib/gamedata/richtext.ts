/**
 * Arknights rich-text markup, shared by every surface that renders game text:
 * operator traits/talents/skills, base skills, enemy abilities, stage
 * briefings, and the EN-localised prose that carries Unity `<i>` (voice lines,
 * handbook stories, module descriptions).
 *
 * Two game openers, one universal closer:
 *   `<@scope.name>text</>`  a style span, coloured from RichTextStyles
 *   `<$scope.name>text</>`  a term link into the in-game glossary
 *
 * Tag names are dot-separated `[A-Za-z0-9_]` segments, one to three deep, and
 * are not uniformly lowercase - the shipped tables carry `<@tutor>`,
 * `<@ba.vup>`, `<@ro2.get>`, `<@act.timeLimit>` and `<@ba.dt.element>` alike.
 * Anything narrower than this grammar leaks the tag to the page as literal
 * text, so keep it as the single source of truth and build call-site regexes
 * from `TAG_NAME` rather than hand-rolling another one.
 *
 * `parseRichText` is the one parser; `foldRichText` is the one walk. Every
 * renderer (`renderTags` to HTML, `plainText` to a string, `GameText` to React
 * nodes) is a visitor over that walk, so they cannot drift from each other.
 */
const TAG_NAME = String.raw`[@$][A-Za-z0-9_]+(?:\.[A-Za-z0-9_]+)*`;

/**
 * One scan of the markup. Alternatives, in order: a game opener (group 1,
 * sigil included), the universal closer (2), a Unity inline opener (3) or
 * closer (4) that is real HTML, and the Unity tags the browser silently drops
 * today (no group - matched only so they stay dropped).
 *
 * Anything else angle-bracketed is TEXT, not markup. The game writes stage and
 * enemy item names that way - `<@lv.item><Roadblock></>`, `<Substitute>`,
 * `<In battle>` - 3509 of them across the tables rendered here. Passed
 * unescaped into innerHTML the browser reads `<Roadblock>` as an unknown
 * element and shows nothing, which is how those names were going missing.
 */
function createTokenRegex(): RegExp {
    return new RegExp(`<(${TAG_NAME})>|(</>)|<(i|b|u)>|</(i|b|u)>|(?:</?(?:color|size)(?:[ =][^<>]*)?>|<break>)`, "gi");
}

/** One whole tag, opener through closer, anchored at the start of the input. */
export const atomicTagRegex = new RegExp(`^<${TAG_NAME}>[\\s\\S]*?<\\/>`);

/** A fresh opener-or-closer scanner; call per use so `lastIndex` is never shared. */
export function tagTokenRegex(): RegExp {
    return new RegExp(`<(${TAG_NAME})>|<\\/>`, "g");
}

function escapeText(text: string): string {
    return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

const DESCRIPTION_COLORS = {
    valueUp: "#6495ED",
    valueDown: "#ff847d",
    reminder: "#da9a46",
    keyword: "#27e8e7",
    potential: "#27e8e7",
    skillTooltip: "#27e8e7",
};

export function colorForTag(tag: string): string {
    if (tag.startsWith("$")) return DESCRIPTION_COLORS.skillTooltip;
    // Trailing tag fragment after `@<scope>.` - `ba` is combat, `cc` is base/RIIC.
    const suffix = tag.includes(".") ? tag.slice(tag.indexOf(".") + 1) : tag;
    if (suffix === "vup") return DESCRIPTION_COLORS.valueUp;
    if (suffix === "vdown") return DESCRIPTION_COLORS.valueDown;
    if (suffix === "rem") return DESCRIPTION_COLORS.reminder;
    if (suffix === "kw") return DESCRIPTION_COLORS.keyword;
    if (suffix === "talpu") return DESCRIPTION_COLORS.potential;
    return DESCRIPTION_COLORS.keyword;
}

export type InlineTag = "i" | "b" | "u";

/**
 * The parsed markup. `text` is literal, unescaped source; `inline` is a Unity
 * `<i>`/`<b>`/`<u>` span; `style` is a game `<@scope.name>`/`<$scope.name>`
 * span with the sigil kept on `tag` so `colorForTag` can read it.
 */
export type RichNode = { kind: "text"; value: string } | { kind: "inline"; tag: InlineTag; children: RichNode[] } | { kind: "style"; tag: string; children: RichNode[] };

type RichBranch = Exclude<RichNode, { kind: "text" }>;

/** Pop the stack down to and including the innermost frame `matches`; a miss leaves it alone. */
function closeNearest<T>(stack: T[], matches: (frame: T) => boolean): void {
    for (let i = stack.length - 1; i >= 0; i--) {
        if (matches(stack[i])) {
            stack.length = i;
            return;
        }
    }
}

/**
 * Parse the markup into a tree in one pass. The stack lets a nesting like
 * `<$scope><@cc.kw>text</></>` close inside out. Malformed input degrades to
 * its text rather than leaking a tag to the page: a stray `</>` or `</i>` is
 * dropped, an unclosed opener is closed at the end of the input, and a `</>`
 * arriving while an inline span is still open closes that span first (the
 * browser's own recovery for `<span><i>x</span>`; 2 of 17634 EN strings).
 */
export function parseRichText(text: string): RichNode[] {
    const token = createTokenRegex();
    const root: RichNode[] = [];
    const open: RichBranch[] = [];
    const siblings = () => (open.length ? open[open.length - 1].children : root);
    const pushText = (value: string) => {
        if (value) siblings().push({ kind: "text", value });
    };
    const openBranch = (node: RichBranch) => {
        siblings().push(node);
        open.push(node);
    };
    let last = 0;

    for (let m = token.exec(text); m !== null; m = token.exec(text)) {
        if (m.index > last) pushText(text.slice(last, m.index));
        last = m.index + m[0].length;

        if (m[1]) {
            openBranch({ kind: "style", tag: m[1], children: [] });
        } else if (m[2]) {
            closeNearest(open, (node) => node.kind === "style");
        } else if (m[3]) {
            openBranch({ kind: "inline", tag: m[3].toLowerCase() as InlineTag, children: [] });
        } else if (m[4]) {
            const tag = m[4].toLowerCase();
            closeNearest(open, (node) => node.kind === "inline" && node.tag === tag);
        }
    }

    if (last < text.length) pushText(text.slice(last));
    return root;
}

/** One renderer over the tree: a leaf becomes a `T`, a branch folds its children's `T`s. */
export interface RichVisitor<T> {
    text: (value: string) => T;
    inline: (tag: InlineTag, children: T[]) => T;
    style: (tag: string, children: T[]) => T;
}

/** Walk the tree depth-first in document order, so a visitor sees text leaves in reading order. */
export function foldRichText<T>(nodes: RichNode[], visitor: RichVisitor<T>): T[] {
    return nodes.map((node) => {
        if (node.kind === "text") return visitor.text(node.value);
        const children = foldRichText(node.children, visitor);
        return node.kind === "inline" ? visitor.inline(node.tag, children) : visitor.style(node.tag, children);
    });
}

const joined = (parts: string[]) => parts.join("");

/** The markup's text with every tag removed, for search and plain-text surfaces. */
export function plainText(text: string): string {
    return joined(foldRichText(parseRichText(text), { text: (value) => value, inline: (_tag, children) => joined(children), style: (_tag, children) => joined(children) }));
}

/**
 * Collapse the markup to HTML for a `dangerouslySetInnerHTML` sink: literal
 * text is escaped, inline spans become their HTML element, and game spans go
 * through `wrap` with their already-rendered content.
 */
export function renderTags(text: string, wrap: (tag: string, content: string) => string): string {
    return joined(
        foldRichText(parseRichText(text), {
            text: escapeText,
            inline: (tag, children) => `<${tag}>${joined(children)}</${tag}>`,
            style: (tag, children) => wrap(tag, joined(children)),
        }),
    );
}

/**
 * Tags as plain emphasis rather than the in-game palette - for muted body copy
 * (enemy abilities, stage briefings) that reads as prose, not as a stat sheet.
 */
export function emphasizeTagsHtml(text: string): string {
    return renderTags(text, (_tag, content) => `<strong style="color: var(--foreground)">${content}</strong>`).replace(/\\n|\n/g, "<br/>");
}
