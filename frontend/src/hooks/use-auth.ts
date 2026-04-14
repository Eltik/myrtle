import { useCallback, useEffect, useState } from "react";
import type { UserProfile } from "~/types/api/impl/user";

const USER_CACHE_KEY = "myrtle_user_cache";

/**
 * Minimal user data cached in localStorage for instant display.
 * Uses v3 field names matching UserProfile.
 */
export interface CachedUserData {
    uid: string;
    nickname: string;
    level: number;
    secretary: string;
    secretary_skin_id: string;
    avatar_id: string;
    server: string;
}

/** User data type - can be full UserProfile from API or minimal cached data */
export type AuthUser = UserProfile | CachedUserData;

/**
 * Check if the auth indicator cookie exists (client-side readable).
 * This cookie is set alongside the httpOnly JWT cookie to allow
 * the client to know if a session exists without exposing sensitive data.
 */
function hasAuthIndicator(): boolean {
    if (typeof document === "undefined") return false;
    return document.cookie.split(";").some((c) => c.trim().startsWith("auth_indicator="));
}

/**
 * Extract minimal user data for caching.
 * Only includes fields used in the UI (header display, avatar).
 */
function extractCacheData(user: UserProfile): CachedUserData {
    return {
        uid: user.uid,
        nickname: user.nickname ?? "",
        level: user.level ?? 0,
        secretary: user.secretary ?? "",
        secretary_skin_id: user.secretary_skin_id ?? "",
        avatar_id: user.avatar_id ?? "",
        server: user.server,
    };
}

/**
 * Get cached user data from localStorage for instant display.
 */
function getCachedUser(): CachedUserData | null {
    if (typeof window === "undefined") return null;
    try {
        const cached = localStorage.getItem(USER_CACHE_KEY);
        return cached ? JSON.parse(cached) : null;
    } catch {
        return null;
    }
}

/**
 * Cache minimal user data to localStorage for faster subsequent loads.
 */
function setCachedUser(user: UserProfile | null): void {
    if (typeof window === "undefined") return;
    if (user) {
        localStorage.setItem(USER_CACHE_KEY, JSON.stringify(extractCacheData(user)));
    } else {
        localStorage.removeItem(USER_CACHE_KEY);
    }
}

export function useAuth() {
    const [user, setUser] = useState<AuthUser | null>(null);
    const [loading, setLoading] = useState(true);

    // Fetch user data from the server (uses cached database data, no game server refresh)
    const fetchUser = useCallback(async (): Promise<UserProfile | null> => {
        try {
            const res = await fetch("/api/auth/me", { method: "POST" });

            if (!res.ok) {
                return null;
            }

            // v3: /api/auth/me returns UserProfile directly (not wrapped in {success, user})
            const data: UserProfile = await res.json();

            // Guard against error responses that slip through
            if (!data.uid) {
                return null;
            }

            return data;
        } catch {
            return null;
        }
    }, []);

    useEffect(() => {
        if (!hasAuthIndicator()) {
            setCachedUser(null);
            setUser(null);
            setLoading(false);
            return;
        }

        const cached = getCachedUser();
        if (cached) {
            setUser(cached);
            setLoading(false);
        }

        fetchUser().then((fetchedUser) => {
            if (fetchedUser) {
                setUser(fetchedUser);
                setCachedUser(fetchedUser);
            } else {
                // Session expired or invalid - clear everything
                setUser(null);
                setCachedUser(null);
            }
            // Only set loading false here if we didn't have cached data
            if (!cached) {
                setLoading(false);
            }
        });
    }, [fetchUser]);

    const login = useCallback(async (email: string, code: string, server: string) => {
        const res = await fetch("/api/auth/login", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ email, code, server }),
        });

        const data = await res.json();

        // v3: login returns { success: true, user: UserProfile }
        if (data.success && data.user) {
            setUser(data.user);
            setCachedUser(data.user);
        }

        return data;
    }, []);

    const logout = useCallback(async () => {
        await fetch("/api/auth/logout", { method: "POST" });
        setUser(null);
        setCachedUser(null);
    }, []);

    const verify = useCallback(async (): Promise<{ valid: boolean; userId?: string; uid?: string; server?: string; role?: string }> => {
        try {
            const res = await fetch("/api/auth/verify", { method: "POST" });
            const data = await res.json();

            // v3: verify returns { success: true, data: { valid, userId, uid, server, role } }
            if (data.success && data.data?.valid) {
                return { valid: true, userId: data.data.userId, uid: data.data.uid, server: data.data.server, role: data.data.role };
            }
            return { valid: false };
        } catch {
            return { valid: false };
        }
    }, []);

    const refreshProfile = useCallback(async (): Promise<{ success: boolean; message?: string }> => {
        try {
            const res = await fetch("/api/settings/refresh-profile", { method: "POST" });
            const data = await res.json();

            if (data.success && data.user) {
                setUser(data.user);
                setCachedUser(data.user);
            }

            return { success: data.success, message: data.message };
        } catch {
            return { success: false, message: "Failed to refresh profile" };
        }
    }, []);

    return { user, loading, login, logout, fetchUser, verify, refreshProfile };
}
