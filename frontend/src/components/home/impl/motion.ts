import * as React from "react";

const REDUCED_MOTION = "(prefers-reduced-motion: reduce)";

/** Reveal once 12% of the element is in view, not counting the bottom 6% of the viewport. */
const REVEAL_OBSERVER_OPTIONS: IntersectionObserverInit = { threshold: 0.12, rootMargin: "0px 0px -6% 0px" };

/** The hero art drifts at a fifth of the scroll speed. */
const PARALLAX_RATE = 0.2;
/** Scroll distance, in viewport heights, after which the drift stops. */
const PARALLAX_RANGE_VH = 1.2;
/** Slight overscale so the drifting layer never shows an edge. */
const PARALLAX_SCALE = 1.02;

/** The visitor's reduced-motion preference, read now. Client only. */
export function prefersReducedMotion(): boolean {
    return window.matchMedia(REDUCED_MOTION).matches;
}

function subscribeReducedMotion(onChange: () => void): () => void {
    const query = window.matchMedia(REDUCED_MOTION);
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
}

/** Whether the visitor asked for reduced motion. `true` on the server, so nothing animates before hydration decides. */
export function usePrefersReducedMotion(): boolean {
    return React.useSyncExternalStore(subscribeReducedMotion, prefersReducedMotion, () => true);
}

/**
 * Reveal-on-scroll. The element renders with `data-reveal="pending"` (hidden by
 * the `.reveal` rule in `shared.module.css`) and flips to `"shown"` the first
 * time it scrolls into view. Reduced motion is handled in CSS, which shows a
 * pending element outright, so the observer still runs but changes nothing visible.
 *
 * Spread the result onto the element: `<div {...useReveal(80)} />`.
 */
export function useReveal<T extends HTMLElement>(delayMs = 0): { ref: React.RefCallback<T>; "data-reveal": "pending"; style: React.CSSProperties } {
    const ref = React.useCallback((el: T | null) => {
        if (!el) return;
        if (typeof IntersectionObserver === "undefined") {
            el.dataset.reveal = "shown";
            return;
        }
        const io = new IntersectionObserver((entries) => {
            for (const entry of entries) {
                if (!entry.isIntersecting) continue;
                el.dataset.reveal = "shown";
                io.disconnect();
            }
        }, REVEAL_OBSERVER_OPTIONS);
        io.observe(el);
        return () => io.disconnect();
    }, []);
    return { ref, "data-reveal": "pending", style: { "--reveal-delay": `${delayMs}ms` } as React.CSSProperties };
}

/**
 * Scroll parallax for the hero art: the layer drifts down slower than the
 * page, capped once the hero is well out of view. rAF-throttled and passive;
 * does nothing under reduced motion.
 */
export function useParallax<T extends HTMLElement>(): React.RefObject<T | null> {
    const ref = React.useRef<T>(null);
    const reduced = usePrefersReducedMotion();

    React.useEffect(() => {
        const el = ref.current;
        if (!el || reduced) return;
        let frame = 0;
        const apply = (): void => {
            frame = 0;
            const y = Math.min(window.scrollY, window.innerHeight * PARALLAX_RANGE_VH);
            el.style.transform = `translate3d(0, ${(y * PARALLAX_RATE).toFixed(2)}px, 0) scale(${PARALLAX_SCALE})`;
        };
        const onScroll = (): void => {
            if (!frame) frame = requestAnimationFrame(apply);
        };
        apply();
        window.addEventListener("scroll", onScroll, { passive: true });
        return () => {
            window.removeEventListener("scroll", onScroll);
            if (frame) cancelAnimationFrame(frame);
            el.style.transform = "";
        };
    }, [reduced]);

    return ref;
}

/** Wall-clock milliseconds, re-read every `intervalMs` (a second by default) while mounted. */
export function useNow(intervalMs = 1000): number {
    const [now, setNow] = React.useState(() => Date.now());
    React.useEffect(() => {
        const id = window.setInterval(() => setNow(Date.now()), intervalMs);
        return () => window.clearInterval(id);
    }, [intervalMs]);
    return now;
}
