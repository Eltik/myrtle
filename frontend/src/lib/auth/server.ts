import { createServerFn } from "@tanstack/react-start";
import { loginSchema, type LoginInput } from "./login";
import { backendFetch } from "../fetch";
import { deleteCookie, getCookie, setCookie } from "@tanstack/react-start/server";
import type { IUserProfile } from "#/types/user";
import { env } from "#/env";

export const loginFn = createServerFn({ method: "POST" })
    .inputValidator((d: LoginInput) => loginSchema.parse(d))
    .handler(async ({ data }) => {
        return await login(data);
    });

export const getSessionFn = createServerFn({ method: "GET" }).handler(async () => {
    return await getSession();
});

export const logoutFn = createServerFn({ method: "POST" }).handler(async () => {
    clearAuthCookies();
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

const login = async (data: LoginInput) => {
    const loginRes = await backendFetch("/login", {
        method: "POST",
        body: JSON.stringify(data),
    });
    if (!loginRes.ok) throw new Error("Invalid credentials");
    const { token, uid } = (await loginRes.json()) as { token: string; uid: string };

    setAuthCookies(token);

    const refreshRes = await backendFetch("/refresh", { method: "POST", bearerToken: token });
    if (!refreshRes.ok) throw new Error("Sync failed");

    const userRes = await backendFetch(`/get-user?uid=${encodeURIComponent(uid)}`);
    if (!userRes.ok) throw new Error("Failed to fetch user data");
    return (await userRes.json()) as IUserProfile;
};

const getSession = async () => {
    const token = getCookie("site_token");
    if (!token) return null;

    const verifyRes = await backendFetch("/auth/verify", { bearerToken: token });
    if (!verifyRes.ok) {
        clearAuthCookies();
        return null;
    }
    const { valid, uid } = (await verifyRes.json()) as { valid: boolean; uid?: string };
    if (!valid || !uid) {
        clearAuthCookies();
        return null;
    }

    const userRes = await backendFetch(`/get-user?uid=${encodeURIComponent(uid)}`);
    if (!userRes.ok) return null;
    return (await userRes.json()) as IUserProfile;
};
