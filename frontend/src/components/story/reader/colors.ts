/**
 * The reader's text colour, and the contrast arithmetic that picked its
 * swatches.
 *
 * ONE hex cannot serve both boxes. The dark box is `bg-black/70`, so over a
 * pure white scene it composites to #4d4d4d (luminance 0.0781) and a 4.5:1
 * foreground there needs luminance at least 0.5090. The light box is
 * `bg-[#f4f1ea]/95`, so over a pure black scene it composites to #e8e5de
 * (luminance 0.7877) and 4.5:1 there needs luminance at most 0.1355. The two
 * bounds do not overlap, so the curated row is SURFACE-aware exactly like
 * `speakerColor`: eight pale tints for the dark box, eight dark tints for the
 * light one, paired by index, and the setting stores the index rather than a
 * hex so flipping "Light reading surface" carries the choice across.
 *
 * Measured ratios against those two worst-case composites (and, in
 * parentheses, against the NOMINAL box, black/70 over a dark scene #060606 and
 * #f4f1ea itself):
 *
 * | # | dark tint | on #4d4d4d | (nominal) | light tint | on #e8e5de | (nominal) |
 * |---|-----------|-----------:|----------:|------------|-----------:|----------:|
 * | 0 | #ffffff   |      8.453 |    20.262 | #1a1a1a    |     13.835 |    15.429 |
 * | 1 | #f3ead6   |      7.065 |    16.934 | #4a3410    |      9.327 |    10.401 |
 * | 2 | #f0d9a0   |      6.093 |    14.606 | #5c3d00    |      7.865 |     8.771 |
 * | 3 | #a8d8f5   |      5.557 |    13.321 | #0d4166    |      8.495 |     9.474 |
 * | 4 | #f7b9c4   |      5.109 |    12.245 | #7a1230    |      8.531 |     9.514 |
 * | 5 | #a9e7c4   |      6.004 |    14.390 | #0a4a2a    |      8.240 |     9.190 |
 * | 6 | #d4bdf2   |      4.976 |    11.926 | #4a1f7a    |      9.441 |    10.529 |
 * | 7 | #f8c9a0   |      5.577 |    13.367 | #7a3300    |      7.255 |     8.091 |
 *
 * The worst row is 4.976 on the dark box and 7.255 on the light one, so every
 * swatch clears 4.5:1 on BOTH surfaces at their worst composite. The native
 * colour input is unconstrained by design: it is the user's own choice and the
 * reader does not second-guess it.
 */

import { hslToRgb, rgbToHsl, toHex } from "#/lib/story/palette";

/** `bg-black/70` over a pure white scene: the dark box at its lightest. */
export const DARK_BOX_WORST = "#4d4d4d";
/** `bg-[#f4f1ea]/95` over a pure black scene: the light box at its darkest. */
export const LIGHT_BOX_WORST = "#e8e5de";

/** WCAG relative luminance of a `#rgb` or `#rrggbb` colour. Unparseable input is black. */
export function relativeLuminance(hex: string): number {
    const parsed = parseHex(hex);
    if (!parsed) return 0;
    const [r, g, b] = parsed.map((v) => {
        const c = v / 255;
        return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio between two colours, 1..21, order-independent. */
export function contrastRatio(a: string, b: string): number {
    const la = relativeLuminance(a);
    const lb = relativeLuminance(b);
    const hi = Math.max(la, lb);
    const lo = Math.min(la, lb);
    return (hi + 0.05) / (lo + 0.05);
}

function parseHex(hex: string): [number, number, number] | null {
    const m = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(hex.trim());
    if (!m) return null;
    const h = m[1];
    const full = h.length === 3 ? `${h[0]}${h[0]}${h[1]}${h[1]}${h[2]}${h[2]}` : h;
    return [Number.parseInt(full.slice(0, 2), 16), Number.parseInt(full.slice(2, 4), 16), Number.parseInt(full.slice(4, 6), 16)];
}

/** True for a value the reader will accept as a custom colour. */
export function isHexColor(value: string): boolean {
    return parseHex(value) !== null;
}

export interface TextSwatch {
    /** Stored in the setting as `s{index}`. */
    id: string;
    /** On the dark box. */
    dark: string;
    /** On the light box. */
    light: string;
}

export const TEXT_SWATCHES: readonly TextSwatch[] = [
    { id: "s0", dark: "#ffffff", light: "#1a1a1a" },
    { id: "s1", dark: "#f3ead6", light: "#4a3410" },
    { id: "s2", dark: "#f0d9a0", light: "#5c3d00" },
    { id: "s3", dark: "#a8d8f5", light: "#0d4166" },
    { id: "s4", dark: "#f7b9c4", light: "#7a1230" },
    { id: "s5", dark: "#a9e7c4", light: "#0a4a2a" },
    { id: "s6", dark: "#d4bdf2", light: "#4a1f7a" },
    { id: "s7", dark: "#f8c9a0", light: "#7a3300" },
];

/**
 * The stored `textColor` to a CSS colour, or `undefined` for "leave the box's
 * own colour alone". The empty string is the DEFAULT and the kill switch: it
 * returns undefined, so the box keeps `text-white` / `text-neutral-900`
 * exactly as it shipped.
 */
export function resolveTextColor(value: string, light: boolean): string | undefined {
    if (value === "") return undefined;
    const swatch = TEXT_SWATCHES.find((s) => s.id === value);
    if (swatch) return light ? swatch.light : swatch.dark;
    return isHexColor(value) ? value : undefined;
}

/** The swatch to paint in the picker row for one surface. */
export function swatchColor(swatch: TextSwatch, light: boolean): string {
    return light ? swatch.light : swatch.dark;
}

/**
 * The contrast target every ink the reader prints has to clear on the box it
 * is printed on. WCAG AA for body text.
 */
export const TEXT_CONTRAST_TARGET = 4.5;

/** One step of the lightness walk. 0.01 is 100 steps across the range, which is under a millisecond and lands within 1% of the edge. */
const FIT_STEP = 0.01;

/**
 * A SAMPLED ink conditioned until it reads on one of the two box surfaces.
 *
 * The ticket's own band (`lib/story/palette.ts`, lightness 0.22..0.70) is the
 * band a colour reads in on a near-black CARD. The text box is not that card:
 * over a pure white scene `bg-black/70` composites to {@link DARK_BOX_WORST}
 * and over a pure black scene `bg-[#f4f1ea]/95` composites to
 * {@link LIGHT_BOX_WORST}, and the two need opposite moves. So the hue and the
 * saturation of the sprite's ink are KEPT, which is what makes the plate
 * recognisable as that character, and only the LIGHTNESS is walked: up on the
 * dark box, down on the light one, until the ratio clears
 * {@link TEXT_CONTRAST_TARGET} against that surface's worst composite.
 *
 * The walk ends at white or at black, which always clear it (8.453:1 and
 * 13.835:1 respectively, `TEXT_SWATCHES` row 0), so it cannot fail to return a
 * legible colour; an unparseable input is handed back untouched for the caller
 * to drop.
 */
export function fitToBox(hex: string, light: boolean): string {
    const parsed = parseHex(hex);
    if (!parsed) return hex;
    const surface = light ? LIGHT_BOX_WORST : DARK_BOX_WORST;
    const [h, s, l0] = rgbToHsl(parsed);
    let l = l0;
    for (let i = 0; i <= 1 / FIT_STEP; i++) {
        const candidate = toHex(hslToRgb([h, s, l]));
        if (contrastRatio(candidate, surface) >= TEXT_CONTRAST_TARGET) return candidate;
        l = light ? l - FIT_STEP : l + FIT_STEP;
        if (l <= 0 || l >= 1) break;
    }
    return light ? "#000000" : "#ffffff";
}
