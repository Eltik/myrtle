/**
 * Two things only a fake AudioContext can show.
 *
 * The gesture gate, which headless Chromium cannot show at all: it runs with
 * the autoplay policy disabled, so an AudioContext created off a gesture
 * reports "running" there and "suspended" in the user's own Chrome. The fake
 * below models the real refusal: `resume()` returns a promise that never
 * settles until a gesture has been seen, and the clock stays at 0.
 *
 * And the SFX ledger's five defects, each of which is a GAIN or a START TIME
 * that a browser run can only see as "that sound was silent". The fake records
 * every ramp target, every scheduled start and every stop, and drives the
 * decode by hand so a test can put a command INSIDE the decode window.
 */

import { afterEach, describe, expect, it, vi } from "vitest";
import { createStoryAudio } from "./audio";

class FakeParam {
    value = 1;
    /** The last `linearRampToValueAtTime` target, which is what a fade asks for. */
    target: number | null = null;
    rampEnd: number | null = null;
    cancelScheduledValues() {}
    setValueAtTime(v: number) {
        this.value = v;
    }
    linearRampToValueAtTime(v: number, t: number) {
        this.target = v;
        this.rampEnd = t;
    }
    setTargetAtTime(v: number) {
        // Only a running clock can actually move the value.
        if (ctxUnderTest?.state === "running") this.value = v;
    }
}

class FakeGain {
    gain = new FakeParam();
    connect() {}
    disconnect() {}
}

class FakeSource {
    buffer: unknown = null;
    loop = false;
    onended: (() => void) | null = null;
    startedAt: number | null = null;
    /** `start(when, offset)`: where in the BUFFER playback begins, which is what a seek moves. */
    offset: number | null = null;
    stoppedAt: number | null = null;
    connect() {}
    start(t?: number, offset?: number) {
        this.startedAt = t ?? ctxUnderTest?.currentTime ?? 0;
        this.offset = offset ?? null;
    }
    stop(t?: number) {
        this.stoppedAt = t ?? ctxUnderTest?.currentTime ?? 0;
    }
}

interface FakeBuffer {
    numberOfChannels: number;
    sampleRate: number;
    duration: number;
}

let ctxUnderTest: FakeAudioContext | null = null;

class FakeAudioContext {
    state: "suspended" | "running" | "closed" = "suspended";
    currentTime = 0;
    destination = {};
    resumeCalls = 0;
    gains: FakeGain[] = [];
    sources: FakeSource[] = [];
    /** Decodes queued by url, resolved by the test so a command can land mid-decode. */
    pending: (() => void)[] = [];
    /** Set once a real gesture has reached the page, as Chrome's policy does. */
    static gestureSeen = false;
    /** A test that cares about two DIFFERENT durations reads them off the bytes; everything else decodes to 3 s. */
    static durations: ((bytes: ArrayBuffer) => number) | null = null;
    private listeners = new Set<() => void>();

    constructor() {
        ctxUnderTest = this;
    }

    addEventListener(_type: string, fn: () => void) {
        this.listeners.add(fn);
    }

    createGain() {
        const g = new FakeGain();
        this.gains.push(g);
        return g;
    }

    createBufferSource() {
        const s = new FakeSource();
        this.sources.push(s);
        return s;
    }

    decodeAudioData(bytes: ArrayBuffer): Promise<FakeBuffer> {
        const duration = FakeAudioContext.durations?.(bytes) ?? 3;
        return new Promise((resolve) => {
            this.pending.push(() => resolve({ numberOfChannels: 2, sampleRate: 44100, duration }));
        });
    }

    /**
     * Drain the fetch microtasks until `decodeAudioData` has been reached, let
     * every queued decode land, then drain again.
     */
    async settle() {
        for (let round = 0; round < 6; round += 1) {
            for (let i = 0; i < 8; i += 1) await Promise.resolve();
            const queued = this.pending;
            this.pending = [];
            for (const fn of queued) fn();
        }
        for (let i = 0; i < 8; i += 1) await Promise.resolve();
    }

    resume(): Promise<void> {
        this.resumeCalls += 1;
        if (!FakeAudioContext.gestureSeen) {
            // Chrome neither resolves nor rejects this one.
            return new Promise<void>(() => undefined);
        }
        this.state = "running";
        this.currentTime = 1;
        for (const fn of this.listeners) fn();
        return Promise.resolve();
    }

    close(): Promise<void> {
        this.state = "closed";
        return Promise.resolve();
    }
}

function install() {
    FakeAudioContext.gestureSeen = false;
    FakeAudioContext.durations = null;
    ctxUnderTest = null;
    (window as unknown as { AudioContext: unknown }).AudioContext = FakeAudioContext;
}

/** A context that is already running, with fetch stubbed to a byte blob. */
function installRunning() {
    install();
    FakeAudioContext.gestureSeen = true;
    vi.stubGlobal("fetch", async () => ({ ok: true, arrayBuffer: async () => new ArrayBuffer(8) }));
}

/**
 * A cue with the shape of a real chapter theme: Near Light's 9.2 s intro over
 * its 177.2 s loop. The two durations have to DIFFER for any of the seek
 * arithmetic to be visible, so the fetch hands back a buffer whose length is
 * the duration in tenths and the decode reads it back off the bytes.
 */
function installCue() {
    install();
    FakeAudioContext.gestureSeen = true;
    FakeAudioContext.durations = (bytes) => bytes.byteLength / 10;
    vi.stubGlobal("fetch", async (url: string) => ({ ok: true, arrayBuffer: async () => new ArrayBuffer(url.includes("intro") ? 92 : 1772) }));
}

const CUE = { intro: "/m/intro.ogg", loop: "/m/loop.ogg", volume: 1, crossfade: 0 };
/** `startAt` is `ctx.currentTime + 0.05`, and a running fake context starts its clock at 1. */
const START_AT = 1.05;

afterEach(() => {
    (window as unknown as { AudioContext?: unknown }).AudioContext = undefined;
    ctxUnderTest = null;
    vi.unstubAllGlobals();
});

describe("story audio gesture gate", () => {
    it("stays suspended when armed off a gesture, then heals on the first click anywhere", async () => {
        install();
        const audio = createStoryAudio();

        // The `?halt=N` deep link arms on mount, with no gesture behind it.
        expect(audio.arm()).toBe(true);
        expect(ctxUnderTest?.state).toBe("suspended");
        expect(ctxUnderTest?.currentTime).toBe(0);

        // A gesture that never reaches `arm()`: the toolbar, a slider, a key.
        FakeAudioContext.gestureSeen = true;
        window.dispatchEvent(new Event("pointerdown"));
        await Promise.resolve();
        await Promise.resolve();

        expect(ctxUnderTest?.state).toBe("running");
        expect(audio.snapshot().contextState).toBe("running");
        audio.dispose();
    });

    it("re-applies the bus volumes on the live clock, so the bus is not left at 1", async () => {
        install();
        const audio = createStoryAudio();
        audio.setMasterMusic(0.4);
        audio.setMasterSfx(0.8);
        audio.arm();

        // Scheduled against a frozen clock: the bus still reads its default.
        expect(audio.snapshot().volumes.musicBusGain).toBe(1);

        FakeAudioContext.gestureSeen = true;
        window.dispatchEvent(new Event("keydown"));
        await Promise.resolve();
        await Promise.resolve();

        const v = audio.snapshot().volumes;
        expect(v.musicBusGain).toBe(0.4);
        expect(v.sfxBusGain).toBe(0.8);
        audio.dispose();
    });

    it("takes its listeners off once the context runs, and on dispose", async () => {
        install();
        const audio = createStoryAudio();
        audio.arm();
        FakeAudioContext.gestureSeen = true;
        window.dispatchEvent(new Event("pointerdown"));
        await Promise.resolve();
        await Promise.resolve();

        const after = ctxUnderTest?.resumeCalls ?? 0;
        window.dispatchEvent(new Event("pointerdown"));
        expect(ctxUnderTest?.resumeCalls).toBe(after);

        audio.dispose();
        window.dispatchEvent(new Event("pointerdown"));
        expect(ctxUnderTest?.resumeCalls).toBe(after);
    });
});

describe("a channel is a bus, not a tag", () => {
    it("raises a voice opened at volume=0 when [soundvolume] levels its channel", async () => {
        installRunning();
        const audio = createStoryAudio();
        audio.arm();
        // main_15-08_end: [playsound(key=$d_avg_labamb, loop=true, channel=bgs, volume=0)]
        audio.playSound("/a/labamb.ogg", { volume: 0, loop: true, channel: "bgs" });
        await ctxUnderTest?.settle();
        const voice = ctxUnderTest?.gains.at(-1);
        expect(voice?.gain.value).toBe(0);

        // ... then [soundvolume(volume=0.7, channel=bgs, fadetime=2)].
        audio.setSfxCueVolume("bgs", 0.7, 2);
        // Under the old multiply the target was 0 * 0.7 and the ambience was
        // never heard: 726 of the 736 EN `volume=0` playsounds are this shape.
        expect(voice?.gain.target).toBe(0.7);
        audio.dispose();
    });

    it("replaces the live voice when a second sound takes the same channel", async () => {
        installRunning();
        const audio = createStoryAudio();
        audio.arm();
        audio.playSound("/a/first.ogg", { volume: 1, loop: true, channel: "a" });
        await ctxUnderTest?.settle();
        const first = ctxUnderTest?.sources.at(-1);
        expect(first?.stoppedAt).toBeNull();

        audio.playSound("/a/second.ogg", { volume: 0.5, loop: false, channel: "a" });
        // The client holds ONE AudioSource per channel; 915 EN reuses.
        expect(first?.stoppedAt).not.toBeNull();
        await ctxUnderTest?.settle();
        expect(audio.snapshot().volumes.sfxCue.a).toBe(0.5);
        audio.dispose();
    });

    it("fades a [stopsound(fadetime=)] instead of cutting it", async () => {
        installRunning();
        const audio = createStoryAudio();
        audio.arm();
        audio.playSound("/a/amb.ogg", { volume: 1, loop: true, channel: "b" });
        await ctxUnderTest?.settle();
        const src = ctxUnderTest?.sources.at(-1);
        const gain = ctxUnderTest?.gains.at(-1);
        const now = ctxUnderTest?.currentTime ?? 0;

        // 2,804 of the 2,937 EN [stopsound] carry a fadetime.
        audio.stopSounds("b", 2);
        expect(gain?.gain.target).toBe(0);
        expect(src?.stoppedAt).toBeCloseTo(now + 2.05, 6);
        audio.dispose();
    });

    it("forgets the script's channel levels on a replay", async () => {
        installRunning();
        const audio = createStoryAudio();
        audio.arm();
        audio.setSfxCueVolume("a", 0, 0);
        expect(audio.snapshot().volumes.sfxCue.a).toBe(0);

        audio.resetSfxLevels();
        audio.playSound("/a/walk.ogg", { volume: 1, loop: false, channel: "a" });
        await ctxUnderTest?.settle();
        expect(ctxUnderTest?.gains.at(-1)?.gain.value).toBe(1);
        audio.dispose();
    });
});

describe("the clock a sound is scheduled against", () => {
    it("anchors delay= to the call, not to the end of the decode", async () => {
        installRunning();
        const audio = createStoryAudio();
        audio.arm();
        const ctx = ctxUnderTest;
        if (!ctx) throw new Error("no context");
        ctx.currentTime = 10;
        audio.playSound("/a/explosion.ogg", { volume: 1, loop: false, delaySec: 0.3 });
        // The fetch and the decode take 0.4 s of audio clock.
        ctx.currentTime = 10.4;
        await ctx.settle();
        // Old code started at decodeEnd + delay = 10.7, so an authored stagger
        // drifted by the whole decode of whichever file was not cached.
        expect(ctx.sources.at(-1)?.startedAt).toBeCloseTo(10.4, 6);
        const row = audio.snapshot().ledger.at(-1);
        expect(row?.scheduledStart).toBeCloseTo(10.3, 6);
        audio.dispose();
    });

    it("keeps a [musicvolume] that landed inside the decode window", async () => {
        installRunning();
        const audio = createStoryAudio();
        audio.arm();
        audio.playMusic({ loop: "/m/loop.ogg", volume: 0.6, crossfade: 1 });
        // The script's next command, before the decode lands.
        audio.setCueVolume(0.2, 1);
        await ctxUnderTest?.settle();
        // Old code ramped back to req.volume and silently dropped the command.
        expect(ctxUnderTest?.gains.at(-1)?.gain.target).toBe(0.2);
        audio.dispose();
    });
});

describe("the dev ledger", () => {
    it("writes one row per effect, with the decode and the start it actually got", async () => {
        installRunning();
        const audio = createStoryAudio();
        audio.arm();
        audio.noteHalt(7);
        audio.playSound("/a/hit.ogg", { volume: 0.9, loop: false, channel: "atk1" });
        await ctxUnderTest?.settle();
        const row = audio.snapshot().ledger.at(-1);
        expect(row?.halt).toBe(7);
        expect(row?.kind).toBe("sound");
        expect(row?.channel).toBe("atk1");
        expect(row?.requestedVolume).toBe(0.9);
        expect(row?.fetchStatus).toBe("ok");
        expect(row?.decoded).toEqual({ channels: 2, sampleRate: 44100, duration: 3 });
        expect(row?.actualStartClock).not.toBeNull();

        audio.stopSounds("atk1", 1);
        expect(row?.killReason).toBe("stopsound");
        audio.dispose();
    });
});

/**
 * A `dispose()` that the reader did not mean as the end.
 *
 * Measured 2026-09-23 in the user's own Chrome, entering the reader by a
 * CLIENT-SIDE navigation from `/stories`: a property setter on
 * `window.__storyAudio` recorded exactly two writes 15 ms apart, one function
 * and one undefined, and no third. The module is created ONCE during render
 * into a `useRef` and the mount effect's cleanup disposes it; the ref survives
 * that cycle, so the render never calls `createStoryAudio()` again. With the
 * old tombstone every later `arm()` returned false and the session had no
 * music AND no SFX: no AudioContext was ever created, `contextState` read
 * "none" and the ledger stayed empty over a resume plus an advance.
 */
describe("a dispose is a teardown, not a tombstone", () => {
    it("arms again after dispose and plays music on the new context", async () => {
        installRunning();
        const audio = createStoryAudio();
        expect(audio.arm()).toBe(true);
        const first = ctxUnderTest;

        audio.dispose();
        expect(first?.state).toBe("closed");
        expect(audio.armed).toBe(false);

        // The Resume button's own `audio.arm()`, one gesture later.
        expect(audio.arm()).toBe(true);
        expect(audio.armed).toBe(true);
        expect(ctxUnderTest).not.toBe(first);
        expect(ctxUnderTest?.state).toBe("running");

        audio.playMusic({ intro: "/m/intro.ogg", loop: "/m/loop.ogg", volume: 0.8, crossfade: 0 });
        await ctxUnderTest?.settle();
        const snap = audio.snapshot();
        expect(snap.contextState).toBe("running");
        expect(snap.music.playing).toBe(true);
        expect(ctxUnderTest?.sources.length).toBe(2);
        expect(ctxUnderTest?.sources.at(-1)?.startedAt).not.toBeNull();
        audio.dispose();
    });

    it("keeps the settings volumes across the rebuild", async () => {
        installRunning();
        const audio = createStoryAudio();
        audio.setMasterMusic(0.55);
        audio.setMasterSfx(0.3);
        audio.arm();
        audio.dispose();
        audio.arm();
        const v = audio.snapshot().volumes;
        expect(v.musicBusGain).toBe(0.55);
        expect(v.sfxBusGain).toBe(0.3);
        audio.dispose();
    });
});

/**
 * THE CUE HAS A TIMELINE, and a listener can be put anywhere on it.
 *
 * Nothing here needs a clock of its own: the position is `ctx.currentTime`
 * minus the clock time the cue's own second 0 maps to, and a seek moves that
 * origin by starting the buffers at an offset. The numbers below are Near
 * Light's, a 9.2 s intro over a 177.2 s loop, which is the one case where the
 * intro and the loop are long enough for the two arms to disagree visibly.
 */
describe("the music timeline", () => {
    it("reports intro plus loop, and walks the position on the audio clock", async () => {
        installCue();
        const audio = createStoryAudio();
        audio.arm();
        const ctx = ctxUnderTest;
        if (!ctx) throw new Error("no context");
        audio.playMusic(CUE);
        await ctx.settle();

        expect(audio.musicIntroSeconds).toBeCloseTo(9.2, 6);
        expect(audio.musicLength).toBeCloseTo(186.4, 6);
        expect(audio.musicPosition).toBe(0);

        ctx.currentTime = START_AT + 3;
        expect(audio.musicPosition).toBeCloseTo(3, 6);

        // Past the join the position WRAPS, because the loop source does: 200 s
        // of playing is 9.2 + (190.8 mod 177.2) = 22.8 s into the timeline.
        ctx.currentTime = START_AT + 200;
        expect(audio.musicPosition).toBeCloseTo(22.8, 6);
        audio.dispose();
    });

    it("has no timeline before the decode lands or after a stop", async () => {
        installCue();
        const audio = createStoryAudio();
        audio.arm();
        const ctx = ctxUnderTest;
        if (!ctx) throw new Error("no context");
        audio.playMusic(CUE);
        expect(audio.musicLength).toBe(0);
        expect(audio.musicPosition).toBe(0);

        await ctx.settle();
        expect(audio.musicLength).toBeCloseTo(186.4, 6);

        audio.stopMusic(0);
        expect(audio.musicLength).toBe(0);
        expect(audio.musicPosition).toBe(0);
        expect(audio.snapshot().music.length).toBe(0);
        audio.dispose();
    });

    it("starts the intro at the offset and leaves the join one intro-remainder later", async () => {
        installCue();
        const audio = createStoryAudio();
        audio.arm();
        const ctx = ctxUnderTest;
        if (!ctx) throw new Error("no context");
        audio.playMusicAt(CUE, 4);
        await ctx.settle();

        const [intro, loop] = ctx.sources;
        expect(ctx.sources).toHaveLength(2);
        expect(intro?.offset).toBeCloseTo(4, 6);
        expect(intro?.startedAt).toBeCloseTo(START_AT, 6);
        // 9.2 - 4 of intro is left, so the join is 5.2 s after the sources start.
        expect((loop?.startedAt ?? 0) - (intro?.startedAt ?? 0)).toBeCloseTo(5.2, 6);
        expect(loop?.offset).toBe(0);

        ctx.currentTime = START_AT;
        expect(audio.musicPosition).toBeCloseTo(4, 6);
        audio.dispose();
    });

    it("skips the intro for an offset past it and starts the loop at that phase", async () => {
        installCue();
        const audio = createStoryAudio();
        audio.arm();
        const ctx = ctxUnderTest;
        if (!ctx) throw new Error("no context");
        audio.playMusicAt(CUE, 100);
        await ctx.settle();

        expect(ctx.sources).toHaveLength(1);
        expect(ctx.sources[0]?.loop).toBe(true);
        expect(ctx.sources[0]?.offset).toBeCloseTo(90.8, 6);
        // There is no join left to schedule, so the instrument says so rather than
        // reporting one that already happened.
        expect(audio.snapshot().music.join).toBeNull();

        ctx.currentTime = START_AT;
        expect(audio.musicPosition).toBeCloseTo(100, 6);
        audio.dispose();
    });

    it("wraps an offset past the end of the timeline into the loop's own phase", async () => {
        installCue();
        const audio = createStoryAudio();
        audio.arm();
        const ctx = ctxUnderTest;
        if (!ctx) throw new Error("no context");
        // 190 is past the 186.4 s timeline: (190 - 9.2) mod 177.2 = 3.6.
        audio.playMusicAt(CUE, 190);
        await ctx.settle();

        expect(ctx.sources).toHaveLength(1);
        expect(ctx.sources[0]?.offset).toBeCloseTo(3.6, 6);
        ctx.currentTime = START_AT;
        expect(audio.musicPosition).toBeCloseTo(12.8, 6);
        audio.dispose();
    });

    it("restarts a cue that is already the one playing, which a repeat of playMusic never does", async () => {
        installCue();
        const audio = createStoryAudio();
        audio.arm();
        const ctx = ctxUnderTest;
        if (!ctx) throw new Error("no context");
        audio.playMusic(CUE);
        await ctx.settle();
        expect(ctx.sources).toHaveLength(2);

        // The replay case: the same cue asked for again keeps sounding.
        audio.playMusic(CUE);
        await ctx.settle();
        expect(ctx.sources).toHaveLength(2);

        // A seek is the same cue too, and it has to build new sources anyway.
        audio.playMusicAt(CUE, 100);
        await ctx.settle();
        expect(ctx.sources).toHaveLength(3);
        expect(ctx.sources.at(-1)?.offset).toBeCloseTo(90.8, 6);
        audio.dispose();
    });
});
