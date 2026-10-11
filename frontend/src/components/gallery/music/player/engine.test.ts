/**
 * The engine driven through its public surface in jsdom. Media elements are
 * stubbed on the prototype (jsdom implements none of play, pause or load);
 * the story engine, the API helpers and the account library are mocked, so
 * nothing here touches a network or an AudioContext.
 */
import { renderHook } from "@testing-library/react";
import type React from "react";
import { createElement } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { I18nProvider } from "#/lib/i18n/context";
import type { ICatalog, ITrack } from "../impl/model";

const channel = {
    arm: vi.fn(() => true),
    playMusic: vi.fn(),
    playMusicAt: vi.fn(),
    stopMusic: vi.fn(),
    setMasterMusic: vi.fn(),
    setMuted: vi.fn(),
    dispose: vi.fn(),
    musicPosition: 0,
    musicIntroSeconds: 0,
    musicLength: 0,
    lastFallback: false,
};

vi.mock("#/lib/story/audio", () => ({ createStoryAudio: () => channel }));
vi.mock("#/components/operators/detail/impl/assets", () => ({ asset: (path: string) => `ASSET${path}` }));
vi.mock("#/lib/api/gamedata", () => ({ isGamedataServer: () => false }));
vi.mock("#/lib/api/music", () => ({ sirenStreamUrl: (cid: string) => `/stream/${cid}`, sirenLyricsUrl: (cid: string) => `/lyrics/${cid}` }));
vi.mock("../impl/library/store", () => ({ logPlay: vi.fn(async () => 1), updatePlay: vi.fn(), toggleLike: vi.fn(() => true) }));

const engine = await import("./engine");

function siren(cid: string): ITrack {
    return { id: `siren:${cid}`, kind: "siren", key: cid, title: `Song ${cid}`, artists: ["House"], albumId: "siren:a1", albumName: "Album", coverUrl: null, instrumental: false, hasLyrics: false, trackOrder: 1 };
}

function game(clip: string): ITrack {
    return {
        id: `game:${clip}`,
        kind: "game",
        key: clip,
        title: clip,
        artists: [],
        albumId: null,
        albumName: null,
        coverUrl: null,
        instrumental: false,
        hasLyrics: false,
        trackOrder: 0,
        game: { id: clip, kinds: ["battle"], banks: [], places: [], loopUrl: `/audio/${clip}_loop.ogg`, introUrl: `/audio/${clip}_intro.ogg` },
    } as ITrack;
}

function catalogOf(tracks: ITrack[]): ICatalog {
    return {
        tracks,
        byId: new Map(tracks.map((t) => [t.id, t])),
        sirenTracks: tracks.filter((t) => t.kind === "siren"),
        gameTracks: tracks.filter((t) => t.kind === "game"),
        albums: [],
        albumById: new Map(),
        tracksByAlbum: new Map(),
        houseArtist: "House",
        artists: [],
        sirenReady: true,
        gameReady: true,
        siren: null,
        game: null,
    };
}

const A = siren("a");
const B = siren("b");
const C = siren("c");

beforeEach(() => {
    window.localStorage.clear();
    vi.spyOn(HTMLMediaElement.prototype, "play").mockImplementation(function (this: HTMLMediaElement) {
        this.dispatchEvent(new Event("playing"));
        return Promise.resolve();
    });
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {});
    vi.spyOn(HTMLMediaElement.prototype, "load").mockImplementation(() => {});
    channel.playMusic.mockClear();
    channel.playMusicAt.mockClear();
});

afterEach(() => {
    engine.resetPlayerForTests();
    vi.restoreAllMocks();
    window.localStorage.clear();
});

describe("player engine", () => {
    it("playQueue makes the chosen track current and plays it", () => {
        engine.playQueue([A, B, C], 1, { kind: "album", name: "Album", id: "siren:a1" });
        const s = engine.playerStore.state;
        expect(s.current?.id).toBe(B.id);
        expect(s.index).toBe(1);
        expect(s.status).toBe("playing");
        expect(s.source.kind).toBe("album");
        expect(HTMLMediaElement.prototype.play).toHaveBeenCalled();
    });

    it("next and previous move through the queue", () => {
        engine.playQueue([A, B, C], 0, { kind: "none" });
        engine.next();
        expect(engine.playerStore.state.current?.id).toBe(B.id);
        engine.next();
        expect(engine.playerStore.state.current?.id).toBe(C.id);
        engine.previous();
        expect(engine.playerStore.state.current?.id).toBe(B.id);
    });

    it("next at the end with repeat off stops on the last track, and repeat-all wraps", () => {
        engine.playQueue([A, B], 1, { kind: "none" });
        engine.next();
        expect(engine.playerStore.state.current?.id).toBe(B.id);
        expect(engine.playerStore.state.status).toBe("paused");
        engine.setRepeat("all");
        engine.next();
        expect(engine.playerStore.state.current?.id).toBe(A.id);
    });

    it("cycleRepeat goes off, all, one, off", () => {
        const seen = [engine.playerStore.state.repeat];
        for (let i = 0; i < 3; i += 1) {
            engine.cycleRepeat();
            seen.push(engine.playerStore.state.repeat);
        }
        expect(seen).toEqual(["off", "all", "one", "off"]);
    });

    it("setVolume clamps to 0..1, ignores NaN, and persists", () => {
        engine.setVolume(2);
        expect(engine.playerStore.state.volume).toBe(1);
        engine.setVolume(-1);
        expect(engine.playerStore.state.volume).toBe(0);
        engine.setVolume(Number.NaN);
        expect(engine.playerStore.state.volume).toBe(0);
        engine.setVolume(0.25);
        expect(JSON.parse(window.localStorage.getItem("myrtle.music.volume") ?? "{}")).toEqual({ volume: 0.25, muted: false });
    });

    it("removing the current track plays the next, and removing the last one empties the queue", () => {
        engine.playQueue([A, B], 0, { kind: "none" });
        engine.removeFromQueue(0);
        expect(engine.playerStore.state.current?.id).toBe(B.id);
        expect(engine.playerStore.state.queue).toHaveLength(1);
        engine.removeFromQueue(0);
        expect(engine.playerStore.state.queue).toHaveLength(0);
        expect(engine.playerStore.state.status).toBe("idle");
    });

    it("an in-game track plays on the story engine, intro and loop together", () => {
        engine.playQueue([game("m_bat_x")], 0, { kind: "game" });
        expect(channel.playMusic).toHaveBeenCalledWith({ intro: "ASSET/audio/m_bat_x_intro.ogg", loop: "ASSET/audio/m_bat_x_loop.ogg", volume: 1, crossfade: 0 });
        expect(engine.playerStore.state.current?.kind).toBe("game");
    });

    it("saves the session to localStorage after a change", async () => {
        engine.playQueue([A, B, C], 2, { kind: "playlist", name: "Mine", id: "p1" });
        await new Promise((r) => setTimeout(r, 260));
        const saved = JSON.parse(window.localStorage.getItem("myrtle.music.session") ?? "null");
        expect(saved).toMatchObject({ trackIds: [A.id, B.id, C.id], index: 2, shuffle: false, order: [0, 1, 2], repeat: "off", source: { kind: "playlist", name: "Mine", id: "p1" } });
    });

    it("restores a saved session only once the catalogue arrives, without playing, dropping unknown ids", () => {
        window.localStorage.setItem("myrtle.music.session", JSON.stringify({ trackIds: [A.id, "siren:gone", C.id], index: 2, position: 42, shuffle: false, order: [0, 1, 2], repeat: "all", source: { kind: "album", name: "Album" }, savedAt: 1 }));
        const messages = {};
        const wrapper = ({ children }: { children: React.ReactNode }) => createElement(I18nProvider, { locale: "en", available: [{ code: "en", nativeName: "English" }], messages, children });
        const hook = renderHook(() => engine.useMusicPlayerLifetime(), { wrapper });

        expect(engine.playerStore.state.restored).toBe(false);
        expect(engine.playerStore.state.queue).toHaveLength(0);

        engine.setCatalog(catalogOf([A, B, C]));
        const s = engine.playerStore.state;
        expect(s.restored).toBe(true);
        expect(s.queue.map((t) => t.id)).toEqual([A.id, C.id]);
        expect(s.current?.id).toBe(C.id);
        expect(s.index).toBe(1);
        expect(s.repeat).toBe("all");
        expect(s.status).toBe("paused");
        expect(engine.playheadStore.state.position).toBe(42);
        expect(HTMLMediaElement.prototype.play).not.toHaveBeenCalled();
        hook.unmount();
    });

    it("a restored in-game track shows its probed length while paused, intro plus two loop passes", async () => {
        const lengths: Record<string, number> = { "ASSET/audio/m_bat_x_intro.ogg": 9.5, "ASSET/audio/m_bat_x_loop.ogg": 64 };
        class ProbeAudio extends EventTarget {
            preload = "";
            duration = Number.NaN;
            removeAttribute(): void {}
            load(): void {}
            set src(url: string) {
                this.duration = lengths[url] ?? Number.NaN;
                queueMicrotask(() => this.dispatchEvent(new Event("loadedmetadata")));
            }
        }
        vi.stubGlobal("Audio", ProbeAudio);
        const X = game("m_bat_x");
        window.localStorage.setItem("myrtle.music.session", JSON.stringify({ trackIds: [X.id], index: 0, position: 1.8, shuffle: false, order: [0], repeat: "off", source: { kind: "game" }, savedAt: 1 }));
        const wrapper = ({ children }: { children: React.ReactNode }) => createElement(I18nProvider, { locale: "en", available: [{ code: "en", nativeName: "English" }], messages: {}, children });
        const hook = renderHook(() => engine.useMusicPlayerLifetime(), { wrapper });
        engine.setCatalog(catalogOf([X]));
        expect(engine.playheadStore.state.duration).toBe(0);
        await new Promise((r) => setTimeout(r, 0));
        expect(engine.playheadStore.state).toMatchObject({ position: 1.8, duration: 9.5 + 2 * 64, loopAt: 9.5 });
        expect(channel.playMusic).not.toHaveBeenCalled();
        hook.unmount();
        vi.unstubAllGlobals();
    });

    it("the keyboard toggles shuffle and opens the shortcuts with ? even on an empty queue", () => {
        engine.handleKeydown(new KeyboardEvent("keydown", { key: "?" }));
        expect(engine.playerStore.state.shortcutsOpen).toBe(true);
        engine.handleKeydown(new KeyboardEvent("keydown", { key: "s" }));
        expect(engine.playerStore.state.shuffle).toBe(false);
        engine.playQueue([A, B, C], 0, { kind: "none" });
        engine.handleKeydown(new KeyboardEvent("keydown", { key: "s" }));
        expect(engine.playerStore.state.shuffle).toBe(true);
        expect(engine.playerStore.state.order[0]).toBe(0);
    });
});
