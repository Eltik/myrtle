/**
 * The three ticket colours a story card is printed in.
 *
 * The reference card at tools.adachurch.com does not theme its tickets from a
 * site palette: it SAMPLES the artwork, so every chapter gets its own tape
 * stripe, badge and gradient. We do the same from our own cover art, which is
 * either the game's Archives picture or the first background of the chapter's
 * first script, served from `/api/assets` on the same site as the page.
 *
 * WHAT THIS FILE OWNS is the RULES: the quantiser, the legibility band, the
 * saturation floor, the separation test and the triad. All of it is pure and
 * DOM-free, so every number below can be argued with in a unit test. The
 * plumbing that gets pixels to it lives in `sampler.ts`, which is one factory
 * both samplers here are built from.
 *
 * It lives in `lib/story` rather than in the library folder because the READER
 * samples the same way: {@link spritePalette} runs this quantiser over a
 * character's body PNG so the speaker plate can be inked in the colours the
 * sprite on stage is actually wearing. `components/story/library/impl/palette.ts`
 * re-exports every ticket symbol, so the card is untouched by the move.
 */

import { useEffect, useState } from "react";
import { clamp } from "./num";
import { createSampler } from "./sampler";

export interface ITicketPalette {
    /** The dominant colour: tape stripe 1, badge fill, progress fill, gradient top. */
    c1: string;
    /** The second colour: tape stripe 2, the spec row's dot, gradient middle. */
    c2: string;
    /** The third colour: tape stripe 3 only. */
    c3: string;
    /** White or the site's dark on-accent, whichever reads on {@link c1}. */
    badgeText: string;
}

/**
 * The versioned localStorage key. A bump invalidates every cached sample at
 * once. v4 (2026-09-24): the ticket samples the KEY VISUAL rather than the
 * derived cover, so every entry a browser already held is a sample of the
 * wrong file.
 */
export const PALETTE_CACHE_KEY = "myrtle.story.palette.v4";

/**
 * The site's own accent triad, used when a card has no cover, the image fails,
 * or the canvas is tainted. `--primary`, `--chart-2` and `--chart-3` resolved
 * from their light-mode oklch: the ticket is dark art in BOTH themes, so the
 * fallback cannot be a token that flips.
 */
export const FALLBACK_PALETTE: ITicketPalette = { c1: "#df202e", c2: "#008c7d", c3: "#006088", badgeText: "#ffffff" };

/** The site's darkest surface, `--bg-base` at its dark-mode value, pinned. Their `panel-3` is `#0a0a0a`. */
export const TICKET_PANEL = "#0a1418";

/** The dark text the site puts on an accent, `--foreground` at its light-mode value. */
export const ON_ACCENT_DARK = "#0a0a0c";

/** The sample grid. 768 pixels is enough to rank dominant colours and costs one `drawImage`. */
export const SAMPLE_W = 32;
export const SAMPLE_H = 24;

/** Badges, stripes and the progress fill all have to read on a near-black ticket, so a sampled colour is pulled into this band. */
const MIN_LIGHT = 0.22;
const MAX_LIGHT = 0.7;

/** Two sampled colours count as the same ink unless they differ by this much hue, or this much lightness. */
const HUE_SEPARATION = 24;
const LIGHT_SEPARATION = 0.16;

/**
 * Below this saturation a colour has no hue worth comparing: at s 0.05 two
 * greys three RGB steps apart read 150 degrees apart. MEASURED on the running
 * page before this gate existed: chapter MAIN_1 sampled #35353b / #393739 /
 * #3b3636 at s 0.054 / 0.018 / 0.044 and the separation rule accepted all
 * three as distinct inks, so the tape's five hard stripes were five identical
 * greys.
 */
const HUE_GATE = 0.12;

/**
 * A TRADE, shipped knowingly. The reference page samples event key visuals,
 * which are saturated; our covers are mostly the first BACKGROUND of a
 * chapter's first script, and 27 inks sampled over the first nine cards came
 * back at saturation 0.000..0.239, median 0.10. Printed as sampled, the tape
 * band, the badge and the progress fill are all the same grey and the
 * construction disappears. The floor keeps the hue the art actually has and
 * only raises its purity; a bin with no chroma at all (`SAT_GATE`) is left
 * grey rather than invented into a red.
 */
const SAT_FLOOR = 0.32;
const SAT_GATE = 0.02;

/** Pixels below this alpha are cover matting, not art. */
const ALPHA_FLOOR = 128;

/**
 * The alpha floor a SPRITE is quantised at, and it is not the cover's.
 *
 * A cover is an opaque rectangle and its only sub-128 pixels are matting. A
 * character body is mostly nothing: the figure sits in a transparent plate and
 * its whole silhouette is a rim of partly transparent pixels that the browser
 * hands back as the colour BLENDED toward the un-drawn canvas. Admitting them
 * bins a dark halo that no part of the art has. 250 keeps the rim out and
 * costs only the outermost pixel of the outline.
 */
const SPRITE_ALPHA_FLOOR = 250;

/**
 * How far apart two PRINTED inks have to be, in plain RGB distance, for the
 * tape band to read as striped rather than as one flat field.
 *
 * The raw separation rule above it is not enough, and MEASURED on the running
 * page it left 17 of 87 cards with a band whose narrowest stripe boundary was
 * under this: 5 at distance 0.000 (Stormwatch, All Quiet Under the Thunder,
 * Absolved Will Be the Seekers, Dissociative Recombination, Delicious On
 * Terra) and 12 more between 1.000 and 19.105. Two causes, both of them the
 * gap between what is JUDGED and what is PRINTED: conditioning pulls two
 * raw-distinct bins into the same lightness band and up to the same saturation
 * floor, and the pad rotated the hue of an ACHROMATIC ink, which
 * {@link hslToRgb} returns unchanged at saturation 0.
 */
const INK_SEPARATION = 24;

/** The saturation a padded ink is printed at, so a rotation off a grey dominant lands on a real hue instead of the same grey. */
const PAD_SAT = 0.42;

/**
 * How far a padded ink is moved in lightness, on top of the hue rotation.
 *
 * The move is what carries the separation, not the rotation: at the bottom of
 * the legible band the whole gamut is compressed, and a 40 degree turn at
 * lightness 0.22 moves an ink 20.616 RGB units, under {@link INK_SEPARATION}.
 * Both pads therefore step AWAY from whichever edge the dominant ink sits on,
 * 0.18 and 0.36, which the 0.48-wide band always has room for.
 */
const PAD_LIGHT = 0.18;

/** Plain RGB distance, which is what the eye reads across a hard stripe boundary. */
export function rgbDistance(a: readonly [number, number, number], b: readonly [number, number, number]): number {
    return Math.sqrt((a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2 + (a[2] - b[2]) ** 2);
}

export interface IColorBin {
    /** The bin's mean colour, 0..255 per channel. */
    rgb: [number, number, number];
    /** How many sampled pixels landed in the bin. */
    count: number;
}

/**
 * The sampled pixels folded into 5-bit-per-channel bins (32^3 = 32,768 cells),
 * most populous first. The bin's colour is the MEAN of its members rather than
 * the bin centre, so a near-uniform sky does not quantise to a visibly
 * different blue.
 */
export function quantise(data: Uint8ClampedArray, alphaFloor: number = ALPHA_FLOOR): IColorBin[] {
    const sums = new Map<number, { r: number; g: number; b: number; n: number }>();
    for (let i = 0; i + 3 < data.length; i += 4) {
        const alpha = data[i + 3] ?? 0;
        if (alpha < alphaFloor) continue;
        const r = data[i] ?? 0;
        const g = data[i + 1] ?? 0;
        const b = data[i + 2] ?? 0;
        const key = ((r >> 3) << 10) | ((g >> 3) << 5) | (b >> 3);
        const cell = sums.get(key);
        if (cell) {
            cell.r += r;
            cell.g += g;
            cell.b += b;
            cell.n += 1;
        } else sums.set(key, { r, g, b, n: 1 });
    }
    const bins: IColorBin[] = [];
    for (const cell of sums.values()) bins.push({ rgb: [Math.round(cell.r / cell.n), Math.round(cell.g / cell.n), Math.round(cell.b / cell.n)], count: cell.n });
    return bins.sort((a, b) => b.count - a.count);
}

export function rgbToHsl([r, g, b]: [number, number, number]): [number, number, number] {
    const rn = r / 255;
    const gn = g / 255;
    const bn = b / 255;
    const max = Math.max(rn, gn, bn);
    const min = Math.min(rn, gn, bn);
    const l = (max + min) / 2;
    const d = max - min;
    if (d === 0) return [0, 0, l];
    const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    let h: number;
    if (max === rn) h = ((gn - bn) / d + (gn < bn ? 6 : 0)) * 60;
    else if (max === gn) h = ((bn - rn) / d + 2) * 60;
    else h = ((rn - gn) / d + 4) * 60;
    return [h, s, l];
}

export function hslToRgb([h, s, l]: [number, number, number]): [number, number, number] {
    if (s === 0) {
        const v = Math.round(l * 255);
        return [v, v, v];
    }
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    const channel = (t: number): number => {
        let x = t;
        if (x < 0) x += 1;
        if (x > 1) x -= 1;
        if (x < 1 / 6) return p + (q - p) * 6 * x;
        if (x < 1 / 2) return q;
        if (x < 2 / 3) return p + (q - p) * (2 / 3 - x) * 6;
        return p;
    };
    const hn = (((h % 360) + 360) % 360) / 360;
    return [Math.round(channel(hn + 1 / 3) * 255), Math.round(channel(hn) * 255), Math.round(channel(hn - 1 / 3) * 255)];
}

/** Degrees between two hues the short way round, 0..180. */
export function hueDistance(a: number, b: number): number {
    const d = Math.abs(((a - b) % 360) + 360) % 360;
    return d > 180 ? 360 - d : d;
}

/** A sampled colour pulled into the legible band, hue and saturation untouched. */
export function clampLightness(rgb: [number, number, number]): [number, number, number] {
    const [h, s, l] = rgbToHsl(rgb);
    if (l >= MIN_LIGHT && l <= MAX_LIGHT) return rgb;
    return hslToRgb([h, s, clamp(l, MIN_LIGHT, MAX_LIGHT)]);
}

/** A sampled colour lifted to {@link SAT_FLOOR}, hue and lightness untouched. An achromatic bin is left alone. */
export function floorSaturation(rgb: [number, number, number]): [number, number, number] {
    const [h, s, l] = rgbToHsl(rgb);
    if (s >= SAT_FLOOR || s < SAT_GATE) return rgb;
    return hslToRgb([h, SAT_FLOOR, l]);
}

/** The whole per-colour rule: into the legible lightness band, then up to the saturation floor. */
export function conditionInk(rgb: [number, number, number]): [number, number, number] {
    return floorSaturation(clampLightness(rgb));
}

export function toHex([r, g, b]: [number, number, number]): string {
    return `#${[r, g, b].map((v) => clamp(Math.round(v), 0, 255).toString(16).padStart(2, "0")).join("")}`;
}

export function fromHex(hex: string): [number, number, number] {
    const h = hex.replace("#", "");
    const full = h.length === 3 ? `${h[0]}${h[0]}${h[1]}${h[1]}${h[2]}${h[2]}` : h;
    return [Number.parseInt(full.slice(0, 2), 16) || 0, Number.parseInt(full.slice(2, 4), 16) || 0, Number.parseInt(full.slice(4, 6), 16) || 0];
}

/** WCAG relative luminance. */
export function luminance([r, g, b]: [number, number, number]): number {
    const lin = (v: number): number => {
        const c = v / 255;
        return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    };
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

export function contrastRatio(a: [number, number, number], b: [number, number, number]): number {
    const la = luminance(a);
    const lb = luminance(b);
    return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

/**
 * White, or the site's dark on-accent, whichever has the higher contrast ratio
 * against the badge fill. COMPUTED rather than assumed: c1 is clamped only as
 * far as lightness 0.7, and white on a 0.7 yellow is 1.9:1.
 */
export function badgeTextFor(fill: string): string {
    const rgb = fromHex(fill);
    return contrastRatio(rgb, [255, 255, 255]) >= contrastRatio(rgb, fromHex(ON_ACCENT_DARK)) ? "#ffffff" : ON_ACCENT_DARK;
}

/**
 * A pad ink: the dominant one rotated off its hue AND moved in lightness, at a
 * saturation floor of its own.
 *
 * The saturation floor is the whole point. `hslToRgb` returns the same grey for
 * every hue at saturation 0, so rotating an achromatic dominant ink by 40
 * degrees is a NO-OP and the three inks come out byte-identical; that is what
 * printed Stormwatch's tape as one flat #383838 field.
 */
export function padInk(base: readonly [number, number, number], step: 1 | 2): [number, number, number] {
    const [h, sat, l] = rgbToHsl(base as [number, number, number]);
    const turn = step === 1 ? 40 : -40;
    const up = MAX_LIGHT - l;
    const down = l - MIN_LIGHT;
    // Opposite sides when both fit, the same side in two steps when the ink is
    // already against an edge: clamping the second pad back onto the first
    // edge is what printed it 20.616 away from the dominant ink.
    const lift = up >= PAD_LIGHT && down >= PAD_LIGHT ? (step === 1 ? PAD_LIGHT : -PAD_LIGHT) : (up >= down ? PAD_LIGHT : -PAD_LIGHT) * step;
    return hslToRgb([h + turn, Math.max(sat, PAD_SAT), clamp(l + lift, MIN_LIGHT, MAX_LIGHT)]);
}

/**
 * The three most populous bins that are visibly different inks, in population
 * order. A cover that carries fewer than three (a monochrome background, a
 * silhouette) pads from the DOMINANT colour through {@link padInk}, so the tape
 * still stripes instead of reading as one flat band.
 */
export function pickTriad(bins: readonly IColorBin[]): [string, string, string] {
    // Separation is judged on the RAW bin colours, never on the conditioned
    // ones: conditioning lifts a grey to saturation 0.32, and hue noise that
    // the gate is there to reject would then read as a real hue difference.
    // It is judged a SECOND time on the conditioned ink, because the raw rule
    // alone admits two bins that conditioning then flattens onto each other.
    const raw: [number, number, number][] = [];
    const inks: [number, number, number][] = [];
    for (const bin of bins) {
        if (inks.length === 3) break;
        const [h, sat, l] = rgbToHsl(bin.rgb);
        const distinct = raw.every((p) => {
            const [ph, psat, pl] = rgbToHsl(p);
            const hueCounts = sat >= HUE_GATE && psat >= HUE_GATE;
            return (hueCounts && hueDistance(h, ph) >= HUE_SEPARATION) || Math.abs(l - pl) >= LIGHT_SEPARATION;
        });
        if (!distinct) continue;
        const ink = conditionInk(bin.rgb);
        if (inks.some((p) => rgbDistance(p, ink) < INK_SEPARATION)) continue;
        raw.push(bin.rgb);
        inks.push(ink);
    }
    if (inks.length === 0) return [FALLBACK_PALETTE.c1, FALLBACK_PALETTE.c2, FALLBACK_PALETTE.c3];
    while (inks.length < 3) {
        // Always rotated off the DOMINANT ink, never off the previous pad: +40 then -40 off
        // the pad itself lands back on the dominant hue and the tape reads as one flat band.
        inks.push(padInk(inks[0] as [number, number, number], inks.length === 1 ? 1 : 2));
    }
    return [toHex(inks[0] as [number, number, number]), toHex(inks[1] as [number, number, number]), toHex(inks[2] as [number, number, number])];
}

/** The whole rule over one sampled bitmap, the entry point the tests drive. */
export function paletteFromPixels(data: Uint8ClampedArray): ITicketPalette {
    const [c1, c2, c3] = pickTriad(quantise(data));
    return { c1, c2, c3, badgeText: badgeTextFor(c1) };
}

/**
 * The ticket sampler. `paletteFromPixels` is the rule; `sampler.ts` is the
 * plumbing, and the sprite sampler below is the same factory with its own four
 * values.
 */
const tickets = createSampler({ storageKey: PALETTE_CACHE_KEY, width: SAMPLE_W, height: SAMPLE_H, derive: paletteFromPixels, fallback: FALLBACK_PALETTE });

/**
 * Sample one cover. Resolves to {@link FALLBACK_PALETTE} rather than rejecting
 * on any failure, including a tainted canvas.
 */
export function samplePalette(url: string): Promise<ITicketPalette> {
    return tickets.sample(url);
}

/** The sampled palette for a cover, the site accent triad until it arrives and for a card with no cover. */
export function useTicketPalette(url: string | null): ITicketPalette {
    const [palette, setPalette] = useState<ITicketPalette>(() => (url ? (tickets.hot(url) ?? FALLBACK_PALETTE) : FALLBACK_PALETTE));

    useEffect(() => {
        if (!url) {
            setPalette(FALLBACK_PALETTE);
            return;
        }
        let live = true;
        void samplePalette(url).then((next) => {
            if (live) setPalette(next);
        });
        return () => {
            live = false;
        };
    }, [url]);

    return palette;
}

/* ------------------------------------------------------------------------ *
 * The STORY SPRITE palette.
 *
 * The reader inks a speaker plate in the colours of the sprite ON SCREEN, and
 * the source is that sprite's own body PNG, not the skin table. The skin
 * table's colour list is per OUTFIT: one entry covers every story sprite of
 * that skin and none of the sprites a character wears in a story she appears
 * in out of costume, so it answers a question nobody asked. The pixels on
 * stage are the ground truth and they are already fetched.
 * ------------------------------------------------------------------------ */

/** The versioned localStorage key for sampled sprites. A bump invalidates every cached body at once. */
export const SPRITE_CACHE_KEY = "myrtle.story.spritePalette.v1";

/**
 * The sprite sample grid, square and much larger than the cover's 32x24.
 *
 * A cover fills its rectangle, so its 768 pixels are 768 samples. A body is a
 * small figure in a large transparent plate and almost all of it is nothing:
 * MEASURED at {@link SPRITE_ALPHA_FLOOR} over the three verification sprites,
 * only 10.3% (Amiya `char_002_amiya_7`), 13.4% (Dobermann
 * `char_130_doberm_ex`) and 16.3% (`avg_npc_001`) of the grid is opaque. At
 * 32x24 that is 79 to 125 pixels for a whole palette; at 64x64 it is 363, 493
 * and 617, and Amiya's most populous bin then holds TWELVE members, which is
 * noise ranking a character's colours. 128x128 is 16,384 samples for one
 * `drawImage`, leaves 1,681 / 2,201 / 2,676 opaque pixels, and puts 49 members
 * in that same top bin.
 */
export const SPRITE_SAMPLE = 128;

/** Lightness at or above this is a highlight, a rim light or the page: no ink. */
const NEAR_WHITE = 0.92;
/** Lightness at or below this is a shadow or an outline: no ink. */
const NEAR_BLACK = 0.08;

/**
 * The SKIN BAND, and it is a TRADE, stated rather than hidden.
 *
 * A story sprite is a person, and a person is a lot of face, neck and hands.
 * Printed with no exclusion a character's second and third inks are her SKIN on
 * sprite after sprite, so the palettes stop telling characters apart. Excluding
 * the band makes a character's palette her OUTFIT and her HAIR, which is what a
 * reader recognises her by. What it costs: a character whose clothing sits
 * inside the band loses that ink too, and a sprite that is nothing but skin
 * falls back to its next colours or to the accent triad.
 *
 * THE FIRST BAND WAS REFUTED ON THE PIXELS. Hue 15..40 with saturation 0.2..0.6
 * and lightness 0.5..0.85 changes NOTHING on any of the three verification
 * sprites: it drops 22, 20 and 66 bins of 596, 238 and 390, and not one of them
 * was reaching the triad. The skin that does reach it sits ABOVE that ceiling,
 * because HSL saturation blows up as lightness rises: Dobermann's face is
 * `#f5e5d4` at lightness 0.896 and saturation 0.623, the npc's is `#fcd5bb` at
 * 0.861 and 0.915, and both clear the 0.85 ceiling AND the 0.6 saturation cap.
 * So the ceiling is {@link NEAR_WHITE}, the saturation FLOOR stays (a warm grey
 * is stone, not a face), and the saturation CAP applies only below
 * {@link SKIN_PALE_LIGHT}, above which a warm hue that pale is a highlight and
 * never a garment colour that identifies anyone.
 *
 * MEASURED with the tuned band, c1 is UNCHANGED on all three sprites, so the
 * plate itself never moved; c2 and c3 do. Dobermann c2 `#e2b483` (her face)
 * becomes `#4a2626` and c3 `#4a2626` becomes `#363869`, both hers; the npc's c3
 * `#cbb29a` becomes `#895e46`. Amiya is unchanged in all three modes: her face
 * never reached her triad to begin with.
 */
const SKIN_HUE_MIN = 15;
const SKIN_HUE_MAX = 40;
const SKIN_SAT_MIN = 0.15;
const SKIN_SAT_MAX = 0.6;
const SKIN_LIGHT_MIN = 0.5;
/** Above this lightness a warm hue is a highlight, so {@link SKIN_SAT_MAX} no longer applies. */
const SKIN_PALE_LIGHT = 0.75;

/** The three inks a sprite is printed in, in population order after the exclusions. */
export interface ISpritePalette {
    c1: string;
    c2: string;
    c3: string;
}

/** The reader's fallback when a body has no usable ink, the site accent triad without the badge text. */
export const FALLBACK_SPRITE: ISpritePalette = { c1: FALLBACK_PALETTE.c1, c2: FALLBACK_PALETTE.c2, c3: FALLBACK_PALETTE.c3 };

/** True for a bin the sprite rule drops before ranking: near-white, near-black, or in the skin band. */
export function isExcludedInk(rgb: [number, number, number], excludeSkin = true): boolean {
    const [h, s, l] = rgbToHsl(rgb);
    if (l >= NEAR_WHITE || l <= NEAR_BLACK) return true;
    if (!excludeSkin) return false;
    if (h < SKIN_HUE_MIN || h > SKIN_HUE_MAX || l < SKIN_LIGHT_MIN || s < SKIN_SAT_MIN) return false;
    return l >= SKIN_PALE_LIGHT || s <= SKIN_SAT_MAX;
}

/**
 * The three inks of one sampled body. The bins are filtered FIRST and then run
 * through the library's own {@link pickTriad}, so the separation rule, the
 * lightness band, the saturation floor and the pad are one shared rule and the
 * reader's ink reads on the box for exactly the reason the ticket's reads on
 * the card. A body with nothing left after the exclusions falls back to the
 * accent triad rather than printing a plate the colour of the outline.
 */
export function spriteInksFromPixels(data: Uint8ClampedArray, excludeSkin = true): ISpritePalette {
    const bins = quantise(data, SPRITE_ALPHA_FLOOR).filter((b) => !isExcludedInk(b.rgb, excludeSkin));
    if (bins.length === 0) return FALLBACK_SPRITE;
    const [c1, c2, c3] = pickTriad(bins);
    return { c1, c2, c3 };
}

const sprites = createSampler({ storageKey: SPRITE_CACHE_KEY, width: SPRITE_SAMPLE, height: SPRITE_SAMPLE, derive: (data) => spriteInksFromPixels(data), fallback: FALLBACK_SPRITE });

/** The palette already known for a body, from memory or storage, or undefined. Never starts a sample. */
export function cachedSpritePalette(url: string): ISpritePalette | undefined {
    return sprites.cached(url);
}

/**
 * Sample one character body. Resolves to {@link FALLBACK_SPRITE} rather than
 * rejecting on any failure, for the same reason {@link samplePalette} does: a
 * deployment that moved `/api/assets` to another origin must grey the plates
 * out, not throw on every line.
 */
export function spritePalette(url: string): Promise<ISpritePalette> {
    return sprites.sample(url);
}
