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
}

const initialLogin: ILoginFormState = {
    email: "",
    server: "en",
    otp: "",
    isOTPSent: false,
    cooldownUntil: 0,
};

export const authStore = new Store<IAuthState>({ user: null, status: "idle", login: initialLogin });

export const authActions = {
    setUser: (user: IUserProfile | null) => authStore.setState((s) => ({ ...s, user, status: "ready" })),
    setLoading: () => authStore.setState((s) => ({ ...s, status: "loading" })),
    clear: () => authStore.setState(() => ({ user: null, status: "ready", login: initialLogin })),

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
};
