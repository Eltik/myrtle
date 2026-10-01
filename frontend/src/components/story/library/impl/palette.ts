/**
 * Re-exports the ticket sampler, moved to `#/lib/story/palette` on 2026-09-23 because the reader
 * samples a story SPRITE with the same quantiser. The card still imports `useTicketPalette` from
 * here and `palette.test.ts` beside it drives the same functions. Three cards' `--ticket-c1` are
 * byte-identical before and after the move (`docs/story-reader.md`, the reader UI paragraphs).
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
