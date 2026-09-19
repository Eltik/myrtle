import { createServerFn } from "@tanstack/react-start";
import { deleteCookie, getCookie, setCookie } from "@tanstack/react-start/server";
import { env } from "#/env";
import { APIError, parseError } from "#/lib/api/_shared";
import type { LoginResponse } from "#/types/generated/LoginResponse";
import type { StatusOk } from "#/types/generated/StatusOk";
import type { VerifySession } from "#/types/generated/VerifySession";
import type { IUserProfile } from "#/types/user";
import { backendFetch } from "../fetch";
import { type AKServer, type BilibiliLoginInput, type BilibiliSmsLoginInput, bilibiliLoginSchema, bilibiliSmsLoginSchema, type CnLoginInput, cnLoginSchema, type LoginInput, loginSchema } from "./login";

export const loginFn = createServerFn({ method: "POST" })
    .inputValidator((d: LoginInput) => loginSchema.parse(d))
    .handler(async ({ data }) => {
        return await login(data);
    });

export const loginBilibiliFn = createServerFn({ method: "POST" })
    .inputValidator((d: BilibiliLoginInput) => bilibiliLoginSchema.parse(d))
    .handler(async ({ data }) => {
        return await loginBilibili(data);
    });

export const loginBilibiliSmsFn = createServerFn({ method: "POST" })
    .inputValidator((d: BilibiliSmsLoginInput) => bilibiliSmsLoginSchema.parse(d))
    .handler(async ({ data }) => {
        return await loginBilibiliSms(data);
    });

export const loginCnFn = createServerFn({ method: "POST" })
    .inputValidator((d: CnLoginInput) => cnLoginSchema.parse(d))
    .handler(async ({ data }) => {
        return await loginCn(data);
    });

export const getSessionFn = createServerFn({ method: "GET" }).handler(async () => {
    return await getSession();
});

export const logoutFn = createServerFn({ method: "POST" }).handler(async () => {
    clearAuthCookies();
});

export const sendCodeFn = createServerFn({ method: "POST" })
    .inputValidator((d: { email: string; server: AKServer }) => d)
    .handler(async ({ data }) => {
        return await sendCode(data);
    });

export const sendCodeCnFn = createServerFn({ method: "POST" })
    .inputValidator((d: { phone: string }) => d)
    .handler(async ({ data }) => {
        return await sendCodeCn(data);
    });

export const sendBiliSmsFn = createServerFn({ method: "POST" })
    .inputValidator((d: { phone: string }) => d)
    .handler(async ({ data }) => {
        return await sendBiliSms(data);
    });

const COOKIE_BASE = {
    secure: env.NODE_ENV === "production",
    sameSite: "strict" as const,
    path: "/",
    maxAge: 60 * 60 * 24 * 7,
};

function setAuthCookies(token: string) {
    setCookie("site_token", token, { ...COOKIE_BASE, httpOnly: true });
    setCookie("auth_indicator", "1", { ...COOKIE_BASE, httpOnly: false });
}

function clearAuthCookies() {
    deleteCookie("site_token", { path: "/" });
    deleteCookie("auth_indicator", { path: "/" });
}

// Shared by every login method: each just gets a {token, uid} pair out of a
// different backend endpoint, then finishes identically from there.
//
// Failures propagate as `APIError` (status + the backend's own message and
// code) or `BackendUnreachableError` from `backendFetch`, so the dialog can say
// WHICH thing failed. This used to throw a bare "Invalid credentials" for every
// non-2xx login response, which hid a backend outage (503), a rate limit (429)
// and Yostar's own reason for rejecting a code (400) behind the same words.
const completeLogin = async (loginRes: Response): Promise<ISession> => {
    if (!loginRes.ok) throw await parseError(loginRes);
    const { token } = (await loginRes.json()) as LoginResponse;

    setAuthCookies(token);

    // The account IS signed in at this point (cookies are set), only the
    // first roster pull failed; a page reload lands the user signed in.
    const refreshRes = await backendFetch("/refresh", { method: "POST", bearerToken: token });
    if (!refreshRes.ok) {
        const err = await parseError(refreshRes);
        throw new APIError(err.status, `Signed in, but the first roster sync failed: ${err.message}`, "SYNC_FAILED");
    }

    // Build the session the same way a page load does, rather than fetching the
    // profile directly: that is what carries the authorisation facts resolved
    // server-side. Fetching the profile row here instead left a just-logged-in
    // translator without `canAccessAdminPanel` until their next full page load.
    //
    // Resolve it from the token we just received, NOT from the request cookie.
    // `setAuthCookies` writes the response's Set-Cookie header; `getCookie`
    // reads the incoming request, which on a fresh login carries no
    // `site_token` yet. Reading the cookie here made every first login fail
    // with "Failed to fetch user data" until the next page load re-sent it.
    const session = await sessionForToken(token);
    if (!session) throw new Error("Failed to fetch user data");
    return session;
};

// The backend field is `save_credentials`; `withSaveFlag` renames it at the
// boundary rather than making every client-side schema carry snake_case.
const withSaveFlag = <T extends { saveCredentials: boolean }>({ saveCredentials, ...rest }: T) => ({ ...rest, save_credentials: saveCredentials });

const login = async (data: LoginInput) =>
    completeLogin(
        await backendFetch("/login", {
            method: "POST",
            body: JSON.stringify(withSaveFlag(data)),
        }),
    );

const loginBilibili = async (data: BilibiliLoginInput) =>
    completeLogin(
        await backendFetch("/login/bilibili", {
            method: "POST",
            body: JSON.stringify(withSaveFlag(data)),
        }),
    );

// Experimental: see the login.ts bilibiliSmsLoginSchema doc comment and the
// backend core::hypergryph::bilibili module docs. The SMS endpoints are an
// unverified structural guess, unlike the username/password flow above.
const loginBilibiliSms = async (data: BilibiliSmsLoginInput) =>
    completeLogin(
        await backendFetch("/login/bilibili/sms", {
            method: "POST",
            body: JSON.stringify(withSaveFlag(data)),
        }),
    );

// Experimental: see the CN entry in SERVERS and the backend's
// core::hypergryph::passport module docs. May fail even with correct
// credentials until the real game-client appCode is known.
const loginCn = async (data: CnLoginInput) =>
    completeLogin(
        await backendFetch("/login/cn", {
            method: "POST",
            body: JSON.stringify(withSaveFlag(data)),
        }),
    );

/**
 * What a signed-in session carries: the public profile row plus the two
 * authorisation facts the server resolved. `canAccessAdminPanel` is a grant of
 * entry only - every admin route still authorises its own request.
 */
export interface ISession extends IUserProfile {
    canAccessAdminPanel: boolean;
}

const getSession = async () => {
    const token = getCookie("site_token");
    if (!token) return null;
    return sessionForToken(token);
};

const sessionForToken = async (token: string): Promise<ISession | null> => {
    const verifyRes = await backendFetch("/auth/verify", { bearerToken: token });
    if (!verifyRes.ok) {
        clearAuthCookies();
        return null;
    }
    // `role` and `canAccessAdminPanel` come from `/auth/verify`, not from the
    // profile below: verify resolves both server-side from the database and
    // from `translation_permissions`, which is what every admin route checks.
    // Reading the role off the profile row instead (what this did before) let
    // the client gate and the server gate drift apart.
    const { valid, uid, role, canAccessAdminPanel } = (await verifyRes.json()) as VerifySession;
    if (!valid || !uid) {
        clearAuthCookies();
        return null;
    }

    // Send the token. `/get-user` runs the shared privacy gate, which only
    // skips the `public_profile` check when it can see that the caller IS the
    // profile's owner. Unauthenticated, that check is decided entirely by
    // `public_profile = true`, which is NULL on a freshly created account until
    // `sp_sync_user_data` backfills `user_settings`, and stays false forever for
    // anyone who set their profile private. Both cases resolved the session to
    // null and surfaced as "Failed to fetch user data" on login, and as a
    // silent sign-out on a later page load.
    const userRes = await backendFetch(`/get-user?uid=${encodeURIComponent(uid)}`, { bearerToken: token });
    if (!userRes.ok) return null;
    const profile = (await userRes.json()) as IUserProfile;
    return { ...profile, role: role ?? profile.role, canAccessAdminPanel: canAccessAdminPanel === true } satisfies ISession;
};

const sendCode = async (data: { email: string; server: AKServer }) => {
    const req = await backendFetch("/login/send-code", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ email: data.email, server: data.server }),
    });

    if (!req.ok) throw await parseError(req);

    return (await req.json()) as StatusOk;
};

const sendCodeCn = async (data: { phone: string }) => {
    const req = await backendFetch("/login/cn/send-code", {
        method: "POST",
        body: JSON.stringify({ phone: data.phone }),
    });

    if (!req.ok) throw await parseError(req);

    return (await req.json()) as StatusOk;
};

const sendBiliSms = async (data: { phone: string }) => {
    const req = await backendFetch("/login/bilibili/send-code", {
        method: "POST",
        body: JSON.stringify({ phone: data.phone }),
    });

    if (!req.ok) throw await parseError(req);

    return (await req.json()) as StatusOk;
};
