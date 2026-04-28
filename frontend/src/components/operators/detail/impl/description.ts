import type { IBlackboard } from "#/types/operators";

const DESCRIPTION_COLORS = {
    valueUp: "#d597da",
    valueDown: "#ff847d",
    reminder: "#da9a46",
    keyword: "#49b3ff",
    potential: "#49b3ff",
    skillTooltip: "#49b3ff",
};

function colorForTag(tag: string): string {
    if (tag === "@ba.vup") return DESCRIPTION_COLORS.valueUp;
    if (tag === "@ba.vdown") return DESCRIPTION_COLORS.valueDown;
    if (tag === "@ba.rem") return DESCRIPTION_COLORS.reminder;
    if (tag === "@ba.kw") return DESCRIPTION_COLORS.keyword;
    if (tag === "@ba.talpu") return DESCRIPTION_COLORS.potential;
    if (tag.startsWith("$")) return DESCRIPTION_COLORS.skillTooltip;
    return DESCRIPTION_COLORS.keyword;
}

const tagRegex = /<((?:@ba\.[a-z]+|\$[^>]+))>([\s\S]*?)<\/>/g;
const interpolationRegex = /-?\{-?([^}:]+)(?::([^}]+))?\}/g;

export function descriptionToHtml(description: string, blackboard: IBlackboard[] | { key: string; value: number }[] = []): string {
    if (!description) return "";

    const interp = new Map<string, number>();
    for (const b of blackboard) {
        if (b?.key) interp.set(b.key.toLowerCase(), Number.parseFloat(b.value.toPrecision(6)));
    }

    let html = description.replace(/&/g, "&amp;");

    html = html.replace(tagRegex, (_match, tag: string, content: string) => {
        return `<span style="color:${colorForTag(tag)}">${content}</span>`;
    });

    html = html.replace(interpolationRegex, (_match, rawKey: string, format?: string) => {
        const key = rawKey.toLowerCase();
        const value = interp.get(key);
        if (value === undefined) return `[${key}]`;
        if (!format) return `${value}`;
        if (format === "0%") return `${Math.round(value * 100)}%`;
        if (format === "0.0") return value.toFixed(1);
        return `${value}`;
    });

    html = html.replace(/\n/g, "<br/>");
    return html;
}

type DiffOp = { type: "equal" | "delete" | "insert"; tokens: string[] };

function tokenizeForDiff(html: string): string[] {
    const re = /<[^>]+>|\s+|[^\s<]+/g;
    const out: string[] = [];
    let m: RegExpExecArray | null;
    // biome-ignore lint/suspicious/noAssignInExpressions: standard regex iteration
    while ((m = re.exec(html)) !== null) out.push(m[0]);
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

    const ops = lcsDiff(tokenizeForDiff(oldHtml), tokenizeForDiff(newHtml));
    return ops
        .map((op) => {
            const segment = op.tokens.join("");
            if (op.type === "equal") return segment;
            if (op.type === "delete") return `<span class="rounded bg-red-500/15 px-0.5 text-red-300 line-through decoration-red-400/70">${segment}</span>`;
            return `<span class="rounded bg-emerald-500/15 px-0.5 text-emerald-300">${segment}</span>`;
        })
        .join("");
}
