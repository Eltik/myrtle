/**
 * A small ICU MessageFormat subset: `{arg}`, `{arg, plural, ...}`,
 * `{arg, select, ...}`, `{arg, number}`, `{arg, date}`, `#` inside a plural
 * branch, and `'` escaping.
 *
 * Hand-written instead of `intl-messageformat`: the backend already rejects
 * undeclared arguments and unbalanced braces, so only what can be stored needs
 * covering, and the hot render path stays dependency-free.
 *
 * Plural selection is `Intl.PluralRules`, so every locale's real rules apply
 * (Polish has four, Japanese has one) without a rules table here.
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
            // ICU 4.8 "real literal" quoting, what every ICU ships today:
            //
            //   ''            -> one literal apostrophe
            //   '{ '} '# '|   -> starts a quoted run, ended by the next '
            //   ' anywhere else -> a plain apostrophe, NOT a quote
            //
            // The last clause matters. Treating every apostrophe as a quote
            // start silently ate them: "Most E2'd" rendered as "Most E2d", and
            // "What's public, and what isn't" lost both and swallowed the text
            // between. That hit every English contraction, with no error.
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
            // Unclosed brace. An argument here would blank the whole tail of a
            // malformed stored translation; keep the raw text so the damage is
            // visible and bounded.
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

/**
 * One argument of a message, for the translation editor to explain. Exported
 * from the parser so a component never re-derives plural/select branches with
 * a regex.
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
 * number, `{name}` for a nested argument. Display only, never parsed back.
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
        // `{count, number} {count, plural, ...}` is the catalogue's commonest
        // shape, so a name can repeat. Keep the branching occurrence: a bare
        // mention never overwrites it, a branching one replaces a bare one.
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
        // `parseNodes`, not `parse`: the editor calls this per keystroke, and
        // the memo would fill the RENDER path's cache with dead entries until
        // it clears wholesale at 5000 and reparses everything on screen. The
        // memo can never hit here anyway; the longest catalogue string parses
        // in 1.5us.
        collectArgs(parseNodes(message, 0, false).nodes, found);
        return [...found.values()];
    } catch {
        return [];
    }
}

/** CLDR's canonical order for plural categories, smallest count outwards. */
const CATEGORY_ORDER = ["zero", "one", "two", "few", "many", "other"] as const;

/**
 * The plural categories a locale uses, from `Intl.PluralRules` (English two,
 * Russian four, Japanese one). A translation missing one is missing a
 * grammatical form, which the editor flags.
 */
const CATEGORY_CACHE = new Map<string, string[]>();
const EXAMPLE_CACHE = new Map<string, IPluralExample[]>();

export function pluralCategoriesFor(locale: string, type: "plural" | "selectordinal" = "plural"): string[] {
    const hit = CATEGORY_CACHE.get(`${locale}:${type}`);
    if (hit) return hit;
    try {
        const rules = new Intl.PluralRules(locale, { type: type === "selectordinal" ? "ordinal" : "cardinal" });
        const categories = new Set(rules.resolvedOptions().pluralCategories);
        // `resolvedOptions` reports categories ALPHABETICALLY, putting Russian's
        // `few` and `many` ahead of `one`. Reorder to CLDR's order, `other`
        // last as every locale's fallback.
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
 * Concrete numbers that land in each category, from the locale's own rules
 * over 0..120. A translator asked for this: "few: 2, 3, 4, 22" reads where
 * "few" does not. Scanning cannot drift from the runtime that selects the
 * branch, a hand table could.
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
