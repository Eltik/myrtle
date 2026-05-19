import { Store, useStore } from "@tanstack/react-store";
import { createContext, useCallback, useContext, useEffect, useMemo, useRef } from "react";
import { createPortal } from "react-dom";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { RARITY_HEX_MUTED } from "#/lib/utils";

const TOUCH_LONG_PRESS_MS = 220;
const DRAG_THRESHOLD_PX = 6;
const SCROLL_EDGE_PX = 80;
const SCROLL_VELOCITY_PX = 18;

export type DragDropTarget = { kind: "tier"; tierId: string; index: number } | { kind: "pool" };

export interface IDragState {
    operatorId: string;
    pointerId: number;
    pointerType: string;
    startX: number;
    startY: number;
    currentX: number;
    currentY: number;
    offsetX: number;
    offsetY: number;
    tileSize: number;
    isLifted: boolean;
    target: DragDropTarget | null;
}

interface IDragControllerCtx {
    store: Store<IDragState | null>;
    startPress: (e: React.PointerEvent<HTMLElement>, operatorId: string) => void;
}

const DragCtx = createContext<IDragControllerCtx | null>(null);

interface IProviderProps {
    operatorById: Record<string, ITierOperator>;
    onPlace: (operatorId: string, tierId: string, index: number) => void;
    onUnplace: (operatorId: string) => void;
    children: React.ReactNode;
}

export function DragControllerProvider({ operatorById, onPlace, onUnplace, children }: IProviderProps) {
    const store = useMemo(() => new Store<IDragState | null>(null), []);
    const longPressTimerRef = useRef<number | null>(null);
    const sourceElRef = useRef<HTMLElement | null>(null);
    const autoScrollRafRef = useRef<number | null>(null);
    const autoScrollDirRef = useRef(0);
    const onPlaceRef = useRef(onPlace);
    const onUnplaceRef = useRef(onUnplace);
    onPlaceRef.current = onPlace;
    onUnplaceRef.current = onUnplace;

    const clearLongPress = useCallback(() => {
        if (longPressTimerRef.current !== null) {
            window.clearTimeout(longPressTimerRef.current);
            longPressTimerRef.current = null;
        }
    }, []);

    const stopAutoScroll = useCallback(() => {
        autoScrollDirRef.current = 0;
        if (autoScrollRafRef.current !== null) {
            cancelAnimationFrame(autoScrollRafRef.current);
            autoScrollRafRef.current = null;
        }
    }, []);

    const ensureAutoScroll = useCallback(() => {
        if (autoScrollRafRef.current !== null) return;
        const tick = () => {
            if (autoScrollDirRef.current === 0) {
                autoScrollRafRef.current = null;
                return;
            }
            window.scrollBy({ top: SCROLL_VELOCITY_PX * autoScrollDirRef.current, behavior: "auto" });
            autoScrollRafRef.current = requestAnimationFrame(tick);
        };
        autoScrollRafRef.current = requestAnimationFrame(tick);
    }, []);

    const updateAutoScroll = useCallback(
        (clientY: number) => {
            const vh = window.innerHeight || document.documentElement.clientHeight;
            if (clientY < SCROLL_EDGE_PX) {
                autoScrollDirRef.current = -1;
                ensureAutoScroll();
            } else if (clientY > vh - SCROLL_EDGE_PX) {
                autoScrollDirRef.current = 1;
                ensureAutoScroll();
            } else if (autoScrollDirRef.current !== 0) {
                stopAutoScroll();
            }
        },
        [ensureAutoScroll, stopAutoScroll],
    );

    const releaseCapture = useCallback(() => {
        const cur = store.state;
        const src = sourceElRef.current;
        if (src && cur) {
            try {
                src.releasePointerCapture(cur.pointerId);
            } catch {
                /* already released */
            }
        }
        sourceElRef.current = null;
    }, [store]);

    const reset = useCallback(() => {
        releaseCapture();
        store.setState(() => null);
        clearLongPress();
        stopAutoScroll();
    }, [clearLongPress, releaseCapture, stopAutoScroll, store]);

    const lift = useCallback(() => {
        clearLongPress();
        store.setState((prev) => (prev ? { ...prev, isLifted: true } : prev));
        try {
            if (typeof navigator !== "undefined" && "vibrate" in navigator) {
                navigator.vibrate?.(10);
            }
        } catch {
            /* haptics unsupported */
        }
    }, [clearLongPress, store]);

    const hitTest = useCallback((x: number, y: number, draggingOpId: string): DragDropTarget | null => {
        const el = document.elementFromPoint(x, y);
        if (!el) return null;

        if ((el as Element).closest("[data-tl-drop-pool]")) {
            return { kind: "pool" };
        }

        const rowEl = (el as Element).closest("[data-tl-drop-tier]") as HTMLElement | null;
        if (!rowEl) return null;
        const tierId = rowEl.getAttribute("data-tl-drop-tier");
        if (!tierId) return null;

        const allChips = Array.from(rowEl.querySelectorAll<HTMLElement>("[data-tl-chip-id]"));
        const otherChips: HTMLElement[] = [];
        const originalIdx: number[] = [];
        for (let i = 0; i < allChips.length; i++) {
            const chip = allChips[i];
            if (!chip) continue;
            if (chip.dataset.tlChipId === draggingOpId) continue;
            otherChips.push(chip);
            originalIdx.push(i);
        }
        if (otherChips.length === 0) {
            return { kind: "tier", tierId, index: 0 };
        }

        let nearestI = 0;
        let nearestDist = Infinity;
        for (let i = 0; i < otherChips.length; i++) {
            const chip = otherChips[i];
            if (!chip) continue;
            const rect = chip.getBoundingClientRect();
            const cx = rect.left + rect.width / 2;
            const cy = rect.top + rect.height / 2;
            const dist = Math.hypot(x - cx, y - cy);
            if (dist < nearestDist) {
                nearestDist = dist;
                nearestI = i;
            }
        }
        const nearestChip = otherChips[nearestI];
        if (!nearestChip) return { kind: "tier", tierId, index: allChips.length };
        const rect = nearestChip.getBoundingClientRect();
        const before = x < rect.left + rect.width / 2;
        const insertBeforeOther = before ? nearestI : nearestI + 1;

        let index: number;
        if (insertBeforeOther >= otherChips.length) {
            index = allChips.length;
        } else {
            index = originalIdx[insertBeforeOther] ?? allChips.length;
        }
        return { kind: "tier", tierId, index };
    }, []);

    const startPress = useCallback(
        (e: React.PointerEvent<HTMLElement>, operatorId: string) => {
            // Mouse uses the native HTML5 Drag-and-Drop path. Pointer-based drag
            // is only for touch / pen, which don't fire HTML5 drag events.
            if (e.pointerType !== "touch" && e.pointerType !== "pen") return;
            const target = e.currentTarget;
            const rect = target.getBoundingClientRect();
            sourceElRef.current = target;

            store.setState(() => ({
                operatorId,
                pointerId: e.pointerId,
                pointerType: e.pointerType,
                startX: e.clientX,
                startY: e.clientY,
                currentX: e.clientX,
                currentY: e.clientY,
                offsetX: e.clientX - rect.left,
                offsetY: e.clientY - rect.top,
                tileSize: Math.max(rect.width, rect.height),
                isLifted: false,
                target: null,
            }));

            try {
                target.setPointerCapture(e.pointerId);
            } catch {
                /* capture unavailable */
            }

            longPressTimerRef.current = window.setTimeout(lift, TOUCH_LONG_PRESS_MS);
        },
        [lift, store],
    );

    useEffect(() => {
        let active = true;
        let didLockBody = false;
        const apply = () => {
            const cur = store.state;
            if (!active) return;
            if (cur?.isLifted) {
                if (!didLockBody) {
                    document.body.style.userSelect = "none";
                    document.body.style.setProperty("-webkit-user-select", "none");
                    document.body.style.cursor = "grabbing";
                    didLockBody = true;
                }
            } else if (didLockBody) {
                document.body.style.userSelect = "";
                document.body.style.removeProperty("-webkit-user-select");
                document.body.style.cursor = "";
                didLockBody = false;
            }
            if (!cur) stopAutoScroll();
        };
        apply();
        const sub = store.subscribe(apply);
        return () => {
            active = false;
            sub.unsubscribe();
            if (didLockBody) {
                document.body.style.userSelect = "";
                document.body.style.removeProperty("-webkit-user-select");
                document.body.style.cursor = "";
            }
        };
    }, [store, stopAutoScroll]);

    useEffect(() => {
        // Coalesce pointermove work to one rAF tick. Pointer events can fire at
        // 120+ Hz on high-refresh devices; collapsing to display refresh rate
        // halves the hit-test + setState load without changing how the drag
        // looks (the ghost is positioned in the same rAF).
        let pendingMove: { x: number; y: number } | null = null;
        let moveRaf: number | null = null;

        const flushMove = () => {
            moveRaf = null;
            const m = pendingMove;
            pendingMove = null;
            if (!m) return;
            const cur = store.state;
            if (!cur || !cur.isLifted) return;
            const target = hitTest(m.x, m.y, cur.operatorId);
            store.setState((prev) => (prev ? { ...prev, currentX: m.x, currentY: m.y, target } : prev));
            updateAutoScroll(m.y);
        };

        const scheduleMove = (x: number, y: number) => {
            pendingMove = { x, y };
            if (moveRaf !== null) return;
            moveRaf = requestAnimationFrame(flushMove);
        };

        const cancelScheduledMove = () => {
            if (moveRaf !== null) {
                cancelAnimationFrame(moveRaf);
                moveRaf = null;
            }
            pendingMove = null;
        };

        const onMove = (e: PointerEvent) => {
            const cur = store.state;
            if (!cur || e.pointerId !== cur.pointerId) return;
            const dx = e.clientX - cur.startX;
            const dy = e.clientY - cur.startY;

            if (!cur.isLifted) {
                // Tiles use `touch-action: none`, so JS owns the gesture from
                // the first move - lift immediately past the threshold instead
                // of waiting out the full long-press timer.
                if (Math.abs(dx) > DRAG_THRESHOLD_PX || Math.abs(dy) > DRAG_THRESHOLD_PX) {
                    lift();
                    e.preventDefault();
                    scheduleMove(e.clientX, e.clientY);
                }
                return;
            }

            e.preventDefault();
            scheduleMove(e.clientX, e.clientY);
        };

        const commit = () => {
            // Drain any queued move so the drop target reflects the final
            // pointer position, not the previous rAF's snapshot.
            if (pendingMove) flushMove();
            const cur = store.state;
            if (!cur) return;
            if (cur.isLifted && cur.target) {
                if (cur.target.kind === "pool") {
                    onUnplaceRef.current(cur.operatorId);
                } else {
                    onPlaceRef.current(cur.operatorId, cur.target.tierId, cur.target.index);
                }
            }
            reset();
        };

        const onUp = (e: PointerEvent) => {
            const cur = store.state;
            if (!cur || e.pointerId !== cur.pointerId) return;
            if (cur.isLifted) e.preventDefault();
            commit();
        };

        const onCancel = (e: PointerEvent) => {
            const cur = store.state;
            if (!cur || e.pointerId !== cur.pointerId) return;
            cancelScheduledMove();
            reset();
        };

        const onContextMenu = (e: Event) => {
            if (store.state?.isLifted) e.preventDefault();
        };

        window.addEventListener("pointermove", onMove, { passive: false });
        window.addEventListener("pointerup", onUp);
        window.addEventListener("pointercancel", onCancel);
        window.addEventListener("contextmenu", onContextMenu);
        return () => {
            cancelScheduledMove();
            window.removeEventListener("pointermove", onMove);
            window.removeEventListener("pointerup", onUp);
            window.removeEventListener("pointercancel", onCancel);
            window.removeEventListener("contextmenu", onContextMenu);
        };
    }, [store, hitTest, reset, updateAutoScroll, lift]);

    useEffect(
        () => () => {
            clearLongPress();
            stopAutoScroll();
        },
        [clearLongPress, stopAutoScroll],
    );

    const value = useMemo<IDragControllerCtx>(() => ({ store, startPress }), [store, startPress]);

    return (
        <DragCtx.Provider value={value}>
            {children}
            <GhostPortal store={store} operatorById={operatorById} />
        </DragCtx.Provider>
    );
}

function useDragCtx(): IDragControllerCtx {
    const ctx = useContext(DragCtx);
    if (!ctx) throw new Error("useDragCtx must be used inside DragControllerProvider");
    return ctx;
}

export function useStartOperatorDrag(): IDragControllerCtx["startPress"] {
    return useDragCtx().startPress;
}

export function useIsDragSource(operatorId: string): boolean {
    const { store } = useDragCtx();
    return useStore(store, (s) => s !== null && s.operatorId === operatorId && s.isLifted);
}

export function useTierDropIndex(tierId: string): number | null {
    const { store } = useDragCtx();
    return useStore(store, (s) => (s?.isLifted && s.target?.kind === "tier" && s.target.tierId === tierId ? s.target.index : null));
}

export function usePoolIsOver(): boolean {
    const { store } = useDragCtx();
    return useStore(store, (s) => Boolean(s?.isLifted && s.target?.kind === "pool"));
}

export function useAnyDragLifted(): boolean {
    const { store } = useDragCtx();
    return useStore(store, (s) => Boolean(s?.isLifted));
}

interface IGhostPortalProps {
    store: Store<IDragState | null>;
    operatorById: Record<string, ITierOperator>;
}

function GhostPortal({ store, operatorById }: IGhostPortalProps) {
    const operatorId = useStore(store, (s) => (s?.isLifted ? s.operatorId : null));
    const tileSize = useStore(store, (s) => (s?.isLifted ? s.tileSize : 0));
    const ghostRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        if (!operatorId) return;
        const apply = () => {
            const s = store.state;
            const el = ghostRef.current;
            if (!s || !el) return;
            const left = s.currentX - s.offsetX;
            const top = s.currentY - s.offsetY;
            el.style.transform = `translate3d(${left}px, ${top}px, 0) scale(1.12)`;
        };
        apply();
        const sub = store.subscribe(apply);
        return () => sub.unsubscribe();
    }, [operatorId, store]);

    if (!operatorId || typeof document === "undefined") return null;
    const op = operatorById[operatorId];
    if (!op) return null;

    const size = tileSize > 0 ? tileSize : 64;
    const accent = RARITY_HEX_MUTED[op.rarity] ?? RARITY_HEX_MUTED[1];

    const style: React.CSSProperties = {
        position: "fixed",
        left: 0,
        top: 0,
        width: size,
        height: size,
        transformOrigin: "center center",
        borderRadius: 10,
        overflow: "hidden",
        zIndex: 1000,
        pointerEvents: "none",
        boxShadow: "0 14px 28px oklch(0 0 0 / 0.32), 0 0 0 1.5px var(--ring)",
        background: "oklch(0.22 0.005 285)",
        willChange: "transform",
    };

    const accentStyle: React.CSSProperties = {
        position: "absolute",
        inset: "auto 0 0 0",
        height: 3,
        background: accent,
        pointerEvents: "none",
    };

    return createPortal(
        <div ref={ghostRef} style={style} aria-hidden="true">
            <OperatorAvatar charId={op.id} name={op.name} />
            <span style={accentStyle} />
        </div>,
        document.body,
    );
}
