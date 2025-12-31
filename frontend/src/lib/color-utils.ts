/**
 * Color utilities for theme customization.
 * Works with OKLch color space for perceptually uniform color manipulation.
 */

/**
 * Default hue value for the primary color (orange).
 */
export const DEFAULT_PRIMARY_HUE = 25;

/**
 * Preset colors with their hue values for quick selection.
 */
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

/**
 * Generates CSS variable values for a given hue.
 * Returns an object with all primary-related color variables.
 */
export function generatePrimaryColors(hue: number) {
    // Secondary accent hue for gradient effects (slightly warmer)
    const secondaryHue = (hue + 15) % 360;

    return {
        // Light mode colors
        light: {
            primary: `oklch(0.65 0.18 ${hue})`,
            primaryForeground: "oklch(0.985 0.002 285)",
            ring: `oklch(0.65 0.18 ${hue})`,
            chart1: `oklch(0.65 0.18 ${hue})`,
            sidebarPrimary: `oklch(0.65 0.18 ${hue})`,
            sidebarPrimaryForeground: "oklch(0.985 0.002 285)",
            sidebarRing: `oklch(0.65 0.18 ${hue})`,
            glowPrimary: `oklch(0.65 0.18 ${hue} / 0.3)`,
            glowPrimaryIntense: `oklch(0.65 0.18 ${hue} / 0.5)`,
        },
        // Dark mode colors
        dark: {
            primary: `oklch(0.75 0.15 ${hue})`,
            primaryForeground: "oklch(0.13 0.005 285)",
            ring: `oklch(0.75 0.15 ${hue})`,
            chart1: `oklch(0.75 0.15 ${hue})`,
            sidebarPrimary: `oklch(0.75 0.15 ${hue})`,
            sidebarPrimaryForeground: "oklch(0.13 0.005 285)",
            sidebarRing: `oklch(0.75 0.15 ${hue})`,
            glowPrimary: `oklch(0.75 0.15 ${hue} / 0.5)`,
            glowPrimaryIntense: `oklch(0.75 0.15 ${hue} / 0.8)`,
        },
        // Nav pill hover (needs both light and dark)
        navPillHover: {
            light: `oklch(0.65 0.18 ${hue} / 0.15)`,
            lightEnd: `oklch(0.65 0.18 ${hue} / 0.1)`,
            dark: `oklch(0.24 0.025 ${hue} / 0.5)`,
            darkEnd: `oklch(0.2 0.02 ${hue} / 0.35)`,
        },
        // Glow effects for headers and nav (theme-independent, uses dark mode values for visibility)
        glowEffects: {
            // Header border gradient
            headerBorder: `oklch(0.75 0.15 ${hue} / 0.5)`,
            headerBorderSecondary: `oklch(0.85 0.12 ${secondaryHue} / 0.3)`,
            // Header blur glow
            headerGlow: `oklch(0.75 0.15 ${hue} / 0.15)`,
            headerGlowSecondary: `oklch(0.80 0.10 ${secondaryHue} / 0.1)`,
            // Nav radial glow
            navGlow: `oklch(0.75 0.15 ${hue} / 0.25)`,
            // Nav active line
            navLineCenter: `oklch(0.75 0.15 ${hue} / 0.8)`,
            navLineShadow: `oklch(0.75 0.15 ${hue} / 0.6)`,
            navLineShadowSoft: `oklch(0.75 0.15 ${hue} / 0.4)`,
            // Text glow effects
            textGlowStrong: `oklch(0.75 0.15 ${hue} / 0.5)`,
            textGlowMedium: `oklch(0.75 0.15 ${hue} / 0.3)`,
            textGlowSoft: `oklch(0.75 0.15 ${hue} / 0.15)`,
            textGlowIcon: `oklch(0.75 0.15 ${hue} / 0.6)`,
        },
    };
}

/**
 * Converts a hue value (0-360) to a preview hex color.
 * Used for displaying a color swatch preview.
 */
export function hueToPreviewColor(hue: number): string {
    // Convert OKLch to approximate RGB for preview
    // Using a simplified conversion at L=0.65, C=0.18
    const h = hue * (Math.PI / 180);
    const L = 0.7;
    const C = 0.15;

    // OKLab a and b from chroma and hue
    const a = C * Math.cos(h);
    const b = C * Math.sin(h);

    // OKLab to linear sRGB (approximate)
    const l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    const m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    const s_ = L - 0.0894841775 * a - 1.291485548 * b;

    const l = l_ * l_ * l_;
    const m = m_ * m_ * m_;
    const s = s_ * s_ * s_;

    const r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    const g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    const B = -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s;

    // Clamp and convert to hex
    const toHex = (c: number) => {
        const clamped = Math.max(0, Math.min(1, c));
        const srgb = clamped <= 0.0031308 ? 12.92 * clamped : 1.055 * clamped ** (1 / 2.4) - 0.055;
        return Math.round(Math.max(0, Math.min(255, srgb * 255)))
            .toString(16)
            .padStart(2, "0");
    };

    return `#${toHex(r)}${toHex(g)}${toHex(B)}`;
}

/**
 * Storage key for persisting accent color preference.
 */
export const ACCENT_COLOR_STORAGE_KEY = "myrtle-accent-hue";

/**
 * Reads the stored accent hue from localStorage.
 * Returns null if not set or if running on server.
 */
export function getStoredAccentHue(): number | null {
    if (typeof window === "undefined") return null;
    const stored = localStorage.getItem(ACCENT_COLOR_STORAGE_KEY);
    if (stored === null) return null;
    const parsed = Number.parseFloat(stored);
    return Number.isNaN(parsed) ? null : parsed;
}

/**
 * Saves the accent hue to localStorage.
 */
export function setStoredAccentHue(hue: number | null): void {
    if (typeof window === "undefined") return;
    if (hue === null) {
        localStorage.removeItem(ACCENT_COLOR_STORAGE_KEY);
    } else {
        localStorage.setItem(ACCENT_COLOR_STORAGE_KEY, hue.toString());
    }
}
