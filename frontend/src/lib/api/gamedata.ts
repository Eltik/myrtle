/**
 * Which Arknights client the game-data endpoints should read.
 *
 * Every game-data route on the backend exists in two shapes: `/static/operators`
 * (the deployment's default client) and `/{server}/static/operators` (an explicit
 * one). The i18n manifest pins each locale to a client via `gamedata_server`, so
 * `/ja/operators` can show Japanese operator names instead of English ones under
 * translated chrome.
 *
 * Two rules keep this from breaking English:
 *
 * 1. {@link gamedataPath} returns the *unprefixed* path for the default server,
 *    so an English render issues byte-identical requests to what it issued
 *    before this module existed.
 * 2. {@link gamedataKey} contributes the server to every game-data query key -
 *    and contributes *nothing* for the default server, so English cache entries
 *    keep the keys they already had while `ja` (→ `jp`) can never be served out
 *    of them.
 */

/** The game regions the backend can serve, matching its `Server` enum. */
export const GAMEDATA_SERVERS = ["en", "jp", "kr", "cn", "tw", "bili"] as const;

export type GamedataServer = (typeof GAMEDATA_SERVERS)[number];

/**
 * The server reached by the unprefixed routes. This is `en` because that is
 * what the deployment's `SERVERS`/`BIN_SERVER` default to and what every call
 * site assumed before `gamedata_server` was wired up.
 */
export const DEFAULT_GAMEDATA_SERVER: GamedataServer = "en";

export function isGamedataServer(value: string | null | undefined): value is GamedataServer {
    return value != null && (GAMEDATA_SERVERS as readonly string[]).includes(value);
}

/**
 * Narrow an arbitrary string (a manifest value, or a `server` that crossed a
 * server-function boundary) to a server we can actually route to.
 *
 * An unrecognised value collapses to the default rather than throwing: a
 * mis-typed `gamedata_server` row should degrade to English data, not take the
 * page down.
 */
export function resolveGamedataServer(value: string | null | undefined): GamedataServer {
    return isGamedataServer(value) ? value : DEFAULT_GAMEDATA_SERVER;
}

/**
 * The backend path for one game-data resource on one server.
 *
 * `gamedataPath("en", "/operators/index")` → `/operators/index`
 * `gamedataPath("jp", "/operators/index")` → `/jp/operators/index`
 */
export function gamedataPath(server: string | null | undefined, path: string): string {
    const resolved = resolveGamedataServer(server);
    const suffix = path.startsWith("/") ? path : `/${path}`;
    return resolved === DEFAULT_GAMEDATA_SERVER ? suffix : `/${resolved}${suffix}`;
}

/**
 * The server segment of a query key.
 *
 * Spread it into the key (`["operators", "index", ...gamedataKey(server)]`).
 * It is empty for the default server on purpose - that keeps every English key
 * byte-identical to the one it had before servers were threaded through, while
 * still guaranteeing that no two servers can share a cache entry.
 */
export function gamedataKey(server: string | null | undefined): readonly GamedataServer[] {
    const resolved = resolveGamedataServer(server);
    return resolved === DEFAULT_GAMEDATA_SERVER ? [] : [resolved];
}

/**
 * Which server one operator's data lives on.
 *
 * `/operators/{id}` tags every operator with the client it resolved on, and a
 * CN-exclusive operator exists *only* there - so that pin wins over the locale.
 * Everything else follows the locale's server, which is what makes a `ja`
 * render show Japanese names instead of English ones.
 */
export function operatorGamedataServer(operatorServer: string | null | undefined, localeServer: string | null | undefined): GamedataServer {
    return operatorServer === "cn" ? "cn" : resolveGamedataServer(localeServer);
}
