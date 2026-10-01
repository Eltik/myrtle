import type { DisplayUser } from "./types";

/**
 * `locale` defaults to `undefined`, which `Intl.Collator` reads as the browser's locale. This is
 * a pure helper with no hooks, so the page's locale arrives as an argument (callers pass
 * `useLocale()`).
 */
export function sortBySubstringMatch(users: DisplayUser[], query: string, locale?: string): DisplayUser[] {
    const q = query.trim().toLowerCase();
    if (!q) return users;

    const collator = new Intl.Collator(locale, { sensitivity: "base", numeric: true });

    const rank = (user: DisplayUser): number => {
        const name = (user.nickname ?? "").toLowerCase();
        if (!name) return 2;
        if (name.startsWith(q)) return 0;
        if (name.includes(q)) return 1;
        return 2;
    };

    return [...users].sort((a, b) => {
        const ra = rank(a);
        const rb = rank(b);
        if (ra !== rb) return ra - rb;
        return collator.compare(a.nickname ?? "", b.nickname ?? "");
    });
}
