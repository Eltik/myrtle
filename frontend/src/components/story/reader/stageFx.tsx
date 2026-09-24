/**
 * THE FOUR THINGS EVERY LAYER ON THE STAGE DOES: blur for focus, dim for
 * lighting, fade for a transition, and stay mounted long enough to cross-fade.
 *
 * They are apart from `Stage.tsx` because the scene layers and the character
 * sprites each use all four and neither owns them. Each carries the read that
 * justifies it: what the client's shader chain actually is, and what
 * `SetColorWithoutAlpha` actually writes.
 */
import type React from "react";
import { useEffect, useRef, useState } from "react";

/**
 * `focusout` amount to a CSS filter. It is a BLUR and nothing else: the render
 * loads `GetBlitAlphaGhostMatPath` and runs a three-round ping-pong downsample
 * chain, so the reader's old brightness ramp was modelling a darken the client
 * never does. 12 px at amount 1 is OUR scale; the chain's radius is not in the
 * binary.
 */
export function focusFilter(amount: number | undefined): string | undefined {
    if (amount === undefined || amount <= 0) return undefined;
    return `blur(${(amount * 12).toFixed(2)}px)`;
}

/**
 * The lighting multiply. `SetColorWithoutAlpha` writes RGB (0.5,0.5,0.5) and
 * leaves alpha alone, which is exactly what `brightness(0.5)` does: a CSS
 * brightness filter multiplies each colour channel and never touches alpha,
 * where an opacity or a black overlay would change the silhouette's edges.
 */
export const DIM_FILTER = "brightness(0.5)";

export function joinFilters(...parts: Array<string | undefined>): string | undefined {
    const on = parts.filter((p): p is string => Boolean(p));
    return on.length > 0 ? on.join(" ") : undefined;
}

/**
 * Keeps the node that just left mounted for `sec`, which is what lets a swap
 * CROSSFADE: `_SwapImages` moves the current image to the back layer and the
 * incoming one fades in over it, rather than the old one being replaced.
 */
export function useOutgoing<T>(current: T | undefined, id: string | undefined, sec: number): { item: T; id: string } | null {
    const [out, setOut] = useState<{ item: T; id: string } | null>(null);
    const held = useRef<{ item: T; id: string } | null>(current !== undefined && id !== undefined ? { item: current, id } : null);
    useEffect(() => {
        const prev = held.current;
        held.current = current !== undefined && id !== undefined ? { item: current, id } : null;
        if (!prev || prev.id === id) return;
        setOut(prev);
        const timer = window.setTimeout(() => setOut((o) => (o === prev ? null : o)), Math.max(0, sec) * 1000 + 80);
        return () => window.clearTimeout(timer);
    }, [current, id, sec]);
    return out;
}

/** A fade of `sec` seconds, ease Linear, which is the serialized `_fadeEase` on every slot and image view. */
export function fade(direction: "in" | "out", sec: number): React.CSSProperties {
    return { animationName: direction === "in" ? "story-fade-in" : "story-fade-out", animationDuration: `${sec}s`, animationTimingFunction: "linear", animationFillMode: "both" };
}
