const ALLOWED_SCHEMES = ["http:", "https:", "mailto:"] as const;

/**
 * Returns the URL if it's a safe link target, otherwise null.
 * Allows absolute http(s)/mailto links and relative paths (/foo, #anchor, foo/bar).
 * Rejects javascript:, data:, vbscript:, etc.
 */
export function sanitizeHref(raw: string): string | null {
    const href = raw.trim();
    if (!href) return null;

    // Relative paths / anchors / query-only
    if (href.startsWith("/") || href.startsWith("#") || href.startsWith("?") || href.startsWith("./") || href.startsWith("../")) {
        return href;
    }

    // Scheme check: anything before the first `:` (no `/` between) is a scheme.
    const schemeMatch = /^([a-z][a-z0-9+.-]*):/i.exec(href);
    if (schemeMatch) {
        const scheme = schemeMatch[1]?.toLowerCase();
        if (!scheme) return null;
        return (ALLOWED_SCHEMES as readonly string[]).includes(`${scheme}:`) ? href : null;
    }

    // No scheme and not a relative path - treat as relative.
    return href;
}
