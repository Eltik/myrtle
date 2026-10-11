import type * as React from "react";
import { useCallback, useEffect, useImperativeHandle, useRef, useState } from "react";
import { cn } from "#/lib/utils";
import { type IBox, zoomBox } from "./geometry";
import type { IView } from "./labels";

/** Screen pixels along each edge that other UI covers; the fitted view keeps content out of them. */
export interface IInsets {
    top: number;
    right: number;
    bottom: number;
    left: number;
}

const NO_INSETS: IInsets = { top: 0, right: 0, bottom: 0, left: 0 };

export interface IPanZoomHandle {
    zoomBy: (factor: number) => void;
    fit: () => void;
}

interface IPanZoomProps {
    /** The content box to fit; a new one refits. */
    content: IBox;
    /** Edges covered by controls drawn over the map; the fit centres content in what is left. */
    insets?: IInsets;
    /** Called with the zoom level as a fraction of the fitted view (1 = fit). */
    onZoom?: (zoom: number) => void;
    minZoom?: number;
    maxZoom?: number;
    className?: string;
    label: string;
    handle?: React.Ref<IPanZoomHandle>;
    children: React.ReactNode;
    /**
     * HTML drawn over the map in screen pixels, given the current view (map
     * labels that keep their size at every zoom). It pans and zooms with the
     * map: drags and wheel turns over it move the map too.
     */
    overlay?: (view: IView) => React.ReactNode;
}

/**
 * An SVG whose viewBox drags and wheel-zooms, with an optional HTML overlay
 * on top. The box keeps the element's aspect ratio, so one SVG unit is the
 * same on both axes and pointer math is a plain scale. A drag of more than a
 * few pixels swallows the click that ends it, so panning never opens what
 * the pointer was released on.
 */
export function PanZoom({ content, insets = NO_INSETS, onZoom, minZoom = 0.5, maxZoom = 8, className, label, handle, children, overlay }: IPanZoomProps): React.ReactElement {
    const stageRef = useRef<HTMLDivElement>(null);
    const [size, setSize] = useState({ w: 1000, h: 500 });
    const [box, setBox] = useState<IBox>(content);
    const fitRef = useRef<IBox>(content);
    const drag = useRef<{ x: number; y: number; moved: boolean } | null>(null);
    const swallowClick = useRef(false);

    useEffect(() => {
        const el = stageRef.current;
        if (!el) return;
        const ro = new ResizeObserver(([entry]) => {
            const r = entry.contentRect;
            if (r.width > 0 && r.height > 0) setSize({ w: r.width, h: r.height });
        });
        ro.observe(el);
        return () => ro.disconnect();
    }, []);

    useEffect(() => {
        const next = fitBox(content, size.w, size.h, insets);
        fitRef.current = next;
        setBox(next);
    }, [content, size, insets]);

    const clampFactor = useCallback(
        (current: IBox, factor: number) => {
            const zoom = fitRef.current.w / (current.w * factor);
            if (zoom > maxZoom) return fitRef.current.w / maxZoom / current.w;
            if (zoom < minZoom) return fitRef.current.w / minZoom / current.w;
            return factor;
        },
        [minZoom, maxZoom],
    );

    useEffect(() => {
        onZoom?.(fitRef.current.w / box.w);
    }, [box, onZoom]);

    useEffect(() => {
        const el = stageRef.current;
        if (!el) return;
        const onWheel = (e: WheelEvent) => {
            e.preventDefault();
            const rect = el.getBoundingClientRect();
            setBox((b) => {
                const factor = clampFactor(b, Math.exp(e.deltaY * 0.0015));
                const anchor = { x: b.x + ((e.clientX - rect.left) / rect.width) * b.w, y: b.y + ((e.clientY - rect.top) / rect.height) * b.h };
                return zoomBox(b, factor, anchor);
            });
        };
        el.addEventListener("wheel", onWheel, { passive: false });
        return () => el.removeEventListener("wheel", onWheel);
    }, [clampFactor]);

    useImperativeHandle(
        handle,
        () => ({
            zoomBy: (factor) => setBox((b) => zoomBox(b, clampFactor(b, factor), { x: b.x + b.w / 2, y: b.y + b.h / 2 })),
            fit: () => setBox(fitRef.current),
        }),
        [clampFactor],
    );

    return (
        <div
            ref={stageRef}
            className={cn("relative overflow-hidden", className)}
            style={{ touchAction: "none", cursor: drag.current?.moved ? "grabbing" : "grab" }}
            onPointerDown={(e) => {
                drag.current = { x: e.clientX, y: e.clientY, moved: false };
                swallowClick.current = false;
            }}
            onPointerMove={(e) => {
                const d = drag.current;
                const el = stageRef.current;
                if (!d || !el) return;
                const dx = e.clientX - d.x;
                const dy = e.clientY - d.y;
                if (!d.moved && Math.hypot(dx, dy) < 4) return;
                if (!d.moved) el.setPointerCapture(e.pointerId);
                d.moved = true;
                d.x = e.clientX;
                d.y = e.clientY;
                const rect = el.getBoundingClientRect();
                setBox((b) => ({ ...b, x: b.x - (dx / rect.width) * b.w, y: b.y - (dy / rect.height) * b.h }));
            }}
            onPointerUp={(e) => {
                if (drag.current?.moved) {
                    swallowClick.current = true;
                    stageRef.current?.releasePointerCapture(e.pointerId);
                }
                drag.current = null;
            }}
            onClickCapture={(e) => {
                if (swallowClick.current) {
                    e.stopPropagation();
                    swallowClick.current = false;
                }
            }}
        >
            {/* biome-ignore lint/a11y/useSemanticElements: an SVG cannot be a <fieldset>; the group names the focusable clusters and cells inside it */}
            <svg role="group" aria-label={label} className="absolute inset-0 size-full" viewBox={`${box.x} ${box.y} ${box.w} ${box.h}`} preserveAspectRatio="none">
                {children}
            </svg>
            {overlay ? <div className="pointer-events-none absolute inset-0">{overlay({ box, width: size.w, height: size.h })}</div> : null}
        </div>
    );
}

/**
 * The view box that shows all of `c` inside a `w` x `h` stage minus `inset`
 * screen pixels, centred in that free area, at one scale on both axes. When
 * the insets leave too little room (a phone), they are dropped so the
 * content still gets the whole stage.
 */
export function fitBox(c: IBox, w: number, h: number, inset: IInsets = NO_INSETS): IBox {
    let { top, right, bottom, left } = inset;
    if (w - left - right < w / 2 || h - top - bottom < h / 2) {
        top = right = bottom = left = 0;
    }
    const freeW = w - left - right;
    const freeH = h - top - bottom;
    const scale = Math.max(c.w / freeW, c.h / freeH);
    return {
        x: c.x - left * scale - (freeW * scale - c.w) / 2,
        y: c.y - top * scale - (freeH * scale - c.h) / 2,
        w: w * scale,
        h: h * scale,
    };
}
