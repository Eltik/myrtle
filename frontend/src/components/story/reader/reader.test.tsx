/**
 * The reader as the browser drives it: press Space, get the next line. The
 * engine's own tests walk all 122 halts of the tutorial without throwing, and
 * they still passed while the reader STALLED at halt 6, because the stall was
 * in the reveal handshake between `TextBox` and `StoryReader`, not in the
 * engine. This test presses through the fixture the way a reader does.
 */
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import welcomeJson from "#/lib/story/__fixtures__/main_0_0_welcome_to_guide.json";
import type { StoryScript } from "#/types/generated/StoryScript";
import { StoryReader } from "./StoryReader";

const welcome = welcomeJson as StoryScript;

vi.mock("@tanstack/react-router", () => ({
    Link: ({ children }: { children?: React.ReactNode }) => <a href="/stories">{children}</a>,
    useNavigate: () => () => undefined,
}));

// The chapter dialog is the only child that queries the index; it is closed here.
vi.mock("@tanstack/react-query", () => ({
    useQuery: () => ({ data: undefined }),
    queryOptions: (o: unknown) => o,
}));

/** Every element the reader scrolled into view, in order: the backlog test reads it. */
const scrolledTo: Element[] = [];

/** Every `setMusicDucked` the reader made, in order: the cutscene test reads it. */
const ducked: boolean[] = [];

// jsdom has no Web Audio and no fullscreen; the reader must not need them.
vi.mock("#/lib/story/audio", () => ({
    createStoryAudio: () => ({
        arm: () => false,
        armed: false,
        playMusic: () => undefined,
        stopMusic: () => undefined,
        setCueVolume: () => undefined,
        playSound: () => undefined,
        stopSounds: () => undefined,
        setSfxCueVolume: () => undefined,
        resetSfxLevels: () => undefined,
        snapshot: () => undefined,
        noteHalt: () => undefined,
        setMasterMusic: () => undefined,
        setMasterSfx: () => undefined,
        setMuted: () => undefined,
        setMusicDucked: (d: boolean) => ducked.push(d),
        dispose: () => undefined,
        lastFallback: false,
    }),
}));

function renderReader(script: StoryScript = welcome, props: Partial<React.ComponentProps<typeof StoryReader>> = {}) {
    return render(
        <I18nProvider locale="en" available={[]} messages={{}}>
            <StoryReader script={script} entry={null} groupName="Prologue" category="main" previous={null} next={null} {...props} />
        </I18nProvider>,
    );
}

/** A two-command script: one cutscene, then one line. */
const CLIP = { webmUrl: "/video/main_10/main_10_enter.webm", mp4Url: "/video/main_10/main_10_enter.mp4" };
function cutsceneScript(res = "video/main_10/main_10_enter.mp4"): StoryScript {
    return {
        id: "cut",
        name: "cut",
        groupId: "g",
        wordCount: 0,
        commands: [
            { kind: "video", args: { res }, line: 1 },
            { kind: "text", args: {}, text: "after the clip", line: 2 },
        ],
        assets: { backgrounds: {}, images: {}, characters: {}, music: {}, sounds: {}, avatars: {}, imageSizes: {}, videos: { [res]: CLIP } },
    };
}

/** Advance real-ish time: the frame timeline uses setTimeout, the reveal uses rAF. */
function tick(ms: number) {
    act(() => {
        vi.advanceTimersByTime(ms);
    });
}

function pressSpace() {
    act(() => {
        fireEvent.keyDown(window, { key: " " });
    });
}

function currentLine(): string {
    return screen.queryByTestId("story-line")?.textContent ?? "";
}

/** Space does nothing on a decision, so a walk-through takes the first option. */
function takeFirstOptionIfOffered(): boolean {
    const option = document.querySelector("[data-story-option]");
    if (!(option instanceof HTMLElement)) return false;
    act(() => {
        fireEvent.click(option);
    });
    return true;
}

function haltLabel(): string {
    return screen.getByTestId("story-halt").textContent ?? "";
}

describe("StoryReader", () => {
    beforeEach(() => {
        window.localStorage.clear();
        ducked.length = 0;
        vi.useFakeTimers();
        // jsdom has no rAF loop under fake timers; drive it off the timer queue.
        vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => window.setTimeout(() => cb(performance.now()), 16) as unknown as number);
        vi.stubGlobal("cancelAnimationFrame", (id: number) => window.clearTimeout(id));
        // jsdom has no Web Animations; the dialogs' scroll area asks for them.
        if (!("getAnimations" in Element.prototype)) Object.defineProperty(Element.prototype, "getAnimations", { value: () => [], configurable: true });
        // Nor scrollIntoView; the backlog's anchor scroll is recorded instead.
        scrolledTo.length = 0;
        Object.defineProperty(Element.prototype, "scrollIntoView", {
            value(this: Element) {
                scrolledTo.push(this);
            },
            configurable: true,
        });
    });
    afterEach(() => {
        cleanup();
        vi.useRealTimers();
        vi.unstubAllGlobals();
    });

    it("advances 30 halts on Space: two presses per line, never stalling", () => {
        renderReader();
        // The title card: the first press starts the story.
        pressSpace();
        tick(50);
        expect(haltLabel()).toBe("0");

        // Each line takes at most two presses: one to finish the reveal, one to
        // advance. 90 presses is a generous budget for 30 halts; the stall made
        // halt 6 absorb every press after it.
        let presses = 0;
        while (Number(haltLabel()) < 29 && presses < 90) {
            if (!takeFirstOptionIfOffered()) pressSpace();
            tick(1600);
            presses += 1;
        }
        expect(Number(haltLabel())).toBeGreaterThanOrEqual(29);
    });

    it("crosses the halt 6 to 7 boundary that stalled: [dialog] then [delay] then a shorter line", () => {
        renderReader();
        pressSpace();
        tick(50);
        for (let i = 0; i < 20 && Number(haltLabel()) < 6; i++) {
            pressSpace();
            tick(1600);
        }
        expect(haltLabel()).toBe("6");
        expect(currentLine()).toContain("She still needs you.");
        // Two presses: complete the reveal, then advance across the 1 s delay.
        pressSpace();
        tick(1600);
        pressSpace();
        tick(1600);
        expect(haltLabel()).toBe("7");
        expect(currentLine()).toContain("December 23rd");
    });

    it("renders an empty speaker line as narration with no speaker plate", () => {
        renderReader();
        pressSpace();
        tick(1600);
        // Halt 0 of the tutorial is `[name=""] Ah, it's you.`: no name, so narration.
        expect(currentLine()).toContain("Ah, it's you.");
        expect(screen.queryByTestId("story-speaker")).toBeNull();
        expect(screen.getByTestId("story-line").getAttribute("data-narration")).toBe("true");
    });

    it("substitutes the Doctor's name from settings", () => {
        window.localStorage.setItem("myrtle.story.settings", JSON.stringify({ v: 1, nickname: "Verifier", cps: 400 }));
        renderReader();
        pressSpace();
        tick(50);
        let presses = 0;
        let found = false;
        while (!found && presses < 200) {
            if (currentLine().includes("Verifier")) found = true;
            if (!takeFirstOptionIfOffered()) pressSpace();
            tick(1600);
            presses += 1;
        }
        expect(found).toBe(true);
    });

    it("a cutscene halt draws the video layer, ducks the music, and Skip advances past it", () => {
        // The bare element is what this case is about, so it is CHOSEN rather
        // than assumed: the shipped default is the full player, a lazy module
        // that draws no element of ours, and `simple` is that default's kill
        // switch. Every other assertion here is the one this test always made.
        window.localStorage.setItem("myrtle.story.settings", JSON.stringify({ cutscenePlayer: "simple" }));
        renderReader(cutsceneScript());
        // The title card: the first press starts the story on the cutscene.
        pressSpace();
        tick(50);
        expect(haltLabel()).toBe("0");

        const layer = document.querySelector("[data-story-cutscene]");
        expect(layer).not.toBeNull();
        const video = layer?.querySelector("video");
        expect(video).not.toBeNull();
        // WebM is offered FIRST, mp4 second, and both are served through /api/assets.
        const sources = Array.from(layer?.querySelectorAll("source") ?? []).map((el) => [el.getAttribute("type"), el.getAttribute("src")]);
        expect(sources[0][0]).toBe("video/webm");
        expect(sources[0][1]).toContain("/api/assets/video/main_10/main_10_enter.webm");
        expect(sources[1][0]).toBe("video/mp4");
        expect(layer?.getAttribute("data-story-cutscene-player")).toBe("simple");
        expect(video?.hasAttribute("controls")).toBe(false);
        expect(video?.getAttribute("preload")).toBe("auto");
        // The BGM bus is held down for the length of the clip.
        expect(ducked[ducked.length - 1]).toBe(true);

        const skip = document.querySelector("[data-story-cutscene-skip]");
        expect(skip).not.toBeNull();
        act(() => {
            fireEvent.click(skip as Element);
        });
        tick(1600);
        expect(haltLabel()).toBe("1");
        expect(currentLine()).toContain("after the clip");
        expect(document.querySelector("[data-story-cutscene]")).toBeNull();
        // And the duck is released when the halt moves off the clip.
        expect(ducked[ducked.length - 1]).toBe(false);
    });

    it("?video=0 skips the cutscene entirely: one halt, no layer", () => {
        renderReader(cutsceneScript(), { video: false });
        pressSpace();
        tick(1600);
        expect(document.querySelector("[data-story-cutscene]")).toBeNull();
        expect(haltLabel()).toBe("0");
        expect(currentLine()).toContain("after the clip");
        expect(ducked).toEqual([]);
    });

    it("Skip shows the story summary, and confirming lands on the end card with the story marked read", () => {
        renderReader({ ...welcome, synopsis: "The Doctor wakes.\nAmiya is there." });
        pressSpace();
        tick(1600);
        for (let i = 0; i < 6; i++) {
            if (!takeFirstOptionIfOffered()) pressSpace();
            tick(1600);
        }
        const before = Number(haltLabel());
        expect(before).toBeGreaterThan(0);
        act(() => {
            fireEvent.keyDown(window, { key: "s" });
        });
        const sheet = document.querySelector("[data-story-synopsis]");
        expect(sheet?.textContent).toContain("The Doctor wakes.");
        expect(sheet?.querySelectorAll("p")).toHaveLength(2);
        // Nothing moved while the sheet is up: no stepping behind it.
        tick(3000);
        expect(Number(haltLabel())).toBe(before);
        act(() => {
            fireEvent.click(document.querySelector("[data-story-skip-confirm]") as Element);
        });
        tick(50);
        expect(document.querySelector('[data-story-card="end"]')).not.toBeNull();
        const saved = JSON.parse(window.localStorage.getItem("myrtle.story.progress") ?? "{}");
        expect(saved.read[welcome.id]).toBeGreaterThan(1);
        expect(saved.pos[welcome.id]).toBeUndefined();
    });

    it("Skip on a story with no summary says so in one line and still skips", () => {
        renderReader();
        pressSpace();
        tick(1600);
        act(() => {
            fireEvent.keyDown(window, { key: "s" });
        });
        expect(document.querySelector("[data-story-synopsis-none]")).not.toBeNull();
        act(() => {
            fireEvent.click(document.querySelector("[data-story-skip-confirm]") as Element);
        });
        tick(50);
        expect(document.querySelector('[data-story-card="end"]')).not.toBeNull();
    });

    it("the log keeps the rows past a jump back, and the saved position carries the reach", () => {
        renderReader();
        pressSpace();
        tick(1600);
        while (Number(haltLabel()) < 12) {
            if (!takeFirstOptionIfOffered()) pressSpace();
            tick(1600);
        }
        const reach = Number(haltLabel());
        for (let i = 0; i < 5; i++) {
            act(() => {
                fireEvent.keyDown(window, { key: "ArrowLeft" });
            });
            tick(50);
        }
        const at = Number(haltLabel());
        expect(at).toBe(reach - 5);
        const saved = JSON.parse(window.localStorage.getItem("myrtle.story.progress") ?? "{}");
        expect(saved.pos[welcome.id].halt).toBe(at);
        expect(saved.pos[welcome.id].reach).toBe(reach);
        act(() => {
            fireEvent.keyDown(window, { key: "l" });
        });
        tick(50);
        const ahead = [...document.querySelectorAll('[data-story-backlog-row][data-story-ahead="true"]')].map((el) => Number(el.getAttribute("data-story-halt-index")));
        expect(ahead.length).toBeGreaterThan(0);
        expect(Math.max(...ahead)).toBe(reach);
        const current = document.querySelector('[data-story-backlog-row][data-story-current="true"]');
        expect(current?.getAttribute("data-story-halt-index")).toBe(String(at));
        // Opening scrolled to the CURRENT row, not the top of the story.
        tick(50);
        expect(scrolledTo.at(-1)?.querySelector("[data-story-backlog-row]")).toBe(current);

        // A reload: the resume rebuilds the log out to the saved reach.
        cleanup();
        renderReader();
        tick(50);
        pressSpace();
        tick(50);
        expect(Number(haltLabel())).toBe(at);
        act(() => {
            fireEvent.keyDown(window, { key: "l" });
        });
        tick(50);
        const resumed = [...document.querySelectorAll('[data-story-backlog-row][data-story-ahead="true"]')].map((el) => Number(el.getAttribute("data-story-halt-index")));
        expect(resumed).toEqual(ahead);
    });
});
