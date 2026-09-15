/**
 * A small ICU MessageFormat subset: `{arg}`, `{arg, plural, ...}`,
 * `{arg, select, ...}`, `{arg, number}`, `{arg, date}`, `#` inside a plural
 * branch, and `'` escaping.
 *
 * Hand-written rather than pulled from `intl-messageformat` on purpose. The
 * grammar the backend will ACCEPT is already narrower than full ICU - it
 * rejects any message referencing an argument the source string didn't
 * declare, and rejects unbalanced braces - so the formatter only has to cover
 * what can actually be stored. That buys a dependency-free runtime on the hot
 * render path, and keeps the lockfile out of the change.
 *
 * Plural category selection is `Intl.PluralRules`, so every locale's real
 * rules apply (Polish has four, Japanese has one) without a rules table here.
 */

export type MessageValues = Record<string, string | number | Date | boolean | null | undefined>;

/** What `parseArg` yields: a real argument, or literal text when malformed. */
type ParsedArg = { node: INode; end: number };

interface IArgNode {
    kind: "arg";
    name: string;
    type: "plain" | "plural" | "selectordinal" | "select" | "number" | "date" | "time";
    branches?: Record<string, INode[]>;
    /** `offset:N` on a plural, which shifts the number used for selection. */
    offset?: number;
}

interface ITextNode {
    kind: "text";
    value: string;
}

/** `#` inside a plural branch: the selected number, locale-formatted. */
interface IPoundNode {
    kind: "pound";
}

type INode = ITextNode | IArgNode | IPoundNode;

/** Parsed messages are reused across renders; parsing is the expensive half. */
const cache = new Map<string, INode[]>();
const MAX_CACHE = 5000;

function parse(message: string): INode[] {
    const hit = cache.get(message);
    if (hit) return hit;

    const nodes = parseNodes(message, 0, false).nodes;
    if (cache.size >= MAX_CACHE) cache.clear();
    cache.set(message, nodes);
    return nodes;
}

function parseNodes(src: string, start: number, nested: boolean): { nodes: INode[]; end: number } {
    const nodes: INode[] = [];
    let text = "";
    let i = start;

    const flush = () => {
        if (text) {
            nodes.push({ kind: "text", value: text });
            text = "";
        }
    };

    while (i < src.length) {
        const ch = src[i];

        if (ch === "'") {
            // ICU 4.8 "real literal" quoting, which is the rule every ICU
            // implementation ships today:
            //
            //   ''            -> one literal apostrophe
            //   '{ '} '# '|   -> starts a quoted run, ended by the next '
            //   ' anywhere else -> a plain apostrophe, NOT a quote
            //
            // That last clause is the one that matters. Treating every
            // apostrophe as a quote start silently ate them: "Most E2'd"
            // rendered as "Most E2d", and a string with two of them
            // ("What's public, and what isn't") lost both and swallowed the
            // text between. English is the source language here, so that was
            // corrupting the default rendering of any message with a
            // contraction or a possessive - with no error anywhere.
            if (src[i + 1] === "'") {
                text += "'";
                i += 2;
                continue;
            }

            const next = src[i + 1];
            if (next !== "{" && next !== "}" && next !== "#" && next !== "|") {
                text += "'";
                i += 1;
                continue;
            }

            const close = src.indexOf("'", i + 1);
            if (close === -1) {
                text += src.slice(i + 1);
                i = src.length;
            } else {
                text += src.slice(i + 1, close);
                i = close + 1;
            }
            continue;
        }

        if (ch === "}" && nested) {
            flush();
            return { nodes, end: i };
        }

        if (ch === "#" && nested) {
            flush();
            nodes.push({ kind: "pound" });
            i += 1;
            continue;
        }

        if (ch === "{") {
            flush();
            const parsed = parseArg(src, i);
            nodes.push(parsed.node);
            i = parsed.end;
            continue;
        }

        text += ch;
        i += 1;
    }

    flush();
    return { nodes, end: i };
}

function parseArg(src: string, start: number): ParsedArg {
    let i = start + 1;
    while (i < src.length && /\s/.test(src[i])) i += 1;

    let name = "";
    while (i < src.length && /[\w$]/.test(src[i])) {
        name += src[i];
        i += 1;
    }
    while (i < src.length && /\s/.test(src[i])) i += 1;

    // `{name}`
    if (src[i] === "}") {
        return { node: { kind: "arg", name, type: "plain" }, end: i + 1 };
    }

    // `{name, type ...}`
    if (src[i] !== ",") {
        const close = src.indexOf("}", i);
        if (close === -1) {
            // The brace never closes. Emitting an argument here would render
            // the whole tail as empty - a malformed stored translation would
            // silently blank the sentence instead of showing anything. Keep
            // the raw text so the damage is visible and bounded.
            return { node: { kind: "text", value: src.slice(start) }, end: src.length };
        }
        return { node: { kind: "arg", name, type: "plain" }, end: close + 1 };
    }
    i += 1;
    while (i < src.length && /\s/.test(src[i])) i += 1;

    let typeName = "";
    while (i < src.length && /[a-zA-Z]/.test(src[i])) {
        typeName += src[i];
        i += 1;
    }
    while (i < src.length && /\s/.test(src[i])) i += 1;

    const type = (["plural", "selectordinal", "select", "number", "date", "time"] as const).find((t) => t === typeName.toLowerCase());

    if (!type) {
        const close = src.indexOf("}", i);
        if (close === -1) return { node: { kind: "text", value: src.slice(start) }, end: src.length };
        return { node: { kind: "arg", name, type: "plain" }, end: close + 1 };
    }

    if (type === "number" || type === "date" || type === "time") {
        const close = src.indexOf("}", i);
        if (close === -1) return { node: { kind: "text", value: src.slice(start) }, end: src.length };
        return { node: { kind: "arg", name, type }, end: close + 1 };
    }

    // plural / selectordinal / select: `, key {branch} key {branch}`
    if (src[i] === ",") i += 1;

    const branches: Record<string, INode[]> = {};
    let offset: number | undefined;

    while (i < src.length) {
        while (i < src.length && /\s/.test(src[i])) i += 1;
        if (src[i] === "}") {
            i += 1;
            break;
        }

        let key = "";
        while (i < src.length && !/[\s{]/.test(src[i])) {
            key += src[i];
            i += 1;
        }
        if (!key) break;

        if (key.startsWith("offset:")) {
            offset = Number.parseInt(key.slice(7), 10) || 0;
            continue;
        }

        while (i < src.length && /\s/.test(src[i])) i += 1;
        if (src[i] !== "{") break;

        const inner = parseNodes(src, i + 1, true);
        branches[key] = inner.nodes;
        i = inner.end + 1;
    }

    return { node: { kind: "arg", name, type, branches, offset }, end: i };
}

function pluralKey(locale: string, value: number, type: "plural" | "selectordinal", branches: Record<string, INode[]>): INode[] | undefined {
    // An `=N` branch is an exact match and outranks the category.
    const exact = branches[`=${value}`];
    if (exact) return exact;

    try {
        const category = new Intl.PluralRules(locale, {
            type: type === "selectordinal" ? "ordinal" : "cardinal",
        }).select(value);
        if (branches[category]) return branches[category];
    } catch {
        // An unknown locale tag falls through to `other`.
    }
    return branches.other;
}

function render(nodes: INode[], locale: string, values: MessageValues, poundValue: number | null): string {
    let out = "";

    for (const node of nodes) {
        if (node.kind === "text") {
            out += node.value;
            continue;
        }
        if (node.kind === "pound") {
            out += poundValue === null ? "" : new Intl.NumberFormat(locale).format(poundValue);
            continue;
        }

        const raw = values[node.name];

        switch (node.type) {
            case "plain":
                out += raw === null || raw === undefined ? "" : String(raw);
                break;
            case "number":
                out += typeof raw === "number" ? new Intl.NumberFormat(locale).format(raw) : String(raw ?? "");
                break;
            case "date":
            case "time": {
                const date = raw instanceof Date ? raw : new Date(String(raw ?? ""));
                out += Number.isNaN(date.getTime()) ? "" : new Intl.DateTimeFormat(locale, node.type === "date" ? { dateStyle: "medium" } : { timeStyle: "short" }).format(date);
                break;
            }
            case "plural":
            case "selectordinal": {
                const num = typeof raw === "number" ? raw : Number(raw ?? 0);
                const selectValue = num - (node.offset ?? 0);
                const branch = pluralKey(locale, selectValue, node.type, node.branches ?? {});
                out += branch ? render(branch, locale, values, selectValue) : "";
                break;
            }
            case "select": {
                const key = String(raw ?? "");
                const branch = node.branches?.[key] ?? node.branches?.other;
                out += branch ? render(branch, locale, values, poundValue) : "";
                break;
            }
        }
    }

    return out;
}

/**
 * Format one message. A message with no braces short-circuits, because the
 * overwhelming majority of UI strings are plain text and should not pay for a
 * parser.
 */
export function formatMessage(message: string, locale: string, values?: MessageValues): string {
    if (!message.includes("{") && !message.includes("'")) return message;
    try {
        return render(parse(message), locale, values ?? {}, null);
    } catch {
        // Never let a malformed stored message blank a subtree; showing the
        // raw pattern is strictly more useful than throwing.
        return message;
    }
}
