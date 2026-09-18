/**
 * Word-level diff between the English a translation was written against and
 * the English it faces now.
 *
 * A stale row previously showed a badge and the NEW source, which tells a
 * translator that something moved but not what: on a long string the only safe
 * response was to re-translate the whole sentence. A diff turns most stale
 * rows into a one-word re-read.
 *
 * Tokens are words plus their trailing whitespace, so reassembling the spans
 * reproduces the original text exactly and a punctuation-only change still
 * shows up attached to its word.
 */

export type DiffOp = "same" | "added" | "removed";

export interface IDiffSpan {
    op: DiffOp;
    text: string;
}

/** Words with their trailing space, so `join("")` is lossless. */
function tokenize(text: string): string[] {
    return text.match(/\S+\s*|\s+/g) ?? [];
}

/**
 * Longest common subsequence over tokens.
 *
 * Source strings in this catalogue are UI copy - a few dozen tokens at the
 * outside - so the O(n*m) table is the right trade against the complexity of a
 * Myers implementation. The guard below is what keeps that true: a pathological
 * pair degrades to a plain replace rather than allocating a huge matrix.
 */
const MAX_TOKENS = 400;

export function diffWords(before: string, after: string): IDiffSpan[] {
    if (before === after) return after ? [{ op: "same", text: after }] : [];

    const a = tokenize(before);
    const b = tokenize(after);

    if (a.length > MAX_TOKENS || b.length > MAX_TOKENS) {
        const spans: IDiffSpan[] = [];
        if (before) spans.push({ op: "removed", text: before });
        if (after) spans.push({ op: "added", text: after });
        return spans;
    }

    // lcs[i][j] = length of the longest common subsequence of a[i..] and b[j..].
    const lcs: number[][] = Array.from({ length: a.length + 1 }, () => new Array<number>(b.length + 1).fill(0));
    for (let i = a.length - 1; i >= 0; i -= 1) {
        for (let j = b.length - 1; j >= 0; j -= 1) {
            lcs[i][j] = a[i] === b[j] ? lcs[i + 1][j + 1] + 1 : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
        }
    }

    const spans: IDiffSpan[] = [];
    const push = (op: DiffOp, text: string) => {
        const last = spans[spans.length - 1];
        if (last && last.op === op) last.text += text;
        else spans.push({ op, text });
    };

    let i = 0;
    let j = 0;
    while (i < a.length && j < b.length) {
        if (a[i] === b[j]) {
            push("same", a[i]);
            i += 1;
            j += 1;
        } else if (lcs[i + 1][j] >= lcs[i][j + 1]) {
            push("removed", a[i]);
            i += 1;
        } else {
            push("added", b[j]);
            j += 1;
        }
    }
    while (i < a.length) {
        push("removed", a[i]);
        i += 1;
    }
    while (j < b.length) {
        push("added", b[j]);
        j += 1;
    }

    return spans;
}
