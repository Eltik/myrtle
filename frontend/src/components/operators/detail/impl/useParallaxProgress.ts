import { type RefObject, useEffect, useRef } from "react";

export function useParallaxProgress<T extends HTMLElement>(cssVar = "--parallax-progress"): RefObject<T | null> {
    const ref = useRef<T>(null);

    useEffect(() => {
        const el = ref.current;
        if (!el) return;

        const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        if (prefersReducedMotion) {
            el.style.setProperty(cssVar, "0");
            return;
        }

        let ticking = false;
        let isVisible = false;

        const update = () => {
            ticking = false;
            const rect = el.getBoundingClientRect();
            const progress = Math.max(0, Math.min(1, -rect.top / Math.max(1, rect.height)));
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
            { rootMargin: "100px 0px" },
        );
        io.observe(el);

        window.addEventListener("scroll", onScroll, { passive: true });
        window.addEventListener("resize", onScroll, { passive: true });
        update();

        return () => {
            io.disconnect();
            window.removeEventListener("scroll", onScroll);
            window.removeEventListener("resize", onScroll);
        };
    }, [cssVar]);

    return ref;
}
