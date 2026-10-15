/**
 * Paths earlier versions of the site served, mapped to the page that replaced them. Each
 * one is still linked from outside: `awesome-arknights` lists `/dps-calculator`, and
 * search engines still index `/operators?id=char_1014_nearl2`. Without this they 404.
 *
 * Sources, all from this repo's history: the Next.js `pages/` tree before 120a0f05
 * (`/dps-calculator`, `/recruitment-calculator`, `/squad-randomizer`), the one before
 * f153b6a2 (`/operators?id=`, `/tools/enemies`, `/users/*`), and the one before
 * dff01d67 (`/collection/*`, `/tier-list`). `/my/*` is left out on purpose: its first
 * segment parses as a locale (`my`), so the locale redirect claims it first.
 */
const LEGACY_PATHS: Readonly<Record<string, string>> = {
    "/dps-calculator": "/tools/dps",
    "/recruitment-calculator": "/tools/recruitment",
    "/squad-randomizer": "/tools/randomizer",
    "/tools/enemies": "/enemies",
    "/collection/enemies": "/enemies",
    "/collection/operators": "/operators",
    "/tier-list": "/tier-lists",
    "/users/leaderboard": "/user/leaderboard",
    "/users/search": "/user/search",
};

/** Both list pages that once took the operator as `?id=`. */
const OPERATOR_QUERY_PATHS = new Set(["/operators", "/collection/operators"]);

/** An operator id as the game writes it, so a crafted `?id=` cannot steer the redirect off-site. */
const OPERATOR_ID = /^[a-z0-9_]+$/i;

/**
 * The page a legacy URL moved to, as a same-origin path, or null when `pathname` is not a
 * legacy one. `search` is the router's parsed search object.
 */
export function legacyRedirect(pathname: string, search: Record<string, unknown>): string | null {
    const path = pathname.length > 1 ? pathname.replace(/\/+$/, "") : pathname;
    if (OPERATOR_QUERY_PATHS.has(path)) {
        const id = search.id;
        if (typeof id === "string" && OPERATOR_ID.test(id)) return `/operators/${id}`;
    }
    return LEGACY_PATHS[path] ?? null;
}
