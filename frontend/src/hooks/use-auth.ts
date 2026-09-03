import { useQueryClient } from "@tanstack/react-query";
import { useRouteContext, useRouter } from "@tanstack/react-router";
import { useStore } from "@tanstack/react-store";
import type { BilibiliLoginInput, BilibiliSmsLoginInput, CnLoginInput, LoginInput } from "#/lib/auth/login";
import { getSessionFn, loginBilibiliFn, loginBilibiliSmsFn, loginCnFn, loginFn, logoutFn } from "#/lib/auth/server";
import { authActions, authStore } from "#/lib/auth/store";

export function useAuth() {
    const router = useRouter();
    const queryClient = useQueryClient();

    // The store starts empty and is only seeded by a post-hydration effect in
    // __root.tsx, but the root beforeLoad already resolved the real session
    // into route context. Until the store leaves "idle", read the context so
    // SSR and the first client render agree (no signed-out flash for
    // logged-in users, no hydration mismatch).
    const contextUser = useRouteContext({ from: "__root__", select: (c) => c.user ?? null });
    const storeSeeded = useStore(authStore, (s) => s.status !== "idle");
    const storeUser = useStore(authStore, (s) => s.user);

    const user = storeSeeded ? storeUser : contextUser;
    const loading = useStore(authStore, (s) => s.status === "loading");
    const isAuthenticated = user !== null;

    const login = async (data: LoginInput) => {
        authActions.setLoading();
        try {
            const u = await loginFn({ data });
            authActions.setUser(u);
            // Drop any cached anon views - server fns now auto-attach the session
            // token, so the next fetch should return owner-scoped data.
            queryClient.removeQueries({ queryKey: ["user"] });
            await router.invalidate();
            return u;
        } catch (err) {
            authActions.clear();
            throw err;
        }
    };

    // Same completion shape as `login` (set user, drop cached anon views,
    // invalidate route data); only which server fn mints the session differs.
    const finishLogin = async (u: Awaited<ReturnType<typeof loginFn>>) => {
        authActions.setUser(u);
        queryClient.removeQueries({ queryKey: ["user"] });
        await router.invalidate();
        return u;
    };

    const loginBilibili = async (data: BilibiliLoginInput) => {
        authActions.setLoading();
        try {
            return await finishLogin(await loginBilibiliFn({ data }));
        } catch (err) {
            authActions.clear();
            throw err;
        }
    };

    // Experimental: see login.ts bilibiliSmsLoginSchema doc comment. May fail
    // even with a correct code until the guessed SMS endpoints are confirmed.
    const loginBilibiliSms = async (data: BilibiliSmsLoginInput) => {
        authActions.setLoading();
        try {
            return await finishLogin(await loginBilibiliSmsFn({ data }));
        } catch (err) {
            authActions.clear();
            throw err;
        }
    };

    // Experimental: see the CN entry in SERVERS. May fail even with correct
    // credentials until the real game-client appCode is known.
    const loginCn = async (data: CnLoginInput) => {
        authActions.setLoading();
        try {
            return await finishLogin(await loginCnFn({ data }));
        } catch (err) {
            authActions.clear();
            throw err;
        }
    };

    const logout = async () => {
        await logoutFn();
        authActions.clear();
        // Evict any private data fetched while authenticated so it can't leak
        // to subsequent anon views on the same browser.
        queryClient.removeQueries({ queryKey: ["user"] });
        await router.invalidate();
    };

    const fetchUser = async () => {
        const u = await getSessionFn();
        authActions.setUser(u);
        return u;
    };

    return {
        user,
        loading,
        isAuthenticated,
        login,
        loginBilibili,
        loginBilibiliSms,
        loginCn,
        logout,
        fetchUser,
    };
}
