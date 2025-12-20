import { useCallback, useEffect, useState } from "react";
import type { User } from "~/types/api";

const USER_CACHE_KEY = "myrtle_user_cache";

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
 * Get cached user data from localStorage for instant display.
 */
function getCachedUser(): User | null {
    if (typeof window === "undefined") return null;
    try {
        const cached = localStorage.getItem(USER_CACHE_KEY);
        return cached ? JSON.parse(cached) : null;
    } catch {
        return null;
    }
}

/**
 * Cache user data to localStorage for faster subsequent loads.
 */
function setCachedUser(user: User | null): void {
    if (typeof window === "undefined") return;
    if (user) {
        localStorage.setItem(USER_CACHE_KEY, JSON.stringify(user));
    } else {
        localStorage.removeItem(USER_CACHE_KEY);
    }
}

export function useAuth() {
    const [user, setUser] = useState<User | null>(null);
    const [loading, setLoading] = useState(true);

    // Refresh session from the server
    const refreshSession = useCallback(async (): Promise<User | null> => {
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

        // Step 3: Refresh session in background to validate and update seqnum
        refreshSession().then((refreshedUser) => {
            if (refreshedUser) {
                setUser(refreshedUser);
                setCachedUser(refreshedUser);
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
    }, [refreshSession]);

    const login = useCallback(async (email: string, code: string, server: string) => {
        const res = await fetch("/api/auth/login", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ email, code, server }),
        });

        const data = await res.json();

        if (data.success) {
            // After successful login, fetch user data
            const meRes = await fetch("/api/auth/me", { method: "POST" });
            const meData = await meRes.json();

            if (meData.success && meData.user) {
                setUser(meData.user);
                setCachedUser(meData.user);
            }
        }

        return data;
    }, []);

    const logout = useCallback(async () => {
        await fetch("/api/auth/logout", { method: "POST" });
        setUser(null);
        setCachedUser(null);
    }, []);

    return { user, loading, login, logout, refreshSession };
}
