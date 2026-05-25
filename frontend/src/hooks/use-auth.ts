import { useQueryClient } from "@tanstack/react-query";
import { useRouter } from "@tanstack/react-router";
import { useStore } from "@tanstack/react-store";
import type { LoginInput } from "#/lib/auth/login";
import { getSessionFn, loginFn, logoutFn } from "#/lib/auth/server";
import { authActions, authStore } from "#/lib/auth/store";

export function useAuth() {
    const router = useRouter();
    const queryClient = useQueryClient();

    const user = useStore(authStore, (s) => s.user);
    const loading = useStore(authStore, (s) => s.status === "loading");
    const isAuthenticated = useStore(authStore, (s) => s.user !== null);

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
        logout,
        fetchUser,
    };
}
