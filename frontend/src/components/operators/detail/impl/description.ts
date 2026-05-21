import type { IBlackboard } from "#/types/operators";

const DESCRIPTION_COLORS = {
    valueUp: "#6495ED",
    valueDown: "#ff847d",
    reminder: "#da9a46",
    keyword: "#27e8e7",
    potential: "#27e8e7",
    skillTooltip: "#27e8e7",
};

function colorForTag(tag: string): string {
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

// Nesting-aware: content cannot contain another opening (`<@` / `<$`). Applied
// repeatedly in renderTagsAndNewlines, this resolves innermost tags first so
// patterns like `<$scope><@cc.kw>text</></>` collapse correctly instead of
// leaking the inner opener and a stray `</>` as raw text.
const tagRegex = /<((?:@[a-z]+\.[a-z]+|\$[^>]+))>((?:(?!<[@$])[\s\S])*?)<\/>/g;
const interpolationRegex = /-?\{-?([^}:]+)(?::([^}]+))?\}/g;
const atomicTagRegex = /^<(?:@[a-z]+\.[a-z]+|\$[^>]+)>[\s\S]*?<\/>/;
const leadingWsRegex = /^\s+/;

function interpolateValues(text: string, blackboard: IBlackboard[] | { key: string; value: number }[]): string {
    const interp = new Map<string, number>();
    for (const b of blackboard) {
        if (b?.key) interp.set(b.key.toLowerCase(), Number.parseFloat(b.value.toPrecision(6)));
    }
    return text.replace(interpolationRegex, (_match, rawKey: string, format?: string) => {
        const key = rawKey.toLowerCase();
        const value = interp.get(key);
        if (value === undefined) return `[${key}]`;
        if (!format) return `${value}`;
        if (format === "0%") return `${Math.round(value * 100)}%`;
        if (format === "0.0") return value.toFixed(1);
        return `${value}`;
    });
}

function renderTagsAndNewlines(text: string): string {
    const replace = (s: string) =>
        s.replace(tagRegex, (_match, tag: string, content: string) => {
            return `<span style="color:${colorForTag(tag)}">${content}</span>`;
        });
    let prev = text;
    let curr = replace(text);
    while (curr !== prev) {
        prev = curr;
        curr = replace(curr);
    }
    return curr.replace(/\n/g, "<br/>");
}

export function descriptionToHtml(description: string, blackboard: IBlackboard[] | { key: string; value: number }[] = []): string {
    if (!description) return "";
    const escaped = description.replace(/&/g, "&amp;");
    return renderTagsAndNewlines(interpolateValues(escaped, blackboard));
}

type DiffOp = { type: "equal" | "delete" | "insert"; tokens: string[] };

function tokenizeRawForDiff(text: string): string[] {
    const out: string[] = [];
    let i = 0;
    while (i < text.length) {
        const rest = text.slice(i);
        const tagMatch = rest.match(atomicTagRegex);
        if (tagMatch) {
            out.push(tagMatch[0]);
            i += tagMatch[0].length;
            continue;
        }
        const wsMatch = rest.match(leadingWsRegex);
        if (wsMatch) {
            out.push(wsMatch[0]);
            i += wsMatch[0].length;
            continue;
        }
        let end = 1;
        while (end < rest.length) {
            const c = rest[end];
            if (/\s/.test(c)) break;
            if (c === "<" && atomicTagRegex.test(rest.slice(end))) break;
            end++;
        }
        out.push(rest.slice(0, end));
        i += end;
    }
    return out;
}

function lcsDiff(a: string[], b: string[]): DiffOp[] {
    const m = a.length;
    const n = b.length;
    const dp: Uint16Array[] = [];
    for (let i = 0; i <= m; i++) dp.push(new Uint16Array(n + 1));
    for (let i = 1; i <= m; i++) {
        for (let j = 1; j <= n; j++) {
            dp[i][j] = a[i - 1] === b[j - 1] ? dp[i - 1][j - 1] + 1 : Math.max(dp[i - 1][j], dp[i][j - 1]);
        }
    }
    const stack: { type: DiffOp["type"]; tok: string }[] = [];
    let i = m;
    let j = n;
    while (i > 0 || j > 0) {
        if (i > 0 && j > 0 && a[i - 1] === b[j - 1]) {
            stack.push({ type: "equal", tok: a[i - 1] });
            i--;
            j--;
        } else if (j > 0 && (i === 0 || dp[i][j - 1] >= dp[i - 1][j])) {
            stack.push({ type: "insert", tok: b[j - 1] });
            j--;
        } else {
            stack.push({ type: "delete", tok: a[i - 1] });
            i--;
        }
    }
    const ops: DiffOp[] = [];
    for (let k = stack.length - 1; k >= 0; k--) {
        const last = ops[ops.length - 1];
        if (last && last.type === stack[k].type) last.tokens.push(stack[k].tok);
        else ops.push({ type: stack[k].type, tokens: [stack[k].tok] });
    }
    return ops;
}

export function renderDescriptionDiffHtml(oldDesc: string | null | undefined, newDesc: string | null | undefined, oldBlackboard: IBlackboard[] | { key: string; value: number }[] = [], newBlackboard: IBlackboard[] | { key: string; value: number }[] = []): string {
    const newHtml = descriptionToHtml(newDesc ?? "", newBlackboard);
    if (!oldDesc) return newHtml;
    const oldHtml = descriptionToHtml(oldDesc, oldBlackboard);
    if (!oldHtml) return newHtml;
    if (!newHtml) return oldHtml;

    const oldPre = interpolateValues((oldDesc ?? "").replace(/&/g, "&amp;"), oldBlackboard);
    const newPre = interpolateValues((newDesc ?? "").replace(/&/g, "&amp;"), newBlackboard);

    const ops = lcsDiff(tokenizeRawForDiff(oldPre), tokenizeRawForDiff(newPre));
    return ops
        .map((op) => {
            const html = renderTagsAndNewlines(op.tokens.join(""));
            if (op.type === "equal") return html;
            if (op.type === "delete") return `<span class="rounded bg-red-500/15 px-0.5 text-red-300 line-through decoration-red-400/70">${html}</span>`;
            return `<span class="rounded bg-emerald-500/15 px-0.5 text-emerald-300">${html}</span>`;
        })
        .join("");
}
