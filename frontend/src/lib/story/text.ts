/**
 * Story line markup to a small node tree the TextBox renders. The corpus
 * (EN, 4,860 scripts, counted 2026-09-21) carries these openers inside line
 * text: `<i>` 3,393, `</>` 1,651, `<@tu.kw>` 1,351 (tutorial keyword, closed
 * by `</>`), `<color=#hex>` 397 with `</color>` 397, `<p=N>` 283 (a paragraph
 * span, closed by `</>`), `<@tu.imp>` 20, `<b>` 8, plus one `<i/>` and two
 * literal `<The Romance of Ives>` (a book title, kept as text). Substitutions:
 * `{@nickname}` 830 and `{@Nickname}` 3 (the Doctor's name), `{@nbs}` 22 (a
 * non-breaking space). Sticker and subtitle text carries literal `\n`.
 *
 * `lib/gamedata/richtext.ts` is not reused because it DROPS `<color>` on
 * purpose (the game tables render colour from `<@scope>` styles), and story
 * text needs the colour and the paragraph split. Its `colorForTag` is reused
 * so `<@tu.kw>` matches the rest of the site.
 */
import { colorForTag } from "#/lib/gamedata/richtext";

export type TextNode = { kind: "text"; value: string } | { kind: "inline"; tag: "i" | "b" | "u"; children: TextNode[] } | { kind: "color"; color: string; children: TextNode[] } | { kind: "paragraph"; index: number; children: TextNode[] };

interface Frame {
    node: TextNode | null;
    children: TextNode[];
}

const TOKEN = /<(i|b|u)>|<\/(i|b|u)>|<color=(#[0-9a-fA-F]{3,8}|[a-zA-Z]+)>|<\/color>|<p=(\d+)>|<(@[A-Za-z0-9_.]+)>|<\/>|<i\/>/g;

/** `{@nickname}` -> the Doctor's name, `{@nbs}` -> a non-breaking space, literal `\n` -> newline. */
export function substitute(text: string, nickname: string): string {
    return text
        .replace(/\{@nickname\}/gi, nickname)
        .replace(/\{@nbs\}/g, " ")
        .replace(/\\n/g, "\n");
}

/** Parse one line's markup into nodes. Unknown angle-bracket text stays literal. */
export function parseStoryText(text: string): TextNode[] {
    const root: Frame = { node: null, children: [] };
    const stack: Frame[] = [root];
    let last = 0;
    const pushText = (value: string) => {
        if (value.length === 0) return;
        const top = stack[stack.length - 1];
        const prev = top.children[top.children.length - 1];
        if (prev && prev.kind === "text") prev.value += value;
        else top.children.push({ kind: "text", value });
    };
    const open = (node: TextNode & { children: TextNode[] }) => {
        stack[stack.length - 1].children.push(node);
        stack.push({ node, children: node.children });
    };
    const close = () => {
        if (stack.length > 1) stack.pop();
    };
    TOKEN.lastIndex = 0;
    for (let m = TOKEN.exec(text); m !== null; m = TOKEN.exec(text)) {
        pushText(text.slice(last, m.index));
        last = m.index + m[0].length;
        const [whole, openInline, closeInline, color, paragraph, styleTag] = m;
        if (openInline) open({ kind: "inline", tag: openInline as "i" | "b" | "u", children: [] });
        else if (closeInline || whole === "</color>" || whole === "</>") close();
        else if (color) open({ kind: "color", color, children: [] });
        else if (paragraph) open({ kind: "paragraph", index: Number(paragraph), children: [] });
        else if (styleTag) open({ kind: "color", color: colorForTag(styleTag), children: [] });
        // `<i/>` is a self-closing typo (1 use): dropped.
    }
    pushText(text.slice(last));
    return root.children;
}

/** The line as plain text, for the backlog copy and the auto-play timer. */
export function plainStoryText(nodes: TextNode[]): string {
    let out = "";
    for (const n of nodes) {
        if (n.kind === "text") out += n.value;
        else {
            out += plainStoryText(n.children);
            if (n.kind === "paragraph") out += "\n";
        }
    }
    return out;
}

/** Substitute, then parse: the one call the reader makes per line. */
export function renderLine(text: string, nickname: string): TextNode[] {
    return parseStoryText(substitute(text, nickname));
}
