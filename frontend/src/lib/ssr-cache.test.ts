import { describe, expect, it, vi } from "vitest";
import { type ISsrCacheConfig, isCacheableResponse, parseCookies, SsrMicroCache, ssrCacheConfig, ssrCacheKey } from "./ssr-cache";

const ON: ISsrCacheConfig = { enabled: true, ttlMs: 45_000, maxBytes: 1024 * 1024 };

function req(path: string, init: { cookie?: string; method?: string; auth?: string } = {}): Request {
    const headers = new Headers();
    if (init.cookie) headers.set("cookie", init.cookie);
    if (init.auth) headers.set("authorization", init.auth);
    return new Request(`http://localhost:3000${path}`, { method: init.method ?? "GET", headers });
}

function html(body: string, headers: Record<string, string> = {}, status = 200): Response {
    return new Response(body, { status, headers: { "content-type": "text/html; charset=utf-8", ...headers } });
}

describe("ssrCacheConfig", () => {
    it("is on in production, off in development, and SSR_CACHE overrides both", () => {
        expect(ssrCacheConfig({ NODE_ENV: "production" }).enabled).toBe(true);
        expect(ssrCacheConfig({ NODE_ENV: "development" }).enabled).toBe(false);
        expect(ssrCacheConfig({ NODE_ENV: "production", SSR_CACHE: "0" }).enabled).toBe(false);
        expect(ssrCacheConfig({ NODE_ENV: "production", SSR_CACHE: "false" }).enabled).toBe(false);
        expect(ssrCacheConfig({ NODE_ENV: "development", SSR_CACHE: "1" }).enabled).toBe(true);
    });

    it("takes the defaults for a missing or empty TTL, never 0", () => {
        expect(ssrCacheConfig({}).ttlMs).toBe(45_000);
        expect(ssrCacheConfig({ SSR_CACHE_TTL_MS: "" }).ttlMs).toBe(45_000);
        expect(ssrCacheConfig({ SSR_CACHE_TTL_MS: "0" }).ttlMs).toBe(45_000);
        expect(ssrCacheConfig({ SSR_CACHE_TTL_MS: "30000" }).ttlMs).toBe(30_000);
        expect(ssrCacheConfig({ SSR_CACHE_TTL_MS: "9999999" }).ttlMs).toBe(300_000);
        expect(ssrCacheConfig({}).maxBytes).toBe(48 * 1024 * 1024);
    });
});

describe("ssrCacheKey", () => {
    it("keys the operators pages by path, query, locale cookie and server cookie", () => {
        expect(ssrCacheKey(req("/operators"))).toBe("/operators\u0000\u0000\u0000h");
        expect(ssrCacheKey(req("/ja/operators/char_002_amiya?tab=skills", { cookie: "locale=ja; gamedata_server=cn" }))).toBe("/ja/operators/char_002_amiya?tab=skills\u0000ja\u0000cn\u0000h");
        expect(ssrCacheKey(req("/operators/char_002_amiya", { cookie: "locale=en" }))).not.toBe(ssrCacheKey(req("/operators/char_002_amiya", { cookie: "locale=ja" })));
    });

    it("keys a crawler apart from a browser: the renderer sends bots the fully resolved page", () => {
        const bot = new Request("http://localhost:3000/operators", { headers: { "user-agent": "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)" } });
        const human = new Request("http://localhost:3000/operators", { headers: { "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0 Safari/537.36" } });
        expect(ssrCacheKey(bot)?.endsWith("\u0000b")).toBe(true);
        expect(ssrCacheKey(human)?.endsWith("\u0000h")).toBe(true);
    });

    it("bypasses a session, a non-GET and every other path", () => {
        expect(ssrCacheKey(req("/operators", { cookie: "locale=en; site_token=abc" }))).toBeNull();
        expect(ssrCacheKey(req("/operators", { cookie: "auth_indicator=1" }))).toBeNull();
        expect(ssrCacheKey(req("/operators", { auth: "Bearer x" }))).toBeNull();
        expect(ssrCacheKey(req("/operators", { method: "POST" }))).toBeNull();
        expect(ssrCacheKey(req("/"))).toBeNull();
        expect(ssrCacheKey(req("/operators/a/b"))).toBeNull();
        expect(ssrCacheKey(req("/user/123"))).toBeNull();
        expect(ssrCacheKey(req("/_serverFn/abc"))).toBeNull();
        expect(ssrCacheKey(req("/api/og/operator/char_002_amiya"))).toBeNull();
    });
});

describe("isCacheableResponse", () => {
    it("keeps only a plain 200 HTML page", () => {
        expect(isCacheableResponse(html("x"))).toBe(true);
        expect(isCacheableResponse(html("x", {}, 404))).toBe(false);
        expect(isCacheableResponse(html("x", { "set-cookie": "a=b" }))).toBe(false);
        expect(isCacheableResponse(html("x", { "content-encoding": "br" }))).toBe(false);
        expect(isCacheableResponse(html("x", { "cache-control": "private, max-age=0" }))).toBe(false);
        expect(isCacheableResponse(new Response("{}", { headers: { "content-type": "application/json" } }))).toBe(false);
        expect(isCacheableResponse(new Response(null, { status: 307, headers: { location: "/ja/operators" } }))).toBe(false);
    });
});

describe("parseCookies", () => {
    it("reads the first value of each cookie", () => {
        const c = parseCookies("a=1; b=%E3%81%82; a=2; broken; =x");
        expect(c.get("a")).toBe("1");
        expect(c.get("b")).toBe("あ");
        expect(c.size).toBe(2);
    });
});

describe("SsrMicroCache.wrap", () => {
    it("returns the handler itself when disabled, so the kill switch is the stock server", () => {
        const handler = vi.fn(async () => html("x"));
        expect(new SsrMicroCache({ ...ON, enabled: false }).wrap(handler)).toBe(handler);
    });

    it("renders once, serves the copy until the TTL, then renders again", async () => {
        let t = 1_000_000;
        const cache = new SsrMicroCache(ON, () => t);
        let n = 0;
        const handler = vi.fn(async () => html(`<p>render ${++n}</p>`));
        const fetch = cache.wrap(handler);

        const first = await fetch(req("/operators"));
        expect(first.headers.get("x-ssr-cache")).toBe("MISS");
        expect(await first.text()).toBe("<p>render 1</p>");
        await vi.waitFor(() => expect(cache.snapshot().entries).toBe(1));

        t += 44_000;
        const second = await fetch(req("/operators"));
        expect(second.headers.get("x-ssr-cache")).toBe("HIT");
        expect(second.headers.get("age")).toBe("44");
        expect(second.headers.get("content-type")).toBe("text/html; charset=utf-8");
        expect(await second.text()).toBe("<p>render 1</p>");

        t += 1_000;
        expect(await (await fetch(req("/operators"))).text()).toBe("<p>render 2</p>");
        expect(handler).toHaveBeenCalledTimes(2);
    });

    it("never stores or serves a session's page", async () => {
        const cache = new SsrMicroCache(ON);
        const handler = vi.fn(async () => html("mine"));
        const fetch = cache.wrap(handler);
        await (await fetch(req("/operators", { cookie: "site_token=t" }))).text();
        await (await fetch(req("/operators", { cookie: "site_token=t" }))).text();
        expect(handler).toHaveBeenCalledTimes(2);
        expect(cache.snapshot()).toMatchObject({ entries: 0, bypasses: 2, hits: 0 });
    });

    it("does not keep a 404 or a response that sets a cookie", async () => {
        const cache = new SsrMicroCache(ON);
        const fetch = cache.wrap(async (r: Request) => (r.url.endsWith("/missing") ? html("nf", {}, 404) : html("c", { "set-cookie": "x=1" })));
        await (await fetch(req("/operators/missing"))).text();
        await (await fetch(req("/operators/cookie"))).text();
        expect(cache.snapshot().entries).toBe(0);
    });

    it("shares one in-flight render between concurrent identical requests", async () => {
        const cache = new SsrMicroCache(ON);
        let release: () => void = () => {};
        const gate = new Promise<void>((resolve) => {
            release = resolve;
        });
        const handler = vi.fn(async () => {
            await gate;
            return html("once");
        });
        const fetch = cache.wrap(handler);
        const a = fetch(req("/operators/char_002_amiya"));
        const b = fetch(req("/operators/char_002_amiya"));
        release();
        const [ra, rb] = await Promise.all([a, b]);
        expect(await ra.text()).toBe("once");
        expect(await rb.text()).toBe("once");
        expect(rb.headers.get("x-ssr-cache")).toBe("HIT");
        expect(handler).toHaveBeenCalledTimes(1);
    });

    it("releases a stalled render's slot instead of making every later request wait on it", async () => {
        vi.useFakeTimers();
        try {
            const cache = new SsrMicroCache(ON);
            let calls = 0;
            const fetch = cache.wrap(async () => {
                calls++;
                // The first render never finishes its body.
                if (calls === 1) return new Response(new ReadableStream(), { headers: { "content-type": "text/html" } });
                return html("ok");
            });
            await fetch(req("/operators/x"));
            await vi.advanceTimersByTimeAsync(10_000);
            // The slot is free: the next request renders at once and registers itself.
            const second = fetch(req("/operators/x"));
            await vi.advanceTimersByTimeAsync(0);
            expect(await (await second).text()).toBe("ok");
            expect(calls).toBe(2);
        } finally {
            vi.useRealTimers();
        }
    });

    it("evicts the least recently used page past the byte budget", async () => {
        const cache = new SsrMicroCache({ ...ON, maxBytes: 25 });
        const fetch = cache.wrap(async (r: Request) => html("x".repeat(10) + new URL(r.url).pathname.slice(-1)));
        for (const id of ["a", "b", "c"]) {
            await (await fetch(req(`/operators/${id}`))).text();
            await vi.waitFor(() => expect(cache.stats.stores).toBeGreaterThan(0));
        }
        await vi.waitFor(() => expect(cache.snapshot()).toMatchObject({ entries: 2, bytes: 22, evictions: 1 }));
        expect((await fetch(req("/operators/a"))).headers.get("x-ssr-cache")).toBe("MISS");
    });
});
