import { useEffect, useRef } from "react";

export function useHeroTilt() {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const el = ref.current;
        if (!el) return;
        const MAX_X = 12,
            MAX_Y = 14,
            LIFT = 24;
        let raf = 0;
        let tx = 6,
            ty = -4,
            tz = 0;
        let dx = 6,
            dy = -4,
            dz = 0;
        const step = () => {
            tx += (dx - tx) * 0.18;
            ty += (dy - ty) * 0.18;
            tz += (dz - tz) * 0.18;
            el.style.setProperty("--tilt-x", tx.toFixed(2) + "deg");
            el.style.setProperty("--tilt-y", ty.toFixed(2) + "deg");
            el.style.setProperty("--tilt-z", tz.toFixed(1) + "px");
            if (Math.abs(dx - tx) > 0.05 || Math.abs(dy - ty) > 0.05 || Math.abs(dz - tz) > 0.2) {
                raf = requestAnimationFrame(step);
            } else {
                raf = 0;
            }
        };
        const onMove = (e: MouseEvent) => {
            const r = el.getBoundingClientRect();
            const nx = (e.clientX - r.left) / r.width - 0.5;
            const ny = (e.clientY - r.top) / r.height - 0.5;
            dy = -nx * MAX_Y;
            dx = ny * MAX_X;
            dz = LIFT;
            el.classList.add("is-tilting");
            if (!raf) raf = requestAnimationFrame(step);
        };
        const onLeave = () => {
            dx = 6;
            dy = -4;
            dz = 0;
            el.classList.remove("is-tilting");
            if (!raf) raf = requestAnimationFrame(step);
        };
        el.addEventListener("mousemove", onMove);
        el.addEventListener("mouseleave", onLeave);
        return () => {
            el.removeEventListener("mousemove", onMove);
            el.removeEventListener("mouseleave", onLeave);
            if (raf) cancelAnimationFrame(raf);
        };
    }, []);
    return ref;
}
