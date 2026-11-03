import { describe, expect, it } from "vitest";
import { accentKit } from "./accent-kit";
import { COLOR_PRESETS, generateAccentColors, readableOn } from "./color-utils";

// WCAG measured here independently of the kit: oklch / hex / oklab color-mix -> sRGB -> luminance.
type Rgb = [number, number, number];

function oklabToLinear(L: number, a: number, b: number): Rgb {
    const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
    const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
    const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
    const out = [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s, -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s];
    return out.map((x) => Math.min(1, Math.max(0, x))) as Rgb;
}

function linearToOklab(c: Rgb): Rgb {
    const l = Math.cbrt(0.4122214708 * c[0] + 0.5363325363 * c[1] + 0.0514459929 * c[2]);
    const m = Math.cbrt(0.2119034982 * c[0] + 0.6806995451 * c[1] + 0.1073969566 * c[2]);
    const s = Math.cbrt(0.0883024619 * c[0] + 0.2817188376 * c[1] + 0.6299787005 * c[2]);
    return [0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s, 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s, 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s];
}

function parse(css: string): Rgb {
    const mix = /^color-mix\(in oklab, (.+) (\d+)%, (black|white)\)$/.exec(css);
    if (mix) {
        const [L, a, b] = linearToOklab(parse(mix[1]));
        const t = Number(mix[2]) / 100;
        return oklabToLinear(L * t + (mix[3] === "white" ? 1 - t : 0), a * t, b * t);
    }
    const lch = /^oklch\(([\d.]+) ([\d.]+) ([\d.]+)\)$/.exec(css);
    if (lch) {
        const h = (Number(lch[3]) * Math.PI) / 180;
        return oklabToLinear(Number(lch[1]), Number(lch[2]) * Math.cos(h), Number(lch[2]) * Math.sin(h));
    }
    const hex = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(css);
    if (!hex) throw new Error(`unparsed colour ${css}`);
    return hex.slice(1).map((x) => {
        const c = Number.parseInt(x, 16) / 255;
        return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    }) as Rgb;
}

function contrast(x: string, y: string): number {
    const lum = (c: Rgb) => 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    const a = lum(parse(x));
    const b = lum(parse(y));
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

const SURFACES = {
    light: { bg: "oklch(0.985 0.002 285)", card: "oklch(1 0 0)", muted: "oklch(0.94 0.005 285)" },
    dark: { bg: "oklch(0.13 0.005 285)", card: "oklch(0.17 0.005 285)", muted: "oklch(0.22 0.005 285)" },
};
const HEXES = ["#ff9800", "#2196f3", "#4caf50", "#ffc107", "#9c27b0"];
const THEMES = [false, true] as const;

describe("readableOn (K1)", () => {
    it("picks whichever of black and white has the higher ratio", () => {
        const pairs = [...COLOR_PRESETS.flatMap((p) => THEMES.map((dark) => generateAccentColors({ type: "preset", hue: p.hue }, dark))).map((c) => [c.primary, c.primaryForeground]), ...HEXES.map((hex) => [hex, readableOn(hex)])];
        for (const [fill, fg] of pairs) {
            const black = contrast(fill, "#000000");
            const white = contrast(fill, "#ffffff");
            expect(fg, fill).toBe(black >= white ? "#000000" : "#ffffff");
            expect(Math.max(black, white), fill).toBeGreaterThanOrEqual(4.5);
        }
        // The old `lum > 0.55` rule put white on all of these: 2.06 to 2.99.
        expect(HEXES.slice(0, 4).map(readableOn)).toEqual(["#000000", "#000000", "#000000", "#000000"]);
        expect(readableOn("#9c27b0")).toBe("#ffffff");
    });
});

describe("presets (K3) and hover (K4)", () => {
    for (const { name, hue } of COLOR_PRESETS) {
        for (const dark of THEMES) {
            it(`${name} ${dark ? "dark" : "light"}`, () => {
                const c = generateAccentColors({ type: "preset", hue }, dark);
                const s = dark ? SURFACES.dark : SURFACES.light;
                expect(contrast(c.primaryInk, s.bg)).toBeGreaterThanOrEqual(4.5);
                expect(contrast(c.primaryInk, s.card)).toBeGreaterThanOrEqual(4.5);
                expect(contrast(c.primaryForeground, c.primary)).toBeGreaterThanOrEqual(4.5);
                expect(contrast(c.ring, s.muted)).toBeGreaterThanOrEqual(3);
                expect(contrast(c.primaryForeground, c.primaryHover)).toBeGreaterThanOrEqual(4.5);
            });
        }
    }

    it("leaves the brand red at its stylesheet numbers", () => {
        const light = generateAccentColors({ type: "preset", hue: 25 }, false);
        const dark = generateAccentColors({ type: "preset", hue: 25 }, true);
        expect(light.primary).toBe("oklch(0.58 0.22 25)");
        expect(dark.primary).toBe("oklch(0.75 0.15 25)");
        expect(contrast("oklch(0.58 0.22 25)", SURFACES.light.bg).toFixed(2)).toBe("4.59");
        expect(contrast("oklch(0.58 0.22 25)", SURFACES.light.card).toFixed(2)).toBe("4.79");
        expect(contrast("oklch(0.75 0.15 25)", SURFACES.dark.bg).toFixed(2)).toBe("8.49");
        expect(contrast("oklch(0.75 0.15 25)", SURFACES.dark.card).toFixed(2)).toBe("8.07");
        // The stylesheet's own hover tokens, with its near-white / near-black labels.
        expect(contrast(SURFACES.light.bg, "color-mix(in oklab, oklch(0.58 0.22 25) 88%, black)")).toBeGreaterThanOrEqual(6);
        expect(contrast(SURFACES.dark.bg, "color-mix(in oklab, oklch(0.75 0.15 25) 88%, white)")).toBeGreaterThanOrEqual(9);
    });
});

describe("custom hex (K2)", () => {
    for (const hex of [...HEXES, "#22c55e", "#06b6d4", "#808080", "#000080", "#ffeb3b"]) {
        for (const dark of THEMES) {
            it(`${hex} ${dark ? "dark" : "light"}`, () => {
                const c = generateAccentColors({ type: "custom", hex }, dark);
                const s = dark ? SURFACES.dark : SURFACES.light;
                expect(c.primary).toBe(hex);
                expect(c.chart1).toBe(hex);
                expect(contrast(c.primaryInk, s.bg)).toBeGreaterThanOrEqual(4.5);
                expect(contrast(c.primaryInk, s.card)).toBeGreaterThanOrEqual(4.5);
                expect(contrast(c.ring, s.bg)).toBeGreaterThanOrEqual(3);
                expect(contrast(c.ring, s.muted)).toBeGreaterThanOrEqual(3);
                expect(contrast(c.primaryForeground, c.primary)).toBeGreaterThanOrEqual(4.5);
                expect(contrast(c.primaryForeground, c.primaryHover)).toBeGreaterThanOrEqual(4.5);
            });
        }
    }
});

describe("the pre-paint script", () => {
    it("runs the same kit after toString(), with nothing from outside its body", () => {
        // `__root.tsx` inlines `accentKit.toString()`; a reference to a module-level
        // name would throw here, where the function is rebuilt with no closure.
        const rebuilt = new Function(`return (${accentKit.toString()})();`)() as ReturnType<typeof accentKit>;
        const live = accentKit();
        for (const raw of ["h:25", "h:85", "h:200", "c:#ffc107", "c:#9c27b0", "c:bad", "x"]) {
            for (const dark of THEMES) expect(rebuilt.accentVars(raw, dark)).toEqual(live.accentVars(raw, dark));
        }
    });
});
