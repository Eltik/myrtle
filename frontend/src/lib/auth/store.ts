import { Store } from "@tanstack/store";
import type { IUserProfile } from "#/types/user";

interface AuthState {
    user: IUserProfile | null;
    status: "idle" | "loading" | "ready";
}

export const authStore = new Store<AuthState>({ user: null, status: "idle" });

export const authActions = {
    setUser: (user: IUserProfile | null) => authStore.setState((s) => ({ ...s, user, status: "ready" })),
    setLoading: () => authStore.setState((s) => ({ ...s, status: "loading" })),
    clear: () => authStore.setState(() => ({ user: null, status: "ready" })),
};
