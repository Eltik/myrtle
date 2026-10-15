import { type KeyboardEvent, type PointerEvent as ReactPointerEvent, type RefObject, useCallback, useEffect, useId, useRef, useState } from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import type { IUserProfile } from "#/types/user";
import { croppableAxes, deadDragAxis, type IArtGeometry, keyAdjust, panFocus, wheelScale, withScale, zoomOf } from "../background";
import { Hero } from "../components/Hero";
import artStyles from "../components/HeroArt.module.css";
import type { messages } from "./BackgroundEditor.messages";
import { PreviewFrame } from "./PreviewFrame";
import { previewScale } from "./preview-geometry";

interface IPreviewStageProps {
    profile: IUserProfile;
    background: ProfileBackground | null;
    /** The viewport width the header is laid out at (see `previewViewport`). */
    viewportWidth: number;
    maxHeight: number;
    onChange: (next: ProfileBackground) => void;
    /** Told the dead axis the author just tried to move along (no slack there at this zoom), so the toolbar can say why nothing moved. */
    onDeadAxis: (axis: "x" | "y") => void;
}

/** The header's box inside the frame, in the frame's own (unscaled) CSS px. */
interface IHeaderBox {
    left: number;
    top: number;
    width: number;
    height: number;
}

/** The previewed header inside the frame: the `Hero` section under the preview root. */
const HEADER_SELECTOR = "[data-preview-root] > section";

/**
 * How long a burst of layout changes is coalesced before one measure. A timer, not a frame:
 * an editor in a tab that paints no frames (behind another window, under automation) still
 * measures.
 */
const MEASURE_DELAY_MS = 16;

/** The frame's height before the header is measured, tall enough for any header to lay out in. */
const UNMEASURED_FRAME_HEIGHT = 800;
/** The stage's height before the header is measured; the stage is transparent until then. */
const UNMEASURED_STAGE_HEIGHT = 200;

const sameHeaderBox = (a: IHeaderBox, b: IHeaderBox) => a.left === b.left && a.top === b.top && a.width === b.width && a.height === b.height;
const sameGeometry = (a: IArtGeometry, b: IArtGeometry) => a.boxWidth === b.boxWidth && a.boxHeight === b.boxHeight && a.naturalWidth === b.naturalWidth && a.naturalHeight === b.naturalHeight;

/**
 * The editor's life-size header: the real `Hero` (as a visitor sees it, so without the
 * owner's buttons), laid out inside a `PreviewFrame` at the previewed viewport and scaled
 * down only when the editor is narrower. A layer over it takes the gestures: one pointer
 * pans, two pinch, the wheel zooms, and when focused the arrow keys pan (Shift for ten)
 * and +/- zoom. Every delta is divided back by the scale and read against the art's
 * geometry measured INSIDE the frame, so the focus and scale it stores are the ones the
 * same drag on the profile page would store.
 */
export function PreviewStage({ profile, background, viewportWidth, maxHeight, onChange, onDeadAxis }: IPreviewStageProps) {
    const t: TypedT<typeof messages> = useT("user");
    const keysId = useId();
    const [doc, setDoc] = useState<Document | null>(null);
    const { header, geometry } = useFrameMeasure(doc);
    const stageRef = useRef<HTMLDivElement>(null);
    const stageWidth = useWidth(stageRef);

    const scale = header && stageWidth > 0 ? previewScale(header.width, header.height, stageWidth, maxHeight) : 1;
    const axes = background ? croppableAxes(background.kind, background.scale, geometry) : { x: false, y: false };
    const latest = useRef({ background, geometry, scale, axes, onChange, onDeadAxis });
    latest.current = { background, geometry, scale, axes, onChange, onDeadAxis };

    const surface = useGestures(latest);

    const onKeyDown = (e: KeyboardEvent<HTMLButtonElement>) => {
        if (!background || e.altKey || e.metaKey || e.ctrlKey) return;
        const result = keyAdjust(background, e.key, e.shiftKey, axes);
        if (!result) return;
        e.preventDefault();
        if (result.next) onChange(result.next);
        else onDeadAxis(result.dead);
    };

    const width = header ? header.width * scale : "100%";
    const height = header ? header.height * scale : UNMEASURED_STAGE_HEIGHT;
    return (
        <div ref={stageRef} className="flex w-full min-w-0 justify-center">
            <div className={header ? "relative shrink-0 overflow-hidden rounded-2xl sm:rounded-3xl" : "relative shrink-0 overflow-hidden rounded-2xl opacity-0 sm:rounded-3xl"} style={{ width, height, maxWidth: "100%" }}>
                <PreviewFrame
                    title={t("profile.background.preview.role")}
                    viewportWidth={viewportWidth}
                    height={header ? header.top + header.height + 1 : UNMEASURED_FRAME_HEIGHT}
                    onDocument={setDoc}
                    style={{ position: "absolute", left: header ? -header.left * scale : 0, top: header ? -header.top * scale : 0, transform: `scale(${scale})`, transformOrigin: "0 0" }}
                >
                    {/* The page's own shell, so the header takes the width the page gives it at this viewport; no vertical rhythm, the frame shows the header alone. */}
                    <main className="page-shell [--page-max:1440px]" style={{ paddingBlock: 0 }} data-preview-root="" inert>
                        <Hero profile={profile} background={background} />
                    </main>
                </PreviewFrame>
                {background && (
                    // A button: the one focusable element with no action of its own to fight the drag. Its keys are described by `aria-describedby`.
                    <button
                        ref={surface.ref}
                        type="button"
                        aria-roledescription={t("profile.background.preview.role")}
                        aria-label={t("profile.background.preview.label")}
                        aria-describedby={keysId}
                        data-grabbing={surface.grabbing}
                        onKeyDown={onKeyDown}
                        onPointerDown={surface.onPointerDown}
                        onPointerMove={surface.onPointerMove}
                        onPointerUp={surface.onPointerEnd}
                        onPointerCancel={surface.onPointerEnd}
                        className="absolute inset-0 cursor-grab touch-none select-none rounded-[inherit] outline-none ring-inset focus-visible:ring-2 focus-visible:ring-ring data-[grabbing=true]:cursor-grabbing"
                    />
                )}
                <p id={keysId} className="sr-only">
                    {t("profile.background.preview.keys")}
                </p>
            </div>
        </div>
    );
}

/** The element's content width, kept current by a ResizeObserver. */
function useWidth(ref: RefObject<HTMLElement | null>): number {
    const [width, setWidth] = useState(0);
    useEffect(() => {
        const el = ref.current;
        if (!el) return;
        const observer = new ResizeObserver(() => setWidth(el.clientWidth));
        observer.observe(el);
        setWidth(el.clientWidth);
        return () => observer.disconnect();
    }, [ref]);
    return width;
}

/**
 * The header's box and the art's geometry inside the frame, re-read whenever the frame's
 * layout, the art's element or its load state changes. Read with the frame's own
 * observers, so they report the frame's viewport, unscaled.
 */
function useFrameMeasure(doc: Document | null): { header: IHeaderBox | null; geometry: IArtGeometry | null } {
    const [header, setHeader] = useState<IHeaderBox | null>(null);
    const [geometry, setGeometry] = useState<IArtGeometry | null>(null);
    useEffect(() => {
        const view = doc?.defaultView;
        if (!doc || !view) {
            setHeader(null);
            setGeometry(null);
            return;
        }
        let timer: ReturnType<typeof setTimeout> | undefined;
        const measure = () => {
            timer = undefined;
            const section = doc.querySelector<HTMLElement>(HEADER_SELECTOR);
            if (!section) return;
            const r = section.getBoundingClientRect();
            const box = { left: r.left, top: r.top, width: r.width, height: r.height };
            setHeader((prev) => (prev && sameHeaderBox(prev, box) ? prev : box));
            const img = section.querySelector<HTMLImageElement>(`.${artStyles.art}`);
            const next = img && img.dataset.loaded === "true" && img.naturalWidth > 0 ? { boxWidth: img.offsetWidth, boxHeight: img.offsetHeight, naturalWidth: img.naturalWidth, naturalHeight: img.naturalHeight } : null;
            setGeometry((prev) => (prev === next || (prev && next && sameGeometry(prev, next)) ? prev : next));
        };
        const schedule = () => {
            timer ??= setTimeout(measure, MEASURE_DELAY_MS);
        };
        const resize = new view.ResizeObserver(schedule);
        resize.observe(doc.body);
        const watch = () => {
            const section = doc.querySelector(HEADER_SELECTOR);
            if (section) resize.observe(section);
        };
        watch();
        // A new pick remounts the art; its load flips `data-loaded`. Neither resizes anything.
        const mutations = new view.MutationObserver(() => {
            watch();
            schedule();
        });
        mutations.observe(doc.body, { childList: true, subtree: true, attributes: true, attributeFilter: ["data-loaded", "src"] });
        doc.addEventListener("load", schedule, true);
        schedule();
        return () => {
            resize.disconnect();
            mutations.disconnect();
            doc.removeEventListener("load", schedule, true);
            clearTimeout(timer);
        };
    }, [doc]);
    return { header, geometry };
}

/** The gesture's point (the first pointer, or the midpoint of the first two), their distance (0 for one) and whether two are down; `null` with none. */
function pointerSpread(points: readonly { x: number; y: number }[]): { x: number; y: number; distance: number; pinch: boolean } | null {
    const [a, b] = points;
    if (!a) return null;
    if (!b) return { x: a.x, y: a.y, distance: 0, pinch: false };
    return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2, distance: Math.hypot(a.x - b.x, a.y - b.y), pinch: true };
}

interface IGestureState {
    background: ProfileBackground | null;
    geometry: IArtGeometry | null;
    scale: number;
    axes: { x: boolean; y: boolean };
    onChange: (next: ProfileBackground) => void;
    onDeadAxis: (axis: "x" | "y") => void;
}

/**
 * Pan, pinch and wheel on the preview's layer. A pointer's travel is in screen px; it is
 * divided by the preview's scale before `panFocus` reads it, so one header px of drag is
 * one header px whatever the scale. Each gesture restarts from the background it began
 * on, so rounding never accumulates.
 */
function useGestures(latest: RefObject<IGestureState>) {
    const pointers = useRef(new Map<number, { x: number; y: number }>());
    const gesture = useRef<{ start: ProfileBackground; x: number; y: number; distance: number } | null>(null);
    const [grabbing, setGrabbing] = useState(false);
    const [el, setEl] = useState<HTMLButtonElement | null>(null);

    // React's wheel listener is passive, and a zoom must not scroll the editor.
    useEffect(() => {
        if (!el) return;
        const onWheel = (e: WheelEvent) => {
            const { background, onChange } = latest.current;
            if (!background) return;
            e.preventDefault();
            const next = withScale(background, wheelScale(zoomOf(background), e.deltaY, e.deltaMode, e.ctrlKey));
            if (zoomOf(next) !== zoomOf(background)) onChange(next);
        };
        el.addEventListener("wheel", onWheel, { passive: false });
        return () => el.removeEventListener("wheel", onWheel);
    }, [el, latest]);

    const restart = () => {
        const spread = pointerSpread([...pointers.current.values()]);
        const start = latest.current.background;
        gesture.current = spread && start ? { start, x: spread.x, y: spread.y, distance: spread.distance } : null;
    };

    const onPointerDown = (e: ReactPointerEvent<HTMLButtonElement>) => {
        if (e.button !== 0) return;
        e.currentTarget.setPointerCapture(e.pointerId);
        pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
        setGrabbing(true);
        restart();
    };
    const onPointerMove = (e: ReactPointerEvent<HTMLButtonElement>) => {
        if (!pointers.current.has(e.pointerId)) return;
        pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
        const g = gesture.current;
        const { background: shown, geometry, scale, axes, onChange, onDeadAxis } = latest.current;
        const now = pointerSpread([...pointers.current.values()]);
        if (!g || !geometry || !now || !shown) return;
        let next = g.start;
        if (now.pinch && g.distance > 0) next = withScale(next, zoomOf(g.start) * (now.distance / g.distance));
        const dx = now.x - g.x;
        const dy = now.y - g.y;
        next = panFocus(next, dx / scale, dy / scale, geometry);
        const dead = now.pinch ? null : deadDragAxis(dx, dy, axes);
        if (dead) onDeadAxis(dead);
        if (next.focus_x !== shown.focus_x || next.focus_y !== shown.focus_y || zoomOf(next) !== zoomOf(shown)) onChange(next);
    };
    const onPointerEnd = (e: ReactPointerEvent<HTMLButtonElement>) => {
        pointers.current.delete(e.pointerId);
        if (pointers.current.size === 0) setGrabbing(false);
        restart();
    };

    const ref = useCallback((node: HTMLButtonElement | null) => setEl(node), []);
    return { ref, grabbing, onPointerDown, onPointerMove, onPointerEnd };
}
