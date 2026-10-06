/**
 * Where the editor's header preview gets its size. The preview renders the real header
 * inside an iframe whose viewport is the one being previewed, so the header's own CSS
 * (its breakpoints, the page gutter, the art's 62% box from `sm` up) lays it out exactly
 * as on the profile page. These helpers pick that viewport and the one CSS scale that
 * fits the result into the editor. Every measurement the crop math reads is taken inside
 * the iframe, before the scale, so a focus stored from the preview is the focus a drag on
 * the real header would store.
 */

export type PreviewMode = "desktop" | "phone";

/** The phone viewport previewed from a wider screen: a 390 px phone, where the header's art box measured 362 x 299 (2026-10-06). */
export const PHONE_VIEWPORT = 390;
/** Under this viewport the profile lays out as on a phone: Tailwind's `sm`, where the header switches to its side-by-side layout. */
export const SM_BREAKPOINT = 640;
/** The desktop viewport previewed from a phone, which has no desktop width of its own to show. */
export const DESKTOP_FALLBACK_VIEWPORT = 1280;

/**
 * The viewport width a mode previews, given the editor's own (the page's `clientWidth`).
 * On a desktop: Desktop is the page as it is now, Phone a 390 px phone. On a phone the
 * page already IS the phone layout, so Phone is its own width and Desktop a 1280 px
 * desktop.
 */
export function previewViewport(mode: PreviewMode, clientWidth: number): number {
    const onPhone = clientWidth < SM_BREAKPOINT;
    if (mode === "phone") return onPhone ? clientWidth : PHONE_VIEWPORT;
    return onPhone ? DESKTOP_FALLBACK_VIEWPORT : clientWidth;
}

/** The mode the editor opens in: the one that matches the screen it is on. */
export function defaultPreviewMode(clientWidth: number): PreviewMode {
    return clientWidth < SM_BREAKPOINT ? "phone" : "desktop";
}

/**
 * The CSS scale that fits a `width` x `height` header into `maxWidth` x `maxHeight`:
 * never above 1, so a header that fits is shown life-size. A size not measured yet
 * (0) scales by 1.
 */
export function previewScale(width: number, height: number, maxWidth: number, maxHeight: number): number {
    if (!(width > 0 && height > 0)) return 1;
    const fit = Math.min(1, maxWidth / width, maxHeight / height);
    return fit > 0 ? fit : 1;
}

/**
 * The tallest the preview may draw, so the art browser keeps the rest of the editor:
 * 42% of the viewport's height, never under 160 px.
 */
export function previewMaxHeight(viewportHeight: number): number {
    return Math.max(160, Math.round(viewportHeight * 0.42));
}

/** The collapsed preview strip's height, and the tallest its header may draw inside it. */
export const STRIP_HEIGHT = 148;
export const STRIP_PREVIEW_MAX = 116;

/**
 * How far past the strip's bottom the preview's bottom may still sit and the strip show.
 * The scroller's `scrollTop` and `scrollHeight` are whole px while the preview's height is
 * not (290.75 at 1500 x 716, 373 in Phone mode), so at the end of a source that fills the
 * height under the strip the preview's bottom lands anywhere in 147 to 149 px. One px of
 * slack shows the strip at that end whatever the fraction.
 */
export const STRIP_SLACK = 1;

/**
 * Whether the collapsed strip shows: once the full preview's bottom edge (`previewBottom`,
 * measured from the scroller's top) has scrolled up under where the strip sits, so the
 * strip only ever covers a preview already out of view. The strip overlays the scroller,
 * and nothing in the scroller's flow depends on whether it shows (the sticky rows sit
 * `STRIP_HEIGHT` down and the filled heights end there, shown or not), so showing it
 * cannot change the scroll range, clamp the scroll back and flip it off again.
 */
export function stripShown(previewBottom: number): boolean {
    return previewBottom < STRIP_HEIGHT + STRIP_SLACK;
}
