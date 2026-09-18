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

// ---------------------------------------------------------------- description

/**
 * One argument of a message, as the translation editor needs to explain it.
 *
 * The editor used to show the declared placeholder names and nothing else, so
 * a translator meeting `{count, plural, one {player} other {players}}` was
 * handed the raw ICU and left to infer the grammar from it. The parser already
 * knows the answer; this is that knowledge exported rather than re-derived by
 * a regex in a component.
 */
export interface IMessageArgument {
    name: string;
    type: IArgNode["type"];
    /** Branch keys in source order. Empty for anything but plural/select. */
    branches: string[];
    /** Each branch's wording, flattened to plain text for display. */
    branchText: Record<string, string>;
    /** `offset:N` on a plural, or `null` when there is none. */
    offset: number | null;
}

/**
 * A branch's wording as a translator reads it: literal text, `#` for the
 * number, and `{name}` for a nested argument. Not a re-serialiser - it is only
 * ever shown, never parsed back.
 */
function nodesToText(nodes: INode[]): string {
    let out = "";
    for (const node of nodes) {
        if (node.kind === "text") out += node.value;
        else if (node.kind === "pound") out += "#";
        else out += `{${node.name}}`;
    }
    return out;
}

function collectArgs(nodes: INode[], into: Map<string, IMessageArgument>): void {
    for (const node of nodes) {
        if (node.kind !== "arg") continue;
        const branches = node.branches ? Object.keys(node.branches) : [];
        const branchText: Record<string, string> = {};
        if (node.branches) {
            for (const [key, inner] of Object.entries(node.branches)) branchText[key] = nodesToText(inner);
        }
        const existing = into.get(node.name);
        // A name can appear more than once - `{count, number} {count, plural,
        // ...}` is the catalogue's commonest shape. The branching occurrence is
        // the one worth describing, so a bare mention never overwrites it and
        // a branching one always replaces a bare one.
        if (!existing || (existing.branches.length === 0 && branches.length > 0)) {
            into.set(node.name, { name: node.name, type: node.type, branches, branchText, offset: node.offset ?? null });
        }
        if (node.branches) {
            for (const inner of Object.values(node.branches)) collectArgs(inner, into);
        }
    }
}

/**
 * The arguments a message takes, in first-appearance order. Malformed input
 * yields whatever parsed cleanly rather than throwing, matching
 * `formatMessage`'s refusal to let a bad stored string break a screen.
 */
export function describeMessage(message: string): IMessageArgument[] {
    if (!message.includes("{")) return [];
    try {
        const found = new Map<string, IMessageArgument>();
        // `parseNodes` rather than `parse`: the caller is the translation editor,
        // which calls this on every keystroke, so going through the memo would
        // put one dead entry per keystroke into a cache the RENDER path shares.
        // At 5000 entries that cache clears wholesale and every message on
        // screen is reparsed. Parsing the longest string in the catalogue
        // outright measures 1.5us, and the editor's input is different on every
        // keystroke, so the memo could never hit here: it bought nothing and
        // charged the hot path.
        collectArgs(parseNodes(message, 0, false).nodes, found);
        return [...found.values()];
    } catch {
        return [];
    }
}

/** CLDR's canonical order for plural categories, smallest count outwards. */
const CATEGORY_ORDER = ["zero", "one", "two", "few", "many", "other"] as const;

/**
 * The plural categories a locale actually uses, from `Intl.PluralRules` rather
 * than a table - English has two, Russian has four, Japanese has one. A
 * translation that omits one of these is missing a grammatical form, which is
 * a real defect the editor can point at.
 */
const CATEGORY_CACHE = new Map<string, string[]>();
const EXAMPLE_CACHE = new Map<string, IPluralExample[]>();

export function pluralCategoriesFor(locale: string, type: "plural" | "selectordinal" = "plural"): string[] {
    const hit = CATEGORY_CACHE.get(`${locale}:${type}`);
    if (hit) return hit;
    try {
        const rules = new Intl.PluralRules(locale, { type: type === "selectordinal" ? "ordinal" : "cardinal" });
        const categories = new Set(rules.resolvedOptions().pluralCategories);
        // `resolvedOptions` reports the categories ALPHABETICALLY, which puts
        // Russian's `few` and `many` ahead of `one` - an order no grammar book
        // uses and a confusing thing to hand a translator. CLDR's own order is
        // the one below, and `other` ends it because it is the fallback every
        // locale must have.
        const order = CATEGORY_ORDER.filter((c) => categories.has(c));
        CATEGORY_CACHE.set(`${locale}:${type}`, order);
        return order;
    } catch {
        return ["other"];
    }
}

/** How many integers each scanned category claims, and the first few of them. */
export interface IPluralExample {
    category: string;
    examples: number[];
}

const EXAMPLE_SCAN_LIMIT = 120;
const EXAMPLES_PER_CATEGORY = 4;

/**
 * Concrete numbers that land in each category, found by running the locale's
 * own rules over 0..120.
 *
 * This is the part a translator asked for in so many words: "one" and "other"
 * are opaque names, but "few: 2, 3, 4, 22" is a rule you can read. Scanning
 * beats a hand-written table because it cannot drift from the runtime that
 * will actually select the branch.
 */
export function pluralExamples(locale: string, type: "plural" | "selectordinal" = "plural"): IPluralExample[] {
    // A locale's plural rules do not change, so this is computed once per
    // locale for the life of the tab. Without it the editor re-ran 121
    // `Intl.PluralRules.select` calls on every keystroke, per argument.
    const cached = EXAMPLE_CACHE.get(`${locale}:${type}`);
    if (cached) return cached;
    const order = pluralCategoriesFor(locale, type);
    const found = new Map<string, number[]>(order.map((c) => [c, []]));
    try {
        const rules = new Intl.PluralRules(locale, { type: type === "selectordinal" ? "ordinal" : "cardinal" });
        for (let n = 0; n <= EXAMPLE_SCAN_LIMIT; n += 1) {
            const bucket = found.get(rules.select(n));
            if (bucket && bucket.length < EXAMPLES_PER_CATEGORY) bucket.push(n);
        }
    } catch {
        // Fall through with empty examples; the category list is still useful.
    }
    const result = order.map((category) => ({ category, examples: found.get(category) ?? [] }));
    EXAMPLE_CACHE.set(`${locale}:${type}`, result);
    return result;
}
