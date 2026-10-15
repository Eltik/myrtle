import { BacklogDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// The reader's Log: every beat of this story so far, each row a jump back to
// that line. The current line is ringed and does not jump; choices sit on a
// muted band; rows past the current line (after a jump back) fall under an
// "Ahead" divider. Lines are 15-8 (Decompression Syndrome) verbatim. The
// dialog is controlled, so `open` mounts it; the stage gives the portalled
// backdrop a height and the auto-focus ring is cleared after the transition.

const noop = () => {};
type Entry = Parameters<typeof BacklogDialog>[0]["entries"][number];

function Stage({ children }: { children: ReactNode }) {
    useEffect(() => {
        const timers = [60, 180, 400].map((ms) => window.setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => {
            for (const t of timers) window.clearTimeout(t);
        };
    }, []);
    return <div className="relative w-full bg-black" style={{ minHeight: 620 }}>{children}</div>;
}

const line = (haltIndex: number, speaker: string | undefined, text: string): Entry => ({ haltIndex, kind: "line", speaker, text, isNarration: false }) as Entry;
const ENTRIES: Entry[] = [
    line(0, "???", "......"),
    line(1, "???", "Doctor..."),
    { haltIndex: 2, kind: "choice", text: "...Amiya?", isNarration: false, value: "2" } as Entry,
    line(3, "Amiya", "Doctor! I'm so relieved..."),
    line(4, "Amiya", "You're finally awake."),
    line(5, "Amiya", "The evacuation hasn't been smooth. Our neural link was abruptly severed, and it's hard to maintain stability due to the information impact at the boundaries."),
    line(6, "Amiya", "Don't worry about me. I'm doing much better, thanks to Theresa's protection."),
    line(7, "Approaching Voice", "I started preparing cognition baseline tests as soon as I confirmed that you're awake."),
    line(8, "Kal'tsit", "Welcome back, Doctor."),
];

// Read to the latest line: the last row is current.
export const AtLatest = () => (
    <Stage>
        <BacklogDialog open onOpenChange={noop} entries={ENTRIES} title="Decompression Syndrome" currentHaltIndex={8} onJump={noop} />
    </Stage>
);

// After jumping back to Amiya's first line: the rows past it are "Ahead".
export const JumpedBack = () => (
    <Stage>
        <BacklogDialog open onOpenChange={noop} entries={ENTRIES} title="Decompression Syndrome" currentHaltIndex={3} onJump={noop} />
    </Stage>
);

// Opened on the first line, before anything is logged.
export const Empty = () => (
    <Stage>
        <BacklogDialog open onOpenChange={noop} entries={[]} title="Decompression Syndrome" currentHaltIndex={0} onJump={noop} />
    </Stage>
);
