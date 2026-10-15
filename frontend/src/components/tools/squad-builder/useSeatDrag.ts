import { useCallback, useEffect, useRef, useState } from "react";

const DRAG_THRESHOLD_PX = 6;
const TOUCH_LONG_PRESS_MS = 320;
const LONG_PRESS_CANCEL_PX = 10;

export interface ISeatDragState {
    from: number;
    x: number;
    y: number;
    /** The seat under the pointer, `null` off the grid. */
    over: number | null;
}

interface IPress {
    seat: number;
    pointerId: number;
    pointerType: string;
    startX: number;
    startY: number;
    lifted: boolean;
}

function seatAt(x: number, y: number): number | null {
    const el = document.elementFromPoint(x, y)?.closest<HTMLElement>("[data-seat]");
    if (!el) return null;
    const n = Number(el.dataset.seat);
    return Number.isInteger(n) ? n : null;
}

/**
 * Drag a seat card onto another to swap them. A mouse or pen lifts after a
 * 6px move; a finger lifts after a 320ms press, so a swipe still scrolls.
 * `consumeClick` tells a card's click handler a drag just ended on it, so a
 * drop does not also open the seat editor.
 */
export function useSeatDrag(onSwap: (from: number, to: number) => void, enabled: boolean) {
    const [drag, setDrag] = useState<ISeatDragState | null>(null);
    const pressRef = useRef<IPress | null>(null);
    const timerRef = useRef<number | null>(null);
    const suppressClickRef = useRef(false);
    const onSwapRef = useRef(onSwap);
    onSwapRef.current = onSwap;

    const clearTimer = useCallback(() => {
        if (timerRef.current !== null) {
            window.clearTimeout(timerRef.current);
            timerRef.current = null;
        }
    }, []);

    const end = useCallback(() => {
        clearTimer();
        pressRef.current = null;
        setDrag(null);
    }, [clearTimer]);

    useEffect(() => {
        if (!enabled) return;
        const onMove = (e: PointerEvent) => {
            const p = pressRef.current;
            if (!p || e.pointerId !== p.pointerId) return;
            const dx = e.clientX - p.startX;
            const dy = e.clientY - p.startY;
            const dist = Math.hypot(dx, dy);
            if (!p.lifted) {
                if (p.pointerType === "touch") {
                    // Moving before the long press fires is a scroll, not a drag.
                    if (dist > LONG_PRESS_CANCEL_PX) end();
                    return;
                }
                if (dist < DRAG_THRESHOLD_PX) return;
                p.lifted = true;
            }
            setDrag({ from: p.seat, x: e.clientX, y: e.clientY, over: seatAt(e.clientX, e.clientY) });
        };
        const onUp = (e: PointerEvent) => {
            const p = pressRef.current;
            if (!p || e.pointerId !== p.pointerId) return;
            if (p.lifted) {
                suppressClickRef.current = true;
                window.setTimeout(() => {
                    suppressClickRef.current = false;
                }, 0);
                const to = seatAt(e.clientX, e.clientY);
                if (to !== null && to !== p.seat) onSwapRef.current(p.seat, to);
            }
            end();
        };
        const onCancel = () => end();
        // While a finger drags, the page must not scroll under it.
        const onTouchMove = (e: TouchEvent) => {
            if (pressRef.current?.lifted) e.preventDefault();
        };
        const onKey = (e: KeyboardEvent) => {
            if (e.key === "Escape") end();
        };
        window.addEventListener("pointermove", onMove);
        window.addEventListener("pointerup", onUp);
        window.addEventListener("pointercancel", onCancel);
        window.addEventListener("touchmove", onTouchMove, { passive: false });
        window.addEventListener("keydown", onKey);
        return () => {
            window.removeEventListener("pointermove", onMove);
            window.removeEventListener("pointerup", onUp);
            window.removeEventListener("pointercancel", onCancel);
            window.removeEventListener("touchmove", onTouchMove);
            window.removeEventListener("keydown", onKey);
        };
    }, [enabled, end]);

    useEffect(() => clearTimer, [clearTimer]);

    // No text selection under a mouse drag.
    const lifted = drag !== null;
    useEffect(() => {
        if (!lifted) return;
        const prev = document.body.style.userSelect;
        document.body.style.userSelect = "none";
        return () => {
            document.body.style.userSelect = prev;
        };
    }, [lifted]);

    const onPointerDown = useCallback(
        (e: React.PointerEvent<HTMLElement>, seat: number) => {
            if (!enabled || e.button !== 0) return;
            const press: IPress = { seat, pointerId: e.pointerId, pointerType: e.pointerType, startX: e.clientX, startY: e.clientY, lifted: false };
            pressRef.current = press;
            if (e.pointerType === "touch") {
                clearTimer();
                timerRef.current = window.setTimeout(() => {
                    if (pressRef.current !== press) return;
                    press.lifted = true;
                    setDrag({ from: seat, x: press.startX, y: press.startY, over: seat });
                }, TOUCH_LONG_PRESS_MS);
            }
        },
        [enabled, clearTimer],
    );

    const consumeClick = useCallback((): boolean => suppressClickRef.current, []);

    return { drag, onPointerDown, consumeClick };
}
