/**
 * Server-side input sanitizer for markdown fields.
 *
 * Runs on TanStack Start server functions (Node/Nitro) before forwarding to the backend.
 * Defense in depth: the Markdown renderer is already XSS-safe by construction (no
 * `dangerouslySetInnerHTML`), but we strip dangerous content at the storage boundary so:
 *   - third-party consumers of our API can't be tricked by raw HTML in our data
 *   - the stored value never contains active payloads (`<script>`, `javascript:` …)
 *   - data integrity stays clean (no NULs / control characters)
 *
 * This is intentionally not a CommonMark validator. It's a focused stripper
 * for the specific threats markdown text can carry, with everything else
 * passed through untouched.
 */

const ALLOWED_LINK_SCHEMES = new Set(["http:", "https:", "mailto:"]);

// Matches HTML opening / closing tags: <tag>, <tag attr="x">, </tag>.
// Critically does NOT match autolinks like <https://example.com> (their `:` after
// the tag name doesn't satisfy the trailing `(\s[^>]*)?` group).
const HTML_TAG_RE = /<\/?[a-zA-Z][a-zA-Z0-9-]*(\s[^>]*)?>/g;
const HTML_COMMENT_RE = /<!--[\s\S]*?-->/g;
const HTML_CDATA_RE = /<!\[CDATA\[[\s\S]*?\]\]>/g;
const PROCESSING_INSTRUCTION_RE = /<\?[\s\S]*?\?>/g;

// Control characters except \n (0x0A) and \t (0x09).
// biome-ignore lint/suspicious/noControlCharactersInRegex: stripping control chars is the whole point
const CONTROL_CHARS_RE = /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]/g;

// Markdown link target: `]( url )`
const MARKDOWN_LINK_TARGET_RE = /\]\(\s*([^)\s]+)(\s+"[^"]*")?\s*\)/g;

// HTML/HEX entity decode for the scheme check: someone could write `[x](&#x6A;avascript:…)`
// to evade the scheme check. We decode entities ONLY for the check, not for storage.
const NUMERIC_ENTITY_RE = /&#(x?)([0-9a-fA-F]+);/g;
const NAMED_ENTITY_RE = /&([a-zA-Z]+);/g;
const NAMED_ENTITIES: Record<string, string> = {
    amp: "&",
    lt: "<",
    gt: ">",
    quot: '"',
    apos: "'",
    nbsp: " ",
};

const DEFAULT_MAX_LENGTH = 10_000;

export interface ISanitizeOptions {
    /** Hard cap on stored length. Defaults to 10,000. */
    maxLength?: number;
    /**
     * When true, returns null if the input was non-empty but became empty after
     * sanitization (e.g., the string was nothing but stripped tags). Defaults false.
     */
    nullOnEmpty?: boolean;
}

/**
 * Sanitize text destined for storage as markdown.
 *
 * - null / undefined -> null (unchanged sentinel)
 * - Normalizes CRLF -> LF
 * - Strips control characters (except `\n`, `\t`)
 * - Strips HTML tags, comments, CDATA blocks, processing instructions
 * - Replaces dangerous URL schemes in `[text](url)` links with `#`
 * - Drops autolinks with non-allowlisted schemes
 * - Collapses runs of 3+ blank lines
 * - Truncates to maxLength
 */
export function sanitizeMarkdownForStorage(input: string | null | undefined, opts: ISanitizeOptions = {}): string | null {
    if (input == null) return null;
    if (input === "") return "";

    const max = opts.maxLength ?? DEFAULT_MAX_LENGTH;

    let s = String(input);

    // Normalize line endings first so subsequent regexes only deal with \n.
    s = s.replace(/\r\n?/g, "\n");

    s = s.replace(CONTROL_CHARS_RE, "");

    // Strip HTML structures. Order matters: comments and CDATA can wrap arbitrary
    // content (including angle brackets) so they're stripped first.
    s = s.replace(HTML_COMMENT_RE, "");
    s = s.replace(HTML_CDATA_RE, "");
    s = s.replace(PROCESSING_INSTRUCTION_RE, "");
    s = s.replace(HTML_TAG_RE, "");

    s = s.replace(MARKDOWN_LINK_TARGET_RE, (_match, rawURL: string, title: string | undefined) => {
        const decoded = decodeEntities(rawURL);
        return isAllowedLinkTarget(decoded) ? `](${rawURL}${title ?? ""})` : "](#)";
    });

    s = s.replace(/<([a-zA-Z][a-zA-Z0-9+.-]*):([^>\s]+)>/g, (match, scheme: string) => {
        const lower = `${scheme.toLowerCase()}:`;
        return ALLOWED_LINK_SCHEMES.has(lower) ? match : "";
    });

    s = s.replace(/\n{3,}/g, "\n\n");

    s = s.replace(/^\s+|\s+$/g, "");

    if (s.length > max) s = s.slice(0, max).replace(/\s+$/, "");

    if (s.length === 0 && opts.nullOnEmpty) return null;
    return s;
}

/**
 * Sanitize a plain-text "name" field (titles, tier labels). Strips HTML and
 * control characters, collapses internal whitespace, truncates. Returns the
 * trimmed string (or "" if it became empty).
 */
export function sanitizePlainName(input: string | null | undefined, maxLength: number): string {
    if (input == null) return "";
    const s = String(input).replace(/\r\n?/g, " ").replace(CONTROL_CHARS_RE, "").replace(HTML_COMMENT_RE, "").replace(HTML_TAG_RE, "").replace(/\s+/g, " ").trim();
    return truncateCodePoints(s, maxLength).trimEnd();
}

/** The first `max` code points of `s`. A UTF-16 `slice` can cut a surrogate pair in half; the backend counts chars, which are code points. */
export function truncateCodePoints(s: string, max: number): string {
    if (s.length <= max) return s;
    const points = Array.from(s);
    return points.length <= max ? s : points.slice(0, max).join("");
}

function isAllowedLinkTarget(href: string): boolean {
    const trimmed = href.trim();
    if (!trimmed) return false;
    // Relative URLs, anchors, query-only - always safe.
    if (trimmed.startsWith("/") || trimmed.startsWith("#") || trimmed.startsWith("?") || trimmed.startsWith("./") || trimmed.startsWith("../")) return true;
    const schemeMatch = /^([a-z][a-z0-9+.-]*):/i.exec(trimmed);
    if (schemeMatch?.[1]) {
        return ALLOWED_LINK_SCHEMES.has(`${schemeMatch[1].toLowerCase()}:`);
    }
    // No scheme = treat as relative.
    return true;
}

function decodeEntities(s: string): string {
    let out = s.replace(NUMERIC_ENTITY_RE, (_m, hex: string, code: string) => {
        const num = hex ? Number.parseInt(code, 16) : Number.parseInt(code, 10);
        if (!Number.isFinite(num) || num < 0 || num > 0x10ffff) return "";
        try {
            return String.fromCodePoint(num);
        } catch {
            return "";
        }
    });
    out = out.replace(NAMED_ENTITY_RE, (m, name: string) => NAMED_ENTITIES[name.toLowerCase()] ?? m);
    return out;
}
