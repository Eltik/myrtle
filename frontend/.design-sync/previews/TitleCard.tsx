import { TitleCard } from "frontend";
import type { ReactNode } from "react";

// The sheet over the whole stage before the first line. It is `absolute
// inset-0`, so every story sits it in the reader's 16:9 stage box. The meta
// line is the group, code and AVG tag already joined by the route; category is
// the translated catalogue name.

function StageBox({ children }: { children: ReactNode }) {
    return <div className="relative aspect-video w-full overflow-hidden rounded-lg bg-black">{children}</div>;
}

const noop = () => {};

// A first visit: nothing saved, so the card only invites a click to begin.
export const FirstVisit = () => (
    <StageBox>
        <TitleCard meta="Hortus de Escapismo · 15-8 · After Operation" title="Decompression Syndrome" category="Main story" savedHalt={null as unknown as number} totalHalts={312} onResume={noop} onStart={noop} />
    </StageBox>
);

// A returning reader: the session stopped on line 148 of 312, so the card
// becomes a two-button choice between resuming and starting over.
export const ResumeSaved = () => (
    <StageBox>
        <TitleCard meta="Hortus de Escapismo · 15-8 · After Operation" title="Decompression Syndrome" category="Main story" savedHalt={147} totalHalts={312} onResume={noop} onStart={noop} />
    </StageBox>
);

// The game's first story: a short title and a shorter meta line, so the card
// is mostly the stage's black.
export const Prologue = () => (
    <StageBox>
        <TitleCard meta="Episode 00 · Prologue" title="Prologue, Part 1" category="Main story" savedHalt={null as unknown as number} totalHalts={64} onResume={noop} onStart={noop} />
    </StageBox>
);
