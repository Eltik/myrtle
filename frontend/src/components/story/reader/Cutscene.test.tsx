/**
 * The three cutscene layers, at the level the reader chooses between them.
 *
 * Two claims are worth a test and the rest is Vidstack's own. First, `native`
 * is the browser's control bar over OUR element, so the attribute has to be on
 * the element and a level dragged on that bar has to reach the settings. Second,
 * the full player is LAZY: it is a separate module and nothing may pull it in
 * until a reader who asked for it meets a clip, which is the whole reason the
 * two files are split.
 */
import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { VideoSources } from "#/types/generated/VideoSources";
import { Cutscene, cutsceneSources, volumeWriteback } from "./Cutscene";

/** Every time the lazy module was actually evaluated: the import is what this counts. */
const loads = vi.hoisted(() => ({ count: 0 }));

vi.mock("./CutscenePlayer", () => {
    loads.count += 1;
    return {
        default: ({ label }: { label: string }) => <div data-testid="vidstack-player" data-label={label} />,
    };
});

const SOURCES: VideoSources = { webmUrl: "/video/a.webm", mp4Url: "/video/a.mp4" };

function renderCutscene(player: "simple" | "native" | "vidstack", onVolumeChange?: (v: number, m: boolean) => void) {
    return render(<Cutscene sources={SOURCES} label="Cutscene" skipLabel="Skip video" playLabel="Play" onSkip={() => undefined} onEnded={() => undefined} volume={0.6} muted={false} player={player} onVolumeChange={onVolumeChange} />);
}

function video(): HTMLVideoElement {
    const el = document.querySelector("video");
    if (!(el instanceof HTMLVideoElement)) throw new Error("no video element");
    return el;
}

describe("the cutscene layers", () => {
    afterEach(() => {
        cleanup();
        loads.count = 0;
    });

    it("simple draws the element the reader shipped: no controls, one Skip pill", () => {
        renderCutscene("simple");
        expect(video().hasAttribute("controls")).toBe(false);
        expect(document.querySelector("[data-story-cutscene-skip]")).not.toBeNull();
    });

    it("native puts the browser's own controls on that same element, download suppressed", () => {
        renderCutscene("native");
        expect(video().controls).toBe(true);
        expect(video().getAttribute("controlslist")).toBe("nodownload");
        // Still one element with the same two sources: `native` is an attribute,
        // not a second player.
        expect(document.querySelectorAll("video source")).toHaveLength(2);
    });

    // Dragging the browser's own volume slider IS this: the element's level
    // changes and it fires `volumechange`, which jsdom models faithfully, so
    // the write here is the one a real drag makes.
    it("a volumechange on the native element writes the level and the mute back", () => {
        const wrote: [number, boolean][] = [];
        renderCutscene("native", (v, m) => wrote.push([v, m]));
        video().volume = 0.3;
        expect(wrote).toEqual([[0.3, false]]);
    });

    it("the browser's own mute button writes the mute flag back at an unchanged level", () => {
        const wrote: [number, boolean][] = [];
        renderCutscene("native", (v, m) => wrote.push([v, m]));
        video().muted = true;
        expect(wrote).toEqual([[0.6, true]]);
    });

    // The feedback loop, and the reason for the epsilon: the layer pushes the
    // settings level onto the element, the element reports it straight back,
    // and a writeback there would ping-pong with the prop that caused it.
    it("a volumechange that only repeats the level the layer was GIVEN writes nothing", () => {
        const wrote: [number, boolean][] = [];
        renderCutscene("native", (v, m) => wrote.push([v, m]));
        const el = video();
        el.volume = 0.6;
        fireEvent.volumeChange(el);
        expect(wrote).toEqual([]);
    });

    // The point of the split. `simple` and `native` are the same file the
    // reader already has; nothing of the full player is fetched for them.
    it("neither simple nor native loads the full player module", () => {
        renderCutscene("simple");
        cleanup();
        renderCutscene("native");
        expect(loads.count).toBe(0);
        expect(document.querySelector("[data-testid=vidstack-player]")).toBeNull();
    });

    it("the full player module is loaded only once a vidstack cutscene renders", async () => {
        expect(loads.count).toBe(0);
        renderCutscene("vidstack");
        await waitFor(() => expect(document.querySelector("[data-testid=vidstack-player]")).not.toBeNull());
        expect(loads.count).toBe(1);
        // The pill stays OURS: it is a sibling of the player, never inside its
        // layout, so it survives whatever the layout draws.
        expect(document.querySelector("[data-story-cutscene-skip]")).not.toBeNull();
        expect(document.querySelector("video")).toBeNull();
    });
});

describe("the cutscene volume writeback", () => {
    it("writes a level that moved further than the epsilon and drops one that did not", () => {
        expect(volumeWriteback({ volume: 0.7, muted: false }, { volume: 0.6, muted: false })).toEqual({ volume: 0.7, muted: false });
        expect(volumeWriteback({ volume: 0.604, muted: false }, { volume: 0.6, muted: false })).toBeNull();
        expect(volumeWriteback({ volume: 0.6, muted: false }, { volume: 0.6, muted: false })).toBeNull();
    });

    it("writes a mute FLIP at any level, because muting is not a level change", () => {
        expect(volumeWriteback({ volume: 0.6, muted: true }, { volume: 0.6, muted: false })).toEqual({ volume: 0.6, muted: true });
        expect(volumeWriteback({ volume: 0.6, muted: false }, { volume: 0.6, muted: true })).toEqual({ volume: 0.6, muted: false });
    });

    // The coercion trap: a player with no level reports one that is not a
    // number, and 0 is SILENCE that would then persist into the soundtrack.
    it("drops a level that is not a finite number instead of persisting silence", () => {
        expect(volumeWriteback({ volume: Number.NaN, muted: false }, { volume: 0.6, muted: false })).toBeNull();
        expect(volumeWriteback({ volume: undefined as unknown as number, muted: false }, { volume: 0.6, muted: false })).toBeNull();
    });

    it("clamps a level a player reports outside 0..1", () => {
        expect(volumeWriteback({ volume: 2, muted: false }, { volume: 0.6, muted: false })).toEqual({ volume: 1, muted: false });
        expect(volumeWriteback({ volume: -1, muted: false }, { volume: 0.6, muted: false })).toEqual({ volume: 0, muted: false });
    });
});

describe("the cutscene source list", () => {
    it("offers WebM first and MP4 second, which is the order that keeps Safari off the VP9 file", () => {
        expect(cutsceneSources(SOURCES).map((s) => s.type)).toEqual(["video/webm", "video/mp4"]);
    });

    it("drops a transcode the wire did not carry rather than offering an empty src", () => {
        expect(cutsceneSources({ webmUrl: undefined, mp4Url: "/video/a.mp4" })).toHaveLength(1);
        expect(cutsceneSources({ webmUrl: undefined, mp4Url: undefined })).toEqual([]);
    });
});
