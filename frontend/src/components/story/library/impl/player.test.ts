/**
 * The library's music channel, driven without a browser.
 *
 * Everything here is about WHICH cue holds the one channel and what the
 * reader's volume does to it, so the audio module is a spy: the real one
 * wants an AudioContext, a decode and a user gesture, none of which jsdom
 * has. What the spy records is exactly the contract the store depends on,
 * `arm` then `playMusic`, and the levels pushed before it.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_SETTINGS, SETTINGS_KEY } from "#/lib/story/settings";
import type { LibGroup } from "./derive";

interface FakeAudio {
    arm: ReturnType<typeof vi.fn>;
    playMusic: ReturnType<typeof vi.fn>;
    playMusicAt: ReturnType<typeof vi.fn>;
    stopMusic: ReturnType<typeof vi.fn>;
    setMasterMusic: ReturnType<typeof vi.fn>;
    setMuted: ReturnType<typeof vi.fn>;
    dispose: ReturnType<typeof vi.fn>;
    /** The three the timeline is read through, written by the test where a real channel would decode them. */
    musicPosition: number;
    musicIntroSeconds: number;
    musicLength: number;
}

const channels: FakeAudio[] = [];
/** A browser that refuses an AudioContext answers false to `arm`, and every channel opened after this is set does. */
let armAnswer = true;

vi.mock("#/components/operators/detail/impl/assets", () => ({ asset: (path: string) => `ASSET${path}` }));
vi.mock("#/lib/story/audio", () => ({
    createStoryAudio: () => {
        const fake: FakeAudio = { arm: vi.fn(() => armAnswer), playMusic: vi.fn(), playMusicAt: vi.fn(), stopMusic: vi.fn(), setMasterMusic: vi.fn(), setMuted: vi.fn(), dispose: vi.fn(), musicPosition: 0, musicIntroSeconds: 0, musicLength: 0 };
        channels.push(fake);
        return fake;
    },
}));

const player = await import("./player");

function group(id: string, over: Partial<LibGroup> = {}): LibGroup {
    return { id, name: `${id} name`, category: "side", entryType: "ACTIVITY", actType: "ACTIVITY_STORY", startTime: 0, stories: [], music: { loopUrl: `/audio/${id}.ogg` }, ...over };
}

beforeEach(() => {
    window.localStorage.clear();
    channels.length = 0;
    armAnswer = true;
});

afterEach(() => {
    player.dispose();
});

describe("themeTrack", () => {
    it("keys a theme by its group, so two chapters can never collide on one channel", () => {
        expect(themeOf("act13side").key).toBe("act13side::theme");
        expect(themeOf("main_0").key).toBe("main_0::theme");
    });

    it("carries the music table's own title where there is one and null where the table writes a single space", () => {
        expect(themeOf("act13side", { title: "The Grand Knight Territory", loopUrl: "/a.ogg" }).title).toBe("The Grand Knight Territory");
        expect(themeOf("main_0", { title: " ", loopUrl: "/a.ogg" }).title).toBeNull();
    });

    it("returns nothing for the one group whose bank names a loop clip the tree does not hold", () => {
        expect(player.themeTrack(group("act24side", { music: undefined }))).toBeNull();
    });
});

describe("one channel", () => {
    it("replaces the playing cue when a second group's theme is pressed, and never opens a second channel", () => {
        player.play(themeOf("act13side"));
        player.play(themeOf("main_0"));

        expect(channels).toHaveLength(1);
        expect(channels[0]?.playMusic).toHaveBeenCalledTimes(2);
        expect(player.getState().track?.key).toBe("main_0::theme");
        // The replacement crossfades because something was sounding; the first press did not.
        expect(channels[0]?.playMusic.mock.calls[0]?.[0]).toMatchObject({ loop: "ASSET/audio/act13side.ogg", crossfade: 0 });
        expect(channels[0]?.playMusic.mock.calls[1]?.[0]).toMatchObject({ loop: "ASSET/audio/main_0.ogg", crossfade: 0.35 });
    });

    it("pauses the cue it already holds and resumes it, rather than stacking a second copy", () => {
        const track = themeOf("act13side");
        player.toggle(track);
        expect(player.isSounding(player.getState(), track.key)).toBe(true);

        player.toggle(track);
        expect(player.getState().paused).toBe(true);
        expect(player.getState().track?.key).toBe("act13side::theme");
        expect(channels[0]?.stopMusic).toHaveBeenCalledTimes(1);

        player.toggle(track);
        expect(player.getState().paused).toBe(false);
        // The resume goes through the OFFSET arm, because it continues rather than replays.
        expect(channels[0]?.playMusic).toHaveBeenCalledTimes(1);
        expect(channels[0]?.playMusicAt).toHaveBeenCalledTimes(1);
    });

    it("forgets the cue on stop, which is what dismisses the bar", () => {
        player.play(themeOf("act13side"));
        player.stop();
        expect(player.getState().track).toBeNull();
        expect(player.getState().paused).toBe(false);
    });

    it("leaves the channel idle when the browser refuses an AudioContext", () => {
        armAnswer = false;
        player.play(themeOf("act13side"));
        expect(channels).toHaveLength(1);
        expect(channels[0]?.playMusic).not.toHaveBeenCalled();
        expect(player.getState().track).toBeNull();
    });
});

describe("volume", () => {
    it("starts a cue at the level the reader's own settings document holds", () => {
        window.localStorage.setItem(SETTINGS_KEY, JSON.stringify({ ...DEFAULT_SETTINGS, musicVolume: 0.25, muted: true }));
        player.play(themeOf("act13side"));

        expect(channels[0]?.setMasterMusic).toHaveBeenCalledWith(0.25);
        expect(channels[0]?.setMuted).toHaveBeenCalledWith(true);
        expect(player.getState().volume).toBe(0.25);
    });

    it("applies a level change to the live gain and writes it to the reader's document, with no second key", () => {
        window.localStorage.setItem(SETTINGS_KEY, JSON.stringify({ ...DEFAULT_SETTINGS, musicVolume: 0.6, nickname: "Kal'tsit" }));
        player.play(themeOf("act13side"));
        player.setVolume(0);

        expect(channels[0]?.setMasterMusic).toHaveBeenLastCalledWith(0);
        expect(player.getState().volume).toBe(0);

        const keys = Object.keys(window.localStorage).filter((k) => k.startsWith("myrtle.story"));
        expect(keys).toEqual([SETTINGS_KEY]);
        const stored = JSON.parse(window.localStorage.getItem(SETTINGS_KEY) ?? "{}");
        expect(stored.musicVolume).toBe(0);
        // The rest of the reader's document survives the write: this is a merge, not an overwrite.
        expect(stored.nickname).toBe("Kal'tsit");
        expect(stored.sfxVolume).toBe(DEFAULT_SETTINGS.sfxVolume);
    });

    it("clamps to 0..1, because a slider is not the only caller a store has", () => {
        player.setVolume(4);
        expect(player.getState().volume).toBe(1);
        player.setVolume(-1);
        expect(player.getState().volume).toBe(0);
    });

    it("hydrates from the document without sounding anything", () => {
        window.localStorage.setItem(SETTINGS_KEY, JSON.stringify({ ...DEFAULT_SETTINGS, musicVolume: 0.15 }));
        player.hydrate();
        expect(player.getState().volume).toBe(0.15);
        expect(channels).toHaveLength(0);
    });
});

/**
 * WHERE THE CUE IS, which is the half of the state the bar's seek slider
 * reads. The channel's own clock is the spy's three numbers here; what is
 * being tested is that the store reads them at the right moment and hands the
 * right second back to the channel.
 */
describe("position", () => {
    it("samples the channel's timeline only while something sounds", () => {
        const channel = playThenSample("act13side", { position: 41.5, intro: 9.2, length: 186.4 });
        expect(player.getState().positionSeconds).toBeCloseTo(41.5, 6);
        expect(player.getState().introSeconds).toBeCloseTo(9.2, 6);
        expect(player.getState().lengthSeconds).toBeCloseTo(186.4, 6);

        // A paused channel has no timeline to read, and the mark must survive that.
        player.pause();
        channel.musicPosition = 0;
        channel.musicLength = 0;
        player.samplePosition();
        expect(player.getState().positionSeconds).toBeCloseTo(41.5, 6);
        expect(player.getState().lengthSeconds).toBeCloseTo(186.4, 6);
    });

    it("pauses on the second it reached and resumes from there, rather than replaying the intro", () => {
        const channel = playThenSample("act13side", { position: 110.25, intro: 9.2, length: 186.4 });

        player.pause();
        expect(player.getState().paused).toBe(true);
        expect(player.getState().positionSeconds).toBeCloseTo(110.25, 6);

        player.resume();
        expect(channel.playMusicAt).toHaveBeenCalledTimes(1);
        expect(channel.playMusicAt.mock.calls[0]?.[1]).toBeCloseTo(110.25, 6);
        // A seek is the track over itself, so it never crossfades.
        expect(channel.playMusicAt.mock.calls[0]?.[0]).toMatchObject({ crossfade: 0 });
    });

    it("seeks to the second asked for, clamped to the timeline", () => {
        const channel = playThenSample("act13side", { position: 3, intro: 9.2, length: 186.4 });

        player.seek(100);
        expect(channel.playMusicAt).toHaveBeenLastCalledWith(expect.anything(), 100);
        expect(player.getState().positionSeconds).toBe(100);
        // The length survives a seek within one cue: the decode already said what it is.
        expect(player.getState().lengthSeconds).toBeCloseTo(186.4, 6);

        player.seek(1000);
        expect(channel.playMusicAt).toHaveBeenLastCalledWith(expect.anything(), 186.4);
        player.seek(-5);
        expect(channel.playMusicAt).toHaveBeenLastCalledWith(expect.anything(), 0);
    });

    it("moves the mark without sounding when the cue is paused", () => {
        const channel = playThenSample("act13side", { position: 20, intro: 9.2, length: 186.4 });
        player.pause();

        player.seek(60);
        expect(channel.playMusicAt).not.toHaveBeenCalled();
        expect(player.getState().paused).toBe(true);
        expect(player.getState().positionSeconds).toBe(60);

        // The resume then starts where the seek left the mark.
        player.resume();
        expect(channel.playMusicAt).toHaveBeenLastCalledWith(expect.anything(), 60);
    });

    it("forgets the timeline on a stop and on a second group's theme", () => {
        playThenSample("act13side", { position: 41.5, intro: 9.2, length: 186.4 });
        player.stop();
        expect(player.getState()).toMatchObject({ positionSeconds: 0, introSeconds: 0, lengthSeconds: 0 });

        playThenSample("act13side", { position: 41.5, intro: 9.2, length: 186.4 });
        player.play(themeOf("main_0"));
        expect(player.getState()).toMatchObject({ positionSeconds: 0, introSeconds: 0, lengthSeconds: 0 });
    });
});

describe("lifetime", () => {
    it("takes the channel down when the route leaves, so the reader's own music is alone", () => {
        player.play(themeOf("act13side"));
        const channel = channels[0];
        player.dispose();

        expect(channel?.dispose).toHaveBeenCalledTimes(1);
        expect(player.getState().track).toBeNull();
        // The NEXT press opens a fresh channel rather than reviving a disposed one.
        player.play(themeOf("main_0"));
        expect(channels).toHaveLength(2);
    });

    it("tells its subscribers on every state change and stops telling them once they leave", () => {
        const seen = vi.fn();
        const off = player.subscribe(seen);
        player.play(themeOf("act13side"));
        player.pause();
        expect(seen).toHaveBeenCalledTimes(2);

        off();
        player.stop();
        expect(seen).toHaveBeenCalledTimes(2);
    });

    it("renders an idle channel on the server, so the bar is absent from the markup the client hydrates", () => {
        player.play(themeOf("act13side"));
        expect(player.getServerState().track).toBeNull();
    });
});

/** Open a theme, put the channel's clock where the test wants it, and take one sample. */
function playThenSample(id: string, at: { position: number; intro: number; length: number }): FakeAudio {
    player.play(themeOf(id));
    const channel = channels.at(-1);
    if (!channel) throw new Error("no channel");
    channel.musicPosition = at.position;
    channel.musicIntroSeconds = at.intro;
    channel.musicLength = at.length;
    player.samplePosition();
    return channel;
}

function themeOf(id: string, music?: { title?: string; introUrl?: string; loopUrl: string }): NonNullable<ReturnType<typeof player.themeTrack>> {
    const track = player.themeTrack(group(id, music ? { music } : {}));
    if (track === null) throw new Error(`no theme for ${id}`);
    return track;
}
