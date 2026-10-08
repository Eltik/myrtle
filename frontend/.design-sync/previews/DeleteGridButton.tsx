import { DeleteGridButton } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

// The trigger, the confirmation and the mutation for deleting one grid.
// `icon` is the quiet trash button on My grids cards; `labelled` is the
// destructive outline button on a grid's page and in its editor.

const GRID = { slug: "about-me-i95xd5", title: "About Me" };
const noop = () => {};

/** Clicks the trigger two frames after mount (an effect alone is dropped), then blurs the dialog's auto-focused control so no red ring reads as an error. */
const OpenOnMount = ({ children }: { children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const ids: ReturnType<typeof setTimeout>[] = [];
        requestAnimationFrame(() =>
            requestAnimationFrame(() => {
                ref.current?.querySelector("button")?.click();
                for (const ms of [60, 180, 400]) ids.push(setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
            }),
        );
        return () => ids.forEach(clearTimeout);
    }, []);
    return (
        <div ref={ref} className="min-h-dvh p-4">
            {children}
        </div>
    );
};

export const Icon = () => (
    <div className="p-4">
        <DeleteGridButton grid={GRID} />
    </div>
);

export const Labelled = () => (
    <div className="p-4">
        <DeleteGridButton grid={GRID} variant="labelled" onDeleted={noop} />
    </div>
);

export const Disabled = () => (
    <div className="flex gap-3 p-4">
        <DeleteGridButton grid={GRID} disabled />
        <DeleteGridButton grid={GRID} variant="labelled" disabled />
    </div>
);

/** The confirmation open over the page. */
export const Confirming = () => (
    <OpenOnMount>
        <DeleteGridButton grid={GRID} variant="labelled" />
    </OpenOnMount>
);
