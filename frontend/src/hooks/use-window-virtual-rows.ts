"use client";

import { useWindowVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";

interface IWindowVirtualRowsOptions<TRow> {
    /** Build the flattened row list from the measured parent width. Memoized on `[width, ...rowDeps]`. */
    buildRows: (width: number) => TRow[];
    /** External inputs (besides width) that `buildRows` reads. */
    rowDeps: unknown[];
    /** Per-row height estimate. `width` is supplied so callers can size cards without re-measuring. */
    estimateSize: (row: TRow | undefined, index: number, width: number) => number;
    overscan: number;
    /** Extra re-measure triggers beyond width and row count (e.g. view mode, breakpoint). */
    measureDeps?: unknown[];
    /** Extra scroll-offset triggers beyond width and row count (e.g. view mode). */
    scrollMarginDeps?: unknown[];
}

/**
 * Window-virtualized rows anchored to their document position. Owns the parent's
 * measured width (for column math), its distance from the document top
 * (`scrollMargin`) so window scrolling maps to this list even as surrounding
 * content changes height, and the virtualizer itself. Derives the row list from
 * width internally so callers never face the width -> count/estimate cycle.
 */
export function useWindowVirtualRows<TRow>({ buildRows, rowDeps, estimateSize, overscan, measureDeps = [], scrollMarginDeps = [] }: IWindowVirtualRowsOptions<TRow>) {
    const parentRef = useRef<HTMLDivElement>(null);
    const [width, setWidth] = useState(0);
    const [scrollMargin, setScrollMargin] = useState(0);

    useEffect(() => {
        const el = parentRef.current;
        if (!el) return;
        const observer = new ResizeObserver((entries) => {
            const w = entries[0]?.contentRect.width ?? 0;
            if (w > 0) setWidth(w);
        });
        observer.observe(el);
        return () => observer.disconnect();
    }, []);

    // biome-ignore lint/correctness/useExhaustiveDependencies: buildRows is intentionally re-run only on width/rowDeps changes
    const rows = useMemo(() => buildRows(width), [width, ...rowDeps]);

    const virtualizer = useWindowVirtualizer({
        count: rows.length,
        estimateSize: (index) => estimateSize(rows[index], index, width),
        overscan,
        scrollMargin,
    });

    // biome-ignore lint/correctness/useExhaustiveDependencies: caller-supplied deps plus width/row count drive the offset recompute
    useLayoutEffect(() => {
        const measure = () => {
            if (!parentRef.current) return;
            const top = parentRef.current.getBoundingClientRect().top + window.scrollY;
            setScrollMargin((prev) => (Math.abs(prev - top) > 1 ? top : prev));
        };
        measure();
        window.addEventListener("resize", measure);
        return () => window.removeEventListener("resize", measure);
    }, [width, rows.length, ...scrollMarginDeps]);

    // biome-ignore lint/correctness/useExhaustiveDependencies: caller-supplied deps plus width/row count invalidate cached row heights
    useLayoutEffect(() => {
        virtualizer.measure();
    }, [virtualizer, width, rows.length, ...measureDeps]);

    return { parentRef, width, scrollMargin, rows, virtualizer };
}
