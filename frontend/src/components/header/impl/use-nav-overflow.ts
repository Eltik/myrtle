import { type RefObject, useLayoutEffect, useState } from "react";

interface INavOverflowRefs {
    /** The header row holding the brand, the nav and the controls. */
    row: RefObject<HTMLElement | null>;
    brand: RefObject<HTMLElement | null>;
    nav: RefObject<HTMLElement | null>;
    controls: RefObject<HTMLElement | null>;
}

/**
 * Whether the desktop nav would overflow the header row, measured rather than
 * set by a breakpoint: nav labels differ per locale (Polish runs ~25% wider
 * than English), and the set of locales lives in the database, so no single
 * width is right for all of them.
 *
 * The sum is the same whether the nav is shown or collapsed (a collapsed nav
 * stays laid out, absolutely positioned and invisible, so it keeps its
 * natural width, and the hamburger is left out), so the answer never flips
 * back and forth on its own result.
 */
export function useNavOverflow({ row, brand, nav, controls }: INavOverflowRefs): boolean {
    const [overflows, setOverflows] = useState(false);

    useLayoutEffect(() => {
        const rowEl = row.current;
        const brandEl = brand.current;
        const navEl = nav.current;
        const controlsEl = controls.current;
        if (!rowEl || !brandEl || !navEl || !controlsEl || typeof ResizeObserver === "undefined") return;

        const measure = () => {
            const gap = Number.parseFloat(getComputedStyle(rowEl).columnGap) || 0;
            const needed = brandEl.offsetWidth + navEl.offsetWidth + controlsEl.offsetWidth + gap * 2;
            setOverflows(needed > rowEl.clientWidth);
        };

        measure();
        const observer = new ResizeObserver(measure);
        for (const el of [rowEl, brandEl, navEl, controlsEl]) observer.observe(el);
        return () => observer.disconnect();
    }, [row, brand, nav, controls]);

    return overflows;
}
