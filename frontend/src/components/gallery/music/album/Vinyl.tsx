import type React from "react";
import { useEffect, useState } from "react";
import { cn } from "#/lib/utils";
import { sizedCover } from "../impl/cover";

/** Groove radii on a 100-unit disc, outermost first. */
const GROOVES = [46, 42, 38.5, 35, 31.5, 28, 24.5];

/**
 * The sleeve: a square cover with a record peeking out to the right. The disc
 * slides out once the sleeve mounts (300 ms, the page's longest transition;
 * it was 700 ms and still moving well after the dialog had opened) and spins
 * while `spinning`, the page's one perpetual motion, a compositor-only
 * rotation. Reduced motion keeps it still (the slide and the spin both stop).
 */
export function Sleeve({ coverUrl, alt, spinning, size = 196 }: { coverUrl: string; alt: string; spinning: boolean; size?: number }): React.ReactElement {
    const [out, setOut] = useState(false);
    useEffect(() => {
        const id = requestAnimationFrame(() => setOut(true));
        return () => cancelAnimationFrame(id);
    }, []);
    return (
        <div className="relative shrink-0" style={{ width: size, height: size }}>
            <div className={cn("absolute inset-[5px] transition-transform duration-300 ease-out motion-reduce:transition-none", out ? "translate-x-[32%]" : "translate-x-0")}>
                <div className={cn("size-full", spinning && "music-spin")}>
                    <Disc />
                </div>
            </div>
            <img referrerPolicy="no-referrer" src={sizedCover(coverUrl, size)} alt={alt} decoding="async" className="relative size-full rounded-[6px] bg-muted object-cover shadow-lg/25" />
        </div>
    );
}

function Disc(): React.ReactElement {
    return (
        <svg viewBox="0 0 100 100" className="size-full drop-shadow-md" aria-hidden="true">
            <circle cx="50" cy="50" r="50" className="fill-neutral-900" />
            {GROOVES.map((r) => (
                <circle key={r} cx="50" cy="50" r={r} fill="none" stroke="white" strokeOpacity="0.07" strokeWidth="0.6" />
            ))}
            <path d="M50 4 A46 46 0 0 1 96 50" fill="none" stroke="white" strokeOpacity="0.12" strokeWidth="5" />
            <circle cx="50" cy="50" r="17" className="fill-primary" />
            <circle cx="50" cy="50" r="12.5" fill="none" stroke="black" strokeOpacity="0.15" strokeWidth="0.5" />
            <circle cx="50" cy="50" r="1.8" className="fill-neutral-900" />
        </svg>
    );
}
