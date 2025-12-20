import { useEffect, useState } from "react";
import type { User } from "~/types/api";

export function useAuth() {
    const [user, setUser] = useState<User | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        fetch("/api/auth/me")
            .then((res) => res.json())
            .then((data) => setUser(data.user))
            .finally(() => setLoading(false));
    }, []);

    const login = async (email: string, code: string, server: string) => {
        const res = await fetch("/api/auth/login", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                email,
                code,
                server,
            }),
        });

        const data = await res.json();
        if (data.success) setUser(data.user);

        return data;
    };

    const logout = async () => {
        await fetch("/api/auth/logout", { method: "POST" });
        setUser(null);
    };

    return { user, loading, login, logout };
}
