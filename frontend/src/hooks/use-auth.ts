import { useCallback, useEffect, useState } from "react";
import type { User } from "~/types/api";

const USER_CACHE_KEY = "myrtle_user_cache";

/**
 * Minimal user data cached in localStorage for instant display.
 * Only includes fields actually used in the UI to avoid quota issues.
 * The full User object can be 100-300KB which exceeds localStorage limits.
 */
export interface CachedUserData {
    status: {
        uid: string;
        nickName: string;
        level: number;
        secretary: string;
        secretarySkinId: string;
    };
}

/** User data type - can be full User from API or minimal cached data */
export type AuthUser = User | CachedUserData;

/**
 * Check if the auth indicator cookie exists (client-side readable).
 * This cookie is set alongside the httpOnly session cookie to allow
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
function extractCacheData(user: User): CachedUserData {
    return {
        status: {
            uid: user.status.uid,
            nickName: user.status.nickName,
            level: user.status.level,
            secretary: user.status.secretary,
            secretarySkinId: user.status.secretarySkinId,
        },
    };
}

/**
 * Get cached user data from localStorage for instant display.
 * Returns a partial User object with only the cached fields.
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
 * Only caches fields actually used in the UI to avoid quota issues.
 */
function setCachedUser(user: User | null): void {
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
    const fetchUser = useCallback(async (): Promise<User | null> => {
        try {
            const res = await fetch("/api/auth/me", { method: "POST" });
            const data = await res.json();

            if (data.success && data.user) {
                return data.user;
            }
            return null;
        } catch {
            return null;
        }
    }, []);

    useEffect(() => {
        // Step 1: Check if auth indicator cookie exists
        if (!hasAuthIndicator()) {
            // No session cookie - skip API call entirely
            setCachedUser(null);
            setUser(null);
            setLoading(false);
            return;
        }

        // Step 2: Use cached user for instant display (if available)
        const cached = getCachedUser();
        if (cached) {
            setUser(cached);
            setLoading(false);
        }

        // Step 3: Fetch user data in background to validate session
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

        if (data.success && data.user) {
            // Login response includes user data (refreshed during login)
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

    const verify = useCallback(async (): Promise<{ valid: boolean; role?: string }> => {
        try {
            const res = await fetch("/api/auth/verify", { method: "POST" });
            const data = await res.json();

            if (data.success && data.data?.valid) {
                return { valid: true, role: data.data.role };
            }
            return { valid: false };
        } catch {
            return { valid: false };
        }
    }, []);

    // Refresh profile from game servers and update local state
    const refreshProfile = useCallback(async (): Promise<{ success: boolean; message?: string }> => {
        try {
            const res = await fetch("/api/settings/refresh-profile", { method: "POST" });
            const data = await res.json();

            if (data.success && data.user) {
                // Update local state with refreshed user data
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
