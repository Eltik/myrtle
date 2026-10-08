// Design-bundle stand-in for `media-captions` (Vidstack's subtitle parser and
// renderer, ~62 KB): the cutscene player loads no text tracks, and the bundle must
// stay under the 12 MiB upload cap. Vidstack imports it lazily; the default layout's
// captions component instantiates `CaptionsRenderer` on mount, so that is a no-op
// object, and the parsers report no cues.
const noop = () => {};

export class CaptionsRenderer {
    currentTime = 0;
    dir: string | undefined;
    constructor(_overlay?: unknown) {}
    changeTrack = noop;
    addCue = noop;
    removeCue = noop;
    update = noop;
    reset = noop;
    detach = noop;
    destroy = noop;
}

const empty = () => Promise.resolve({ cues: [], regions: [], errors: [] });
export const parseText = empty;
export const parseResponse = empty;
export const parseByteStream = empty;
export const renderVTTCueString = () => "";
export const tokenizeVTTCue = () => [];
type Ctor = new (...args: never[]) => object;
const g = globalThis as unknown as Record<string, Ctor | undefined>;
export const VTTCue: Ctor = g.VTTCue ?? class {};
export const VTTRegion: Ctor = g.VTTRegion ?? class {};
