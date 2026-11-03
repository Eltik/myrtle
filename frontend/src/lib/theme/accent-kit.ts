/**
 * The accent maths, shared by `color-utils.ts` and the pre-paint script in
 * `routes/__root.tsx`. That script is built from `accentKit.toString()`, so this
 * function MUST stay self-contained: no imports, no module-level names, nothing
 * from outside its own body. One copy means the first paint and the hydrated
 * page cannot pick different colours.
 */
export function accentKit() {
    // Page, card and muted per theme, as oklch triples. Mirrors `styles.css`.
    const SURFACES = {
        light: { bg: [0.985, 0.002, 285], card: [1, 0, 0], muted: [0.94, 0.005, 285] },
        dark: { bg: [0.13, 0.005, 285], card: [0.17, 0.005, 285], muted: [0.22, 0.005, 285] },
    };
    // The preset formula before the per-hue solve. Hue 25 (the brand red) passes
    // as is: 4.59 on the light page, 8.49 on the dark one.
    const PRESET = { light: { l: 0.58, c: 0.22 }, dark: { l: 0.75, c: 0.15 } };
    const TEXT_MIN = 4.5;
    const RING_MIN = 3;

    function lchToLinear(L: number, C: number, hueDeg: number): number[] {
        const h = (hueDeg * Math.PI) / 180;
        const a = C * Math.cos(h);
        const b = C * Math.sin(h);
        const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
        const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
        const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
        return [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s, -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s];
    }

    function linearToLch(rgb: number[]): number[] {
        const l = Math.cbrt(0.4122214708 * rgb[0] + 0.5363325363 * rgb[1] + 0.0514459929 * rgb[2]);
        const m = Math.cbrt(0.2119034982 * rgb[0] + 0.6806995451 * rgb[1] + 0.1073969566 * rgb[2]);
        const s = Math.cbrt(0.0883024619 * rgb[0] + 0.2817188376 * rgb[1] + 0.6299787005 * rgb[2]);
        const L = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
        const a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
        const b = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
        return [L, Math.sqrt(a * a + b * b), ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360];
    }

    function inGamut(rgb: number[]): boolean {
        return rgb[0] >= -1e-4 && rgb[0] <= 1.0001 && rgb[1] >= -1e-4 && rgb[1] <= 1.0001 && rgb[2] >= -1e-4 && rgb[2] <= 1.0001;
    }

    function luminance(rgb: number[]): number {
        const c = rgb.map((x) => Math.min(1, Math.max(0, x)));
        return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    }

    function ratio(y1: number, y2: number): number {
        return (Math.max(y1, y2) + 0.05) / (Math.min(y1, y2) + 0.05);
    }

    function hexToLinear(hex: string): number[] | null {
        if (!/^#[0-9a-f]{6}$/i.test(hex)) return null;
        const out = [];
        for (let i = 1; i < 7; i += 2) {
            const c = Number.parseInt(hex.slice(i, i + 2), 16) / 255;
            out.push(c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
        }
        return out;
    }

    function linearToHex(rgb: number[]): string {
        let out = "#";
        for (const x of rgb) {
            const c = Math.min(1, Math.max(0, x));
            const s = c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055;
            out += Math.round(Math.min(255, Math.max(0, s * 255)))
                .toString(16)
                .padStart(2, "0");
        }
        return out;
    }

    /** The most chroma, up to `C`, that keeps (L, hue) inside sRGB, floored to 0.001. */
    function clampChroma(L: number, C: number, hue: number): number {
        if (inGamut(lchToLinear(L, C, hue))) return C;
        let lo = 0;
        let hi = C;
        for (let i = 0; i < 20; i++) {
            const mid = (lo + hi) / 2;
            if (inGamut(lchToLinear(L, mid, hue))) lo = mid;
            else hi = mid;
        }
        return Math.floor(lo * 1000) / 1000;
    }

    function surfaceLum(dark: boolean): number[] {
        const s = dark ? SURFACES.dark : SURFACES.light;
        return [s.bg, s.card, s.muted].map((t) => luminance(lchToLinear(t[0], t[1], t[2])));
    }

    /** Black or white, whichever has the higher WCAG ratio against a fill of luminance `y`. */
    function readableOnLum(y: number): string {
        return ratio(y, 0) >= ratio(y, 1) ? "#000000" : "#ffffff";
    }

    function readableOn(hex: string): string {
        const rgb = hexToLinear(hex);
        return rgb ? readableOnLum(luminance(rgb)) : "#ffffff";
    }

    /** Clears `min` against the page and the card (and muted, for a ring) in that theme. */
    function clears(y: number, dark: boolean, min: number, withMuted: boolean): boolean {
        const s = surfaceLum(dark);
        return ratio(y, s[0]) >= min && ratio(y, s[1]) >= min && (!withMuted || ratio(y, s[2]) >= min);
    }

    /** Hover and pressed fill: the fill moved 12% away from its label, so the label only gains contrast. */
    function hoverOf(fill: string, fg: string): string {
        return `color-mix(in oklab, ${fill} 88%, ${fg === "#000000" ? "white" : "black"})`;
    }

    /**
     * A variant of the hex at the same hue and chroma (chroma clamped to sRGB),
     * stepping oklch L away from the page until it clears `min`. Dark themes step
     * up, light themes down. The hex is returned untouched when it already clears.
     */
    function solveHex(hex: string, dark: boolean, min: number, withMuted: boolean): string {
        const rgb = hexToLinear(hex);
        if (!rgb) return hex;
        if (clears(luminance(rgb), dark, min, withMuted)) return hex;
        const lch = linearToLch(rgb);
        let step = Math.round(lch[0] * 200);
        while (step >= 0 && step <= 200) {
            step += dark ? 1 : -1;
            const L = step / 200;
            const out = linearToHex(lchToLinear(L, clampChroma(L, lch[1], lch[2]), lch[2]));
            const y = luminance(hexToLinear(out) as number[]);
            if (clears(y, dark, min, withMuted)) return out;
        }
        return dark ? "#ffffff" : "#000000";
    }

    /**
     * A preset hue's fill per theme. It doubles as text and focus ring, so it has
     * to clear 4.5 on the page and the card, 3 on muted, and 4.5 under its own
     * label. L steps from the formula (down in light, up in dark) until all four
     * hold; chroma is clamped to sRGB so what is measured is what renders.
     */
    function presetFill(hue: number, dark: boolean): { L: number; C: number; fg: string } {
        const base = dark ? PRESET.dark : PRESET.light;
        let step = Math.round(base.l * 200);
        for (;;) {
            const L = step / 200;
            const C = clampChroma(L, base.c, hue);
            const y = luminance(lchToLinear(L, C, hue));
            const fg = readableOnLum(y);
            const fgOk = ratio(y, fg === "#000000" ? 0 : 1) >= TEXT_MIN;
            if ((fgOk && clears(y, dark, TEXT_MIN, false) && clears(y, dark, RING_MIN, true)) || step <= 0 || step >= 200) return { L, C, fg };
            step += dark ? 1 : -1;
        }
    }

    /**
     * The CSS custom properties for a stored accent (`h:<hue>` or `c:#rrggbb`),
     * or null when the string is not one.
     */
    function accentVars(raw: string, dark: boolean): Record<string, string> | null {
        let fill: string;
        let fg: string;
        let ink: string;
        let ring: string;
        let glow: (a: string) => string;
        if (raw.indexOf("h:") === 0) {
            const parsed = Number.parseFloat(raw.slice(2));
            if (!Number.isFinite(parsed)) return null;
            const hue = ((parsed % 360) + 360) % 360;
            const p = presetFill(hue, dark);
            fill = `oklch(${p.L} ${p.C} ${hue})`;
            fg = p.fg;
            ink = fill;
            ring = fill;
            glow = (a) => `oklch(${p.L} ${p.C} ${hue} / ${a})`;
        } else if (raw.indexOf("c:") === 0) {
            const hex = raw.slice(2).toLowerCase();
            const rgb = hexToLinear(hex);
            if (!rgb) return null;
            fill = hex;
            fg = readableOn(hex);
            ink = solveHex(hex, dark, TEXT_MIN, false);
            ring = solveHex(hex, dark, RING_MIN, true);
            const ch = [1, 3, 5].map((i) => Number.parseInt(hex.slice(i, i + 2), 16)).join(" ");
            glow = (a) => `rgb(${ch} / ${a})`;
        } else {
            return null;
        }
        return {
            "--primary": fill,
            "--primary-foreground": fg,
            "--primary-hover": hoverOf(fill, fg),
            "--primary-ink": ink,
            "--ring": ring,
            "--chart-1": fill,
            "--sidebar-primary": fill,
            "--sidebar-primary-foreground": fg,
            "--sidebar-ring": ring,
            "--glow-primary": glow(dark ? "0.5" : "0.35"),
            "--glow-primary-intense": glow(dark ? "0.8" : "0.55"),
        };
    }

    return { accentVars, readableOn, presetFill, lchToLinear, linearToHex };
}
