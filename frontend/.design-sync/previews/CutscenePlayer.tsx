import { CutscenePlayer } from "frontend";
import { type ReactNode, useRef } from "react";

// The full cutscene player (Vidstack's default video layout, themed to the
// reader), lazy-loaded by `Cutscene` when Settings picks it. It fills its
// parent, so the story gives it the reader's 16:9 box. No clip is loaded: a
// streaming video keeps the capture page from settling, so the card shows the
// player's own layout over the black frame.

const SOURCES: { src: string; type: "video/webm" | "video/mp4" }[] = [];
const noop = () => {};

function StageBox({ children }: { children: ReactNode }) {
    return <div className="relative aspect-video w-full overflow-hidden rounded-lg bg-black">{children}</div>;
}

function Player({ muted }: { muted: boolean }) {
    const startRef = useRef<() => void>(null);
    const toggleRef = useRef<() => void>(null);
    return (
        <StageBox>
            <CutscenePlayer sources={SOURCES} label="Cutscene" volume={0.8} muted={muted} onEnded={noop} onVolume={noop} onBlocked={noop} startRef={startRef as never} toggleRef={toggleRef as never} />
        </StageBox>
    );
}

// Before the clip's first frame: the player's buffering ring on black. (A
// real clip never settles in the capture harness: page.goto times out.)
export const Loading = () => <Player muted />;
