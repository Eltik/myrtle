/**
 * The `.m4a` sibling order, and the music status a failed fallback reports.
 *
 * jsdom answers "" to every `canPlayType`, so the order is passed in rather
 * than probed; `prefersM4a()` itself is checked to keep the `.ogg` first there.
 */

import { afterEach, describe, expect, it, vi } from "vitest";
import { createStoryAudio } from "./audio";
import { candidatesFor, createBufferCache, m4aSibling, prefersM4a } from "./audioBuffers";

const LOOP = "https://x/api/assets/audio/audio/sound_beta_2/music/act1break/m_bat2_vectorbreak2/m_bat2_vectorbreak2_loop.ogg";

describe("m4aSibling", () => {
    it("maps a music clip and the anon title theme, and nothing else", () => {
        expect(m4aSibling(LOOP)).toBe(LOOP.replace(/\.ogg$/, ".m4a"));
        expect(m4aSibling("https://x/api/cn/assets/audio/anon/13a1/m_sys_title_intro.ogg?v=2")).toBe("https://x/api/cn/assets/audio/anon/13a1/m_sys_title_intro.m4a?v=2");
        expect(m4aSibling("https://x/api/assets/audio/anon/5335/g_ui_back.ogg")).toBeNull();
        expect(m4aSibling("https://x/api/assets/audio/audio/sound_beta_2/voice/char_002/cn_001.ogg")).toBeNull();
        expect(m4aSibling("https://x/api/assets/audio/audio/sound_beta_2/music/a/x.wav")).toBeNull();
    });

    it("keeps the .ogg first unless the platform lacks Vorbis", () => {
        expect(prefersM4a()).toBe(false);
        expect(candidatesFor(LOOP, false)).toEqual([LOOP, LOOP.replace(/\.ogg$/, ".m4a")]);
        expect(candidatesFor(LOOP, true)).toEqual([LOOP.replace(/\.ogg$/, ".m4a"), LOOP]);
        expect(candidatesFor("https://x/se.ogg", true)).toEqual(["https://x/se.ogg"]);
    });
});

/** A context whose decode takes only `.m4a` bytes, the iOS shape under test. */
function m4aOnlyContext() {
    return {
        decodeAudioData: vi.fn(async (bytes: ArrayBuffer) => {
            if (new TextDecoder().decode(bytes) !== "m4a") throw new Error("EncodingError");
            return { duration: 3 } as AudioBuffer;
        }),
    } as unknown as AudioContext;
}

function fetchServing(files: Record<string, string>) {
    return vi.fn(async (url: string) => {
        const body = files[url];
        return body === undefined ? new Response(null, { status: 404 }) : new Response(body);
    });
}

describe("createBufferCache", () => {
    afterEach(() => vi.unstubAllGlobals());

    it("decodes the .m4a when the .ogg decode rejects, without reporting a failure", async () => {
        const fetchMock = fetchServing({ [LOOP]: "ogg", [LOOP.replace(/\.ogg$/, ".m4a")]: "m4a" });
        vi.stubGlobal("fetch", fetchMock);
        const onFailure = vi.fn();
        const cache = createBufferCache(m4aOnlyContext, onFailure);
        const buf = await cache.get(LOOP);
        expect(buf?.duration).toBe(3);
        expect(fetchMock.mock.calls.map((c) => c[0])).toEqual([LOOP, LOOP.replace(/\.ogg$/, ".m4a")]);
        expect(cache.statusOf(LOOP)).toBe("ok");
        expect(onFailure).not.toHaveBeenCalled();
    });

    it("stops at an .ogg that does not fetch: one request, as before", async () => {
        const fetchMock = fetchServing({});
        vi.stubGlobal("fetch", fetchMock);
        const onFailure = vi.fn();
        const cache = createBufferCache(m4aOnlyContext, onFailure);
        expect(await cache.get(LOOP)).toBeNull();
        expect(fetchMock).toHaveBeenCalledTimes(1);
        expect(cache.statusOf(LOOP)).toBe("http-error");
        expect(onFailure).toHaveBeenCalledTimes(1);
    });

    it("reports the .ogg decode-error when the .m4a is missing too", async () => {
        vi.stubGlobal("fetch", fetchServing({ [LOOP]: "ogg" }));
        const cache = createBufferCache(m4aOnlyContext, () => {});
        expect(await cache.get(LOOP)).toBeNull();
        expect(cache.statusOf(LOOP)).toBe("decode-error");
    });
});

describe("musicStatus", () => {
    afterEach(() => vi.unstubAllGlobals());

    class FakeGain {
        gain = { value: 0, cancelScheduledValues() {}, setValueAtTime() {}, linearRampToValueAtTime() {}, setTargetAtTime() {} };
        connect() {}
    }

    class RejectingContext {
        state = "running";
        currentTime = 0;
        destination = {};
        createGain() {
            return new FakeGain();
        }
        addEventListener() {}
        decodeAudioData(): Promise<AudioBuffer> {
            return Promise.reject(new Error("EncodingError"));
        }
        close() {
            return Promise.resolve();
        }
    }

    async function settle() {
        for (let i = 0; i < 40; i += 1) await new Promise((r) => setTimeout(r, 0));
    }

    function run(play: () => Promise<void>) {
        vi.stubGlobal("AudioContext", RejectingContext);
        vi.stubGlobal("fetch", fetchServing({ [LOOP]: "ogg", [LOOP.replace(/\.ogg$/, ".m4a")]: "m4a" }));
        vi.stubGlobal(
            "Audio",
            class {
                loop = false;
                volume = 1;
                src: string;
                constructor(src: string) {
                    this.src = src;
                }
                play = play;
                pause() {}
            },
        );
        const audio = createStoryAudio();
        audio.playMusic({ loop: LOOP, volume: 1, crossfade: 0 });
        return audio;
    }

    it("reads failed when both decodes lose and the media element refuses to play", async () => {
        const audio = run(() => Promise.reject(new DOMException("not allowed", "NotAllowedError")));
        expect(audio.musicStatus).toBe("loading");
        await settle();
        expect(audio.musicStatus).toBe("failed");
        expect(audio.musicLength).toBe(0);
        audio.dispose();
    });

    it("reads playing only once the fallback element's play() resolves", async () => {
        const audio = run(() => Promise.resolve());
        await settle();
        expect(audio.lastFallback).toBe(true);
        expect(audio.musicStatus).toBe("playing");
        audio.dispose();
    });
});
