/**
 * The ticket sampler moved to `#/lib/story/palette` on 2026-09-23 so the reader can sample a
 * story SPRITE with the same quantiser. The card and `palette.test.ts` still import from here.
 * Three cards' `--ticket-c1` are byte-identical across the move (`docs/story-reader.md`).
 */

export {
    badgeTextFor,
    clampLightness,
    conditionInk,
    contrastRatio,
    FALLBACK_PALETTE,
    floorSaturation,
    fromHex,
    hslToRgb,
    hueDistance,
    type IColorBin,
    type ITicketPalette,
    luminance,
    ON_ACCENT_DARK,
    PALETTE_CACHE_KEY,
    padInk,
    paletteFromPixels,
    pickTriad,
    quantise,
    rgbDistance,
    rgbToHsl,
    SAMPLE_H,
    SAMPLE_W,
    samplePalette,
    TICKET_PANEL,
    toHex,
    useTicketPalette,
} from "#/lib/story/palette";
