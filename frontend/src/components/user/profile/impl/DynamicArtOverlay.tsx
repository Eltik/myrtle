import { useEffect, useRef, useState } from "react";
import type { ISpineFit } from "#/components/operators/detail/impl/components/chibi/helpers";
import { SceneIllustPlayer } from "#/components/operators/detail/impl/components/dynillust/SceneIllust.lazy";
import { cn } from "#/lib/utils";
import { useDynamicArt } from "./dynamic-art";

interface IDynamicArtOverlayProps {
    operatorCode: string | null | undefined;
    skinId: string | null | undefined;
    /** Framing to match the static image this overlays (its object-fit/position). */
    fit?: ISpineFit;
    /**
     * Defer mounting the (multi-MB) player until the host scrolls into view.
     * Use for grid/list surfaces where many overlays exist at once; leave off
     * for single-focus surfaces (dialogs) that are always visible.
     */
    viewportGated?: boolean;
    /**
     * Fired with `true` once the animation is rendering and `false` when it's
     * gone (toggle off, no dynamic art, or scrolled out of view). Parents use it
     * to fade the static image out so the two don't show through each other.
     */
    onActiveChange?: (active: boolean) => void;
    className?: string;
}

/**
 * Plays an operator's dynamic (L2D) illustration on top of whatever static art
 * it's layered over, when the page-wide toggle is on and the operator/skin has
 * a dynamic set. Absolutely fills its positioned parent and is click-through.
 */
export function DynamicArtOverlay({ operatorCode, skinId, fit, viewportGated = false, onActiveChange, className }: IDynamicArtOverlayProps) {
    const ctx = useDynamicArt();
    const files = ctx?.getDynamicFiles(operatorCode, skinId) ?? null;

    const hostRef = useRef<HTMLDivElement>(null);
    const [inView, setInView] = useState(!viewportGated);

    // Each mounted player holds a WebGL context, and browsers cap those (~16).
    // So in gated mode we track intersection live and UNMOUNT when scrolled away,
    // keeping concurrent contexts bounded to what's actually on screen. The
    // assets are HTTP-cached, so re-entry reloads cheaply.
    useEffect(() => {
        if (!viewportGated) return;
        const el = hostRef.current;
        if (!el) return;
        const observer = new IntersectionObserver((entries) => setInView(!!entries[0]?.isIntersecting), { rootMargin: "150px" });
        observer.observe(el);
        return () => observer.disconnect();
    }, [viewportGated]);

    const mounted = !!files && inView;

    // Report "not active" whenever the player isn't mounted, so the static image
    // comes back on toggle-off / scroll-out. The player reports "active" via
    // onReady once it's actually rendering.
    useEffect(() => {
        if (!mounted) onActiveChange?.(false);
    }, [mounted, onActiveChange]);

    if (!files) return null;

    return (
        <div ref={hostRef} className={cn("pointer-events-none absolute inset-0", className)}>
            {mounted && <SceneIllustPlayer files={files} server={ctx?.server} fit={fit} onReady={() => onActiveChange?.(true)} />}
        </div>
    );
}
