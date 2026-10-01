import { type RefObject, useEffect, useRef } from "react";

/**
 * SCROLL PROGRESS ACROSS ONE ELEMENT, written to a custom property the CSS reads. 0 while the
 * element's top edge is at the scroller's top edge, 1 once a full element height has gone past.
 *
 * `scrollRoot` names WHICH scroller; the default is the window. A dialog needs the other
 * answer: its content box scrolls and the window does not move, so a window listener would
 * never fire and the progress would stay pinned at 0. With a root the offset is measured
 * against THAT box (`rect.top - root.top`), because the element's viewport-relative top does
 * not change when a scroller inside the viewport moves.
 *
 * `prefers-reduced-motion` pins the property at 0 and attaches nothing, so the listener and
 * observer are inert too.
 */
export function useParallaxProgress<T extends HTMLElement>(cssVar = "--parallax-progress", scrollRoot?: RefObject<HTMLElement | null>): RefObject<T | null> {
    const ref = useRef<T>(null);

    useEffect(() => {
        const el = ref.current;
        if (!el) return;

        const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        if (prefersReducedMotion) {
            el.style.setProperty(cssVar, "0");
            return;
        }

        // Read once per effect rather than once per frame: the scroller is the
        // dialog's content box, which is mounted for as long as this is.
        const root = scrollRoot?.current ?? null;
        const target: HTMLElement | Window = root ?? window;

        let ticking = false;
        let isVisible = false;

        const update = () => {
            ticking = false;
            const rect = el.getBoundingClientRect();
            const top = root === null ? rect.top : rect.top - root.getBoundingClientRect().top;
            const progress = Math.max(0, Math.min(1, -top / Math.max(1, rect.height)));
            el.style.setProperty(cssVar, progress.toFixed(4));
        };

        const onScroll = () => {
            if (!isVisible || ticking) return;
            ticking = true;
            requestAnimationFrame(update);
        };

        const io = new IntersectionObserver(
            ([entry]) => {
                isVisible = entry.isIntersecting;
                if (isVisible) update();
            },
            { root, rootMargin: "100px 0px" },
        );
        io.observe(el);

        target.addEventListener("scroll", onScroll, { passive: true });
        window.addEventListener("resize", onScroll, { passive: true });
        update();

        return () => {
            io.disconnect();
            target.removeEventListener("scroll", onScroll);
            window.removeEventListener("resize", onScroll);
        };
    }, [cssVar, scrollRoot]);

    return ref;
}
