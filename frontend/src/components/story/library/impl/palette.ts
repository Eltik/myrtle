/**
 * The ticket sampler MOVED to `#/lib/story/palette` (2026-09-23), because the
 * reader now samples a story SPRITE with the same quantiser and a library
 * folder is the wrong home for a module two features share.
 *
 * Nothing about the ticket changed: this file re-exports the same symbols, the
 * card still imports `useTicketPalette` from here, and `palette.test.ts` beside
 * it still drives the same functions. Three cards' `--ticket-c1` measured
 * before and after the move are byte-identical (`docs/story-reader.md`, the
 * reader UI paragraphs).
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
