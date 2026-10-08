import { SkipDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// The Skip sheet: a "Story summary" kicker, where the story sits, its title,
// the game's own synopsis, and two answers (Keep reading / Skip to end). It is
// a controlled Base UI dialog, so `open` mounts it directly; the stage gives
// the portalled backdrop a height, and the auto-focus ring is cleared once the
// open transition lands.

const noop = () => {};

function Stage({ children }: { children: ReactNode }) {
    useEffect(() => {
        const timers = [60, 180, 400].map((ms) => window.setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => {
            for (const t of timers) window.clearTimeout(t);
        };
    }, []);
    return <div className="relative min-h-[520px] w-full bg-black">{children}</div>;
}

// A main-story stage with the game's synopsis, two paragraphs.
export const WithSynopsis = () => (
    <Stage>
        <SkipDialog
            open
            onCancel={noop}
            onConfirm={noop}
            title="Decompression Syndrome"
            node="15-8 · After Operation"
            synopsis={"PRTS completes its reboot inside Rhodes Island's dormant systems and flags an unauthorized third-party plugin.\nThe Doctor and Kal'tsit retrace the corridors toward the CEO's office, where an old record waits to be opened."}
        />
    </Stage>
);

// A story the game ships no synopsis for: the panel says so instead.
export const NoSynopsis = () => (
    <Stage>
        <SkipDialog open onCancel={noop} onConfirm={noop} title="Prologue, Part 1" node="Episode 00 · Prologue" />
    </Stage>
);

// A story outside a numbered chapter: the node line is empty and prints nothing.
export const NoNode = () => (
    <Stage>
        <SkipDialog open onCancel={noop} onConfirm={noop} title="Decompression Syndrome" node="" synopsis="PRTS completes its reboot inside Rhodes Island's dormant systems and flags an unauthorized third-party plugin." />
    </Stage>
);
