/**
 * The bar's VISIBILITY RULE, which is the whole of its layout contract: there
 * is no bar until the library's channel holds a cue, and stopping is what
 * takes it away again. Everything else on it is a label over the store, so the
 * store is driven directly here and the bar is only read.
 */
import { cleanup, render, screen } from "@testing-library/react";
import { act } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import { DEFAULT_SETTINGS, SETTINGS_KEY } from "#/lib/story/settings";

/** The one channel the mock hands out, so a test can put its clock where it likes. */
const channel = { arm: () => true, playMusic: () => {}, playMusicAt: () => {}, stopMusic: () => {}, setMasterMusic: () => {}, setMuted: () => {}, dispose: () => {}, musicPosition: 0, musicIntroSeconds: 0, musicLength: 0 };

vi.mock("#/components/operators/detail/impl/assets", () => ({ asset: (path: string) => `ASSET${path}` }));
vi.mock("#/lib/story/audio", () => ({ createStoryAudio: () => channel }));

const player = await import("./player");
const { NowPlayingBar } = await import("./NowPlayingBar");
const { messages } = await import("./NowPlayingBar.messages");
const { messages: archiveMessages } = await import("./Archive.messages");

const catalog = Object.fromEntries([...Object.entries(messages), ...Object.entries(archiveMessages)].map(([key, value]) => [`story.${key}`, value.text]));

const nearLight = { key: "act13side::theme", groupId: "act13side", groupName: "Near Light", title: "The Grand Knight Territory", cue: { loop: "/audio/act13side.ogg" } };
const chapterOne = { key: "main_1::theme", groupId: "main_1", groupName: "Chapter 1", title: null, cue: { loop: "/audio/main_1.ogg" } };

function mount(): void {
    render(
        <I18nProvider locale="en" available={[{ code: "en", nativeName: "English" }]} messages={catalog}>
            <NowPlayingBar />
        </I18nProvider>,
    );
}

beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify({ ...DEFAULT_SETTINGS, musicVolume: 0.6 }));
    channel.musicPosition = 0;
    channel.musicIntroSeconds = 0;
    channel.musicLength = 0;
    player.hydrate();
});

afterEach(() => {
    cleanup();
    player.dispose();
});

describe("NowPlayingBar", () => {
    it("is absent until something is playing and gone again once it is stopped", () => {
        mount();
        expect(screen.queryByRole("region", { name: "Now playing" })).toBeNull();

        act(() => player.play(nearLight));
        expect(screen.getByRole("region", { name: "Now playing" })).toBeTruthy();

        act(() => player.stop());
        expect(screen.queryByRole("region", { name: "Now playing" })).toBeNull();
    });

    it("survives a pause, because a paused cue is still the one the channel holds", () => {
        mount();
        act(() => player.play(nearLight));
        act(() => player.pause());

        expect(screen.getByRole("region", { name: "Now playing" })).toBeTruthy();
        expect(screen.getByRole("button", { name: "Resume The Grand Knight Territory" })).toBeTruthy();
        expect(screen.getByText("Paused")).toBeTruthy();
    });

    it("names the track and the chapter it belongs to, and says what a theme with no title of its own is", () => {
        mount();
        act(() => player.play(nearLight));
        expect(screen.getByText("The Grand Knight Territory")).toBeTruthy();
        expect(screen.getByText("Near Light")).toBeTruthy();
        expect(screen.getByRole("button", { name: "Pause The Grand Knight Territory" })).toBeTruthy();

        act(() => player.play(chapterOne));
        expect(screen.getByText("Chapter theme")).toBeTruthy();
        expect(screen.getByText("Chapter 1")).toBeTruthy();
        expect(screen.getByRole("button", { name: "Stop Chapter theme" })).toBeTruthy();
    });

    it("carries the reader's music volume on one slider, and follows it when the store moves", () => {
        mount();
        act(() => player.play(nearLight));

        // By LABEL, not by role and name: the primitive parks its thumb at
        // `visibility: hidden` until it has measured the track, which never
        // happens under jsdom, and a hidden node computes no accessible name
        // there. The `<input type="range">` inside it is the control a reader
        // operates and `aria-labelledby` is what names it.
        const slider = screen.getByLabelText("Music volume", { selector: "input" });
        expect(slider.getAttribute("type")).toBe("range");
        expect(slider.getAttribute("aria-valuenow")).toBe("0.6");
        expect(screen.getByText("60%")).toBeTruthy();

        act(() => player.setVolume(0));
        expect(screen.getByLabelText("Music volume", { selector: "input" }).getAttribute("aria-valuenow")).toBe("0");
        expect(screen.getByText("0%")).toBeTruthy();
        expect(JSON.parse(window.localStorage.getItem(SETTINGS_KEY) ?? "{}").musicVolume).toBe(0);
    });

    it("reads its position as a TIME, not as a number of seconds on a scale of 186.4", () => {
        mount();
        act(() => player.play(nearLight));
        // Near Light: a 9.2 s intro over a 177.2 s loop, 110.25 s in.
        channel.musicPosition = 110.25;
        channel.musicIntroSeconds = 9.2;
        channel.musicLength = 186.4;
        act(() => player.samplePosition());

        const seek = screen.getByLabelText("Position in track", { selector: "input" });
        expect(seek.getAttribute("type")).toBe("range");
        expect(seek.getAttribute("aria-valuetext")).toBe("1:50 of 3:06");
        expect(seek.getAttribute("max")).toBe("186.4");
        // Five seconds an arrow, which is the grid the keyboard walks.
        expect(seek.getAttribute("step")).toBe("5");
        expect(screen.getByText("1:50")).toBeTruthy();
        expect(screen.getByText("3:06")).toBeTruthy();

        // A cue with no timeline yet shows 0:00 of 0:00 rather than a slider that lies.
        act(() => player.play(chapterOne));
        expect(screen.getByLabelText("Position in track", { selector: "input" }).getAttribute("aria-valuetext")).toBe("0:00 of 0:00");
    });
});
