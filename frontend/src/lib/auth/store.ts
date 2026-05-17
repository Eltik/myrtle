import { Store } from "@tanstack/store";
import type { AKServer } from "#/lib/auth/login";
import type { IUserProfile } from "#/types/user";

interface ILoginFormState {
    email: string;
    server: AKServer;
    otp: string;
    isOTPSent: boolean;
    cooldownUntil: number;
}

interface IAuthState {
    user: IUserProfile | null;
    status: "idle" | "loading" | "ready";
    login: ILoginFormState;
    dialogOpen: boolean;
    postLoginRedirect: string | null;
}

const initialLogin: ILoginFormState = {
    email: "",
    server: "en",
    otp: "",
    isOTPSent: false,
    cooldownUntil: 0,
};

export const authStore = new Store<IAuthState>({ user: null, status: "idle", login: initialLogin, dialogOpen: false, postLoginRedirect: null });

export const authActions = {
    setUser: (user: IUserProfile | null) => authStore.setState((s) => ({ ...s, user, status: "ready" })),
    setLoading: () => authStore.setState((s) => ({ ...s, status: "loading" })),
    clear: () => authStore.setState((s) => ({ ...s, user: null, status: "ready", login: initialLogin, dialogOpen: false, postLoginRedirect: null })),

    setLoginEmail: (email: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, email } })),
    setLoginServer: (server: AKServer) => authStore.setState((s) => ({ ...s, login: { ...s.login, server } })),
    setLoginOTP: (otp: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, otp } })),
    markOTPSent: (cooldownSeconds = 60) =>
        authStore.setState((s) => ({
            ...s,
            login: { ...s.login, isOTPSent: true, cooldownUntil: Date.now() + cooldownSeconds * 1000 },
        })),
    resetLoginOTP: () => authStore.setState((s) => ({ ...s, login: { ...s.login, otp: "", isOTPSent: false, cooldownUntil: 0 } })),
    resetLoginForm: () => authStore.setState((s) => ({ ...s, login: initialLogin })),

    openLoginDialog: (postLoginRedirect: string | null = null) => authStore.setState((s) => ({ ...s, dialogOpen: true, postLoginRedirect })),
    closeLoginDialog: () => authStore.setState((s) => ({ ...s, dialogOpen: false, postLoginRedirect: null })),
    setDialogOpen: (open: boolean) =>
        authStore.setState((s) => {
            if (open) return { ...s, dialogOpen: true };
            const isLoggedIn = s.user !== null;
            return { ...s, dialogOpen: false, postLoginRedirect: isLoggedIn ? s.postLoginRedirect : null };
        }),
    consumePostLoginRedirect: () => {
        const target = authStore.state.postLoginRedirect;
        if (target) authStore.setState((s) => ({ ...s, postLoginRedirect: null }));
        return target;
    },
};
