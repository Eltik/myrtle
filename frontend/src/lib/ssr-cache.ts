/**
 * A short-lived cache of rendered HTML for anonymous visits to the busiest
 * static pages: the operators list and each operator's page.
 *
 * Those pages are a pure function of the URL, the `locale` cookie and the
 * `gamedata_server` cookie for a visitor with no session, so for that visitor
 * the server renders the same bytes again and again. Rendering one is the most
 * CPU the frontend spends per request; serving it from memory is a copy.
 *
 * What is never cached, by construction:
 * - any request carrying a session (`site_token`, `auth_indicator`, an
 *   `Authorization` header), because the page renders the viewer;
 * - anything but a GET of an allowlisted path;
 * - any response that is not a 200 `text/html`, that sets a cookie, that is
 *   content-encoded, or that says `private` / `no-store`.
 *
 * Kill switch: `SSR_CACHE=0` restores the uncached server exactly (the wrapper
 * returns the handler's own response untouched). Off in development unless
 * `SSR_CACHE=1` is set. `SSR_CACHE_TTL_MS` (default 45,000) and
 * `SSR_CACHE_MAX_MB` (default 48) size it.
 */
import { isbot } from "isbot";
import { GAMEDATA_SERVER_COOKIE, LOCALE_COOKIE, parseLocaleFromPath } from "#/lib/i18n/locale";

export interface ISsrCacheConfig {
    enabled: boolean;
    ttlMs: number;
    maxBytes: number;
}

const DEFAULT_TTL_MS = 45_000;
const DEFAULT_MAX_MB = 48;
/** A page larger than this is served but not kept: one entry must not evict the whole cache. */
const MAX_ENTRY_BYTES = 4 * 1024 * 1024;
/** How long a request waits for an identical render already in flight before rendering its own. */
const INFLIGHT_WAIT_MS = 10_000;

/** Paths (after the locale prefix is stripped) whose anonymous render is cached. */
const CACHEABLE_PATH = /^\/operators(?:\/[^/]+)?\/?$/;

/** Framing headers that describe the original stream, not the stored bytes. */
const HOP_HEADERS: ReadonlySet<string> = new Set(["connection", "content-length", "keep-alive", "transfer-encoding"]);

/** Cookies that mean the page renders a signed-in viewer. */
const SESSION_COOKIES = ["site_token", "auth_indicator"] as const;

function positiveNumber(raw: string | undefined, fallback: number): number {
    // A MISSING variable takes the default; `Number("")` is 0 and finite, so test presence first.
    if (raw === undefined || raw.trim() === "") return fallback;
    const n = Number(raw);
    return Number.isFinite(n) && n > 0 ? n : fallback;
}

export function ssrCacheConfig(env: Record<string, string | undefined> = process.env): ISsrCacheConfig {
    const flag = env.SSR_CACHE?.trim();
    const enabled = flag === undefined || flag === "" ? env.NODE_ENV === "production" : flag !== "0" && flag.toLowerCase() !== "false";
    return {
        enabled,
        ttlMs: Math.min(positiveNumber(env.SSR_CACHE_TTL_MS, DEFAULT_TTL_MS), 300_000),
        maxBytes: positiveNumber(env.SSR_CACHE_MAX_MB, DEFAULT_MAX_MB) * 1024 * 1024,
    };
}

export function parseCookies(header: string | null): Map<string, string> {
    const out = new Map<string, string>();
    if (!header) return out;
    for (const part of header.split(";")) {
        const eq = part.indexOf("=");
        if (eq <= 0) continue;
        const name = part.slice(0, eq).trim();
        if (!name || out.has(name)) continue;
        let value = part.slice(eq + 1).trim();
        try {
            value = decodeURIComponent(value);
        } catch {
            // Keep the raw value: it is only a key component.
        }
        out.set(name, value);
    }
    return out;
}

/** The cache key for `request`, or `null` when the request must not be served from or stored in the cache. */
export function ssrCacheKey(request: Request): string | null {
    if (request.method !== "GET") return null;
    if (request.headers.has("authorization")) return null;
    const url = new URL(request.url);
    if (!CACHEABLE_PATH.test(parseLocaleFromPath(url.pathname).rest)) return null;
    const cookies = parseCookies(request.headers.get("cookie"));
    if (SESSION_COOKIES.some((name) => cookies.has(name))) return null;
    // The renderer waits for the whole page before sending a byte to a bot (TanStack's
    // renderRouterToStream: `if (isbot(UA)) await stream.allReady`) and streams suspense
    // fallbacks to everyone else, so the two are different bytes and are keyed apart.
    const bot = isbot(request.headers.get("user-agent")) ? "b" : "h";
    return `${url.pathname}${url.search}\u0000${cookies.get(LOCALE_COOKIE) ?? ""}\u0000${cookies.get(GAMEDATA_SERVER_COOKIE) ?? ""}\u0000${bot}`;
}

export function isCacheableResponse(res: Response): boolean {
    if (res.status !== 200 || !res.body) return false;
    if (!(res.headers.get("content-type") ?? "").toLowerCase().startsWith("text/html")) return false;
    if (res.headers.has("set-cookie")) return false;
    // The key has no Accept-Encoding: an encoded body would reach a client that never asked for it.
    // (Nothing encodes today, nginx compresses in front; this keeps it that way if that changes.)
    if (res.headers.has("content-encoding")) return false;
    return !/\b(?:private|no-store)\b/i.test(res.headers.get("cache-control") ?? "");
}

interface IEntry {
    body: Uint8Array;
    headers: [string, string][];
    storedAt: number;
    expiresAt: number;
}

export interface ISsrCacheStats {
    hits: number;
    misses: number;
    bypasses: number;
    stores: number;
    evictions: number;
    entries: number;
    bytes: number;
}

export class SsrMicroCache {
    private readonly entries = new Map<string, IEntry>();
    private readonly inflight = new Map<string, Promise<IEntry | null>>();
    private bytes = 0;
    readonly stats: Omit<ISsrCacheStats, "entries" | "bytes"> = { hits: 0, misses: 0, bypasses: 0, stores: 0, evictions: 0 };

    constructor(
        readonly config: ISsrCacheConfig,
        private readonly now: () => number = Date.now,
    ) {}

    snapshot(): ISsrCacheStats {
        return { ...this.stats, entries: this.entries.size, bytes: this.bytes };
    }

    private get(key: string): IEntry | null {
        const entry = this.entries.get(key);
        if (!entry) return null;
        if (this.now() >= entry.expiresAt) {
            this.delete(key);
            return null;
        }
        // Least recently used goes first: re-insert to move it to the back.
        this.entries.delete(key);
        this.entries.set(key, entry);
        return entry;
    }

    private delete(key: string): void {
        const entry = this.entries.get(key);
        if (!entry) return;
        this.entries.delete(key);
        this.bytes -= entry.body.byteLength;
    }

    private set(key: string, entry: IEntry): void {
        this.delete(key);
        this.entries.set(key, entry);
        this.bytes += entry.body.byteLength;
        this.stats.stores++;
        for (const oldest of this.entries.keys()) {
            if (this.bytes <= this.config.maxBytes) break;
            this.delete(oldest);
            this.stats.evictions++;
        }
    }

    private respond(entry: IEntry): Response {
        const headers = new Headers(entry.headers);
        headers.set("x-ssr-cache", "HIT");
        headers.set("age", String(Math.floor((this.now() - entry.storedAt) / 1000)));
        return new Response(entry.body.slice(), { status: 200, headers });
    }

    /** Wrap a request handler so anonymous renders of the cacheable paths are reused for `ttlMs`. */
    wrap<A extends unknown[]>(handler: (request: Request, ...rest: A) => Response | Promise<Response>): (request: Request, ...rest: A) => Response | Promise<Response> {
        if (!this.config.enabled) return handler;
        return async (request, ...rest) => {
            const key = ssrCacheKey(request);
            if (key === null) {
                this.stats.bypasses++;
                return handler(request, ...rest);
            }

            const hit = this.get(key);
            if (hit) {
                this.stats.hits++;
                return this.respond(hit);
            }

            // An identical render is already running: wait for it rather than
            // rendering the same bytes twice. A burst of shares of one link is
            // exactly this shape.
            const pending = this.inflight.get(key);
            if (pending) {
                const shared = await Promise.race([pending, new Promise<null>((resolve) => setTimeout(resolve, INFLIGHT_WAIT_MS, null))]);
                if (shared) {
                    this.stats.hits++;
                    return this.respond(shared);
                }
            }

            this.stats.misses++;
            let settle: (entry: IEntry | null) => void = () => {};
            const fill = new Promise<IEntry | null>((resolve) => {
                settle = resolve;
            });
            // Register unless another live render holds the key. A stalled render must not
            // hold it forever: after INFLIGHT_WAIT_MS its slot is released (waiters render
            // their own), and a later finish still stores the page.
            if (!this.inflight.has(key)) this.inflight.set(key, fill);
            const release = () => {
                if (this.inflight.get(key) === fill) this.inflight.delete(key);
            };
            const timer = setTimeout(release, INFLIGHT_WAIT_MS);
            const done = (entry: IEntry | null) => {
                clearTimeout(timer);
                release();
                settle(entry);
            };

            let res: Response;
            try {
                res = await handler(request, ...rest);
            } catch (err) {
                done(null);
                throw err;
            }
            if (!isCacheableResponse(res) || !res.body) {
                done(null);
                return res;
            }

            // Stream to the visitor as before and collect a copy on the side.
            const [toClient, toCache] = res.body.tee();
            const headers: [string, string][] = [];
            res.headers.forEach((value, name) => {
                if (!HOP_HEADERS.has(name)) headers.push([name, value]);
            });
            void collect(toCache, MAX_ENTRY_BYTES).then(
                (body) => {
                    if (!body) return done(null);
                    const storedAt = this.now();
                    const entry: IEntry = { body, headers, storedAt, expiresAt: storedAt + this.config.ttlMs };
                    this.set(key, entry);
                    done(entry);
                },
                () => done(null),
            );

            const out = new Headers(res.headers);
            out.set("x-ssr-cache", "MISS");
            return new Response(toClient, { status: res.status, statusText: res.statusText, headers: out });
        };
    }
}

/** Every byte of `stream`, or `null` when it errors or exceeds `limit`. */
async function collect(stream: ReadableStream<Uint8Array>, limit: number): Promise<Uint8Array | null> {
    const reader = stream.getReader();
    const chunks: Uint8Array[] = [];
    let size = 0;
    try {
        for (;;) {
            const { done, value } = await reader.read();
            if (done) break;
            size += value.byteLength;
            if (size > limit) {
                await reader.cancel();
                return null;
            }
            chunks.push(value);
        }
    } catch {
        return null;
    }
    const body = new Uint8Array(size);
    let offset = 0;
    for (const chunk of chunks) {
        body.set(chunk, offset);
        offset += chunk.byteLength;
    }
    return body;
}
