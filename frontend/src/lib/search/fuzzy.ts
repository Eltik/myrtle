/**
 * Fuzzy scorer tuned for short search-palette items.
 *
 * Each candidate exposes a primary `name` (shown to the user, matched strongest),
 * optional `aliases` that are other names for the same thing and score on the
 * same tiers as `name` (an operator's appellation: `予愿安洁莉娜` is also
 * `Angelina the Mellow Wish`), and an auxiliary `extra` string that
 * concatenates every other searchable field (tags, keywords, profession, etc.)
 * for secondary matches.
 */

export interface IScoreTarget {
    name: string;
    aliases?: readonly string[];
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

const STRIP_PUNCT_RE = /['’.-]/g;

export function normalizeForSearch(input: string): string {
    return input
        .normalize("NFD")
        .replace(/\p{Diacritic}/gu, "")
        .replace(LATIN_FOLD_RE, (c) => LATIN_FOLD[c] ?? c)
        .replace(STRIP_PUNCT_RE, "")
        .toLowerCase();
}

/**
 * Whitespace-insensitive variant of {@link normalizeForSearch}: collapses all
 * spacing too, so `miss christine` matches `Miss.Christine` (-> `misschristine`)
 * regardless of how the user spaces or punctuates the query.
 */
export function compactForSearch(input: string): string {
    return normalizeForSearch(input).replace(/\s+/g, "");
}

const SCORE_NAME_EXACT = 1000;
const SCORE_NAME_PREFIX = 600;
const SCORE_NAME_WORD_PREFIX = 450;
const SCORE_NAME_CONTAINS = 300;
const SCORE_NAME_SUBSEQUENCE = 180;
const SCORE_EXTRA_CONTAINS = 90;

/**
 * A target with its haystacks normalised ONCE. `scoreMatch` normalises the
 * query and every field of the target on every call, which for the story
 * library is 451 targets carrying 1,887 story names re-folded per keystroke;
 * a caller that scores the same list against many queries prepares it once
 * with {@link prepareTarget} and scores with {@link scorePrepared}, which gives
 * the SAME number as `scoreMatch` for every query (pinned in `fuzzy.test.ts`).
 */
export interface IPreparedTarget {
    names: readonly IPreparedName[];
    extra: string;
    extraCompact: string;
}

interface IPreparedName {
    name: string;
    compact: string;
}

/** A query normalised once, for {@link scorePrepared}. `null` is the empty query. */
export interface IPreparedQuery {
    q: string;
    qc: string;
}

function prepareName(raw: string): IPreparedName {
    const name = normalizeForSearch(raw);
    return { name, compact: name.replace(/\s+/g, "") };
}

export function prepareTarget(target: IScoreTarget): IPreparedTarget {
    const extra = normalizeForSearch(target.extra ?? "");
    const names = [prepareName(target.name)];
    for (const alias of target.aliases ?? []) {
        if (alias.trim().length === 0) continue;
        names.push(prepareName(alias));
    }
    return { names, extra, extraCompact: extra.replace(/\s+/g, "") };
}

export function prepareQuery(query: string): IPreparedQuery | null {
    const q = normalizeForSearch(query.trim());
    if (q.length === 0) return null;
    return { q, qc: q.replace(/\s+/g, "") };
}

/** {@link scoreMatch} over a prepared target and query. `null` (the empty query) scores 1, as there. */
export function scorePrepared(query: IPreparedQuery | null, target: IPreparedTarget): number {
    if (query === null) return 1;
    const { q, qc } = query;
    // Best of the name and its aliases: an alias is another spelling of the
    // same name, so it earns the same tiers, and the strongest one wins.
    let best = 0;
    for (const name of target.names) best = Math.max(best, scoreName(q, qc, name.name, name.compact));
    if (best > 0) return best;

    // `extra` gets contiguous-substring matching only. Subsequence matching is
    // reserved for `name`: `extra` concatenates unrelated fields, so a
    // subsequence can assemble its letters across field boundaries ("myr" out
    // of "amiya caster …"), which matches nothing meaningful.
    if (target.extra.length > 0) {
        if (target.extra.includes(q)) return SCORE_EXTRA_CONTAINS;
        if (target.extraCompact.includes(qc)) return SCORE_EXTRA_CONTAINS;
    }

    return 0;
}

/**
 * Returns a score ≥ 0 for how well `target` matches `query`. 0 means "no match".
 * An empty query returns a small positive score so the list still renders.
 */
export function scoreMatch(query: string, target: IScoreTarget): number {
    return scorePrepared(prepareQuery(query), prepareTarget(target));
}

function scoreName(q: string, qc: string, name: string, nameCompact: string): number {
    if (name === q) return SCORE_NAME_EXACT + lengthBonus(name);
    if (name.startsWith(q)) return SCORE_NAME_PREFIX + lengthBonus(name);

    const wordIdx = findWordPrefix(name, q);
    if (wordIdx >= 0) return SCORE_NAME_WORD_PREFIX - wordIdx + lengthBonus(name);

    const nameIdx = name.indexOf(q);
    if (nameIdx >= 0) return SCORE_NAME_CONTAINS - nameIdx + lengthBonus(name);

    const ncIdx = nameCompact.indexOf(qc);
    if (ncIdx >= 0) return SCORE_NAME_CONTAINS - ncIdx + lengthBonus(name);

    if (isSubsequence(q, name)) return SCORE_NAME_SUBSEQUENCE + lengthBonus(name);
    return 0;
}

/** Scores and filters an array, returning matches sorted best-first. */
export function searchAndRank<T>(query: string, items: readonly T[], getTarget: (item: T) => IScoreTarget, limit?: number): IScored<T>[] {
    const out: IScored<T>[] = [];
    const prepared = prepareQuery(query);
    for (const item of items) {
        const score = scorePrepared(prepared, prepareTarget(getTarget(item)));
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
