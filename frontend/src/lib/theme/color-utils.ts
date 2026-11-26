/**
 * Accent colours. A "preset" is a stored hue (0-360) rendered through OKLch, so
 * it shifts with light/dark like the stylesheet defaults. A "custom" is a raw
 * sRGB hex applied as picked, with its label picked by the higher WCAG ratio.
 */

import { accentKit } from "./accent-kit";

export const DEFAULT_PRIMARY_HUE = 25;

export const PRESET_MATCH_TOLERANCE = 5;

export const COLOR_PRESETS = [
    { name: "Orange", hue: 25 },
    { name: "Red", hue: 15 },
    { name: "Rose", hue: 350 },
    { name: "Pink", hue: 330 },
    { name: "Purple", hue: 300 },
    { name: "Violet", hue: 280 },
    { name: "Blue", hue: 250 },
    { name: "Cyan", hue: 200 },
    { name: "Teal", hue: 180 },
    { name: "Green", hue: 145 },
    { name: "Lime", hue: 120 },
    { name: "Yellow", hue: 85 },
] as const;

export type Accent = { type: "preset"; hue: number } | { type: "custom"; hex: string };

export interface IThemeColorSet {
    primary: string;
    primaryForeground: string;
    primaryHover: string;
    primaryInk: string;
    ring: string;
    chart1: string;
    sidebarPrimary: string;
    sidebarPrimaryForeground: string;
    sidebarRing: string;
    glowPrimary: string;
    glowPrimaryIntense: string;
}

const kit = accentKit();

/** Black or white, whichever has the higher WCAG ratio on the given `#rrggbb` fill. */
export const readableOn: (hex: string) => string = kit.readableOn;

const CSS_VAR_MAP: Record<keyof IThemeColorSet, string> = {
    primary: "--primary",
    primaryForeground: "--primary-foreground",
    primaryHover: "--primary-hover",
    primaryInk: "--primary-ink",
    ring: "--ring",
    chart1: "--chart-1",
    sidebarPrimary: "--sidebar-primary",
    sidebarPrimaryForeground: "--sidebar-primary-foreground",
    sidebarRing: "--sidebar-ring",
    glowPrimary: "--glow-primary",
    glowPrimaryIntense: "--glow-primary-intense",
};

/**
 * A preset renders its hue through OKLch with L solved per hue and theme (see
 * `accentKit`). A custom hex stays the fill (`--primary`, `--chart-1`); text
 * (`--primary-ink`, what `text-primary` reads) and the focus ring get per-theme
 * variants at the same hue that clear 4.5 and 3 against the page.
 */
export function generateAccentColors(accent: Accent, isDark: boolean): IThemeColorSet {
    const raw = accent.type === "preset" ? `h:${accent.hue}` : `c:${expandHex(accent.hex)}`;
    const vars = kit.accentVars(raw, isDark);
    if (!vars) {
        // Only reachable for a custom hex that bypassed `parseAccent`: paint it as given.
        const hex = accent.type === "custom" ? accent.hex : "";
        return { primary: hex, primaryForeground: "#ffffff", primaryHover: hex, primaryInk: hex, ring: hex, chart1: hex, sidebarPrimary: hex, sidebarPrimaryForeground: "#ffffff", sidebarRing: hex, glowPrimary: hex, glowPrimaryIntense: hex };
    }
    const out = {} as IThemeColorSet;
    for (const key of Object.keys(CSS_VAR_MAP) as Array<keyof IThemeColorSet>) {
        out[key] = vars[CSS_VAR_MAP[key]];
    }
    return out;
}

function expandHex(hex: string): string {
    return /^#[0-9a-f]{3}$/i.test(hex) ? `#${hex[1]}${hex[1]}${hex[2]}${hex[2]}${hex[3]}${hex[3]}` : hex;
}

export function applyAccentColors(root: HTMLElement, colors: IThemeColorSet): void {
    for (const key of Object.keys(CSS_VAR_MAP) as Array<keyof IThemeColorSet>) {
        root.style.setProperty(CSS_VAR_MAP[key], colors[key]);
    }
}

export function clearAccentColors(root: HTMLElement): void {
    for (const key of Object.keys(CSS_VAR_MAP) as Array<keyof IThemeColorSet>) {
        root.style.removeProperty(CSS_VAR_MAP[key]);
    }
}

/** The hex an accent actually renders as in the given theme, for the preset swatches and the native colour input. */
export function accentToRenderedHex(accent: Accent | null, isDark: boolean): string {
    if (accent?.type === "custom") return accent.hex;
    const hue = accent?.hue ?? DEFAULT_PRIMARY_HUE;
    const { L, C } = kit.presetFill(hue, isDark);
    return kit.linearToHex(kit.lchToLinear(L, C, hue));
}

export const ACCENT_STORAGE_KEY = "myrtle-accent";

export function getStoredAccent(): Accent | null {
    if (typeof window === "undefined") return null;
    const raw = window.localStorage.getItem(ACCENT_STORAGE_KEY);
    if (raw === null) return null;
    return parseAccent(raw);
}

export function setStoredAccent(accent: Accent | null): void {
    if (typeof window === "undefined") return;
    if (accent === null) {
        window.localStorage.removeItem(ACCENT_STORAGE_KEY);
    } else {
        window.localStorage.setItem(ACCENT_STORAGE_KEY, serializeAccent(accent));
    }
}

export function serializeAccent(accent: Accent): string {
    return accent.type === "preset" ? `h:${accent.hue}` : `c:${accent.hex}`;
}

export function parseAccent(raw: string): Accent | null {
    if (raw.startsWith("h:")) {
        const hue = Number.parseFloat(raw.slice(2));
        return !Number.isFinite(hue) ? null : { type: "preset", hue: ((hue % 360) + 360) % 360 };
    }
    if (raw.startsWith("c:")) {
        const hex = raw.slice(2);
        return /^#[0-9a-fA-F]{6}$/.test(hex) ? { type: "custom", hex: hex.toLowerCase() } : null;
    }
    // Legacy: bare number was previously stored as a hue.
    const legacy = Number.parseFloat(raw);
    if (Number.isFinite(legacy)) return { type: "preset", hue: ((legacy % 360) + 360) % 360 };
    return null;
}
