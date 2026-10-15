import { Cutscene } from "frontend";
import type { ReactNode } from "react";

// A cutscene over the stage: the game's clip letterboxed in black, a frosted
// "Skip video" pill in the corner. Three players, from Settings: `simple` (a
// bare video, the pill at the bottom), `native` (the browser's own controls,
// pill at the top) and `vidstack` (the full player with its own control bar,
// pill clear of it). It is `absolute inset-0`, so each story sits it in the
// reader's 16:9 box. The clip itself is left out (`sources={}`): a streaming
// video keeps the capture page from settling, so the cards show each player's
// chrome over the black letterbox, which is also the first frame a reader sees.

const SOURCES = {};
const noop = () => {};

function StageBox({ children }: { children: ReactNode }) {
    return <div className="relative aspect-video w-full overflow-hidden rounded-lg bg-black">{children}</div>;
}

const Clip = ({ player }: { player: "simple" | "native" | "vidstack" }) => (
    <StageBox>
        <Cutscene sources={SOURCES} label="Cutscene" skipLabel="Skip video" playLabel="Play" onSkip={noop} onEnded={noop} volume={0.8} muted player={player} />
    </StageBox>
);

// The default player: the clip and the Skip pill, nothing else.
export const Simple = () => <Clip player="simple" />;

// The browser's native controls along the bottom, the pill moved to the top.
export const Native = () => <Clip player="native" />;

// The full player with its control bar; the pill sits below its top strip.
export const Vidstack = () => <Clip player="vidstack" />;
