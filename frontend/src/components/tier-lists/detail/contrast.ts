export type ReadableTextColor = "white" | "black";

/**
 * The text colour for a label painted on `color`: whichever of white and black
 * has the higher WCAG contrast ratio against it. The crossover sits at a
 * relative luminance of about 0.179. The old rule picked black only above
 * luminance 0.55 (or oklch L 0.72), so every default tier colour got white at
 * 1.98 to 2.83 where black measured 7.51 to 10.63.
 */
export function readableTextColor(color: string | null | undefined): ReadableTextColor {
    const luminance = color ? colorLuminance(color) : null;
    if (luminance === null) return "white";
    return contrastRatio(luminance, 0) > contrastRatio(luminance, 1) ? "black" : "white";
}

/** WCAG contrast ratio between two relative luminances. */
export function contrastRatio(a: number, b: number): number {
    const [hi, lo] = a > b ? [a, b] : [b, a];
    return (hi + 0.05) / (lo + 0.05);
}

/** WCAG relative luminance of a `#hex` or `oklch()` colour, or null when it cannot be read. */
export function colorLuminance(color: string): number | null {
    const trimmed = color.trim();
    if (trimmed.startsWith("#")) return hexLuminance(trimmed.slice(1));
    const oklch = trimmed.match(/^oklch\(\s*([0-9.]+)(%?)\s+([0-9.]+)\s+([0-9.]+)/i);
    if (oklch) {
        const L = Number.parseFloat(oklch[1] ?? "") / (oklch[2] === "%" ? 100 : 1);
        const C = Number.parseFloat(oklch[3] ?? "");
        const H = Number.parseFloat(oklch[4] ?? "");
        if (![L, C, H].every(Number.isFinite)) return null;
        return linearLuminance(oklchToLinearSrgb(L, C, H));
    }
    return null;
}

function hexLuminance(hex: string): number | null {
    const expanded =
        hex.length === 3 || hex.length === 4
            ? hex
                  .slice(0, 3)
                  .split("")
                  .map((c) => c + c)
                  .join("")
            : hex.length >= 6
              ? hex.slice(0, 6)
              : null;
    if (!expanded) return null;
    const channels = [0, 2, 4].map((i) => Number.parseInt(expanded.slice(i, i + 2), 16) / 255);
    if (channels.some((c) => Number.isNaN(c))) return null;
    const lin = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
    return linearLuminance(channels.map(lin) as [number, number, number]);
}

function oklchToLinearSrgb(L: number, C: number, H: number): [number, number, number] {
    const a = C * Math.cos((H * Math.PI) / 180);
    const b = C * Math.sin((H * Math.PI) / 180);
    const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
    const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
    const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
    return [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s, -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s];
}

function linearLuminance([r, g, b]: [number, number, number]): number {
    const clamp = (c: number) => Math.min(1, Math.max(0, c));
    return 0.2126 * clamp(r) + 0.7152 * clamp(g) + 0.0722 * clamp(b);
}
