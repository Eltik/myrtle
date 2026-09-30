/**
 * A TINY ESCAPING XML BUILDER. Every piece of story text reaches the output
 * as a TEXT NODE and is escaped here, once; nothing concatenates user text
 * into markup. It exists instead of `XMLSerializer` because the book is built
 * without a DOM (tests, and a server route later), and it serialises the
 * polyglot subset both XHTML and HTML parse the same way: void elements
 * self-close, every other element gets an explicit end tag even when empty
 * (`<div/>` would open a div in an HTML parser).
 */

export interface XElement {
    name: string;
    attrs: Record<string, string | undefined>;
    children: XNode[];
}

/** A string child is TEXT, never markup. */
export type XNode = XElement | string | null | undefined | false;

export function el(name: string, attrs: Record<string, string | undefined> = {}, ...children: (XNode | XNode[])[]): XElement {
    return { name, attrs, children: children.flat() };
}

const VOID = new Set(["br", "hr", "img", "meta", "link", "col", "area", "base", "input", "source", "wbr"]);

/**
 * Drop what XML 1.0 forbids outright (C0 controls other than tab, newline and
 * carriage return, U+FFFE, U+FFFF) and lone surrogates. They are dropped
 * rather than escaped, because no escape makes them legal. A loop over code
 * units, not a regex: the linter rejects control characters in a pattern.
 */
function stripIllegal(value: string): string {
    let out = "";
    for (let i = 0; i < value.length; i += 1) {
        const c = value.charCodeAt(i);
        if (c >= 0xd800 && c <= 0xdbff) {
            const next = value.charCodeAt(i + 1);
            if (next >= 0xdc00 && next <= 0xdfff) {
                out += value[i] + value[i + 1];
                i += 1;
            }
            continue;
        }
        if (c >= 0xdc00 && c <= 0xdfff) continue;
        if (c === 0x9 || c === 0xa || c === 0xd || (c >= 0x20 && c !== 0xfffe && c !== 0xffff)) out += value[i];
    }
    return out;
}

export function escapeText(value: string): string {
    return stripIllegal(value).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

export function escapeAttr(value: string): string {
    return escapeText(value).replace(/"/g, "&quot;").replace(/\n/g, "&#10;").replace(/\t/g, "&#9;");
}

/** A valid XML attribute or element name; anything else is a programming error, not content. */
const NAME = /^[A-Za-z_][A-Za-z0-9_.:-]*$/;

export function serialize(node: XNode | XNode[]): string {
    if (Array.isArray(node)) return node.map(serialize).join("");
    if (node === null || node === undefined || node === false) return "";
    if (typeof node === "string") return escapeText(node);
    if (!NAME.test(node.name)) throw new Error(`bad element name ${node.name}`);
    let out = `<${node.name}`;
    for (const [k, v] of Object.entries(node.attrs)) {
        if (v === undefined) continue;
        if (!NAME.test(k)) throw new Error(`bad attribute name ${k}`);
        out += ` ${k}="${escapeAttr(v)}"`;
    }
    if (VOID.has(node.name) && node.children.every((c) => c === null || c === undefined || c === false || c === "")) return `${out}/>`;
    return `${out}>${node.children.map(serialize).join("")}</${node.name}>`;
}

/** An XML document: the declaration, an optional doctype, and the root. */
export function xmlDocument(root: XElement, doctype?: string): string {
    return `<?xml version="1.0" encoding="UTF-8"?>\n${doctype ? `${doctype}\n` : ""}${serialize(root)}\n`;
}
