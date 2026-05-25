/**
 * Fuzzy scorer tuned for short search-palette items.
 *
 * Each candidate exposes a primary `name` (shown to the user, matched strongest)
 * and an auxiliary `extra` string that concatenates every other searchable field
 * (tags, keywords, profession, appellation, etc.) for secondary matches.
 */

export interface IScoreTarget {
    name: string;
    extra?: string;
}

export interface IScored<T> {
    item: T;
    score: number;
}

// Non-decomposable Latin letters: NFD leaves these as single base codepoints,
// so the diacritic-strip regex below doesn't touch them. Map them manually.
const LATIN_FOLD: Record<string, string> = {
    ł: "l",
    Ł: "L",
    ø: "o",
    Ø: "O",
    æ: "ae",
    Æ: "AE",
    œ: "oe",
    Œ: "OE",
    ß: "ss",
    đ: "d",
    Đ: "D",
    ð: "d",
    Ð: "D",
    þ: "th",
    Þ: "Th",
    ı: "i",
    İ: "I",
};
const LATIN_FOLD_RE = new RegExp(`[${Object.keys(LATIN_FOLD).join("")}]`, "g");

/**
 * Folds a string into a diacritic-free, lowercase form suitable for search
 * comparison: `Młynar` → `mlynar`, `Vigïl` → `vigil`, `Ægir` → `aegir`.
 */
export function normalizeForSearch(input: string): string {
    return input
        .normalize("NFD")
        .replace(/\p{Diacritic}/gu, "")
        .replace(LATIN_FOLD_RE, (c) => LATIN_FOLD[c] ?? c)
        .toLowerCase();
}

const SCORE_NAME_EXACT = 1000;
const SCORE_NAME_PREFIX = 600;
const SCORE_NAME_WORD_PREFIX = 450;
const SCORE_NAME_CONTAINS = 300;
const SCORE_NAME_SUBSEQUENCE = 180;
const SCORE_EXTRA_CONTAINS = 90;
const SCORE_EXTRA_SUBSEQUENCE = 40;

/**
 * Returns a score ≥ 0 for how well `target` matches `query`. 0 means "no match".
 * An empty query returns a small positive score so the list still renders.
 */
export function scoreMatch(query: string, target: IScoreTarget): number {
    const q = normalizeForSearch(query.trim());
    if (q.length === 0) return 1;

    const name = normalizeForSearch(target.name);
    const extra = normalizeForSearch(target.extra ?? "");

    if (name === q) return SCORE_NAME_EXACT + lengthBonus(name);
    if (name.startsWith(q)) return SCORE_NAME_PREFIX + lengthBonus(name);

    const wordIdx = findWordPrefix(name, q);
    if (wordIdx >= 0) return SCORE_NAME_WORD_PREFIX - wordIdx + lengthBonus(name);

    const nameIdx = name.indexOf(q);
    if (nameIdx >= 0) return SCORE_NAME_CONTAINS - nameIdx + lengthBonus(name);

    if (isSubsequence(q, name)) return SCORE_NAME_SUBSEQUENCE + lengthBonus(name);

    if (extra.length > 0) {
        if (extra.includes(q)) return SCORE_EXTRA_CONTAINS;
        if (isSubsequence(q, extra)) return SCORE_EXTRA_SUBSEQUENCE;
    }

    return 0;
}

/** Scores and filters an array, returning matches sorted best-first. */
export function searchAndRank<T>(query: string, items: readonly T[], getTarget: (item: T) => IScoreTarget, limit?: number): IScored<T>[] {
    const out: IScored<T>[] = [];
    for (const item of items) {
        const score = scoreMatch(query, getTarget(item));
        if (score > 0) out.push({ item, score });
    }
    out.sort((a, b) => b.score - a.score);
    return typeof limit === "number" ? out.slice(0, limit) : out;
}

function isSubsequence(needle: string, haystack: string): boolean {
    let i = 0;
    for (let j = 0; j < haystack.length && i < needle.length; j++) {
        if (haystack.charCodeAt(j) === needle.charCodeAt(i)) i++;
    }
    return i === needle.length;
}

function findWordPrefix(name: string, q: string): number {
    let i = 0;
    while (i < name.length) {
        if (i === 0 || isBoundary(name.charCodeAt(i - 1))) {
            if (name.startsWith(q, i)) return i;
        }
        i++;
    }
    return -1;
}

function isBoundary(code: number): boolean {
    return code === 32 || code === 45 || code === 95 || code === 46; // space, -, _, .
}

function lengthBonus(name: string): number {
    return Math.max(0, 20 - Math.min(name.length, 20));
}
