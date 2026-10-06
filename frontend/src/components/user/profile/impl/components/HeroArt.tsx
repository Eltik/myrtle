import { type ReactNode, type PointerEvent as ReactPointerEvent, type RefObject, useEffect, useRef, useState } from "react";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import { artStyle, backgroundSources, type IArtGeometry, panFocus, withScale, zoomOf } from "../background";
import styles from "./HeroArt.module.css";

interface IHeroArtProps {
    background: ProfileBackground;
    /** The header's plain look: drawn under the art, so it shows through a cut-out figure, while the art loads, and alone when no source loads. */
    fallback: ReactNode;
    /**
     * Passed while the owner's picker is open: the art takes drags (pan) and the
     * wheel or a pinch (zoom) and reports each step. Absent, nothing interactive
     * renders and the markup is what it was before.
     */
    onAdjust?: (next: ProfileBackground) => void;
    /**
     * Passed while the owner's picker is open: told the art's drawn box and file size on
     * load and on every resize (`null` while no source has loaded), so the picker enables
     * a crop slider only along an axis the art overflows. Effects only: the markup is the
     * same with or without it.
     */
    onGeometry?: (geometry: IArtGeometry | null) => void;
}

/**
 * The owner's art behind the header, under a scrim mixed from the card colour. Absolutely
 * positioned inside the header, so neither loading nor failing moves anything. Each source
 * is tried in turn (see `backgroundSources`); when none loads the header keeps its plain
 * look. Mount it with a `key` per picture, so a new pick starts over at the first source.
 */
export function HeroArt({ background, fallback, onAdjust, onGeometry }: IHeroArtProps) {
    const sources = backgroundSources(background);
    const imgRef = useRef<HTMLImageElement | null>(null);
    const [index, setIndex] = useState(0);
    const [loaded, setLoaded] = useState(false);
    const src = sources[index];
    useReportGeometry(imgRef, loaded, onGeometry);
    if (src === undefined) return fallback;
    return (
        <>
            {fallback}
            <div className={styles.frame} aria-hidden="true">
                <img
                    key={src}
                    src={src}
                    alt=""
                    decoding="async"
                    fetchPriority="low"
                    draggable={false}
                    data-loaded={loaded}
                    // An image the browser finished before hydration fires no load event React sees.
                    ref={(img) => {
                        imgRef.current = img;
                        if (!img?.complete) return;
                        if (img.naturalWidth > 0) setLoaded(true);
                        else setIndex((i) => (sources[i] === img.getAttribute("src") ? i + 1 : i));
                    }}
                    className={styles.art}
                    style={artStyle(background)}
                    onLoad={(e) => {
                        if (e.currentTarget.naturalWidth === 0) setIndex((i) => i + 1);
                        else setLoaded(true);
                    }}
                    onError={() => setIndex((i) => i + 1)}
                />
            </div>
            <div className={styles.scrim} aria-hidden="true" />
            {onAdjust && <AdjustSurface background={background} imgRef={imgRef} onAdjust={onAdjust} />}
        </>
    );
}

/** Reports the art's geometry to `onGeometry` once it has loaded and on every resize of its box; `null` before that and on unmount. */
function useReportGeometry(imgRef: RefObject<HTMLImageElement | null>, loaded: boolean, onGeometry: ((geometry: IArtGeometry | null) => void) | undefined) {
    const report = useRef(onGeometry);
    report.current = onGeometry;
    const listening = onGeometry !== undefined;
    useEffect(() => {
        if (!listening) return;
        const img = imgRef.current;
        if (!loaded || !img) {
            report.current?.(null);
            return;
        }
        const send = () => report.current?.(geometryOf(img));
        send();
        const observer = new ResizeObserver(send);
        observer.observe(img);
        return () => {
            observer.disconnect();
            report.current?.(null);
        };
    }, [imgRef, loaded, listening]);
}

/** The art's box before the zoom transform (`offset*` ignore transforms) and the file's size, or `null` before it has loaded. */
function geometryOf(img: HTMLImageElement | null): IArtGeometry | null {
    if (!img || img.naturalWidth === 0) return null;
    return { boxWidth: img.offsetWidth, boxHeight: img.offsetHeight, naturalWidth: img.naturalWidth, naturalHeight: img.naturalHeight };
}

/** Wheel notches to zoom: one 100 px notch is about 15 points of scale. A trackpad pinch arrives as a ctrl-wheel with small deltas, so it is scaled up. */
function wheelZoom(scale: number, e: WheelEvent): number {
    const pixels = e.deltaMode === 1 ? e.deltaY * 16 : e.deltaY;
    return scale * Math.exp(-pixels * (e.ctrlKey ? 0.01 : 0.0015));
}

/** The gesture's point (the first pointer, or the midpoint of the first two), their distance (0 for one) and whether two are down; `null` with none. */
function pointerSpread(points: readonly { x: number; y: number }[]): { x: number; y: number; distance: number; pinch: boolean } | null {
    const [a, b] = points;
    if (!a) return null;
    if (!b) return { x: a.x, y: a.y, distance: 0, pinch: false };
    return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2, distance: Math.hypot(a.x - b.x, a.y - b.y), pinch: true };
}

interface IAdjustSurfaceProps {
    background: ProfileBackground;
    imgRef: RefObject<HTMLImageElement | null>;
    onAdjust: (next: ProfileBackground) => void;
}

/**
 * The drag-and-zoom layer over the art's box while the picker is open. It sits
 * above the header's content (which it covers only inside the art's box) so a
 * press anywhere on the picture grabs it. One pointer pans; two pointers (a
 * touch pinch) zoom by the change in their distance; the wheel zooms. The
 * picker's sliders are the accessible way to the same values, so this layer is
 * hidden from assistive tech.
 */
function AdjustSurface({ background, imgRef, onAdjust }: IAdjustSurfaceProps) {
    const surfaceRef = useRef<HTMLDivElement | null>(null);
    const latest = useRef({ background, onAdjust });
    latest.current = { background, onAdjust };
    const pointers = useRef(new Map<number, { x: number; y: number }>());
    const gesture = useRef<{ start: ProfileBackground; x: number; y: number; distance: number } | null>(null);
    const [grabbing, setGrabbing] = useState(false);

    // React's wheel listener is passive, and a zoom must stop the page scrolling.
    useEffect(() => {
        const el = surfaceRef.current;
        if (!el) return;
        const onWheel = (e: WheelEvent) => {
            e.preventDefault();
            const { background: bg, onAdjust: report } = latest.current;
            const next = withScale(bg, wheelZoom(zoomOf(bg), e));
            if (zoomOf(next) !== zoomOf(bg)) report(next);
        };
        el.addEventListener("wheel", onWheel, { passive: false });
        return () => el.removeEventListener("wheel", onWheel);
    }, []);

    const restart = () => {
        const spread = pointerSpread([...pointers.current.values()]);
        gesture.current = spread && { start: latest.current.background, x: spread.x, y: spread.y, distance: spread.distance };
    };
    const release = (e: ReactPointerEvent<HTMLDivElement>) => {
        pointers.current.delete(e.pointerId);
        if (pointers.current.size === 0) setGrabbing(false);
        restart();
    };

    return (
        <div
            ref={surfaceRef}
            className={styles.adjust}
            data-grabbing={grabbing}
            aria-hidden="true"
            onPointerDown={(e) => {
                if (e.button !== 0) return;
                e.preventDefault();
                e.currentTarget.setPointerCapture(e.pointerId);
                pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
                setGrabbing(true);
                restart();
            }}
            onPointerMove={(e) => {
                if (!pointers.current.has(e.pointerId)) return;
                pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
                const g = gesture.current;
                const geometry = geometryOf(imgRef.current);
                const now = pointerSpread([...pointers.current.values()]);
                if (!g || !geometry || !now) return;
                let next = g.start;
                if (now.pinch && g.distance > 0) next = withScale(next, zoomOf(g.start) * (now.distance / g.distance));
                next = panFocus(next, now.x - g.x, now.y - g.y, geometry);
                const shown = latest.current.background;
                if (next.focus_x !== shown.focus_x || next.focus_y !== shown.focus_y || zoomOf(next) !== zoomOf(shown)) latest.current.onAdjust(next);
            }}
            onPointerUp={release}
            onPointerCancel={release}
        />
    );
}
