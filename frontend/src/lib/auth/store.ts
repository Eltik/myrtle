import { Store } from "@tanstack/store";
import type { AKServer } from "#/lib/auth/login";
import type { ISession } from "#/lib/auth/server";

interface ILoginFormState {
    email: string;
    server: AKServer;
    otp: string;
    isOTPSent: boolean;
    cooldownUntil: number;
    // Bilibili: username/password, or phone plus an SMS code requested for
    // that phone (mirrors the CN flow below).
    biliUsername: string;
    biliPassword: string;
    biliPhone: string;
    biliSmsCode: string;
    biliUseSms: boolean;
    isBiliCodeSent: boolean;
    biliCooldownUntil: number;
    // CN (Hypergryph passport, experimental): phone plus either a password or
    // an SMS code requested for that phone.
    cnPhone: string;
    cnPassword: string;
    cnSmsCode: string;
    cnUseSms: boolean;
    isCnCodeSent: boolean;
    cnCooldownUntil: number;
    /**
     * Whether this login may keep the player's game credentials on the server.
     *
     * Shared by all four login methods, and deliberately NOT reset by
     * `resetLoginForm`: the choice is the point of the control, and having to
     * re-tick it every time is how people stop making it. It is seeded from
     * `readRememberedSaveCredentials()` and written back on every change.
     */
    saveCredentials: boolean;
}

interface IAuthState {
    user: ISession | null;
    status: "idle" | "loading" | "ready";
    login: ILoginFormState;
    dialogOpen: boolean;
    postLoginRedirect: string | null;
    /**
     * A session this client minted itself, from a login response that has not
     * yet been echoed back by a route load.
     *
     * The login response writes `site_token` with `SameSite=Strict`; the
     * `router.invalidate()` that follows re-runs the root `beforeLoad`, which
     * reads that cookie off the NEW request. When the browser has not committed
     * the cookie jar by then, `getSessionFn` returns null and the route context
     * would otherwise overwrite the user we just signed in - no error, no
     * console output, the spinner just turns back into a "login" button.
     * While this flag is set, a null route session is treated as "the route has
     * not caught up", not as "signed out".
     */
    clientAuthoritative: boolean;
}

const initialLogin: ILoginFormState = {
    email: "",
    server: "en",
    otp: "",
    isOTPSent: false,
    cooldownUntil: 0,
    biliUsername: "",
    biliPassword: "",
    biliPhone: "",
    biliSmsCode: "",
    biliUseSms: false,
    isBiliCodeSent: false,
    biliCooldownUntil: 0,
    cnPhone: "",
    cnPassword: "",
    cnSmsCode: "",
    cnUseSms: false,
    isCnCodeSent: false,
    cnCooldownUntil: 0,
    saveCredentials: true,
};

/**
 * Where the remembered answer lives.
 *
 * localStorage, per browser, not per account: the choice has to be readable
 * BEFORE anyone is signed in, which is exactly when no account-scoped setting
 * can be reached. A wrapped read because a private window or blocked site data
 * makes the accessor itself throw, and the default when it does is the old
 * behaviour, never a silent opt-out.
 */
const SAVE_CREDENTIALS_KEY = "auth:save-credentials";

export function readRememberedSaveCredentials(): boolean {
    if (typeof window === "undefined") return true;
    try {
        return window.localStorage.getItem(SAVE_CREDENTIALS_KEY) !== "0";
    } catch {
        return true;
    }
}

function rememberSaveCredentials(value: boolean): void {
    if (typeof window === "undefined") return;
    try {
        window.localStorage.setItem(SAVE_CREDENTIALS_KEY, value ? "1" : "0");
    } catch {
        // Private window or blocked storage: the choice still applies to this
        // login, it just is not remembered for the next one.
    }
}

export const authStore = new Store<IAuthState>({ user: null, status: "idle", login: initialLogin, dialogOpen: false, postLoginRedirect: null, clientAuthoritative: false });

export const authActions = {
    setUser: (user: ISession | null) => authStore.setState((s) => ({ ...s, user, status: "ready", clientAuthoritative: user !== null })),
    /**
     * Apply the session the route context resolved. A non-null session always
     * wins and retires the client-authoritative flag. A null one is IGNORED
     * while that flag is set: see `clientAuthoritative`.
     */
    setUserFromRoute: (user: ISession | null) =>
        authStore.setState((s) => {
            // Exactly ONE null is absorbed, and it retires the flag. Bounding
            // it to a single route load keeps a genuinely dead session from
            // being papered over: the next null after this one signs out.
            if (user === null && s.clientAuthoritative) return { ...s, clientAuthoritative: false };
            return { ...s, user, status: "ready", clientAuthoritative: false };
        }),
    setLoading: () => authStore.setState((s) => ({ ...s, status: "loading" })),
    clear: () => authStore.setState((s) => ({ ...s, user: null, status: "ready", login: { ...initialLogin, saveCredentials: s.login.saveCredentials }, dialogOpen: false, postLoginRedirect: null, clientAuthoritative: false })),

    setLoginEmail: (email: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, email } })),
    setLoginServer: (server: AKServer) => authStore.setState((s) => ({ ...s, login: { ...s.login, server } })),
    setLoginOTP: (otp: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, otp } })),
    markOTPSent: (cooldownSeconds = 60) =>
        authStore.setState((s) => ({
            ...s,
            login: { ...s.login, isOTPSent: true, cooldownUntil: Date.now() + cooldownSeconds * 1000 },
        })),
    resetLoginOTP: () => authStore.setState((s) => ({ ...s, login: { ...s.login, otp: "", isOTPSent: false, cooldownUntil: 0 } })),

    setBiliUsername: (biliUsername: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, biliUsername } })),
    setBiliPassword: (biliPassword: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, biliPassword } })),

    setBiliPhone: (biliPhone: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, biliPhone } })),
    setBiliSmsCode: (biliSmsCode: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, biliSmsCode } })),
    setBiliUseSms: (biliUseSms: boolean) => authStore.setState((s) => ({ ...s, login: { ...s.login, biliUseSms } })),
    markBiliCodeSent: (cooldownSeconds = 60) =>
        authStore.setState((s) => ({
            ...s,
            login: { ...s.login, isBiliCodeSent: true, biliCooldownUntil: Date.now() + cooldownSeconds * 1000 },
        })),
    resetBiliCode: () => authStore.setState((s) => ({ ...s, login: { ...s.login, biliSmsCode: "", isBiliCodeSent: false, biliCooldownUntil: 0 } })),

    setCnPhone: (cnPhone: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, cnPhone } })),
    setCnPassword: (cnPassword: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, cnPassword } })),
    setCnSmsCode: (cnSmsCode: string) => authStore.setState((s) => ({ ...s, login: { ...s.login, cnSmsCode } })),
    setCnUseSms: (cnUseSms: boolean) => authStore.setState((s) => ({ ...s, login: { ...s.login, cnUseSms } })),
    markCnCodeSent: (cooldownSeconds = 60) =>
        authStore.setState((s) => ({
            ...s,
            login: { ...s.login, isCnCodeSent: true, cnCooldownUntil: Date.now() + cooldownSeconds * 1000 },
        })),
    resetCnCode: () => authStore.setState((s) => ({ ...s, login: { ...s.login, cnSmsCode: "", isCnCodeSent: false, cnCooldownUntil: 0 } })),

    setSaveCredentials: (saveCredentials: boolean) => {
        rememberSaveCredentials(saveCredentials);
        authStore.setState((s) => ({ ...s, login: { ...s.login, saveCredentials } }));
    },
    /** Applies the remembered answer to the form, on mount. */
    hydrateSaveCredentials: () => authStore.setState((s) => ({ ...s, login: { ...s.login, saveCredentials: readRememberedSaveCredentials() } })),

    // Keeps `saveCredentials`: see its doc comment on ILoginFormState.
    resetLoginForm: () => authStore.setState((s) => ({ ...s, login: { ...initialLogin, saveCredentials: s.login.saveCredentials } })),

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
