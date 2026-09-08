/**
 * Arknights rich-text markup, shared by every surface that renders game
 * description text (operator traits/talents/skills, base skills, enemy
 * abilities, stage briefings).
 *
 * Two openers, one universal closer:
 *   `<@scope.name>text</>`  a style span, coloured from RichTextStyles
 *   `<$scope.name>text</>`  a term link into the in-game glossary
 *
 * Tag names are dot-separated `[A-Za-z0-9_]` segments, one to three deep, and
 * are not uniformly lowercase - the shipped tables carry `<@tutor>`,
 * `<@ba.vup>`, `<@ro2.get>`, `<@act.timeLimit>` and `<@ba.dt.element>` alike.
 * Anything narrower than this grammar leaks the tag to the page as literal
 * text, so keep it as the single source of truth and build call-site regexes
 * from `TAG_NAME` rather than hand-rolling another one.
 */
const TAG_NAME = String.raw`[@$][A-Za-z0-9_]+(?:\.[A-Za-z0-9_]+)*`;

/**
 * One scan of the markup. Alternatives, in order: a game opener (group 1,
 * sigil included), the universal closer (2), the Unity inline tags that are
 * real HTML and already render (3), and the Unity tags the browser silently
 * drops today (4, no group - matched only so they stay dropped).
 *
 * Anything else angle-bracketed is TEXT, not markup, and is escaped. The game
 * writes stage and enemy item names that way - `<@lv.item><Roadblock></>`,
 * `<Substitute>`, `<In battle>` - 3509 of them across the tables rendered here.
 * Passed unescaped into innerHTML the browser reads `<Roadblock>` as an unknown
 * element and shows nothing, which is how those names were going missing.
 */
function createTokenRegex(): RegExp {
    return new RegExp(`<(${TAG_NAME})>|(</>)|(</?(?:i|b|u)>)|(?:</?(?:color|size)(?:[ =][^<>]*)?>|<break>)`, "gi");
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

/**
 * Collapse the markup to HTML in one pass: literal text is escaped, balanced
 * game tags go through `wrap`, and the tag stack lets a nesting like
 * `<$scope><@cc.kw>text</></>` close inside out. A stray `</>` is dropped and
 * an unclosed opener is flushed wrapped, so malformed input degrades to its
 * text rather than leaking a tag to the page.
 */
export function renderTags(text: string, wrap: (tag: string, content: string) => string): string {
    const token = createTokenRegex();
    const stack: { tag: string; buf: string }[] = [{ tag: "", buf: "" }];
    const push = (s: string) => {
        stack[stack.length - 1].buf += s;
    };
    let last = 0;

    for (let m = token.exec(text); m !== null; m = token.exec(text)) {
        if (m.index > last) push(escapeText(text.slice(last, m.index)));
        last = m.index + m[0].length;

        if (m[1]) {
            stack.push({ tag: m[1], buf: "" });
        } else if (m[2]) {
            if (stack.length === 1) continue;
            const done = stack.pop();
            if (done) push(wrap(done.tag, done.buf));
        } else if (m[3]) {
            push(m[3].toLowerCase());
        }
    }

    if (last < text.length) push(escapeText(text.slice(last)));
    while (stack.length > 1) {
        const done = stack.pop();
        if (done) push(wrap(done.tag, done.buf));
    }
    return stack[0].buf;
}

/**
 * Tags as plain emphasis rather than the in-game palette - for muted body copy
 * (enemy abilities, stage briefings) that reads as prose, not as a stat sheet.
 */
export function emphasizeTagsHtml(text: string): string {
    return renderTags(text, (_tag, content) => `<strong style="color: var(--foreground)">${content}</strong>`).replace(/\\n|\n/g, "<br/>");
}
