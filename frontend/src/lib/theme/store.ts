import { Store } from "@tanstack/store";
import { type Accent, applyAccentColors, clearAccentColors, DEFAULT_PRIMARY_HUE, generateAccentColors, getStoredAccent, setStoredAccent } from "#/lib/theme/color-utils";

export type ThemeMode = "light" | "dark" | "auto";
export type ResolvedTheme = "light" | "dark";

interface IThemeState {
    mode: ThemeMode;
    resolved: ResolvedTheme;
    accent: Accent | null;
    hydrated: boolean;
}

const THEME_MODE_STORAGE_KEY = "theme";

function getInitialMode(): ThemeMode {
    if (typeof window === "undefined") return "auto";
    const stored = window.localStorage.getItem(THEME_MODE_STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "auto") return stored;
    return "auto";
}

function getSystemResolved(): ResolvedTheme {
    if (typeof window === "undefined") return "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function resolveMode(mode: ThemeMode): ResolvedTheme {
    return mode === "auto" ? getSystemResolved() : mode;
}

export const themeStore = new Store<IThemeState>({
    mode: "auto",
    resolved: "dark",
    accent: null,
    hydrated: false,
});

function applyMode(mode: ThemeMode): ResolvedTheme {
    if (typeof document === "undefined") return resolveMode(mode);
    const resolved = resolveMode(mode);
    const root = document.documentElement;
    root.classList.remove("light", "dark");
    root.classList.add(resolved);
    if (mode === "auto") {
        root.removeAttribute("data-theme");
    } else {
        root.setAttribute("data-theme", mode);
    }
    root.style.colorScheme = resolved;
    return resolved;
}

function applyAccent(accent: Accent | null, resolved: ResolvedTheme): void {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    if (accent === null) {
        clearAccentColors(root);
        return;
    }
    applyAccentColors(root, generateAccentColors(accent, resolved === "dark"));
}

export function hydrateTheme(): void {
    if (typeof window === "undefined") return;
    const mode = getInitialMode();
    const accent = getStoredAccent();
    const resolved = applyMode(mode);
    applyAccent(accent, resolved);
    themeStore.setState(() => ({ mode, resolved, accent, hydrated: true }));
}

let systemMediaQuery: MediaQueryList | null = null;
let systemListener: ((ev: MediaQueryListEvent) => void) | null = null;

export function subscribeSystemTheme(): () => void {
    if (typeof window === "undefined") return () => {};
    systemMediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    systemListener = () => {
        const { mode, accent } = themeStore.state;
        if (mode !== "auto") return;
        const resolved = applyMode(mode);
        applyAccent(accent, resolved);
        themeStore.setState((s) => ({ ...s, resolved }));
    };
    systemMediaQuery.addEventListener("change", systemListener);
    return () => {
        if (systemMediaQuery && systemListener) systemMediaQuery.removeEventListener("change", systemListener);
        systemMediaQuery = null;
        systemListener = null;
    };
}

export const themeActions = {
    setMode(mode: ThemeMode): void {
        const resolved = applyMode(mode);
        applyAccent(themeStore.state.accent, resolved);
        themeStore.setState((s) => ({ ...s, mode, resolved }));
        if (typeof window !== "undefined") {
            window.localStorage.setItem(THEME_MODE_STORAGE_KEY, mode);
        }
    },
    setPresetHue(hue: number): void {
        const normalized = ((Math.round(hue) % 360) + 360) % 360;
        const accent: Accent = { type: "preset", hue: normalized };
        applyAccent(accent, themeStore.state.resolved);
        themeStore.setState((s) => ({ ...s, accent }));
        setStoredAccent(accent);
    },
    setCustomHex(hex: string): void {
        const normalized = hex.startsWith("#") ? hex.toLowerCase() : `#${hex.toLowerCase()}`;
        if (!/^#[0-9a-f]{6}$/.test(normalized)) return;
        const accent: Accent = { type: "custom", hex: normalized };
        applyAccent(accent, themeStore.state.resolved);
        themeStore.setState((s) => ({ ...s, accent }));
        setStoredAccent(accent);
    },
    resetAccent(): void {
        applyAccent(null, themeStore.state.resolved);
        themeStore.setState((s) => ({ ...s, accent: null }));
        setStoredAccent(null);
    },
};

export { DEFAULT_PRIMARY_HUE };
