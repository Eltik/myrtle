/**
 * The three class strings every control OVER THE SCENE shares, in one place
 * because the toolbar, the title card, the end card and the decision options
 * all sit on the same black.
 *
 * Under 640 px every control is 44 px TALL, the touch minimum. It is not also
 * 44 px wide: nine controls at 44 px plus their gaps need 396 px of a 375 px
 * viewport, and the two clusters have to share one row, so the width is 36 px
 * and the height carries the target. Above 640 the pills keep the measured
 * 32 px desktop height.
 */
export const TOUCH_TARGET = "max-sm:h-11 pointer-coarse:h-11";

/** The focus ring is white because `ring-ring` is a theme token that disappears against the frosted black glass. */
export const DARK_FOCUS = "focus-visible:ring-white/80 focus-visible:ring-offset-black/40";

/**
 * The frosted glass both pills are made of. The client draws no bar at all,
 * only icons over the scene; ours keeps the glass because a web page has no
 * guaranteed dark scene behind a white glyph.
 */
export const PILL = "pointer-events-auto flex items-center gap-0.5 rounded-lg bg-black/45 p-1 backdrop-blur-sm pointer-coarse:gap-1 max-sm:gap-0";
