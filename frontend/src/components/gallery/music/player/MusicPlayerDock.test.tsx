/**
 * The dock's visibility rule and its wiring to the engine: no bar until there
 * is a queue, the bar's buttons drive the engine, and Stop and close takes the
 * bar away again. The engine is real; its outside world is mocked as in
 * engine.test.ts.
 */
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { ITrack } from "../impl/model";

vi.mock("#/lib/story/audio", () => ({
    createStoryAudio: () => ({ arm: () => true, playMusic: () => {}, playMusicAt: () => {}, stopMusic: () => {}, setMasterMusic: () => {}, setMuted: () => {}, dispose: () => {}, musicPosition: 0, musicIntroSeconds: 0, musicLength: 0, lastFallback: false }),
}));
vi.mock("#/components/operators/detail/impl/assets", () => ({ asset: (path: string) => `ASSET${path}` }));
vi.mock("#/lib/api/gamedata", () => ({ isGamedataServer: () => false }));
vi.mock("#/lib/api/music", () => ({ sirenStreamUrl: (cid: string) => `/stream/${cid}`, sirenLyricsUrl: (cid: string) => `/lyrics/${cid}` }));
vi.mock("../impl/library/store", () => ({ logPlay: vi.fn(async () => 1), updatePlay: vi.fn(), toggleLike: vi.fn(() => true), useIsLiked: () => false, createPlaylist: vi.fn(async () => null) }));

const engine = await import("./engine");
const { MusicPlayerDock } = await import("./MusicPlayerDock");
const { messages } = await import("./Player.messages");
const { messages: shortcutMessages } = await import("./Shortcuts.messages");

const catalog = Object.fromEntries([...Object.entries(messages), ...Object.entries(shortcutMessages)].map(([key, value]) => [`gallery.${key}`, value.text]));

function siren(cid: string): ITrack {
    return { id: `siren:${cid}`, kind: "siren", key: cid, title: `Song ${cid}`, artists: ["House", "Guest"], albumId: "siren:a1", albumName: "Album", coverUrl: null, instrumental: true, hasLyrics: false, trackOrder: 1 };
}

function mount(): void {
    render(
        <I18nProvider available={[{ code: "en", nativeName: "English" }]} locale="en" messages={catalog}>
            <MusicPlayerDock />
        </I18nProvider>,
    );
}

beforeEach(() => {
    window.localStorage.clear();
    vi.spyOn(HTMLMediaElement.prototype, "play").mockImplementation(() => Promise.resolve());
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
    vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
});

afterEach(() => {
    cleanup();
    engine.resetPlayerForTests();
    vi.restoreAllMocks();
});

describe("MusicPlayerDock", () => {
    it("has no bar until a queue exists, and Stop and close removes it", () => {
        mount();
        expect(screen.queryByRole("region", { name: "Music player" })).toBeNull();

        act(() => engine.playQueue([siren("a"), siren("b")], 0, { kind: "album", name: "Album" }));
        expect(screen.getByRole("region", { name: "Music player" })).toBeTruthy();
        expect(screen.getByText("Song a")).toBeTruthy();
        expect(screen.getByText("Playing from album Album")).toBeTruthy();

        fireEvent.click(screen.getByRole("button", { name: "Next" }));
        expect(engine.playerStore.state.current?.key).toBe("b");

        fireEvent.click(screen.getByRole("button", { name: "Repeat: off" }));
        expect(engine.playerStore.state.repeat).toBe("all");

        fireEvent.click(screen.getByRole("button", { name: "Stop and close" }));
        expect(screen.queryByRole("region", { name: "Music player" })).toBeNull();
        expect(engine.playerStore.state.queue).toHaveLength(0);
    });
});
