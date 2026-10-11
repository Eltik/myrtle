import { Store } from "@tanstack/store";
import { type Accent, applyAccentColors, clearAccentColors, generateAccentColors, getStoredAccent, setStoredAccent } from "#/lib/theme/color-utils";

export type ThemeMode = "light" | "dark" | "auto";
export type ResolvedTheme = "light" | "dark";

interface IThemeState {
    mode: ThemeMode;
    resolved: ResolvedTheme;
    accent: Accent | null;
    /** Play animated L2D (dynamic) operator art site-wide. On by default; users can opt out. */
    dynamicArtwork: boolean;
    /**
     * Show operators that only exist on the CN server under their Latin
     * appellation instead of the Han name. On by default: the site is read in
     * English, and nobody types 予愿安洁莉娜 to find Angelina.
     */
    latinNames: boolean;
    /**
     * Show the hidden factions (`SubPower` ids the card never shows) on
     * operator pages. Off by default: some of them are story spoilers.
     */
    spoilers: boolean;
    /** Show the game maintenance countdown in the top bar. On by default. */
    maintenanceCountdown: boolean;
    hydrated: boolean;
}

const THEME_MODE_STORAGE_KEY = "theme";
const DYNAMIC_ART_STORAGE_KEY = "myrtle-dynamic-art";
const LATIN_NAMES_STORAGE_KEY = "myrtle-latin-names";
const SPOILERS_STORAGE_KEY = "myrtle-spoilers";
const MAINTENANCE_COUNTDOWN_STORAGE_KEY = "myrtle-maintenance-countdown";

function getInitialMode(): ThemeMode {
    if (typeof window === "undefined") return "auto";
    const stored = window.localStorage.getItem(THEME_MODE_STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "auto") return stored;
    return "auto";
}

function getInitialDynamicArtwork(): boolean {
    // On by default; only an explicit opt-out ("false") disables it.
    if (typeof window === "undefined") return true;
    const stored = window.localStorage.getItem(DYNAMIC_ART_STORAGE_KEY);
    return stored === null ? true : stored === "true";
}

function getInitialLatinNames(): boolean {
    // On by default; only an explicit opt-out ("false") disables it.
    if (typeof window === "undefined") return true;
    const stored = window.localStorage.getItem(LATIN_NAMES_STORAGE_KEY);
    return stored === null ? true : stored === "true";
}

function getInitialSpoilers(): boolean {
    // Off by default; only an explicit opt-in ("true") enables it.
    if (typeof window === "undefined") return false;
    return window.localStorage.getItem(SPOILERS_STORAGE_KEY) === "true";
}

function getInitialMaintenanceCountdown(): boolean {
    // On by default; only an explicit opt-out ("false") disables it.
    if (typeof window === "undefined") return true;
    const stored = window.localStorage.getItem(MAINTENANCE_COUNTDOWN_STORAGE_KEY);
    return stored === null ? true : stored === "true";
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
    dynamicArtwork: true,
    latinNames: true,
    spoilers: false,
    maintenanceCountdown: true,
    hydrated: false,
});

function applyMode(mode: ThemeMode): ResolvedTheme {
    if (typeof document === "undefined") return resolveMode(mode);
    const resolved = resolveMode(mode);
    const root = document.documentElement;
    // The `.light`/`.dark` class is the only theme switch: Tailwind's `dark:` variant,
    // the semantic tokens in styles.css and the CSS modules all key off it.
    // Write the class only when it changes. Removing and re-adding the class
    // that is already there restyled the whole document on every hydration
    // (4,982 elements, 96 to 159 ms on the story library, measured 2026-09-29),
    // and the SSR shell already carries the right class on most loads.
    if (!root.classList.contains(resolved) || root.classList.contains(resolved === "dark" ? "light" : "dark")) {
        root.classList.remove("light", "dark");
        root.classList.add(resolved);
    }
    if (root.style.colorScheme !== resolved) root.style.colorScheme = resolved;
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
    themeStore.setState(() => ({ mode, resolved, accent, dynamicArtwork: getInitialDynamicArtwork(), latinNames: getInitialLatinNames(), spoilers: getInitialSpoilers(), maintenanceCountdown: getInitialMaintenanceCountdown(), hydrated: true }));
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
    setDynamicArtwork(enabled: boolean): void {
        themeStore.setState((s) => ({ ...s, dynamicArtwork: enabled }));
        if (typeof window !== "undefined") {
            window.localStorage.setItem(DYNAMIC_ART_STORAGE_KEY, String(enabled));
        }
    },
    setLatinNames(enabled: boolean): void {
        themeStore.setState((s) => ({ ...s, latinNames: enabled }));
        if (typeof window !== "undefined") {
            window.localStorage.setItem(LATIN_NAMES_STORAGE_KEY, String(enabled));
        }
    },
    setSpoilers(enabled: boolean): void {
        themeStore.setState((s) => ({ ...s, spoilers: enabled }));
        if (typeof window !== "undefined") {
            window.localStorage.setItem(SPOILERS_STORAGE_KEY, String(enabled));
        }
    },

    setMaintenanceCountdown(enabled: boolean): void {
        themeStore.setState((s) => ({ ...s, maintenanceCountdown: enabled }));
        if (typeof window !== "undefined") {
            window.localStorage.setItem(MAINTENANCE_COUNTDOWN_STORAGE_KEY, String(enabled));
        }
    },
};
