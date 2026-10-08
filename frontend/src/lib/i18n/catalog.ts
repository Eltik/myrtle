import { env } from "#/env";
import { DEFAULT_GAMEDATA_SERVER } from "#/lib/api/gamedata";
import type { I18nManifest } from "#/types/generated/I18nManifest";

export type Catalog = Record<string, string>;

/**
 * The every-namespace catalog. The backend splits catalogs by namespace and
 * the manifest carries a hash per namespace, so per-route lazy loading is
 * available the moment it is worth doing, but until the catalog is measured
 * large enough to need it, one request for everything is fewer moving parts
 * and one fewer round trip on first paint.
 */
export const ALL_NAMESPACES = "-";

interface ICacheEntry {
    catalog: Catalog;
    hash: string;
}

/**
 * Module-scope caches in the SSR process, mirroring the `operatorsListCache`
 * pattern already used for the operator index. A page render is then a map
 * lookup rather than a backend round trip.
 *
 * Catalogs are keyed by content hash, so an entry can never be stale; it can
 * only become unreferenced once the manifest publishes a new hash.
 */
const catalogCache = new Map<string, ICacheEntry>();
let manifestCache: { manifest: I18nManifest; fetchedAt: number } | null = null;

/**
 * How long the SSR process may serve a remembered manifest. This is the whole
 * latency budget between a translator pressing Save and the next server render
 * showing the new text, so it is deliberately short; the backend's own
 * manifest cache and `Cache-Control` are 30s for the same reason.
 */
const MANIFEST_TTL_MS = 30_000;

function apiBase(): string {
    return (typeof window === "undefined" ? env.BACKEND_URL : env.VITE_BACKEND_URL) ?? "";
}

export async function fetchManifest(force = false): Promise<I18nManifest | null> {
    const now = Date.now();
    if (!force && manifestCache && now - manifestCache.fetchedAt < MANIFEST_TTL_MS) {
        return manifestCache.manifest;
    }

    try {
        const res = await fetch(`${apiBase()}/api/i18n/manifest`);
        if (!res.ok) return manifestCache?.manifest ?? null;
        const manifest = (await res.json()) as I18nManifest;
        manifestCache = { manifest, fetchedAt: now };
        return manifest;
    } catch {
        // A backend blip must not take the site down: an un-translated page is
        // a worse page, not a broken one, because the bundled source catalog
        // still renders English.
        return manifestCache?.manifest ?? null;
    }
}

/**
 * Fetch one catalog by content hash. The URL contains the hash, so the
 * response is `immutable` and every layer between here and Postgres (the
 * backend cache, Cloudflare, the browser) can hold it forever without any
 * purge step. This project has no Cloudflare purge automation at all, which is
 * exactly why the hash is in the path.
 */
export async function fetchCatalog(locale: string, hash: string, namespace: string = ALL_NAMESPACES): Promise<Catalog> {
    const cacheKey = `${locale}:${namespace}:${hash}`;
    const hit = catalogCache.get(cacheKey);
    if (hit) return hit.catalog;

    try {
        const res = await fetch(`${apiBase()}/api/i18n/${encodeURIComponent(locale)}/${encodeURIComponent(namespace)}/${encodeURIComponent(hash)}`);
        if (!res.ok) return {};
        const catalog = (await res.json()) as Catalog;
        // Bound the SSR process: locales are few, but a long-lived process
        // that has served many hashes should not keep every superseded one.
        if (catalogCache.size > 32) catalogCache.clear();
        catalogCache.set(cacheKey, { catalog, hash });
        return catalog;
    } catch {
        return {};
    }
}

export interface IBootstrap {
    locale: string;
    /**
     * Where to send the visitor when the path claimed a locale the backend
     * does not serve: `/ja/operators` when `ja` is missing, disabled, or the
     * backend predates it.
     *
     * Without this the request renders English under a `/ja` URL with
     * `lang="en"`, which is indistinguishable from "my translation did not
     * apply".
     */
    redirectTo: string | null;
    /** The switcher's entries. */
    available: IAvailableLocale[];
    messages: Catalog;
    /** The hash the messages came from, so the client can refetch by URL. */
    hash: string;
    /**
     * The Arknights client whose text serves this locale's game data
     * (operator names, skill descriptions, stage text), so the game-data API
     * layer can pick the `/{server}/...` endpoints (see `lib/api/gamedata.ts`).
     * The visitor's own pick (the `gamedata_server` cookie) wins over the
     * locale's row when the backend has that server loaded; see
     * {@link pickGamedataServer}.
     */
    gamedataServer: string;
    /** The server this locale reads when the visitor has not picked one. */
    localeGamedataServer: string;
    /** Whether {@link gamedataServer} came from the visitor's pick. */
    gamedataServerPicked: boolean;
    /**
     * The servers the backend has loaded, the default first: the picker's
     * entries. Empty when the backend predates the field, which hides the
     * picker and leaves the locale's row in charge.
     */
    gamedataServers: string[];
}

export interface IAvailableLocale {
    code: string;
    nativeName: string;
    /**
     * Share of the source catalog this locale translates, 0 to 1, floored to a
     * whole percent so an unfinished locale never reads as 100%. Absent for the
     * source locale, which is not a translation of anything.
     */
    completion?: number;
}

export interface IGamedataServerChoice {
    server: string;
    localeServer: string;
    picked: boolean;
}

/**
 * Which server's game data a render reads.
 *
 * The visitor's pick wins, then the locale's row, then the backend's default
 * (its first loaded server, else English). The pick and the row each count
 * only when the backend reports that server loaded, which is what keeps a
 * `ko -> kr` row from sending every Korean visitor to a server that answers
 * 404 on a deployment without KR data.
 *
 * `loaded === undefined` is a backend older than the `gamedata_servers` field.
 * The locale's row is then trusted as-is and the pick ignored.
 */
export function pickGamedataServer(localeRow: string | undefined, picked: string | undefined, loaded: string[] | undefined): IGamedataServerChoice {
    const row = localeRow ?? DEFAULT_GAMEDATA_SERVER;
    if (loaded === undefined) return { server: row, localeServer: row, picked: false };

    const fallback = loaded[0] ?? DEFAULT_GAMEDATA_SERVER;
    const localeServer = loaded.includes(row) ? row : fallback;
    if (picked && loaded.includes(picked)) return { server: picked, localeServer, picked: true };
    return { server: localeServer, localeServer, picked: false };
}

/**
 * Everything a render needs to show text in one locale. Falls back to an empty
 * catalog, not an error, when the backend cannot be reached, because the
 * bundled source catalog covers English either way.
 */
export async function loadBootstrap(requested: string, fallback: string, claimedPath?: string, pickedServer?: string): Promise<IBootstrap> {
    const manifest = await fetchManifest();

    const available = (manifest?.locales ?? []).map(
        (l): IAvailableLocale => ({
            code: l.code,
            nativeName: l.native_name,
            // `!l.total` also covers a backend older than these fields.
            completion: l.code === manifest?.default_locale || !l.total ? undefined : Math.floor((l.translated / l.total) * 100) / 100,
        }),
    );

    const entry = manifest?.locales.find((l) => l.code === requested) ?? manifest?.locales.find((l) => l.code === fallback);

    // The path claimed a locale and the backend does not serve it. Send the
    // visitor to the unprefixed URL rather than rendering English at a URL
    // that promises otherwise.
    const served = manifest?.locales.some((l) => l.code === requested) ?? false;
    const redirectTo = !served && claimedPath !== undefined ? claimedPath : null;

    const loaded = manifest?.gamedata_servers;
    const choice = pickGamedataServer(entry?.gamedata_server, pickedServer, loaded);
    const servers = { gamedataServer: choice.server, localeGamedataServer: choice.localeServer, gamedataServerPicked: choice.picked, gamedataServers: loaded ?? [] };

    if (!entry) {
        return { locale: fallback, available, messages: {}, hash: "empty", ...servers, redirectTo };
    }

    const hash = entry.namespaces[ALL_NAMESPACES] ?? "latest";
    const messages = await fetchCatalog(entry.code, hash);
    return { locale: entry.code, available, messages, hash, ...servers, redirectTo };
}

/** Whether a locale code is one the backend currently serves. */
export async function isEnabledLocale(code: string): Promise<boolean> {
    const manifest = await fetchManifest();
    if (!manifest) return false;
    return manifest.locales.some((l) => l.code === code);
}
