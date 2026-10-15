import { ReleaseInfoHint } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The pull planner's attached explanation: an info glyph that opens a popover on
// hover or tap.

function OpenOnMount({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        const timers: number[] = [];
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                ref.current?.querySelector<HTMLButtonElement>("button[aria-label]")?.click();
                for (const ms of [60, 180, 400]) timers.push(window.setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
            for (const t of timers) window.clearTimeout(t);
        };
    }, []);
    return (
        <div ref={ref} className="relative w-full p-4" style={{ minHeight: 520 }}>
            {children}
        </div>
    );
}

// Closed, beside the field label it explains.
export const Closed = () => (
    <div className="p-4">
        <span className="inline-flex items-center gap-1 font-medium font-sans text-[12.5px] text-muted-foreground">
            Pulls since last 6★
            <ReleaseInfoHint label="About the pity counter">Counts on Standard and Kernel banners, which share a counter. Limited and collab banners always start you at zero.</ReleaseInfoHint>
        </span>
    </div>
);

// Opened.
export const Open = () => (
    <OpenOnMount>
        <span className="inline-flex items-center gap-1 font-medium font-sans text-[12.5px] text-muted-foreground">
            Pulls since last 6★
            <ReleaseInfoHint label="About the pity counter">Counts on Standard and Kernel banners, which share a counter. Limited and collab banners always start you at zero.</ReleaseInfoHint>
        </span>
    </OpenOnMount>
);
