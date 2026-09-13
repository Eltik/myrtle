import { ReleaseNoteDialog, WhatsNewButton } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The "what's new" announcement. It has no props: the entry it shows is
// `LATEST_ANNOUNCED_NOTE` from `src/content/changelog/entries.ts`, and whether
// it is open lives in a `@tanstack/store` singleton that only the header bell
// (`WhatsNewButton`) can flip. The root route mounts one `<ReleaseNoteDialog />`
// next to the page and the bell opens it from anywhere.
//
// So this story is the honest composition: the bell in a header strip and the
// dialog beside it, with the bell clicked on mount. The store cannot be reached
// from a preview (the bundle keeps its own module instance), so the click is
// the only way in - which is also how the product works.

const SEEN_KEY = "myrtle-changelog-seen";

function Stage({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);

    // Fresh visitor: the store reads this key in the bell's effect, and a
    // missing key is what lights the dot the click then clears.
    try {
        window.localStorage.removeItem(SEEN_KEY);
    } catch {
        // Storage blocked: the dialog still opens on click.
    }

    useEffect(() => {
        // Base UI wires the trigger after first paint, so a click fired straight
        // from the effect is dropped - wait two frames. The open popup then
        // focuses its first tabbable control (the "All updates" link) once its
        // open transition lands, and paints the brand-red ring on it; a frame
        // later is too early for that, so drop the ring on a short timer.
        let f2 = 0;
        const timers: number[] = [];
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                ref.current?.querySelector<HTMLButtonElement>('button[aria-label^="What\'s new"]')?.click();
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
        <div className="relative min-h-[520px] w-full" ref={ref}>
            <div className="flex h-14 w-full items-center justify-end gap-0.5 border-border border-b bg-background px-4">
                <WhatsNewButton />
            </div>
            {children}
        </div>
    );
}

// The newest announced entry, opened from the bell: date eyebrow, title, the
// markdown lead, the "Also in this update" list, and a footer whose primary
// action deep-links to the thing that changed ("Browse operators") beside a
// quiet "Got it".
export const OpenFromBell = () => (
    <Stage>
        <ReleaseNoteDialog />
    </Stage>
);
