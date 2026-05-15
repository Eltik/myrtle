/**
 * Color utilities for accent customization.
 *
 * Two flavors of accent:
 *  - "preset": stored as a hue (0-360); rendered with the OKLch formula so the
 *    accent adapts to light/dark mode (matching the stylesheet defaults).
 *  - "custom": stored as a raw sRGB hex; applied exactly as picked, with a
 *    luminance-derived foreground so text on the accent stays legible.
 */

export const DEFAULT_PRIMARY_HUE = 25;

export const COLOR_PRESETS = [
    { name: "Orange", hue: 25, color: "#e66e68" },
    { name: "Red", hue: 15, color: "#e65050" },
    { name: "Rose", hue: 350, color: "#e6506e" },
    { name: "Pink", hue: 330, color: "#e650a0" },
    { name: "Purple", hue: 300, color: "#c050e6" },
    { name: "Violet", hue: 280, color: "#9050e6" },
    { name: "Blue", hue: 250, color: "#5070e6" },
    { name: "Cyan", hue: 200, color: "#50b0e6" },
    { name: "Teal", hue: 180, color: "#50d6c6" },
    { name: "Green", hue: 145, color: "#50e680" },
    { name: "Lime", hue: 120, color: "#70e650" },
    { name: "Yellow", hue: 85, color: "#c6d650" },
] as const;

export type Accent = { type: "preset"; hue: number } | { type: "custom"; hex: string };

export interface IThemeColorSet {
    primary: string;
    primaryForeground: string;
    ring: string;
    chart1: string;
    sidebarPrimary: string;
    sidebarPrimaryForeground: string;
    sidebarRing: string;
    glowPrimary: string;
    glowPrimaryIntense: string;
    glowTextIcon: string;
}

const LIGHT_FG = "oklch(0.985 0.002 285)";
const DARK_FG = "oklch(0.13 0.005 285)";

function generateFromHue(hue: number, isDark: boolean): IThemeColorSet {
    const L = isDark ? "0.75" : "0.58";
    const C = isDark ? "0.15" : "0.22";
    const base = `oklch(${L} ${C} ${hue})`;
    return {
        primary: base,
        primaryForeground: isDark ? DARK_FG : LIGHT_FG,
        ring: base,
        chart1: base,
        sidebarPrimary: base,
        sidebarPrimaryForeground: isDark ? DARK_FG : LIGHT_FG,
        sidebarRing: base,
        glowPrimary: `oklch(${L} ${C} ${hue} / ${isDark ? "0.5" : "0.35"})`,
        glowPrimaryIntense: `oklch(${L} ${C} ${hue} / ${isDark ? "0.8" : "0.55"})`,
        glowTextIcon: `oklch(${L} ${C} ${hue} / ${isDark ? "0.6" : "0.55"})`,
    };
}

function generateFromHex(hex: string): IThemeColorSet {
    const fg = pickForeground(hex);
    return {
        primary: hex,
        primaryForeground: fg,
        ring: hex,
        chart1: hex,
        sidebarPrimary: hex,
        sidebarPrimaryForeground: fg,
        sidebarRing: hex,
        glowPrimary: hexWithAlpha(hex, 0.5),
        glowPrimaryIntense: hexWithAlpha(hex, 0.8),
        glowTextIcon: hexWithAlpha(hex, 0.6),
    };
}

/** Pick the appropriate foreground (light vs dark) based on the accent's luminance. */
function pickForeground(hex: string): string {
    const rgb = hexToRgb(hex);
    if (!rgb) return LIGHT_FG;
    const lum = relativeLuminance(rgb.r, rgb.g, rgb.b);
    // WCAG threshold ~0.5 separates "light" surfaces (need dark text) from dark ones.
    return lum > 0.55 ? DARK_FG : LIGHT_FG;
}

function relativeLuminance(r: number, g: number, b: number): number {
    return 0.2126 * srgbToLinear(r) + 0.7152 * srgbToLinear(g) + 0.0722 * srgbToLinear(b);
}

function hexWithAlpha(hex: string, alpha: number): string {
    const rgb = hexToRgb(hex);
    if (!rgb) return hex;
    return `rgb(${Math.round(rgb.r * 255)} ${Math.round(rgb.g * 255)} ${Math.round(rgb.b * 255)} / ${alpha})`;
}

export function generateAccentColors(accent: Accent, isDark: boolean): IThemeColorSet {
    return accent.type === "preset" ? generateFromHue(accent.hue, isDark) : generateFromHex(accent.hex);
}

const CSS_VAR_MAP: Record<keyof IThemeColorSet, string> = {
    primary: "--primary",
    primaryForeground: "--primary-foreground",
    ring: "--ring",
    chart1: "--chart-1",
    sidebarPrimary: "--sidebar-primary",
    sidebarPrimaryForeground: "--sidebar-primary-foreground",
    sidebarRing: "--sidebar-ring",
    glowPrimary: "--glow-primary",
    glowPrimaryIntense: "--glow-primary-intense",
    glowTextIcon: "--glow-text-icon",
};

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

/** Produce a preview swatch hex for a stored accent value. */
export function previewHex(accent: Accent): string {
    if (accent.type === "custom") return accent.hex;
    return oklchToHex(0.7, 0.15, accent.hue);
}

/**
 * Render the accent as the literal hex it produces on screen for a given
 * theme - used to pre-fill the native color input so it tracks the live state.
 */
export function accentToRenderedHex(accent: Accent | null, isDark: boolean): string {
    if (accent === null) {
        return oklchToHex(isDark ? 0.75 : 0.58, isDark ? 0.15 : 0.22, DEFAULT_PRIMARY_HUE);
    }
    if (accent.type === "custom") return accent.hex;
    return oklchToHex(isDark ? 0.75 : 0.58, isDark ? 0.15 : 0.22, accent.hue);
}

function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
    const cleaned = hex.replace(/^#/, "");
    if (cleaned.length !== 3 && cleaned.length !== 6) return null;
    const expanded =
        cleaned.length === 3
            ? cleaned
                  .split("")
                  .map((c) => c + c)
                  .join("")
            : cleaned;
    const n = Number.parseInt(expanded, 16);
    if (Number.isNaN(n)) return null;
    return {
        r: ((n >> 16) & 0xff) / 255,
        g: ((n >> 8) & 0xff) / 255,
        b: (n & 0xff) / 255,
    };
}

function srgbToLinear(c: number): number {
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

function linearToSrgb(c: number): number {
    return c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055;
}

function oklchToHex(L: number, C: number, hueDeg: number): string {
    const h = hueDeg * (Math.PI / 180);
    const a = C * Math.cos(h);
    const b = C * Math.sin(h);

    const l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    const m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    const s_ = L - 0.0894841775 * a - 1.291485548 * b;

    const lc = l_ ** 3;
    const mc = m_ ** 3;
    const sc = s_ ** 3;

    const r = 4.0767416621 * lc - 3.3077115913 * mc + 0.2309699292 * sc;
    const g = -1.2684380046 * lc + 2.6097574011 * mc - 0.3413193965 * sc;
    const bb = -0.0041960863 * lc - 0.7034186147 * mc + 1.707614701 * sc;

    return `#${toHexChannel(r)}${toHexChannel(g)}${toHexChannel(bb)}`;
}

function toHexChannel(linear: number): string {
    const clamped = Math.max(0, Math.min(1, linear));
    const srgb = linearToSrgb(clamped);
    return Math.round(Math.max(0, Math.min(255, srgb * 255)))
        .toString(16)
        .padStart(2, "0");
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
        return Number.isNaN(hue) ? null : { type: "preset", hue: ((hue % 360) + 360) % 360 };
    }
    if (raw.startsWith("c:")) {
        const hex = raw.slice(2);
        return /^#[0-9a-fA-F]{6}$/.test(hex) ? { type: "custom", hex: hex.toLowerCase() } : null;
    }
    // Legacy: bare number was previously stored as a hue.
    const legacy = Number.parseFloat(raw);
    if (!Number.isNaN(legacy)) return { type: "preset", hue: ((legacy % 360) + 360) % 360 };
    return null;
}
