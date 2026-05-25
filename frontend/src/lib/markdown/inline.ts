import { sanitizeHref } from "./sanitize";
import type { InlineNode } from "./types";

/**
 * Parses inline markdown (bold, italic, code, links, autolinks, escapes).
 * Input must NOT contain `\n` - the block tokenizer handles line breaks.
 */
export function parseInline(src: string): InlineNode[] {
    const nodes: InlineNode[] = [];
    let buf = "";
    let i = 0;

    const flush = () => {
        if (buf) {
            nodes.push({ type: "text", value: buf });
            buf = "";
        }
    };

    while (i < src.length) {
        const c = src.charAt(i);

        // Escape: \X → literal X (only escapes a punctuation set; backslash otherwise literal)
        if (c === "\\" && isEscapable(src.charAt(i + 1))) {
            buf += src.charAt(i + 1);
            i += 2;
            continue;
        }

        // Inline code: `text`
        if (c === "`") {
            const end = src.indexOf("`", i + 1);
            if (end > i) {
                flush();
                nodes.push({ type: "code", value: src.slice(i + 1, end) });
                i = end + 1;
                continue;
            }
        }

        // Strong: **text**
        if (c === "*" && src.charAt(i + 1) === "*") {
            const end = findClosing(src, i + 2, "**");
            if (end > 0) {
                flush();
                nodes.push({ type: "strong", children: parseInline(src.slice(i + 2, end)) });
                i = end + 2;
                continue;
            }
        }

        // Emphasis: *text* or _text_
        if ((c === "*" || c === "_") && !isWordChar(src.charAt(i - 1))) {
            const marker = c;
            const end = findEmphasisEnd(src, i + 1, marker);
            if (end > 0) {
                flush();
                nodes.push({ type: "emphasis", children: parseInline(src.slice(i + 1, end)) });
                i = end + 1;
                continue;
            }
        }

        // Link: [text](href)
        if (c === "[") {
            const link = parseLink(src, i);
            if (link) {
                flush();
                nodes.push(link.node);
                i = link.end;
                continue;
            }
        }

        // Autolink: <http://...> or <mailto:...>
        if (c === "<") {
            const end = src.indexOf(">", i + 1);
            if (end > i) {
                const inner = src.slice(i + 1, end);
                if (/^(https?:|mailto:)\S+$/i.test(inner)) {
                    const safe = sanitizeHref(inner);
                    if (safe) {
                        flush();
                        nodes.push({ type: "link", href: safe, children: [{ type: "text", value: inner }] });
                        i = end + 1;
                        continue;
                    }
                }
            }
        }

        buf += c;
        i++;
    }

    flush();
    return nodes;
}

function isEscapable(c: string): boolean {
    return c.length === 1 && "\\`*_{}[]()#+-.!<>|~".includes(c);
}

function isWordChar(c: string): boolean {
    return c.length === 1 && /[a-zA-Z0-9]/.test(c);
}

/** Finds the next occurrence of `needle` not inside backtick spans. */
function findClosing(src: string, from: number, needle: string): number {
    let i = from;
    while (i < src.length) {
        if (src.charAt(i) === "\\" && isEscapable(src.charAt(i + 1))) {
            i += 2;
            continue;
        }
        if (src.charAt(i) === "`") {
            const end = src.indexOf("`", i + 1);
            if (end > 0) {
                i = end + 1;
                continue;
            }
        }
        if (src.startsWith(needle, i)) return i;
        i++;
    }
    return -1;
}

function findEmphasisEnd(src: string, from: number, marker: string): number {
    let i = from;
    while (i < src.length) {
        if (src.charAt(i) === "\\" && isEscapable(src.charAt(i + 1))) {
            i += 2;
            continue;
        }
        if (src.charAt(i) === "`") {
            const end = src.indexOf("`", i + 1);
            if (end > 0) {
                i = end + 1;
                continue;
            }
        }
        // Don't match **; let strong take it
        if (src.charAt(i) === marker && src.charAt(i + 1) !== marker) {
            // Don't close on word-adjacent _ (avoid breaking snake_case)
            if (marker === "_" && isWordChar(src.charAt(i + 1))) {
                i++;
                continue;
            }
            // Must have non-space before close
            const prev = src.charAt(i - 1);
            if (prev === " " || prev === marker) {
                i++;
                continue;
            }
            return i;
        }
        i++;
    }
    return -1;
}

function parseLink(src: string, start: number): { node: InlineNode; end: number } | null {
    // [text]
    let depth = 1;
    let i = start + 1;
    while (i < src.length && depth > 0) {
        const c = src.charAt(i);
        if (c === "\\" && isEscapable(src.charAt(i + 1))) {
            i += 2;
            continue;
        }
        if (c === "[") depth++;
        else if (c === "]") {
            depth--;
            if (depth === 0) break;
        }
        i++;
    }
    if (depth !== 0 || src.charAt(i + 1) !== "(") return null;

    const textInner = src.slice(start + 1, i);

    // (href)
    const hrefStart = i + 2;
    const hrefEnd = src.indexOf(")", hrefStart);
    if (hrefEnd < 0) return null;
    const rawHref = src.slice(hrefStart, hrefEnd).trim();
    const safe = sanitizeHref(rawHref);
    if (!safe) return null;

    return {
        node: { type: "link", href: safe, children: parseInline(textInner) },
        end: hrefEnd + 1,
    };
}
