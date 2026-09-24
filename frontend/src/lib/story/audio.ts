/**
 * Story audio: one BGM channel (intro once, then a seamless loop) and one
 * polyphonic SFX channel.
 *
 * Web Audio, not two HTMLAudioElements, for the BGM. The intro to loop join
 * must be sample-accurate: `AudioBufferSourceNode.start(introStart +
 * intro.duration)` schedules the loop on the audio clock, while an
 * HTMLAudioElement's `ended` event fires tens of ms late and `loop=true`
 * on a media element is not gapless in Firefox or Safari. Both channels
 * fade through GainNodes, so a crossfade is two ramps on one clock. The
 * price is a decode per file (`decodeAudioData`, an 869-file corpus of Ogg
 * Vorbis): where that decode rejects (a browser without Vorbis in Web
 * Audio) the channel falls back to a media element and gives up the gapless
 * join, and `lastFallback` says so.
 *
 * The same two buffers give the channel a TIMELINE, `intro + loop`, which is
 * what `musicPosition`, `musicLength` and `playMusicAt` are: the position is
 * the audio clock minus the clock time offset 0 maps to, so it costs one
 * subtraction and no timer, and a seek moves that origin by starting the
 * sources at a buffer offset instead. The reader never calls any of the three;
 * the library's now-playing bar does.
 *
 * A `channel=` is a BUS, not a tag. `[playsound(volume=v, channel=c)]` SETS
 * channel c's level to v and `[soundvolume(volume=w, channel=c)]` overwrites
 * it; the two never multiply. Measured over the 2,918 EN scripts that carry
 * commands: 736 `[playsound]` name `volume=0` and 726 of them are raised again
 * by a later `[soundvolume]` on the same channel, which under a multiply would
 * be 726 authored fade-ins that can never be heard. A second `[playsound]` on a
 * live channel REPLACES the first (915 occurrences over 227 files, 37 of them
 * over a loop), because the client holds one AudioSource per channel.
 *
 * Nothing plays before `arm()`, which the reader calls from the first
 * click: browsers refuse to start an AudioContext without a user gesture.
 *
 * In a dev build the module publishes a read-only `window.__storyAudio()`
 * snapshot, a per-effect LEDGER on that snapshot, and, through
 * `window.__storyAudioNodes()`, a permanent AnalyserNode per bus with an RMS
 * trace, so a browser run can assert what actually SOUNDED instead of
 * inferring it from the network log.
 */

import { createBufferCache } from "./audioBuffers";
import { type AudioLedgerRow, basename, LEDGER_MAX, ledgerRow, rmsOf, rmsSummary, TRACE_MAX, TRACE_MS } from "./audioLedger";
import { createUnlockGate } from "./audioUnlock";
import { clamp01 } from "./num";

const DEV = import.meta.env.DEV;

export interface MusicRequest {
    intro?: string;
    loop?: string;
    volume: number;
    crossfade: number;
}

interface MusicVoice {
    gain: GainNode;
    sources: AudioBufferSourceNode[];
    fallback?: HTMLAudioElement;
    /** The script's own volume for this cue, before the settings multiplier. */
    cueVolume: number;
    /** The loop (or intro) url, so the same cue asked twice is not restarted. */
    key: string;
    /** False until the decode has landed and the sources are scheduled. */
    started: boolean;
    row: AudioLedgerRow | null;
}

/** What `[playsound]` asks for, past the url. */
export interface SoundRequest {
    volume: number;
    loop: boolean;
    /** `delay=` seconds before the sound starts (2,028 EN uses), clamped by the engine. */
    delaySec?: number;
    /** `channel=` (6,318 EN uses on playsound): what `[stopsound(channel=)]` addresses. */
    channel?: string;
}

/** The DEV instrument's row shape is defined beside the ring that holds it. */
export type { AudioLedgerRow } from "./audioLedger";

/** A read-only snapshot for `window.__storyAudio()` in dev builds. */
export interface AudioSnapshot {
    contextState: string;
    music: {
        key: string | null;
        playing: boolean;
        fallback: boolean;
        /** The scheduled intro to loop join, in audio-clock seconds. */
        join: { introStart: number; loopAt: number; introDuration: number } | null;
        /** Seconds into the cue's own timeline, wrapped at the loop. */
        position: number;
        /** Intro plus loop, in seconds, or 0 while no timeline is known. */
        length: number;
    };
    sounds: { active: number; played: number; failed: number; lastUrl: string | null };
    /** The settings multipliers and the live bus gains, so a run can SEE the volume. */
    volumes: { masterMusic: number; masterSfx: number; muted: boolean; musicBusGain: number; sfxBusGain: number; sfxCue: Record<string, number> };
    /** DEV only: one row per effect, oldest first. */
    ledger: AudioLedgerRow[];
    /** DEV only: the audio clock now, so a reader can date the ledger. */
    now: number;
}

export interface StoryAudio {
    /** Create and resume the context; safe to call on every click. Returns false when audio is unavailable. */
    arm(): boolean;
    readonly armed: boolean;
    playMusic(req: MusicRequest): void;
    /**
     * The same cue, started `offsetSeconds` into its TIMELINE: a seek, which
     * is why it restarts a cue that is already the one playing where
     * `playMusic` would leave it alone. An offset inside the intro starts the
     * intro there and keeps the join sample-accurate; an offset past the intro
     * skips the intro and starts the loop at `(offset - intro) mod loop`, so a
     * seek beyond the end of the timeline lands on the right phase of the loop
     * rather than being refused.
     */
    playMusicAt(req: MusicRequest, offsetSeconds: number): void;
    /**
     * Where the cue is now, in seconds from its own start, read off the audio
     * clock rather than a timer. Past the intro it WRAPS at the loop, so the
     * value walks 0 to `musicLength` and starts over, which is what a bar
     * drawing a looping track has to show. Zero while nothing is decoded and
     * on a fallback channel, whose media element has no timeline here.
     */
    readonly musicPosition: number;
    /** The decoded intro's duration, so a bar can mark where the loop begins. Zero for a loop-only cue. */
    readonly musicIntroSeconds: number;
    /** The cue's timeline, intro plus loop, in seconds. Zero until the decode lands and zero on a fallback channel. */
    readonly musicLength: number;
    stopMusic(fadeSec: number): void;
    /** The script's `[musicvolume]`: scales the current cue, not the settings volume. */
    setCueVolume(volume: number, fadeSec: number): void;
    playSound(url: string, req: SoundRequest): void;
    /** `[stopsound]`: bare stops every voice, `ch=` stops that channel only, over `fadeSec`. */
    stopSounds(channel?: string, fadeSec?: number): void;
    /**
     * `[soundvolume]`: the script's own SFX level for ONE channel, under the
     * settings multiplier. A bare command with no channel sets a global level
     * over every voice instead.
     */
    setSfxCueVolume(channel: string | undefined, volume: number, fadeSec: number): void;
    /** A replay starts the script's SFX levels over; nothing survives a `goTo`. */
    resetSfxLevels(): void;
    snapshot(): AudioSnapshot;
    /** DEV: the halt every subsequent ledger row is stamped with. */
    noteHalt(halt: number): void;
    /** Settings: 0..1 multipliers and a mute, applied over whatever the script asked for. */
    setMasterMusic(volume: number): void;
    setMasterSfx(volume: number): void;
    setMuted(muted: boolean): void;
    /**
     * Hold the MUSIC bus at silence while a cutscene plays, and let it back up
     * after. Separate from the mute so the two cannot clobber each other: the
     * gain reads both, and a ducked bus stays down when the viewer unmutes
     * mid-clip. Nothing is suspended, because the AudioContext carries the SFX
     * bus too and suspending it would stop that as well.
     */
    setMusicDucked(ducked: boolean): void;
    dispose(): void;
    /** True after any decode fell back to a media element (no gapless join). */
    readonly lastFallback: boolean;
}

const RAMP_MIN_SEC = 0.02;

export function createStoryAudio(): StoryAudio {
    let ctx: AudioContext | null = null;
    let musicBus: GainNode | null = null;
    let sfxBus: GainNode | null = null;
    let current: MusicVoice | null = null;
    let masterMusic = 1;
    let masterSfx = 1;
    let muted = false;
    let musicDucked = false;
    let lastFallback = false;
    let disposed = false;
    const cache = createBufferCache(
        () => ctx,
        () => {
            lastFallback = true;
        },
    );
    interface SfxVoice {
        source?: AudioBufferSourceNode;
        element?: HTMLAudioElement;
        gain?: GainNode;
        channel?: string;
        /** The `volume=` the `[playsound]` itself carried. */
        cueVolume: number;
        row: AudioLedgerRow | null;
    }
    const sfxVoices = new Set<SfxVoice>();
    // Playing music requests are numbered so a slow decode cannot resurrect a cue that was replaced.
    let musicSerial = 0;
    // `[soundvolume]` is written PER CHANNEL (150 distinct channels over 1,305
    // EN commands) and a channel is a BUS: the level lives on the channel, and
    // the `[playsound]` that opens it is just the first write of that level.
    // One global level was measured silencing every voice on
    // act29side_level_act29side_10_beg, where the script takes one channel to 0
    // at halt 51 while two unrelated loops are playing.
    const sfxCue = new Map<string, number>();
    /** A bare `[soundvolume]` with no channel: a multiplier over every voice. */
    let sfxGlobal = 1;
    /** The level a voice should sit at, before the settings multiplier. */
    const levelFor = (v: { channel?: string; cueVolume: number }) => (v.channel !== undefined ? (sfxCue.get(v.channel) ?? v.cueVolume) : v.cueVolume) * sfxGlobal;
    // Instrument counters: a browser run reads these instead of guessing from the network log.
    let played = 0;
    let failed = 0;
    let lastUrl: string | null = null;
    let lastJoin: { introStart: number; loopAt: number; introDuration: number } | null = null;
    /**
     * THE CUE'S OWN TIMELINE, which is the whole of what a seek needs.
     *
     * `origin` is the audio-clock time that offset 0 of the cue maps to, so
     * the position is one subtraction and needs no timer of its own; a seek
     * moves the origin rather than counting anything. It is set when the
     * sources are scheduled, because only the decoded buffers carry the two
     * durations, and it is null while nothing is decoded, after a stop, and on
     * a media-element fallback, whose duration this module never reads.
     */
    let timeline: { origin: number; introSeconds: number; loopSeconds: number } | null = null;

    /** Where a cue that started at `origin` sits now, wrapped into the loop past the intro. */
    function musicPosition(): number {
        if (!ctx || !timeline) return 0;
        const { origin, introSeconds, loopSeconds } = timeline;
        const raw = ctx.currentTime - origin;
        if (raw <= 0) return 0;
        if (raw < introSeconds) return raw;
        if (loopSeconds <= 0) return Math.min(raw, introSeconds);
        return introSeconds + ((raw - introSeconds) % loopSeconds);
    }

    /**
     * The point in the cue's timeline a requested offset actually resolves to.
     * Inside the intro it is the offset itself; past it the remainder is the
     * loop's PHASE, so 190 s of a 9.2 + 177.2 cue resolves to 9.2 + 3.6.
     */
    function resolveOffset(offsetSec: number, introSeconds: number, loopSeconds: number): number {
        if (!(offsetSec > 0)) return 0;
        if (offsetSec < introSeconds) return offsetSec;
        if (loopSeconds <= 0) return Math.min(offsetSec, introSeconds);
        return introSeconds + ((offsetSec - introSeconds) % loopSeconds);
    }

    // ---- DEV instrument ----
    const ledger: AudioLedgerRow[] = [];
    let haltNow = -1;
    let musicAnalyser: AnalyserNode | null = null;
    let sfxAnalyser: AnalyserNode | null = null;
    let traceTimer: number | null = null;
    const trace: { t: number; music: number; sfx: number }[] = [];
    const scratch: { buffer: Float32Array<ArrayBuffer> | null } = { buffer: null };

    function push(row: AudioLedgerRow): AudioLedgerRow {
        ledger.push(row);
        if (ledger.length > LEDGER_MAX) ledger.shift();
        return row;
    }

    /** One row on the ledger ring, with the fields this command does not carry already nulled. */
    function log(kind: AudioLedgerRow["kind"], fields: Partial<AudioLedgerRow>): AudioLedgerRow {
        return push(ledgerRow(kind, ctx?.currentTime ?? 0, haltNow, fields));
    }

    const rms = (node: AnalyserNode | null) => rmsOf(node, scratch);

    function startTrace() {
        if (!DEV || traceTimer !== null || typeof window === "undefined") return;
        traceTimer = window.setInterval(() => {
            if (!ctx) return;
            trace.push({ t: ctx.currentTime, music: rms(musicAnalyser), sfx: rms(sfxAnalyser) });
            if (trace.length > TRACE_MAX) trace.shift();
        }, TRACE_MS);
    }

    function applyBusVolumes() {
        if (!ctx || !musicBus || !sfxBus) return;
        const t = ctx.currentTime;
        musicBus.gain.cancelScheduledValues(t);
        musicBus.gain.setTargetAtTime(muted || musicDucked ? 0 : masterMusic, t, 0.03);
        sfxBus.gain.cancelScheduledValues(t);
        sfxBus.gain.setTargetAtTime(muted ? 0 : masterSfx, t, 0.03);
    }

    /**
     * The gesture gate: the listeners that heal a context Chrome refused to
     * start. `audioUnlock.ts` carries the measurement and the rule; here it is
     * wired to the bus ramps, because a resume that lands on the live clock has
     * to re-apply the gains that were scheduled on the frozen one.
     */
    const unlock = createUnlockGate(() => (disposed ? null : ctx), applyBusVolumes);

    /**
     * A `dispose()` is a TEARDOWN, not a tombstone: the next `arm()` builds a
     * new context, new buses and new analysers, and republishes the dev
     * handles. Measured 2026-09-23 in the user's own Chrome, entering the
     * reader by a CLIENT-SIDE navigation from `/stories`: a property setter on
     * `window.__storyAudio` recorded exactly two writes 15 ms apart, one
     * function and one undefined, and no third. The module is created ONCE in
     * render into a `useRef`, the mount effect's cleanup disposes it, and the
     * ref survives that cycle, so the render never calls `createStoryAudio()`
     * again. With the tombstone, every later `arm()` returned false and the
     * reader had no music AND no SFX for the rest of the session: no
     * AudioContext was ever created, `contextState` read "none" and the ledger
     * stayed empty over a resume plus an advance. The same page reached by a
     * FULL LOAD armed normally, which is why eleven browser runs and every
     * vitest case missed it.
     */
    function arm(): boolean {
        if (typeof window === "undefined") return false;
        if (!ctx) {
            disposed = false;
            // A rebuild starts a new clock at 0, so the previous context's RMS
            // rows would read as the future and poison `rmsBetween`.
            trace.length = 0;
            const Ctor = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
            if (!Ctor) return false;
            ctx = new Ctor();
            musicBus = ctx.createGain();
            sfxBus = ctx.createGain();
            musicBus.connect(ctx.destination);
            sfxBus.connect(ctx.destination);
            if (DEV && typeof ctx.createAnalyser === "function") {
                // One analyser per bus, permanently, so a run can read SIGNAL
                // and not just scheduling. An unconnected AnalyserNode still
                // sees everything written into it.
                musicAnalyser = ctx.createAnalyser();
                sfxAnalyser = ctx.createAnalyser();
                musicAnalyser.fftSize = 2048;
                sfxAnalyser.fftSize = 2048;
                musicBus.connect(musicAnalyser);
                sfxBus.connect(sfxAnalyser);
                startTrace();
            }
            applyBusVolumes();
            publish();
            // Chrome also suspends a running context when the output device
            // goes away, so the unlock is re-armed by state, not once at boot.
            ctx.addEventListener?.("statechange", () => {
                if (disposed || !ctx) return;
                if (ctx.state === "running") unlock.settle();
                else if (ctx.state === "suspended") unlock.bind();
            });
        }
        if (ctx.state !== "running") unlock.arm();
        return true;
    }

    function ramp(gain: GainNode, to: number, fadeSec: number) {
        if (!ctx) return;
        const t = ctx.currentTime;
        const from = gain.gain.value;
        gain.gain.cancelScheduledValues(t);
        gain.gain.setValueAtTime(from, t);
        gain.gain.linearRampToValueAtTime(to, t + Math.max(RAMP_MIN_SEC, fadeSec));
    }

    function killVoice(voice: MusicVoice, fadeSec: number, reason: AudioLedgerRow["killReason"]) {
        if (!ctx) return;
        if (DEV && voice.row && voice.row.killedAt === undefined) {
            voice.row.killedAt = ctx.currentTime;
            voice.row.killReason = reason;
        }
        ramp(voice.gain, 0, fadeSec);
        const stopAt = ctx.currentTime + Math.max(RAMP_MIN_SEC, fadeSec) + 0.05;
        for (const s of voice.sources) {
            try {
                s.stop(stopAt);
            } catch {
                // Already stopped or never started.
            }
        }
        if (voice.fallback) {
            const el = voice.fallback;
            window.setTimeout(
                () => {
                    el.pause();
                    el.src = "";
                },
                Math.max(RAMP_MIN_SEC, fadeSec) * 1000 + 50,
            );
        }
    }

    function playMusic(req: MusicRequest, offsetSec = 0): void {
        if (!arm() || !ctx || !musicBus) return;
        if (offsetSec === 0 && current && current.key === (req.loop ?? req.intro ?? "")) {
            // The same cue again (a replay landing on the halt that started it): keep it playing.
            // A SEEK carries an offset and is never this case: it has to restart the sources.
            setCueVolume(req.volume, req.crossfade);
            return;
        }
        timeline = null;
        const serial = ++musicSerial;
        const previous = current;
        const gain = ctx.createGain();
        gain.gain.value = 0;
        gain.connect(musicBus);
        const url = req.loop ?? req.intro ?? "";
        const row = DEV ? log("music", { key: basename(url), url, requestedVolume: req.volume, fetchStatus: cache.statusOf(url), scheduledStart: ctx.currentTime, busGainAtStart: musicBus.gain.value }) : null;
        const voice: MusicVoice = { gain, sources: [], cueVolume: req.volume, key: url, started: false, row };
        current = voice;
        if (previous) killVoice(previous, req.crossfade, "crossfade");

        void (async () => {
            const [intro, loop] = await Promise.all([req.intro ? cache.get(req.intro) : Promise.resolve(null), req.loop ? cache.get(req.loop) : Promise.resolve(null)]);
            if (serial !== musicSerial || disposed || !ctx) {
                if (DEV && row && row.killedAt === undefined) {
                    row.killedAt = ctx?.currentTime ?? 0;
                    row.killReason = "musicSerial";
                }
                return;
            }
            if (DEV && row) {
                row.fetchStatus = cache.statusOf(req.loop ?? req.intro ?? "");
                const b = loop ?? intro;
                if (b) row.decoded = { channels: b.numberOfChannels, sampleRate: b.sampleRate, duration: b.duration };
            }
            if (!intro && !loop) {
                // Decode failed: a media element plays the loop alone (not gapless).
                // It keeps no timeline either, so `musicLength` stays 0 and an
                // offset is DROPPED: a channel with no decoded buffers cannot
                // say where it is, and a bar that offered a seek there would be
                // drawing a position it made up.
                const src = req.loop ?? req.intro;
                if (!src) return;
                const el = new Audio(src);
                el.loop = true;
                el.volume = Math.min(1, voice.cueVolume * (muted || musicDucked ? 0 : masterMusic));
                voice.fallback = el;
                voice.started = true;
                if (DEV && row) {
                    row.fallback = true;
                    row.actualStartClock = ctx.currentTime;
                }
                void el.play().catch(() => undefined);
                return;
            }
            const startAt = ctx.currentTime + 0.05;
            const introSeconds = intro?.duration ?? 0;
            const loopSeconds = loop?.duration ?? 0;
            const into = resolveOffset(offsetSec, introSeconds, loopSeconds);
            let loopAt = startAt;
            if (intro && into < introSeconds) {
                const s = ctx.createBufferSource();
                s.buffer = intro;
                s.connect(gain);
                // The second argument is where in the BUFFER playback begins, so
                // the join still lands one intro-remainder later and stays sample-accurate.
                s.start(startAt, into);
                voice.sources.push(s);
                loopAt = startAt + (introSeconds - into);
                lastJoin = { introStart: startAt - into, loopAt, introDuration: introSeconds };
            } else {
                // A seek past the intro has already passed the join: there is none left to schedule.
                lastJoin = null;
            }
            if (loop) {
                const s = ctx.createBufferSource();
                s.buffer = loop;
                s.loop = true;
                s.connect(gain);
                s.start(loopAt, into < introSeconds ? 0 : into - introSeconds);
                voice.sources.push(s);
            }
            timeline = { origin: startAt - into, introSeconds, loopSeconds };
            voice.started = true;
            // The cue volume, NOT `req.volume`: a `[musicvolume]` that landed
            // inside the decode window already moved it, and ramping back to
            // the request would silently discard that command.
            ramp(gain, voice.cueVolume, req.crossfade);
            if (DEV && row) {
                row.actualStartClock = startAt;
                row.gainAtStart = voice.cueVolume;
            }
        })();
    }

    function stopMusic(fadeSec: number): void {
        musicSerial += 1;
        if (DEV && ctx) {
            log("stopMusic", { key: basename(current?.key ?? null), url: current?.key ?? null, delay: fadeSec, gainAtStart: current?.gain.gain.value ?? null, busGainAtStart: musicBus?.gain.value ?? null });
        }
        if (current) killVoice(current, fadeSec, "stopmusic");
        current = null;
        timeline = null;
    }

    function setCueVolume(volume: number, fadeSec: number): void {
        if (DEV && ctx) {
            log("musicVolume", { key: basename(current?.key ?? null), url: current?.key ?? null, requestedVolume: volume, delay: fadeSec, gainAtStart: current?.gain.gain.value ?? null, busGainAtStart: musicBus?.gain.value ?? null });
        }
        if (!current) return;
        current.cueVolume = volume;
        if (current.fallback) current.fallback.volume = Math.min(1, volume * (muted || musicDucked ? 0 : masterMusic));
        // Before the decode lands there is nothing to ramp; the value is read
        // off `cueVolume` when the sources are scheduled.
        else if (current.started) ramp(current.gain, volume, fadeSec);
    }

    function killSfx(v: SfxVoice, fadeSec: number, reason: AudioLedgerRow["killReason"]) {
        if (DEV && ctx && v.row && v.row.killedAt === undefined) {
            v.row.killedAt = ctx.currentTime;
            v.row.killReason = reason;
        }
        const fade = Math.max(0, fadeSec);
        if (v.source) {
            if (fade > 0 && v.gain && ctx) {
                ramp(v.gain, 0, fade);
                try {
                    v.source.stop(ctx.currentTime + fade + 0.05);
                } catch {
                    // Never started.
                }
            } else {
                try {
                    v.source.stop();
                } catch {
                    // Never started.
                }
            }
        }
        if (v.element) {
            const el = v.element;
            if (fade > 0) {
                window.setTimeout(() => {
                    el.pause();
                    el.src = "";
                }, fade * 1000);
            } else {
                el.pause();
                el.src = "";
            }
        }
        sfxVoices.delete(v);
    }

    function playSound(url: string, req: SoundRequest): void {
        if (!arm() || !ctx || !sfxBus) return;
        const bus = sfxBus;
        const delay = Math.max(0, req.delaySec ?? 0);
        // A `[playsound]` OPENS its channel at its own volume, exactly as a
        // `[soundvolume]` would: the two are the same write, never a product.
        if (req.channel !== undefined) {
            for (const v of sfxVoices) {
                // One AudioSource per channel in the client, so the second
                // sound on a live channel replaces the first.
                if (v.channel === req.channel) killSfx(v, 0, "channel-reuse");
            }
            sfxCue.set(req.channel, req.volume);
        }
        const entry: SfxVoice = { channel: req.channel, cueVolume: req.volume, row: null };
        const level = levelFor(entry);
        // The delay is anchored to the CALL, not to the decode: a staggered set
        // of one-shots must keep its authored offsets whether or not each file
        // was already in the buffer cache.
        const at = ctx.currentTime + delay;
        if (DEV) {
            entry.row = log("sound", { key: basename(url), url, channel: req.channel ?? null, requestedVolume: req.volume, cueLevel: level, delay, fetchStatus: cache.statusOf(url), scheduledStart: at, busGainAtStart: bus.gain.value });
        }
        sfxVoices.add(entry);
        lastUrl = url;
        void cache.get(url).then((buffer) => {
            if (disposed || !ctx || !sfxVoices.has(entry)) return;
            const row = entry.row;
            if (DEV && row) row.fetchStatus = cache.statusOf(url);
            if (!buffer) {
                // The decode lost: one media element for this file, counted as
                // a failure so the instrument says so rather than going quiet.
                failed += 1;
                const el = new Audio(url);
                el.loop = req.loop;
                el.volume = Math.min(1, levelFor(entry) * (muted ? 0 : masterSfx));
                entry.element = el;
                el.addEventListener("ended", () => sfxVoices.delete(entry));
                if (DEV && row) {
                    row.fallback = true;
                    row.actualStartClock = Math.max(ctx.currentTime, at);
                    row.gainAtStart = el.volume;
                }
                const wait = Math.max(0, at - ctx.currentTime);
                if (wait > 0) window.setTimeout(() => void el.play().catch(() => undefined), wait * 1000);
                else void el.play().catch(() => undefined);
                return;
            }
            const g = ctx.createGain();
            g.gain.value = levelFor(entry);
            g.connect(bus);
            const s = ctx.createBufferSource();
            s.buffer = buffer;
            s.loop = req.loop;
            s.connect(g);
            s.onended = () => sfxVoices.delete(entry);
            // `delay=` is scheduled on the audio clock, not a timer, so it
            // keeps its offset against the music that is already playing. A
            // start time already in the past plays immediately, which is what
            // the client does when the step it belongs to is skipped.
            const startAt = Math.max(ctx.currentTime, at);
            s.start(startAt);
            entry.source = s;
            entry.gain = g;
            played += 1;
            if (DEV && row) {
                row.actualStartClock = startAt;
                row.gainAtStart = g.gain.value;
                row.decoded = { channels: buffer.numberOfChannels, sampleRate: buffer.sampleRate, duration: buffer.duration };
            }
        });
    }

    function stopSounds(channel?: string, fadeSec = 0): void {
        if (DEV && ctx) {
            log("stopSound", { key: channel ?? "*", channel: channel ?? null, cueLevel: channel !== undefined ? (sfxCue.get(channel) ?? null) : null, delay: fadeSec, busGainAtStart: sfxBus?.gain.value ?? null });
        }
        for (const v of [...sfxVoices]) {
            if (channel !== undefined && v.channel !== channel) continue;
            killSfx(v, fadeSec, "stopsound");
        }
    }

    function setSfxCueVolume(channel: string | undefined, volume: number, fadeSec: number): void {
        const level = clamp01(volume);
        if (DEV && ctx) {
            log("soundVolume", { key: channel ?? "*", channel: channel ?? null, requestedVolume: level, cueLevel: channel !== undefined ? (sfxCue.get(channel) ?? null) : sfxGlobal, delay: fadeSec, busGainAtStart: sfxBus?.gain.value ?? null });
        }
        if (channel === undefined) sfxGlobal = level;
        else sfxCue.set(channel, level);
        for (const v of sfxVoices) {
            if (channel !== undefined && v.channel !== channel) continue;
            const target = levelFor(v);
            if (v.gain) ramp(v.gain, target, fadeSec);
            if (v.element) v.element.volume = Math.min(1, target * (muted ? 0 : masterSfx));
        }
    }

    function resetSfxLevels(): void {
        sfxCue.clear();
        sfxGlobal = 1;
    }

    /** DEV handles, republished after every `arm()` so a rebuild is reachable. */
    function publish() {
        if (!DEV || typeof window === "undefined") return;
        (window as unknown as { __storyAudio?: () => AudioSnapshot }).__storyAudio = snapshot;
        (window as unknown as { __storyAudioNodes?: typeof nodes }).__storyAudioNodes = nodes;
    }

    function snapshot(): AudioSnapshot {
        return {
            contextState: ctx?.state ?? "none",
            music: { key: current?.key ?? null, playing: current !== null, fallback: current?.fallback !== undefined, join: lastJoin, position: musicPosition(), length: timeline ? timeline.introSeconds + timeline.loopSeconds : 0 },
            sounds: { active: sfxVoices.size, played, failed, lastUrl },
            volumes: { masterMusic, masterSfx, muted, musicBusGain: musicBus?.gain.value ?? 0, sfxBusGain: sfxBus?.gain.value ?? 0, sfxCue: Object.fromEntries(sfxCue) },
            ledger: DEV ? ledger : [],
            now: ctx?.currentTime ?? 0,
        };
    }

    /**
     * A dev-only handle on the live graph. `__storyAudio()` proves a source was
     * SCHEDULED; only an AnalyserNode on these nodes proves a signal reaches the
     * destination, which is what a headless run can never show. `rmsWindow`
     * samples live; `rmsBetween` reads the 50 ms trace that has been running
     * since `arm()`, which is how a ledger row is checked for signal in the
     * 400 ms after its own scheduled start.
     */
    function nodes() {
        return {
            ctx,
            musicBus,
            sfxBus,
            musicVoiceGain: current?.gain ?? null,
            musicAnalyser,
            sfxAnalyser,
            trace,
            rmsBetween(bus: "music" | "sfx", from: number, to: number) {
                const rows = trace.filter((r) => r.t >= from && r.t <= to);
                const vs = rows.map((r) => (bus === "music" ? r.music : r.sfx));
                return rmsSummary(vs);
            },
            rmsWindow(bus: "music" | "sfx", seconds: number): Promise<{ n: number; mean: number; max: number }> {
                return new Promise((resolve) => {
                    const vs: number[] = [];
                    const node = bus === "music" ? musicAnalyser : sfxAnalyser;
                    const id = window.setInterval(() => vs.push(rms(node)), TRACE_MS);
                    window.setTimeout(() => {
                        window.clearInterval(id);
                        resolve(rmsSummary(vs));
                    }, seconds * 1000);
                });
            },
        };
    }

    publish();

    return {
        arm,
        get armed() {
            return ctx !== null;
        },
        playMusic,
        playMusicAt(req, offsetSeconds) {
            playMusic(req, offsetSeconds);
        },
        get musicPosition() {
            return musicPosition();
        },
        get musicIntroSeconds() {
            return timeline?.introSeconds ?? 0;
        },
        get musicLength() {
            return timeline ? timeline.introSeconds + timeline.loopSeconds : 0;
        },
        stopMusic,
        setCueVolume,
        playSound,
        stopSounds,
        setSfxCueVolume,
        resetSfxLevels,
        snapshot,
        noteHalt(halt) {
            haltNow = halt;
        },
        setMasterMusic(v) {
            masterMusic = clamp01(v);
            applyBusVolumes();
            if (current?.fallback) current.fallback.volume = Math.min(1, current.cueVolume * (muted || musicDucked ? 0 : masterMusic));
        },
        setMasterSfx(v) {
            masterSfx = clamp01(v);
            applyBusVolumes();
            for (const s of sfxVoices) {
                if (s.element) s.element.volume = Math.min(1, levelFor(s) * (muted ? 0 : masterSfx));
            }
        },
        setMuted(m) {
            muted = m;
            applyBusVolumes();
            if (current?.fallback) current.fallback.volume = Math.min(1, current.cueVolume * (muted || musicDucked ? 0 : masterMusic));
        },
        setMusicDucked(d) {
            musicDucked = d;
            applyBusVolumes();
            if (current?.fallback) current.fallback.volume = Math.min(1, current.cueVolume * (muted || musicDucked ? 0 : masterMusic));
        },
        dispose() {
            disposed = true;
            unlock.dispose();
            if (traceTimer !== null && typeof window !== "undefined") {
                window.clearInterval(traceTimer);
                traceTimer = null;
            }
            if (DEV && typeof window !== "undefined") {
                const w = window as unknown as { __storyAudio?: () => AudioSnapshot; __storyAudioNodes?: typeof nodes };
                if (w.__storyAudio === snapshot) w.__storyAudio = undefined;
                if (w.__storyAudioNodes === nodes) w.__storyAudioNodes = undefined;
            }
            stopSounds();
            stopMusic(0);
            void ctx?.close().catch(() => undefined);
            ctx = null;
        },
        get lastFallback() {
            return lastFallback;
        },
    };
}
