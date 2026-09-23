import { beforeEach, describe, expect, it, vi } from "vitest";

// `createServerFn` builds an RPC stub outside a Start server; here each fn is
// reduced to its handler so the real session logic runs against mocked I/O.
vi.mock("@tanstack/react-start", () => {
    const builder = () => {
        let validate: (d: unknown) => unknown = (d) => d;
        const chain = {
            inputValidator: (v: (d: unknown) => unknown) => {
                validate = v;
                return chain;
            },
            handler: (h: (ctx: { data: unknown }) => unknown) => (opts?: { data?: unknown }) => h({ data: validate(opts?.data) }),
        };
        return chain;
    };
    return { createServerFn: builder };
});

const cookies = vi.hoisted(() => ({
    jar: new Map<string, string>(),
    set: vi.fn(),
    del: vi.fn(),
}));
vi.mock("@tanstack/react-start/server", () => ({
    getCookie: (name: string) => cookies.jar.get(name),
    setCookie: cookies.set,
    deleteCookie: cookies.del,
}));

vi.mock("#/env", () => ({ env: { NODE_ENV: "production" } }));

const backend = vi.hoisted(() => ({ fetch: vi.fn() }));
vi.mock("../fetch", () => ({ backendFetch: backend.fetch }));

import { getSessionFn, loginFn } from "./server";

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status, headers: { "Content-Type": "application/json" } });
const profile = { uid: "123", nickname: "Doctor", role: "user" };
/** `/auth/verify` accepting the token for `profile`. */
const verifyOk = () => json({ valid: true, uid: "123", role: "user", canAccessAdminPanel: false });
/** The two calls a live session resolves through. */
const liveSession = { "/auth/verify": verifyOk, "/get-user": () => json(profile) };

/** Route the mocked backend by path; anything unrouted is a test bug. */
function routes(table: Record<string, () => Response>) {
    backend.fetch.mockImplementation(async (path: string) => {
        const key = Object.keys(table).find((k) => path.startsWith(k));
        if (!key) throw new Error(`unrouted backend call ${path}`);
        return table[key]();
    });
}

beforeEach(() => {
    cookies.jar.clear();
    cookies.set.mockClear();
    cookies.del.mockClear();
    backend.fetch.mockReset();
});

describe("getSessionFn", () => {
    it("is signed out without touching the backend when there is no cookie", async () => {
        await expect(getSessionFn()).resolves.toBeNull();
        expect(backend.fetch).not.toHaveBeenCalled();
        expect(cookies.del).not.toHaveBeenCalled();
    });

    it("resolves the session for a live token", async () => {
        cookies.jar.set("site_token", "tok");
        routes(liveSession);
        await expect(getSessionFn()).resolves.toMatchObject({ uid: "123", canAccessAdminPanel: false });
        expect(cookies.del).not.toHaveBeenCalled();
    });

    it("clears the cookies when the backend says the token is dead (401)", async () => {
        cookies.jar.set("site_token", "expired");
        routes({ "/auth/verify": () => json({ error: "UNAUTHORIZED" }, 401) });
        await expect(getSessionFn()).resolves.toBeNull();
        expect(cookies.del.mock.calls.map((c) => c[0]).sort()).toEqual(["auth_indicator", "site_token"]);
    });

    it.each([500, 502, 503, 504])("keeps the cookies through a backend %i (restart, database stall)", async (status) => {
        cookies.jar.set("site_token", "tok");
        routes({ "/auth/verify": () => json({ error: "INTERNAL" }, status) });
        await expect(getSessionFn()).resolves.toBeNull();
        expect(cookies.del).not.toHaveBeenCalled();
    });

    it("recovers on the next request once the backend is back", async () => {
        cookies.jar.set("site_token", "tok");
        routes({ "/auth/verify": () => json({}, 503) });
        await expect(getSessionFn()).resolves.toBeNull();
        routes(liveSession);
        await expect(getSessionFn()).resolves.toMatchObject({ uid: "123" });
    });

    it("still clears on a 200 that says the token is invalid", async () => {
        cookies.jar.set("site_token", "tok");
        routes({ "/auth/verify": () => json({ valid: false }) });
        await expect(getSessionFn()).resolves.toBeNull();
        expect(cookies.del).toHaveBeenCalledTimes(2);
    });

    it("does not clear when only the profile fetch fails", async () => {
        cookies.jar.set("site_token", "tok");
        routes({ "/auth/verify": verifyOk, "/get-user": () => json({}, 500) });
        await expect(getSessionFn()).resolves.toBeNull();
        expect(cookies.del).not.toHaveBeenCalled();
    });
});

describe("login cookies", () => {
    it("sets both cookies SameSite=Lax, secure in production, token httpOnly", async () => {
        routes({
            "/login": () => json({ token: "fresh", uid: "123", server: "en" }),
            "/refresh": () => json({}),
            ...liveSession,
        });
        await loginFn({ data: { email: "a@b.co", code: "123456", server: "en", saveCredentials: false } as never });
        const byName = Object.fromEntries(cookies.set.mock.calls.map(([name, value, opts]) => [name, { value, ...opts }]));
        expect(byName.site_token).toMatchObject({ value: "fresh", sameSite: "lax", secure: true, httpOnly: true, path: "/" });
        expect(byName.auth_indicator).toMatchObject({ value: "1", sameSite: "lax", secure: true, httpOnly: false, path: "/" });
    });
});
