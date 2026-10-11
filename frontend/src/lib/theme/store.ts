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

type ToggleField = "dynamicArtwork" | "latinNames" | "spoilers" | "maintenanceCountdown";

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

/**
 * A stored on/off toggle: `fallback` when nothing is stored (or off the client), else on only
 * for the exact string "true".
 */
function readToggle(key: string, fallback: boolean): boolean {
    if (typeof window === "undefined") return fallback;
    const stored = window.localStorage.getItem(key);
    return stored === null ? fallback : stored === "true";
}

/** Sets one toggle in the store and keeps it in localStorage under `key`. */
function writeToggle(field: ToggleField, key: string, enabled: boolean): void {
    themeStore.setState((s) => ({ ...s, [field]: enabled }));
    if (typeof window !== "undefined") {
        window.localStorage.setItem(key, String(enabled));
    }
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
    themeStore.setState(() => ({
        mode,
        resolved,
        accent,
        dynamicArtwork: readToggle(DYNAMIC_ART_STORAGE_KEY, true),
        latinNames: readToggle(LATIN_NAMES_STORAGE_KEY, true),
        // Off by default: only an explicit opt-in shows spoilers.
        spoilers: readToggle(SPOILERS_STORAGE_KEY, false),
        maintenanceCountdown: readToggle(MAINTENANCE_COUNTDOWN_STORAGE_KEY, true),
        hydrated: true,
    }));
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
        writeToggle("dynamicArtwork", DYNAMIC_ART_STORAGE_KEY, enabled);
    },
    setLatinNames(enabled: boolean): void {
        writeToggle("latinNames", LATIN_NAMES_STORAGE_KEY, enabled);
    },
    setSpoilers(enabled: boolean): void {
        writeToggle("spoilers", SPOILERS_STORAGE_KEY, enabled);
    },
    setMaintenanceCountdown(enabled: boolean): void {
        writeToggle("maintenanceCountdown", MAINTENANCE_COUNTDOWN_STORAGE_KEY, enabled);
    },
};
