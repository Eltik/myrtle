import { useStore } from "@tanstack/react-store";
import { Store } from "@tanstack/store";
import { useEffect } from "react";
import { toastManager } from "#/components/ui/toast";
import { readStoredValue, writeStoredValue } from "#/hooks/use-local-storage-state";
import { sirenLyricsUrl } from "#/lib/api/music";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { createStoryAudio, type MusicRequest, type StoryAudio } from "#/lib/story/audio";
import { audioContextConstructor } from "#/lib/story/audioContext";
import { clamp, clamp01 } from "#/lib/story/num";
import { logPlay, toggleLike, updatePlay } from "../impl/library/store";
import { displayArtists, type ICatalog, type ITrack, radioFor, trackSource } from "../impl/model";
import { parseLrc } from "./lrc";
import type { messages } from "./Player.messages";
import { registerMusicPauser, setMusicLive, silenceOtherMusic } from "./presence";
import { type IQueue, identityOrder, insertEnd, insertNext, moveItem, nextIndex, playOrder, previousIndex, removeAt, validOrder } from "./queue";
import { CROSSFADE_MAX, CROSSFADE_MIN, DEFAULT_SETTINGS, type ILyricLine, type IPlayerSettings, type IPlayerState, type IPlayhead, type IQueueSource, type IShortcut, type PanelName, type PreloadChoice, type RepeatMode, type SleepMinutes } from "./types";

/**
 * THE MUSIC PAGE'S PLAYER: one queue, two transports, one set of controls.
 *
 * Label tracks (`trackSource(track).kind === "stream"`) play on two media
 * elements, A and B: one sounds, the other preloads the next track, and a
 * crossfade is the two playing at once with their gains ramped. In-game
 * tracks (`"loop"`) play on the story engine (`#/lib/story/audio`), which is
 * what joins an intro to its loop without a gap; a media element cannot.
 * On iOS label tracks play there too, once through (`elementVolumeFixed`).
 * Only one transport sounds at a time and a change of transport is a switch,
 * never a fade.
 *
 * Everything is module state, so a view that unmounts loses nothing and the
 * audio elements survive a client-side navigation away and back.
 */

const SESSION_KEY = "myrtle.music.session";
const SETTINGS_KEY = "myrtle.music.settings";
const VOLUME_KEY = "myrtle.music.volume";

/** The playhead store's clock: 4 Hz, enough for a bar and a lyric line, cheap enough for a queue. */
const PLAYHEAD_MS = 250;
/** How often the session is written while something plays. */
const SESSION_SAVE_MS = 3000;
/** Previous restarts the track instead when it is further in than this. */
const RESTART_AFTER_S = 3;
/** J and L, and the Media Session's seek buttons. */
const SKIP_S = 10;
/** Consecutive load failures before the player gives up. */
const MAX_FAILURES = 3;
/** A play is logged after this much heard time, or half the track when that is shorter. */
const LOG_AFTER_S = 30;
/** Tracks whose lyrics stay in memory. */
const LYRICS_CACHE_MAX = 40;
/** Stopping an in-game cue fades this long, the story library's own figure, so a pause is not a click. */
const LOOP_FADE_S = 0.35;
/** A timeupdate delta larger than this is a seek, not listening. */
const MAX_HEARD_STEP_S = 1.5;
/** How many radio tracks are appended at a time, out of how many asked for. */
const RADIO_ASK = 60;
const RADIO_TAKE = 15;
/** Passes of the loop an in-game track plays before a queue moves on. */
const LOOP_PASSES = 2;

/**
 * AUTO VOLUME LEVELING is a compressor on the live signal, not a loudness
 * measurement per track: there is no loudness data for these releases, so it
 * evens loud and quiet passages as they play. Off is the same node set to
 * unity (threshold 0 dB, ratio 1), which passes the signal through.
 */
const LEVELING_ON = { threshold: -24, knee: 6, ratio: 4, attack: 0.003, release: 0.25 } as const;
const LEVELING_OFF = { threshold: 0, knee: 0, ratio: 1, attack: 0.003, release: 0.25 } as const;

const NO_LYRICS: IPlayerState["lyrics"] = { trackId: null, status: "none", lines: [] };

const INITIAL: IPlayerState = {
    queue: [],
    index: -1,
    order: [],
    current: null,
    status: "idle",
    volume: 1,
    muted: false,
    shuffle: false,
    repeat: "off",
    source: { kind: "none" },
    settings: DEFAULT_SETTINGS,
    sleepEndsAt: null,
    panel: null,
    settingsOpen: false,
    shortcutsOpen: false,
    lyrics: NO_LYRICS,
    failures: 0,
    restored: false,
    houseArtist: null,
};

const EMPTY_PLAYHEAD: IPlayhead = { position: 0, duration: 0, loopAt: null, buffered: 0 };

export const playerStore = new Store<IPlayerState>(INITIAL);
export const playheadStore = new Store<IPlayhead>(EMPTY_PLAYHEAD);

function set(patch: Partial<IPlayerState>): void {
    playerStore.setState((s) => ({ ...s, ...patch }));
}

function state(): IPlayerState {
    return playerStore.state;
}

function queueOf(s: IPlayerState): IQueue<ITrack> {
    return { items: s.queue, index: s.index, order: s.order };
}

/** The index after the current one, or the first radio track appended when the queue has run out. */
function nextOrRadio(s: IPlayerState, auto: boolean): number | null {
    return nextIndex(queueOf(s), s.repeat, auto) ?? extendWithRadio();
}

/** Playing, or about to: what a pause would interrupt. */
function wantsPlay(): boolean {
    const st = state().status;
    return st === "playing" || st === "loading";
}

/** `err.name` for a thrown value of any shape. */
function errorName(err: unknown): string | undefined {
    return (err as { name?: string } | null)?.name;
}

/** The shortcuts the engine binds, in the order the dialog lists them. `alt` is a second binding for the same action. */
export const SHORTCUTS: readonly IShortcut[] = [
    { id: "music.shortcut.playPause", keys: ["Space"], alt: ["K"] },
    { id: "music.shortcut.next", keys: ["Shift", "N"] },
    { id: "music.shortcut.previous", keys: ["Shift", "P"] },
    { id: "music.shortcut.back", keys: ["J"] },
    { id: "music.shortcut.forward", keys: ["L"] },
    { id: "music.shortcut.mute", keys: ["M"] },
    { id: "music.shortcut.shuffle", keys: ["S"] },
    { id: "music.shortcut.repeat", keys: ["R"] },
    { id: "music.shortcut.lyrics", keys: ["Y"] },
    { id: "music.shortcut.queue", keys: ["Q"] },
    { id: "music.shortcut.like", keys: ["Shift", "L"] },
    { id: "music.shortcut.help", keys: ["?"] },
    { id: "music.shortcut.close", keys: ["Esc"] },
];

let catalog: ICatalog | null = null;
/** The saved session, read once, held until the catalogue can resolve it. `undefined` = not read yet. */
let pendingSession: ISession | null | undefined;
let hydrated = false;

/** Which track a transport holds, and on which. `null` = nothing loaded (a restored session before play). */
let loadedId: string | null = null;
let transport: "stream" | "loop" | null = null;
/** Where `play()` starts the current track when no transport holds it yet. */
let pendingOffset = 0;
/** Bumped on every track start, so a late failure from an earlier one is not counted against this one. */
let loadToken = 0;
let failureHandledFor = -1;

interface INotice {
    kind: "skip" | "stopped";
    title?: string;
    count?: number;
}
let notifier: ((n: INotice) => void) | null = null;

function notify(n: INotice): void {
    notifier?.(n);
}

let lastPlayheadAt = 0;
let lastPositionStateAt = 0;

function now(): number {
    return typeof performance !== "undefined" ? performance.now() : Date.now();
}

function publishPlayhead(next: Partial<IPlayhead>, force = false): void {
    const t = now();
    if (!force && t - lastPlayheadAt < PLAYHEAD_MS) return;
    lastPlayheadAt = t;
    playheadStore.setState((p) => ({ ...p, ...next }));
    if (force || t - lastPositionStateAt >= 1000) {
        lastPositionStateAt = t;
        mediaPositionState();
    }
}

interface IListen {
    trackId: string | null;
    heard: number;
    logged: boolean;
    play: Promise<number> | null;
    flushedAt: number;
}

let listen: IListen = { trackId: null, heard: 0, logged: false, play: null, flushedAt: 0 };

function beginListen(trackId: string | null): void {
    listen = { trackId, heard: 0, logged: false, play: null, flushedAt: 0 };
}

/** Add heard seconds to the current play and log it once it passes `min(30, duration / 2)`. */
function addHeard(seconds: number): void {
    const s = state();
    const track = s.current;
    if (!track) return;
    if (listen.trackId !== track.id) beginListen(track.id);
    listen.heard += seconds;
    if (listen.logged) return;
    const duration = playheadStore.state.duration;
    const needed = duration > 0 ? Math.min(LOG_AFTER_S, duration / 2) : LOG_AFTER_S;
    if (listen.heard < needed) return;
    listen.logged = true;
    listen.flushedAt = listen.heard;
    listen.play = logPlay(track.id, Math.round(listen.heard)).catch(() => Number.NaN);
}

/** Write the heard time onto the logged play, if there is one and it moved. */
function flushListen(): void {
    const l = listen;
    if (!l.logged || !l.play || l.heard - l.flushedAt < 1) return;
    l.flushedAt = l.heard;
    const heard = Math.round(l.heard);
    void l.play.then((id) => {
        if (Number.isFinite(id)) updatePlay(id, heard);
    });
}

function finalizeListen(): void {
    flushListen();
    beginListen(null);
}

// Stream transport: two media elements and a lazily built Web Audio graph.

interface IDeck {
    el: HTMLAudioElement;
    gain: GainNode | null;
    comp: DynamicsCompressorNode | null;
    /** The track the element's `src` belongs to. */
    trackId: string | null;
    /** Loaded ahead as the next track, not yet started. */
    preloaded: boolean;
    /** Already retried with a fresh `src` after an error, for this start. */
    retried: boolean;
    /** Where to seek once metadata lands. */
    pendingSeek: number | null;
    lastTime: number;
}

let decks: [IDeck, IDeck] | null = null;
let active = 0;
let ctx: AudioContext | null = null;
/** The graph could not be built (no AudioContext, or it threw): elements play directly. */
let graphBroken = false;
/** Cached `elementVolumeFixed()`; null until first asked. */
let volumeFixed: boolean | null = null;

/**
 * Whether this browser ignores a media element's `volume`, which is iOS and
 * iPadOS (every browser there is WebKit): Apple documents the property as not
 * settable and always reading 1. Feature-detected, not sniffed.
 *
 * There a label track does not play on the decks at all but on the loop
 * transport, as a one-shot cue (`cueSource`): the decks route their elements
 * through `createMediaElementSource`, which iOS does not play reliably, while
 * the story engine fetches and decodes into buffers, which is how the in-game
 * tracks already play there. Crossfade and preload are deck features and do
 * not apply on that path.
 */
function elementVolumeFixed(): boolean {
    if (volumeFixed !== null) return volumeFixed;
    volumeFixed = false;
    if (typeof Audio === "undefined") return volumeFixed;
    try {
        const probe = new Audio();
        probe.volume = 0.5;
        volumeFixed = Math.abs(probe.volume - 0.5) > 0.01;
    } catch {
        // A browser that throws on the probe keeps the graph.
    }
    return volumeFixed;
}

interface IFade {
    from: number;
    to: number;
    start: number;
    duration: number;
    raf: number | null;
}
let fade: IFade | null = null;

function makeDeck(i: number): IDeck {
    const el = new Audio();
    el.crossOrigin = "anonymous";
    el.preload = "auto";
    const deck: IDeck = { el, gain: null, comp: null, trackId: null, preloaded: false, retried: false, pendingSeek: null, lastTime: 0 };
    el.addEventListener("timeupdate", () => onDeckTime(i));
    el.addEventListener("playing", () => onDeckPlaying(i));
    el.addEventListener("waiting", () => {
        if (i === active && transport === "stream" && state().status === "playing") set({ status: "loading" });
    });
    el.addEventListener("ended", () => {
        if (i === active && transport === "stream") advance(true);
    });
    el.addEventListener("error", () => onDeckError(i));
    el.addEventListener("loadedmetadata", () => onDeckMeta(i));
    el.addEventListener("durationchange", () => onDeckMeta(i));
    el.addEventListener("progress", () => onDeckProgress(i));
    return deck;
}

function ensureDecks(): [IDeck, IDeck] | null {
    if (decks) return decks;
    if (typeof window === "undefined" || typeof Audio === "undefined") return null;
    decks = [makeDeck(0), makeDeck(1)];
    return decks;
}

function ensureContext(): AudioContext | null {
    if (graphBroken || typeof window === "undefined") return null;
    if (!ctx) {
        try {
            const Ctor = audioContextConstructor();
            if (!Ctor) {
                graphBroken = true;
                return null;
            }
            ctx = new Ctor();
        } catch {
            // Plain element playback, same controls; only leveling is lost.
            graphBroken = true;
            return null;
        }
    }
    if (ctx.state === "suspended") void ctx.resume().catch(() => {});
    return ctx;
}

function applyLeveling(deck: IDeck): void {
    if (!deck.comp) return;
    const p = state().settings.levelingOn ? LEVELING_ON : LEVELING_OFF;
    deck.comp.threshold.value = p.threshold;
    deck.comp.knee.value = p.knee;
    deck.comp.ratio.value = p.ratio;
    deck.comp.attack.value = p.attack;
    deck.comp.release.value = p.release;
}

/** `MediaElementSource -> DynamicsCompressor -> Gain -> destination`, once per element. */
function ensureGraph(deck: IDeck): void {
    if (deck.gain || graphBroken) return;
    const c = ensureContext();
    if (!c) return;
    try {
        const source = c.createMediaElementSource(deck.el);
        const comp = c.createDynamicsCompressor();
        const gain = c.createGain();
        source.connect(comp);
        comp.connect(gain);
        gain.connect(c.destination);
        deck.comp = comp;
        deck.gain = gain;
        applyLeveling(deck);
    } catch {
        graphBroken = true;
    }
}

function userLevel(): number {
    const s = state();
    return s.muted ? 0 : s.volume;
}

/** Set one deck's loudness: the user's level times `factor` (the crossfade's share). */
function deckLevel(i: number, factor: number): void {
    const deck = decks?.[i];
    if (!deck) return;
    const v = clamp01(userLevel() * factor);
    if (deck.gain) {
        deck.gain.gain.value = v;
        deck.el.volume = 1;
    } else {
        deck.el.volume = v;
    }
}

/** An element's duration, or 0 while it is not a finite number (NaN before metadata). */
function knownDuration(el: HTMLAudioElement): number {
    return Number.isFinite(el.duration) ? el.duration : 0;
}

/** The deck already holds `trackId`, loaded ahead and without an error, so a start can use it as is. */
function holdsPreload(deck: IDeck, trackId: string): boolean {
    return deck.trackId === trackId && deck.preloaded && !deck.el.error;
}

function freshSrc(track: ITrack): string | null {
    const src = trackSource(track);
    return src.kind === "stream" ? src.src : null;
}

function playDeck(i: number, factor = 1): void {
    const deck = decks?.[i];
    if (!deck) return;
    // Resumes a context the browser suspended (an interruption, a long background) before it is needed.
    ensureContext();
    ensureGraph(deck);
    deckLevel(i, factor);
    const token = loadToken;
    let p: Promise<void> | undefined;
    try {
        p = deck.el.play();
    } catch (err) {
        onPlayRejected(i, err, token);
        return;
    }
    if (p && typeof p.catch === "function") p.catch((err: unknown) => onPlayRejected(i, err, token));
}

function onPlayRejected(i: number, err: unknown, token: number): void {
    if (i !== active || token !== loadToken || transport !== "stream") return;
    const name = errorName(err);
    if (name === "AbortError") return;
    if (name === "NotAllowedError") {
        // Autoplay blocked: wait for the listener to press play.
        set({ status: "paused" });
        return;
    }
    trackFailed();
}

function startStream(track: ITrack, offset: number, autoplay: boolean): void {
    const d = ensureDecks();
    const src = freshSrc(track);
    if (!d || !src) {
        trackFailed();
        return;
    }
    stopLoopTransport();
    cancelFade();
    const idle = 1 - active;
    const idleDeck = d[idle];
    const usePreload = holdsPreload(idleDeck, track.id);
    const i = usePreload ? idle : active;
    d[1 - i].el.pause();
    active = i;
    const deck = d[i];
    if (!usePreload) {
        // A fresh `src` every start: the backend signs the CDN url per request, so a stale one never sticks.
        deck.el.src = src;
        deck.trackId = track.id;
    }
    deck.preloaded = false;
    deck.retried = false;
    deck.el.preload = "auto";
    deck.lastTime = offset;
    deck.pendingSeek = null;
    if (offset > 0) {
        if (deck.el.readyState >= 1) deck.el.currentTime = offset;
        else deck.pendingSeek = offset;
    } else if (usePreload && deck.el.currentTime > 0) {
        deck.el.currentTime = 0;
    }
    transport = "stream";
    loadedId = track.id;
    deckLevel(i, 1);
    publishPlayhead({ position: offset, duration: knownDuration(deck.el), loopAt: null, buffered: 0 }, true);
    if (autoplay) {
        set({ status: "loading" });
        playDeck(i);
    } else set({ status: "paused" });
}

function onDeckMeta(i: number): void {
    const deck = decks?.[i];
    if (!deck) return;
    if (deck.pendingSeek !== null && deck.el.readyState >= 1) {
        deck.el.currentTime = deck.pendingSeek;
        deck.lastTime = deck.pendingSeek;
        deck.pendingSeek = null;
    }
    if (i !== active || transport !== "stream") return;
    publishPlayhead({ duration: knownDuration(deck.el) }, true);
}

function onDeckProgress(i: number): void {
    const deck = decks?.[i];
    if (!deck || i !== active || transport !== "stream") return;
    const el = deck.el;
    const d = el.duration;
    if (!Number.isFinite(d) || d <= 0 || el.buffered.length === 0) return;
    let end = 0;
    for (let k = 0; k < el.buffered.length; k += 1) if (el.buffered.start(k) <= el.currentTime + 0.5) end = Math.max(end, el.buffered.end(k));
    publishPlayhead({ buffered: Math.min(1, end / d) });
}

function onDeckPlaying(i: number): void {
    if (i !== active || transport !== "stream") return;
    set({ status: "playing", failures: 0 });
    mediaPlaybackState();
    schedulePreload();
}

function onDeckTime(i: number): void {
    const deck = decks?.[i];
    if (!deck || transport !== "stream") return;
    if (fade && fade.to === i) stepFade();
    if (i !== active) return;
    const t = deck.el.currentTime;
    const dt = t - deck.lastTime;
    deck.lastTime = t;
    if (!deck.el.paused && dt > 0 && dt < MAX_HEARD_STEP_S) addHeard(dt);
    publishPlayhead({ position: t, duration: knownDuration(deck.el), loopAt: null });
    maybeCrossfade(deck);
}

function onDeckError(i: number): void {
    const deck = decks?.[i];
    if (!deck) return;
    if (i !== active || transport !== "stream") {
        // A preload that failed is simply not used; the start loads it again.
        deck.preloaded = false;
        deck.trackId = null;
        return;
    }
    const s = state();
    const track = s.current;
    if (!track) return;
    if (!deck.retried) {
        // Once with a fresh url, which is what a preloaded url whose signature expired needs.
        const src = freshSrc(track);
        if (src) {
            deck.retried = true;
            const at = deck.el.currentTime;
            deck.el.src = src;
            if (at > 0) deck.pendingSeek = at;
            if (wantsPlay()) playDeck(i);
            return;
        }
    }
    trackFailed();
}

function pauseDecks(): void {
    cancelFade();
    if (!decks) return;
    for (const d of decks) d.el.pause();
}

function unloadDecks(): void {
    cancelFade();
    if (!decks) return;
    for (const d of decks) {
        d.el.pause();
        d.el.removeAttribute("src");
        d.trackId = null;
        d.preloaded = false;
        try {
            d.el.load();
        } catch {
            // jsdom and some browsers throw on load() with no source; nothing is loaded either way.
        }
    }
}

function preloadEnabled(choice: PreloadChoice): boolean {
    if (choice === "on") return true;
    if (choice === "off") return false;
    if (typeof window === "undefined") return false;
    try {
        if (window.matchMedia?.("(pointer: coarse)").matches) return false;
    } catch {
        // No media queries: treat as a desktop.
    }
    const conn = (navigator as unknown as { connection?: { saveData?: boolean; effectiveType?: string; type?: string } }).connection;
    if (conn?.saveData) return false;
    if (conn?.type === "cellular") return false;
    if (conn?.effectiveType && ["slow-2g", "2g", "3g"].includes(conn.effectiveType)) return false;
    return true;
}

/** Whether the next track is preloaded under these settings on this device (`auto` resolved). */
export function preloadIsOn(settings: IPlayerSettings): boolean {
    return preloadEnabled(settings.preload);
}

/** Buffer the next label track on the idle element, when preloading is on. */
function schedulePreload(): void {
    const s = state();
    if (!decks || fade || transport !== "stream" || !preloadEnabled(s.settings.preload)) return;
    const ni = nextIndex(queueOf(s), s.repeat, true);
    if (ni === null || ni === s.index) return;
    const track = s.queue[ni];
    if (!track) return;
    const src = freshSrc(track);
    if (!src) return;
    const idle = decks[1 - active];
    if (idle.trackId === track.id && idle.preloaded) return;
    idle.el.pause();
    idle.el.preload = "auto";
    idle.el.src = src;
    idle.trackId = track.id;
    idle.preloaded = true;
    idle.retried = false;
}

function maybeCrossfade(deck: IDeck): void {
    const s = state();
    if (fade || !s.settings.crossfadeOn || s.repeat === "one" || deck.el.paused) return;
    const duration = deck.el.duration;
    if (!Number.isFinite(duration) || duration <= 0) return;
    const seconds = s.settings.crossfadeSeconds;
    const remaining = duration - deck.el.currentTime;
    // A track shorter than two fades switches instead: there is no middle to keep.
    if (remaining > seconds || duration < seconds * 2) return;
    const ni = nextOrRadio(s, true);
    if (ni === null) return;
    const track = state().queue[ni];
    if (!track || trackSource(track).kind !== "stream") return;
    startCrossfade(ni, Math.max(0.2, remaining));
}

function startCrossfade(ni: number, seconds: number): void {
    const d = decks;
    const track = state().queue[ni];
    if (!d || !track) return;
    const src = freshSrc(track);
    if (!src) return;
    finalizeListen();
    const from = active;
    const to = 1 - active;
    const next = d[to];
    if (!holdsPreload(next, track.id)) {
        next.el.src = src;
        next.trackId = track.id;
    } else if (next.el.currentTime > 0) next.el.currentTime = 0;
    next.preloaded = false;
    next.retried = false;
    next.pendingSeek = null;
    next.lastTime = 0;
    active = to;
    loadToken += 1;
    loadedId = track.id;
    set({ index: ni, current: track, status: "loading" });
    beginListen(track.id);
    publishPlayhead({ position: 0, duration: knownDuration(next.el), loopAt: null, buffered: 0 }, true);
    playDeck(to, 0);
    fade = { from, to, start: now(), duration: seconds * 1000, raf: null };
    const loop = () => {
        if (!fade) return;
        stepFade();
        if (fade && typeof window !== "undefined" && typeof window.requestAnimationFrame === "function") fade.raf = window.requestAnimationFrame(loop);
    };
    loop();
}

/**
 * One step of the linear ramp. Driven by rAF and ALSO by the incoming
 * element's timeupdate, because a background tab pauses rAF and a fade that
 * only rAF moved would hang at whatever share it had reached.
 */
function stepFade(): void {
    if (!fade) return;
    const p = Math.min(1, (now() - fade.start) / fade.duration);
    deckLevel(fade.from, 1 - p);
    deckLevel(fade.to, p);
    if (p >= 1) endFade();
}

function endFade(): void {
    if (!fade || !decks) return;
    const f = fade;
    fade = null;
    if (f.raf !== null && typeof window !== "undefined") window.cancelAnimationFrame?.(f.raf);
    decks[f.from].el.pause();
    deckLevel(f.to, 1);
    schedulePreload();
}

/** Stop a fade where it is: the outgoing element is silenced and the incoming one goes to full. */
function cancelFade(): void {
    if (!fade) return;
    endFade();
}

// Loop transport: in-game tracks on the story engine.

interface ILoop {
    trackId: string;
    req: MusicRequest;
    /** Sounding now; false while paused (the story engine has no pause, so a pause is a stop that remembers). */
    sounding: boolean;
    /** Seconds into the virtual timeline (intro, then loop passes) where a resume starts. */
    offset: number;
    /** The virtual position at the last tick. */
    virtual: number;
    /** Loop passes completed, counted from the engine's wrapping position. */
    passes: number;
    lastPos: number;
    /** The last decoded timeline, so a seek or a resume can place itself before the next decode lands. */
    intro: number;
    loopLength: number;
}

let story: StoryAudio | null = null;
let loopState: ILoop | null = null;
let loopClock: ReturnType<typeof setInterval> | null = null;

function ensureStory(): StoryAudio {
    story ??= createStoryAudio();
    return story;
}

function applyStoryLevels(): void {
    if (!story) return;
    const s = state();
    story.setMasterMusic(s.volume);
    story.setMuted(s.muted);
}

/**
 * What the loop transport sounds for a track: an in-game track's intro and
 * loop, or, on iOS, a label track's stream as an intro with no loop, which the
 * story engine plays once and holds at its end (`tickLoop` moves on there).
 */
function cueSource(track: ITrack): { intro: string | undefined; loop: string | undefined } | null {
    const src = trackSource(track);
    if (src.kind === "loop") return { intro: src.intro, loop: src.loop };
    if (src.kind === "stream" && elementVolumeFixed()) return { intro: src.src, loop: undefined };
    return null;
}

/** Whether the current in-game track loops until stopped rather than moving on after two passes. */
function loopsForever(): boolean {
    const s = state();
    if (s.repeat === "one") return true;
    if (s.repeat === "all" || s.settings.radioOn) return false;
    return nextIndex(queueOf(s), s.repeat, true) === null;
}

/** The length the bar shows for an in-game track: the intro, then the loop `LOOP_PASSES` times. */
function loopSpan(intro: number, loopLength: number): number {
    return intro + LOOP_PASSES * loopLength;
}

function seedPasses(l: ILoop, offset: number): void {
    if (l.loopLength > 0 && offset > l.intro) {
        l.passes = Math.floor((offset - l.intro) / l.loopLength);
        l.lastPos = l.intro + ((offset - l.intro) % l.loopLength);
    } else {
        l.passes = 0;
        l.lastPos = offset;
    }
    l.virtual = offset;
}

function soundLoop(l: ILoop, offset: number): void {
    const a = ensureStory();
    if (!a.arm()) {
        // No audio context (or the browser refused one): wait for a press.
        set({ status: "paused" });
        return;
    }
    applyStoryLevels();
    if (offset > 0) a.playMusicAt(l.req, offset);
    else {
        // `playMusic` leaves the same cue alone, so a restart stops it first.
        a.stopMusic(0);
        a.playMusic(l.req);
    }
    l.sounding = true;
    seedPasses(l, offset);
    set({ status: "loading" });
    startLoopClock();
}

function startLoop(track: ITrack, offset: number, autoplay: boolean): void {
    const src = cueSource(track);
    if (!src) {
        trackFailed();
        return;
    }
    unloadDecksForSwitch();
    const sameTrack = loopState?.trackId === track.id;
    const l: ILoop = {
        trackId: track.id,
        req: { intro: src.intro, loop: src.loop, volume: 1, crossfade: 0 },
        sounding: false,
        offset,
        virtual: offset,
        passes: 0,
        lastPos: 0,
        intro: sameTrack && loopState ? loopState.intro : 0,
        loopLength: sameTrack && loopState ? loopState.loopLength : 0,
    };
    if (loopState?.sounding && !sameTrack) story?.stopMusic(0);
    loopState = l;
    transport = "loop";
    loadedId = track.id;
    const total = l.intro + l.loopLength > 0 ? loopSpan(l.intro, l.loopLength) : 0;
    publishPlayhead({ position: offset, duration: total, loopAt: l.intro > 0 && l.loopLength > 0 ? l.intro : null, buffered: 0 }, true);
    if (autoplay) soundLoop(l, offset);
    else {
        set({ status: "paused" });
        if (total === 0) probeLength(track);
    }
}

/** A label track's elements fall silent when an in-game track takes over. */
function unloadDecksForSwitch(): void {
    pauseDecks();
}

function stopLoopTransport(): void {
    if (loopState?.sounding) story?.stopMusic(LOOP_FADE_S);
    loopState = null;
    stopLoopClock();
}

function pauseLoop(): void {
    const l = loopState;
    if (!l?.sounding) return;
    // Read before the stop: the stop drops the timeline the position comes from.
    tickLoop(true);
    l.offset = l.virtual;
    l.sounding = false;
    story?.stopMusic(LOOP_FADE_S);
    stopLoopClock();
}

/** The story engine could not sound the track: silence it and fail it like a stream that errored. */
function loopFailed(l: ILoop): void {
    l.sounding = false;
    story?.stopMusic(0);
    stopLoopClock();
    trackFailed();
}

function startLoopClock(): void {
    if (loopClock !== null || typeof window === "undefined") return;
    loopClock = setInterval(() => tickLoop(false), PLAYHEAD_MS);
}

function stopLoopClock(): void {
    if (loopClock === null) return;
    clearInterval(loopClock);
    loopClock = null;
}

/** One reading of the story engine's clock: heard time, the virtual position, and the advance after two passes (or at a one-shot's end). */
function tickLoop(force: boolean): void {
    const l = loopState;
    const a = story;
    if (!l?.sounding || !a || transport !== "loop") return;
    const length = a.musicLength;
    if (length <= 0) {
        // Still decoding, or the engine fell back to a media element, which keeps no timeline.
        // Only a fallback whose play() resolved counts as playing; one that rejected (iOS
        // refuses it outside the gesture) or found nothing to decode is a failed track, where
        // the sticky `lastFallback` used to show "playing" over silence.
        const heard = a.musicStatus;
        if (heard === "failed") loopFailed(l);
        else if (heard === "ended" && !force) advance(true);
        else if (heard === "playing" && state().status === "loading") set({ status: "playing", failures: 0 });
        return;
    }
    const intro = a.musicIntroSeconds;
    const loopLength = length - intro;
    if (l.intro !== intro || l.loopLength !== loopLength) {
        const first = l.loopLength === 0;
        l.intro = intro;
        l.loopLength = loopLength;
        if (first) seedPasses(l, l.virtual);
    }
    if (state().status !== "playing") {
        set({ status: "playing", failures: 0 });
        mediaPlaybackState();
    }
    const pos = a.musicPosition;
    if (loopLength > 0 && pos + 0.5 < l.lastPos) l.passes += 1;
    l.lastPos = pos;
    const virtual = pos < intro ? pos : intro + l.passes * loopLength + (pos - intro);
    const dt = virtual - l.virtual;
    if (dt > 0 && dt < MAX_HEARD_STEP_S) addHeard(dt);
    l.virtual = virtual;
    const total = loopSpan(intro, loopLength);
    // A one-shot cue has nothing to loop: it moves on at its end whatever comes next, like a deck's `ended`.
    const forever = loopLength > 0 && loopsForever();
    if (!forever && virtual >= total && !force) {
        advance(true);
        return;
    }
    // Looping until stopped: the bar wraps within the same intro-plus-two-passes span.
    const shown = forever && virtual >= intro && loopLength > 0 ? intro + ((virtual - intro) % (LOOP_PASSES * loopLength)) : Math.min(virtual, total);
    publishPlayhead({ position: shown, duration: total, loopAt: intro > 0 && loopLength > 0 ? intro : null, buffered: 0 }, force);
}

function seekLoop(seconds: number): void {
    const l = loopState;
    if (!l) return;
    const total = l.intro + l.loopLength > 0 ? loopSpan(l.intro, l.loopLength) : Number.POSITIVE_INFINITY;
    const to = Math.max(0, Math.min(seconds, total - 0.25));
    if (l.sounding && story) story.playMusicAt(l.req, to);
    else l.offset = to;
    seedPasses(l, to);
    publishPlayhead({ position: to }, true);
}

/**
 * A restored session, or an in-game track made current while paused, has no
 * transport that knows its length: a label track's deck is not loaded yet and
 * an in-game track's timeline only exists once the story engine decodes it.
 * The bar then read "0:01 / 0:00" with the seek slider disabled (Ad astra,
 * restored at 1.8 s, 2026-10-11). This reads the length from media metadata
 * alone, without playing: a label track's WAV once, an in-game track's intro
 * and loop each, totalled the way the loop transport does
 * (`intro + LOOP_PASSES * loop`). The transport's own figure replaces it as
 * soon as playback starts.
 */
const PROBE_TIMEOUT_MS = 15_000;
const probedLength = new Map<string, { duration: number; loopAt: number | null }>();
let probeToken = 0;

function mediaDuration(src: string): Promise<number> {
    if (typeof Audio === "undefined") return Promise.resolve(0);
    return new Promise((resolve) => {
        const el = new Audio();
        let timer: ReturnType<typeof setTimeout> | null = null;
        const done = (seconds: number) => {
            if (timer !== null) clearTimeout(timer);
            timer = null;
            el.removeAttribute("src");
            try {
                el.load();
            } catch {
                // Releasing the request is best effort.
            }
            resolve(seconds);
        };
        el.preload = "metadata";
        el.addEventListener("loadedmetadata", () => done(Number.isFinite(el.duration) && el.duration > 0 ? el.duration : 0), { once: true });
        el.addEventListener("error", () => done(0), { once: true });
        timer = setTimeout(() => done(0), PROBE_TIMEOUT_MS);
        el.src = src;
    });
}

function publishProbed(trackId: string, found: { duration: number; loopAt: number | null }): void {
    // Only for the track still current, and never over a length a transport already reported.
    if (state().current?.id !== trackId || playheadStore.state.duration > 0) return;
    publishPlayhead({ duration: found.duration, loopAt: found.loopAt }, true);
}

function probeLength(track: ITrack): void {
    const known = probedLength.get(track.id);
    if (known) {
        publishProbed(track.id, known);
        return;
    }
    const src = trackSource(track);
    const token = ++probeToken;
    let found: Promise<{ duration: number; loopAt: number | null } | null>;
    if (src.kind === "stream") found = mediaDuration(src.src).then((d) => (d > 0 ? { duration: d, loopAt: null } : null));
    else if (src.kind === "loop") found = Promise.all([src.intro ? mediaDuration(src.intro) : Promise.resolve(0), mediaDuration(src.loop)]).then(([intro, loop]) => (loop > 0 ? { duration: loopSpan(intro, loop), loopAt: intro > 0 ? intro : null } : null));
    else return;
    void found.then((r) => {
        if (!r) return;
        probedLength.set(track.id, r);
        if (token === probeToken) publishProbed(track.id, r);
    });
}

function startTrack(track: ITrack, offset: number, autoplay: boolean): void {
    const kind = trackSource(track).kind;
    if (kind === "stream" && !elementVolumeFixed()) startStream(track, offset, autoplay);
    else if (kind !== "none") startLoop(track, offset, autoplay);
    else {
        loadedId = null;
        trackFailed();
    }
}

/** Make `i` the current track and load it; `autoplay` plays it. */
function startIndex(i: number, autoplay: boolean, offset = 0): void {
    const track = state().queue[i];
    if (!track) return;
    finalizeListen();
    loadToken += 1;
    pendingOffset = 0;
    set({ index: i, current: track, status: autoplay ? "loading" : "paused" });
    beginListen(track.id);
    if (autoplay) {
        armAudio();
        silenceOtherMusic();
    }
    startTrack(track, offset, autoplay);
}

/** The queue ran out: append radio tracks when Radio is on, and return the index to play next. */
function extendWithRadio(): number | null {
    const s = state();
    if (!s.settings.radioOn || !catalog || !s.current) return null;
    const queued = new Set(s.queue.map((t) => t.id));
    const extra = radioFor(catalog, s.current, queued, RADIO_ASK).slice(0, RADIO_TAKE);
    if (extra.length === 0) return null;
    const q = insertEnd(queueOf(s), extra, s.shuffle);
    const source: IQueueSource = { kind: "radio", name: s.current.albumName ?? s.current.title, id: s.current.albumId ?? undefined };
    set({ queue: q.items, order: q.order, source });
    return nextIndex(queueOf(state()), state().repeat, false);
}

/** Move on: `auto` is the track ending by itself (repeat-one holds it), otherwise a press of Next. */
function advance(auto: boolean): void {
    const s = state();
    const ni = nextOrRadio(s, auto);
    if (ni === null) {
        endOfQueue();
        return;
    }
    startIndex(ni, auto ? true : wantsPlay() || s.status === "error");
}

/** Repeat is off and nothing comes next: stop on the last track, rewound. */
function endOfQueue(): void {
    finalizeListen();
    if (transport === "stream" && decks) {
        pauseDecks();
        decks[active].el.currentTime = 0;
        decks[active].lastTime = 0;
    } else if (transport === "loop" && loopState) {
        pauseLoop();
        loopState.offset = 0;
        seedPasses(loopState, 0);
    }
    set({ status: "paused" });
    publishPlayhead({ position: 0 }, true);
    mediaPlaybackState();
}

function trackFailed(): void {
    if (failureHandledFor === loadToken) return;
    failureHandledFor = loadToken;
    const s = state();
    const track = s.current;
    const failures = s.failures + 1;
    if (failures >= MAX_FAILURES) {
        pauseTransports();
        set({ failures, status: "error" });
        notify({ kind: "stopped", count: failures });
        return;
    }
    set({ failures });
    if (track) notify({ kind: "skip", title: track.title });
    const ni = nextOrRadio(s, false);
    if (ni === null || ni === s.index) {
        pauseTransports();
        set({ status: "error" });
        return;
    }
    startIndex(ni, true);
}

function pauseTransports(): void {
    if (transport === "stream") pauseDecks();
    else if (transport === "loop") pauseLoop();
}

let armed = false;

/** Create and resume both audio contexts inside a user gesture, so later starts are allowed to sound. */
function armAudio(): void {
    if (typeof window === "undefined") return;
    ensureContext();
    ensureStory().arm();
    armed = true;
}

/** The page hands over the latest catalogue: radio reads it, and a saved session waits for it. */
export function setCatalog(next: ICatalog | null): void {
    catalog = next;
    const houseArtist = next?.houseArtist ?? null;
    if (state().houseArtist !== houseArtist) set({ houseArtist });
    tryRestore();
}

export function playQueue(tracks: readonly ITrack[], from: number, source: IQueueSource): void {
    if (tracks.length === 0) return;
    const items = [...tracks];
    const start = clamp(Math.floor(from) || 0, 0, items.length - 1);
    const order = playOrder(items.length, start, state().shuffle);
    set({ queue: items, order, index: start, source, failures: 0, restored: true });
    startIndex(start, true);
}

export function playAt(queueIndex: number): void {
    if (!state().queue[queueIndex]) return;
    set({ failures: 0 });
    startIndex(queueIndex, true);
}

/** Into an empty queue, enqueued tracks are selected but not played. */
function seedQueue(tracks: readonly ITrack[]): void {
    const items = [...tracks];
    set({ queue: items, order: playOrder(items.length, 0, state().shuffle), index: 0, current: items[0] ?? null, status: "paused", source: { kind: "none" }, restored: true });
    loadedId = null;
    pendingOffset = 0;
    publishPlayhead({ ...EMPTY_PLAYHEAD }, true);
}

export function enqueueNext(tracks: readonly ITrack[]): void {
    if (tracks.length === 0) return;
    const s = state();
    if (s.queue.length === 0) {
        seedQueue(tracks);
        return;
    }
    const q = insertNext(queueOf(s), tracks);
    set({ queue: q.items, order: q.order });
    schedulePreload();
}

export function enqueueEnd(tracks: readonly ITrack[]): void {
    if (tracks.length === 0) return;
    const s = state();
    if (s.queue.length === 0) {
        seedQueue(tracks);
        return;
    }
    const q = insertEnd(queueOf(s), tracks, s.shuffle);
    set({ queue: q.items, order: q.order });
    schedulePreload();
}

export function removeFromQueue(queueIndex: number): void {
    const s = state();
    if (!s.queue[queueIndex]) return;
    const r = removeAt(queueOf(s), queueIndex);
    if (r.items.length === 0) {
        clearQueue();
        return;
    }
    const playing = wantsPlay();
    set({ queue: r.items, order: r.order, index: r.index, current: r.items[r.index] ?? null });
    if (r.removedCurrent) {
        if (r.hasNext) startIndex(r.index, playing);
        else {
            pauseTransports();
            startIndex(r.index, false);
        }
        return;
    }
    schedulePreload();
}

export function reorderQueue(from: number, to: number): void {
    const s = state();
    const q = moveItem(queueOf(s), from, to, s.shuffle);
    if (q === queueOf(s) || q.items === s.queue) return;
    set({ queue: q.items, order: q.order, index: q.index });
    schedulePreload();
}

export function clearQueue(): void {
    finalizeListen();
    stopLoopTransport();
    unloadDecks();
    transport = null;
    loadedId = null;
    pendingOffset = 0;
    loadToken += 1;
    set({ queue: [], order: [], index: -1, current: null, status: "idle", source: { kind: "none" }, failures: 0, panel: null, lyrics: NO_LYRICS });
    publishPlayhead({ ...EMPTY_PLAYHEAD }, true);
    saveSession();
    mediaClear();
}

export function togglePlay(): void {
    if (wantsPlay()) pause();
    else play();
}

export function play(): void {
    const s = state();
    if (s.queue.length === 0) return;
    armAudio();
    // One music at a time: a story theme paused the player, and the player resuming stops it.
    silenceOtherMusic();
    if (!s.current || s.index < 0) {
        startIndex(s.order[0] ?? 0, true);
        return;
    }
    if (s.failures >= MAX_FAILURES || s.status === "error") set({ failures: 0 });
    if (loadedId !== s.current.id || transport === null) {
        const offset = pendingOffset;
        pendingOffset = 0;
        loadToken += 1;
        beginListen(s.current.id);
        set({ status: "loading" });
        startTrack(s.current, offset, true);
        return;
    }
    if (transport === "stream" && decks) {
        set({ status: "loading" });
        playDeck(active);
    } else if (transport === "loop" && loopState) {
        if (!loopState.sounding) soundLoop(loopState, loopState.offset);
    }
}

export function pause(): void {
    const s = state();
    if (!s.current) return;
    pauseTransports();
    if (s.status !== "idle") set({ status: "paused" });
    flushListen();
    mediaPlaybackState();
}

export function stop(): void {
    pause();
    seek(0);
}

export function next(): void {
    if (state().queue.length === 0) return;
    advance(false);
}

export function previous(): void {
    const s = state();
    if (s.queue.length === 0) return;
    if (playheadStore.state.position > RESTART_AFTER_S) {
        if (transport === "loop" && loopState?.sounding) {
            loopState.offset = 0;
            soundLoop(loopState, 0);
        } else seek(0);
        return;
    }
    const pi = previousIndex(queueOf(s), s.repeat);
    if (pi === null) {
        seek(0);
        return;
    }
    startIndex(pi, wantsPlay());
}

export function seek(seconds: number): void {
    const s = state();
    if (!s.current || !Number.isFinite(seconds)) return;
    const target = Math.max(0, seconds);
    if (loadedId !== s.current.id || transport === null) {
        pendingOffset = target;
        publishPlayhead({ position: target }, true);
        return;
    }
    if (transport === "stream" && decks) {
        cancelFade();
        const deck = decks[active];
        const d = deck.el.duration;
        const to = Number.isFinite(d) && d > 0 ? Math.min(target, Math.max(0, d - 0.05)) : target;
        if (deck.el.readyState >= 1) deck.el.currentTime = to;
        else deck.pendingSeek = to;
        deck.lastTime = to;
        publishPlayhead({ position: to }, true);
        return;
    }
    if (transport === "loop") seekLoop(target);
}

export function skipBy(seconds: number): void {
    seek(playheadStore.state.position + seconds);
}

function applyLevels(): void {
    if (decks && !fade) deckLevel(active, 1);
    applyStoryLevels();
}

export function setVolume(volume: number): void {
    if (!Number.isFinite(volume)) return;
    set({ volume: clamp01(volume), muted: false });
    applyLevels();
    saveVolume();
}

export function toggleMute(): void {
    set({ muted: !state().muted });
    applyLevels();
    saveVolume();
}

/** 100% -> 40% -> silent -> 100%, by the level the listener hears now. */
export function cycleVolume(): void {
    const level = userLevel();
    if (level > 0.4 + 1e-6) setVolume(0.4);
    else if (level > 0) setVolume(0);
    else setVolume(1);
}

export function toggleShuffle(): void {
    const s = state();
    const shuffle = !s.shuffle;
    const n = s.queue.length;
    set({ shuffle, order: playOrder(n, s.index, shuffle) });
    schedulePreload();
}

export function cycleRepeat(): void {
    const order: RepeatMode[] = ["off", "all", "one"];
    setRepeat(order[(order.indexOf(state().repeat) + 1) % order.length] ?? "off");
}

export function setRepeat(mode: RepeatMode): void {
    set({ repeat: mode });
    schedulePreload();
}

function cleanSettings(raw: Partial<IPlayerSettings> | null | undefined): IPlayerSettings {
    const v = raw && typeof raw === "object" ? raw : {};
    const bool = (x: unknown, d: boolean) => (typeof x === "boolean" ? x : d);
    const seconds = typeof v.crossfadeSeconds === "number" && Number.isFinite(v.crossfadeSeconds) ? Math.round(clamp(v.crossfadeSeconds, CROSSFADE_MIN, CROSSFADE_MAX)) : DEFAULT_SETTINGS.crossfadeSeconds;
    const preload: PreloadChoice = v.preload === "on" || v.preload === "off" || v.preload === "auto" ? v.preload : DEFAULT_SETTINGS.preload;
    return {
        crossfadeOn: bool(v.crossfadeOn, DEFAULT_SETTINGS.crossfadeOn),
        crossfadeSeconds: seconds,
        radioOn: bool(v.radioOn, DEFAULT_SETTINGS.radioOn),
        levelingOn: bool(v.levelingOn, DEFAULT_SETTINGS.levelingOn),
        preload,
        resumePosition: bool(v.resumePosition, DEFAULT_SETTINGS.resumePosition),
    };
}

export function updateSettings(patch: Partial<IPlayerSettings>): void {
    const settings = cleanSettings({ ...state().settings, ...patch });
    set({ settings });
    writeStoredValue(SETTINGS_KEY, settings);
    if (decks) for (const d of decks) applyLeveling(d);
    schedulePreload();
}

let sleepClock: ReturnType<typeof setInterval> | null = null;

function stopSleepClock(): void {
    if (sleepClock !== null) clearInterval(sleepClock);
    sleepClock = null;
}

export function setSleepTimer(minutes: SleepMinutes): void {
    stopSleepClock();
    if (!minutes) {
        set({ sleepEndsAt: null });
        return;
    }
    set({ sleepEndsAt: Date.now() + minutes * 60_000 });
    if (typeof window === "undefined") return;
    sleepClock = setInterval(() => {
        const ends = state().sleepEndsAt;
        if (ends === null || Date.now() < ends) return;
        stopSleepClock();
        set({ sleepEndsAt: null });
        pause();
    }, 1000);
}

export function openPanel(panel: PanelName | null): void {
    set({ panel });
}

export function togglePanel(panel: PanelName): void {
    set({ panel: state().panel === panel ? null : panel });
}

export function setSettingsOpen(open: boolean): void {
    set({ settingsOpen: open });
}

export function setShortcutsOpen(open: boolean): void {
    set({ shortcutsOpen: open });
}

interface ISession {
    trackIds: string[];
    index: number;
    position: number;
    shuffle: boolean;
    order: number[];
    repeat: RepeatMode;
    source: IQueueSource;
    savedAt: number;
}

function saveVolume(): void {
    const s = state();
    writeStoredValue(VOLUME_KEY, { volume: s.volume, muted: s.muted });
}

function parseSession(raw: string): ISession | undefined {
    const v = JSON.parse(raw) as Partial<ISession> | null;
    if (!v || typeof v !== "object" || !Array.isArray(v.trackIds)) return undefined;
    const trackIds = v.trackIds.filter((x): x is string => typeof x === "string");
    const repeat: RepeatMode = v.repeat === "all" || v.repeat === "one" ? v.repeat : "off";
    const source: IQueueSource = v.source && typeof v.source === "object" && typeof v.source.kind === "string" ? v.source : { kind: "none" };
    return {
        trackIds,
        index: typeof v.index === "number" && Number.isInteger(v.index) ? v.index : 0,
        position: typeof v.position === "number" && Number.isFinite(v.position) && v.position > 0 ? v.position : 0,
        shuffle: v.shuffle === true,
        order: Array.isArray(v.order) ? v.order : [],
        repeat,
        source,
        savedAt: typeof v.savedAt === "number" ? v.savedAt : 0,
    };
}

/** Read settings, volume and the saved session once per page load. Nothing sounds. */
function hydrate(): void {
    if (hydrated || typeof window === "undefined") return;
    hydrated = true;
    const settings = readStoredValue<IPlayerSettings>(SETTINGS_KEY, DEFAULT_SETTINGS, (raw) => cleanSettings(JSON.parse(raw) as Partial<IPlayerSettings>));
    const vol = readStoredValue<{ volume: number; muted: boolean }>(VOLUME_KEY, { volume: 1, muted: false }, (raw) => {
        const v = JSON.parse(raw) as { volume?: unknown; muted?: unknown } | null;
        if (!v || typeof v !== "object") return undefined;
        return { volume: typeof v.volume === "number" && Number.isFinite(v.volume) ? clamp01(v.volume) : 1, muted: v.muted === true };
    });
    set({ settings, volume: vol.volume, muted: vol.muted });
    pendingSession = readStoredValue<ISession | null>(SESSION_KEY, null, parseSession);
}

/** The catalogue has the kinds the saved session names; until then the restore waits. */
function catalogCovers(session: ISession, c: ICatalog): boolean {
    const needsSiren = session.trackIds.some((id) => id.startsWith("siren:"));
    const needsGame = session.trackIds.some((id) => id.startsWith("game:"));
    return (!needsSiren || c.sirenReady) && (!needsGame || c.gameReady);
}

function tryRestore(): void {
    if (pendingSession === undefined || state().restored) return;
    const session = pendingSession;
    if (session === null || session.trackIds.length === 0 || state().queue.length > 0) {
        pendingSession = null;
        if (catalog || state().queue.length > 0) set({ restored: true });
        return;
    }
    const c = catalog;
    if (!c || !catalogCovers(session, c)) return;
    pendingSession = null;
    // Drop ids the catalogue no longer has, and renumber the order and the index around them.
    const keep: number[] = [];
    const items: ITrack[] = [];
    session.trackIds.forEach((id, i) => {
        const t = c.byId.get(id);
        if (t) {
            keep.push(i);
            items.push(t);
        }
    });
    if (items.length === 0) {
        set({ restored: true });
        return;
    }
    const newIndexOf = new Map(keep.map((oldI, newI) => [oldI, newI]));
    const oldOrder = validOrder(session.order, session.trackIds.length);
    let order = oldOrder ? oldOrder.filter((i) => newIndexOf.has(i)).map((i) => newIndexOf.get(i) as number) : null;
    const keptCurrent = newIndexOf.get(session.index);
    const index = keptCurrent ?? 0;
    if (!order || validOrder(order, items.length) === null) order = playOrder(items.length, index, session.shuffle);
    if (!session.shuffle) order = identityOrder(items.length);
    const position = keptCurrent !== undefined && state().settings.resumePosition ? session.position : 0;
    loadedId = null;
    transport = null;
    pendingOffset = position;
    set({ queue: items, order, index, current: items[index] ?? null, shuffle: session.shuffle, repeat: session.repeat, source: session.source, status: "paused", restored: true });
    publishPlayhead({ position, duration: 0, loopAt: null, buffered: 0 }, true);
    const restoredTrack = items[index];
    if (restoredTrack) probeLength(restoredTrack);
}

function saveSession(): void {
    const s = state();
    if (!s.restored || typeof window === "undefined") return;
    const session: ISession = {
        trackIds: s.queue.map((t) => t.id),
        index: s.index,
        position: Math.max(0, Math.round(playheadStore.state.position * 10) / 10),
        shuffle: s.shuffle,
        order: s.order,
        repeat: s.repeat,
        source: s.source,
        savedAt: Date.now(),
    };
    try {
        writeStoredValue(SESSION_KEY, session);
    } catch {
        // A full or blocked storage loses the session, not the playback.
    }
}

const lyricsCache = new Map<string, ILyricLine[]>();
let lyricsAbort: AbortController | null = null;

/** The lyrics state for `trackId` once its lines are in: an LRC with no timed line reads as empty. */
function lyricsOf(trackId: string, lines: ILyricLine[]): IPlayerState["lyrics"] {
    return { trackId, status: lines.length > 0 ? "ready" : "empty", lines };
}

function cacheLyrics(id: string, lines: ILyricLine[]): void {
    lyricsCache.delete(id);
    lyricsCache.set(id, lines);
    while (lyricsCache.size > LYRICS_CACHE_MAX) {
        const oldest = lyricsCache.keys().next().value;
        if (oldest === undefined) break;
        lyricsCache.delete(oldest);
    }
}

function loadLyrics(track: ITrack | null): void {
    lyricsAbort?.abort();
    lyricsAbort = null;
    if (!track || track.kind !== "siren" || !track.hasLyrics) {
        set({ lyrics: { trackId: track?.id ?? null, status: "none", lines: [] } });
        return;
    }
    const cached = lyricsCache.get(track.id);
    if (cached) {
        cacheLyrics(track.id, cached);
        set({ lyrics: lyricsOf(track.id, cached) });
        return;
    }
    if (typeof fetch !== "function") return;
    set({ lyrics: { trackId: track.id, status: "loading", lines: [] } });
    const ctrl = new AbortController();
    lyricsAbort = ctrl;
    fetch(sirenLyricsUrl(track.key), { signal: ctrl.signal })
        .then(async (res) => {
            if (!res.ok) throw new Error(`lyrics ${res.status}`);
            const lines = parseLrc(await res.text());
            cacheLyrics(track.id, lines);
            if (state().current?.id === track.id) set({ lyrics: lyricsOf(track.id, lines) });
        })
        .catch((err: unknown) => {
            if (errorName(err) === "AbortError") return;
            if (state().current?.id === track.id) set({ lyrics: { trackId: track.id, status: "failed", lines: [] } });
        });
}

function mediaSession(): MediaSession | null {
    if (typeof navigator === "undefined") return null;
    return (navigator as Navigator & { mediaSession?: MediaSession }).mediaSession ?? null;
}

function mediaMetadata(track: ITrack | null): void {
    const ms = mediaSession();
    if (!ms) return;
    try {
        if (!track || typeof MediaMetadata === "undefined") {
            ms.metadata = null;
            return;
        }
        ms.metadata = new MediaMetadata({
            title: track.title,
            artist: displayArtists(track, catalog?.houseArtist ?? null).join(", "),
            album: track.albumName ?? "",
            artwork: track.coverUrl ? [{ src: track.coverUrl }] : [],
        });
    } catch {
        // Unsupported fields or a browser without the API.
    }
}

function mediaPlaybackState(): void {
    const ms = mediaSession();
    if (!ms) return;
    try {
        ms.playbackState = wantsPlay() ? "playing" : state().current ? "paused" : "none";
    } catch {
        // Ignored where unsupported.
    }
}

function mediaPositionState(): void {
    const ms = mediaSession();
    if (!ms || typeof ms.setPositionState !== "function") return;
    const p = playheadStore.state;
    try {
        if (p.duration > 0) ms.setPositionState({ duration: p.duration, position: Math.min(p.position, p.duration), playbackRate: 1 });
    } catch {
        // A position past the duration or an unsupported call.
    }
}

function mediaClear(): void {
    mediaMetadata(null);
    mediaPlaybackState();
}

function bindMediaSession(): () => void {
    const ms = mediaSession();
    if (!ms) return () => {};
    const handlers: [MediaSessionAction, MediaSessionActionHandler][] = [
        ["play", () => play()],
        ["pause", () => pause()],
        ["previoustrack", () => previous()],
        ["nexttrack", () => next()],
        ["stop", () => stop()],
        ["seekto", (d) => (typeof d.seekTime === "number" ? seek(d.seekTime) : undefined)],
        ["seekbackward", (d) => skipBy(-(d.seekOffset ?? SKIP_S))],
        ["seekforward", (d) => skipBy(d.seekOffset ?? SKIP_S)],
    ];
    for (const [action, fn] of handlers) {
        try {
            ms.setActionHandler(action, fn);
        } catch {
            // This browser does not support the action.
        }
    }
    return () => {
        for (const [action] of handlers) {
            try {
                ms.setActionHandler(action, null);
            } catch {
                // Same.
            }
        }
    };
}

// Lyrics, Media Session metadata and the saved session follow the store.

let watchedTrackId: string | null = null;
let watched: Pick<IPlayerState, "queue" | "index" | "order" | "shuffle" | "repeat" | "source" | "status"> | null = null;
let saveTimer: ReturnType<typeof setTimeout> | null = null;

function saveSessionSoon(): void {
    if (typeof window === "undefined" || saveTimer !== null) return;
    saveTimer = setTimeout(() => {
        saveTimer = null;
        saveSession();
    }, 200);
}

/** Asked to sound since the queue last emptied: what makes the player bar follow the visitor off the music page. */
let engaged = false;

playerStore.subscribe((s: IPlayerState) => {
    if (s.current === null) engaged = false;
    else if (s.status === "loading" || s.status === "playing") engaged = true;
    setMusicLive(engaged);
    const id = s.current?.id ?? null;
    if (id !== watchedTrackId) {
        watchedTrackId = id;
        loadLyrics(s.current);
        mediaMetadata(s.current);
    }
    if (!watched || watched.queue !== s.queue || watched.index !== s.index || watched.order !== s.order || watched.shuffle !== s.shuffle || watched.repeat !== s.repeat || watched.source !== s.source || watched.status !== s.status) {
        const statusChanged = watched?.status !== s.status;
        watched = { queue: s.queue, index: s.index, order: s.order, shuffle: s.shuffle, repeat: s.repeat, source: s.source, status: s.status };
        if (statusChanged) mediaPlaybackState();
        saveSessionSoon();
    }
});

function isTypingTarget(target: EventTarget | null): boolean {
    const el = target as HTMLElement | null;
    if (!el || typeof el.tagName !== "string") return false;
    const tag = el.tagName.toLowerCase();
    return tag === "input" || tag === "textarea" || tag === "select" || el.isContentEditable === true || el.closest?.("[contenteditable=''],[contenteditable='true']") != null;
}

function isPressable(target: EventTarget | null): boolean {
    const el = target as HTMLElement | null;
    if (!el || typeof el.tagName !== "string") return false;
    const tag = el.tagName.toLowerCase();
    return tag === "button" || tag === "a" || el.getAttribute?.("role") === "button" || el.getAttribute?.("role") === "link";
}

/** The window's keydown, exported for the tests; `useMusicPlayerLifetime` binds it. */
export function handleKeydown(e: KeyboardEvent): void {
    if (e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey || e.isComposing) return;
    if (isTypingTarget(e.target)) return;
    const s = state();
    const key = e.key;
    if (key === "?") {
        e.preventDefault();
        setShortcutsOpen(!s.shortcutsOpen);
        return;
    }
    if (key === "Escape") {
        if (s.shortcutsOpen) setShortcutsOpen(false);
        else if (s.settingsOpen) setSettingsOpen(false);
        else if (s.panel) openPanel(null);
        return;
    }
    if (s.queue.length === 0) return;
    const lower = key.length === 1 ? key.toLowerCase() : key;
    let handled = true;
    if (e.shiftKey) {
        if (lower === "n") next();
        else if (lower === "p") previous();
        else if (lower === "l") {
            if (s.current) toggleLike(s.current.id);
        } else handled = false;
    } else if (key === " " || e.code === "Space") {
        if (isPressable(e.target)) return;
        togglePlay();
    } else if (lower === "k") togglePlay();
    else if (lower === "j") skipBy(-SKIP_S);
    else if (lower === "l") skipBy(SKIP_S);
    else if (lower === "m") toggleMute();
    else if (lower === "s") toggleShuffle();
    else if (lower === "r") cycleRepeat();
    else if (lower === "y") togglePanel("lyrics");
    else if (lower === "q") togglePanel("queue");
    else handled = false;
    if (handled) e.preventDefault();
}

export function usePlayer<T>(selector: (s: IPlayerState) => T): T {
    return useStore(playerStore, selector);
}

export function usePlayhead(): IPlayhead {
    return useStore(playheadStore, (s) => s);
}

/** True when `trackId` is the current track (playing or paused). */
export function useIsCurrent(trackId: string | null): boolean {
    return useStore(playerStore, (s) => trackId !== null && s.current?.id === trackId);
}

type PlayerT = TypedT<typeof messages>;

// A story's music pauses the player rather than sounding over it (see `./presence`).
registerMusicPauser(pause);

/** How many mounted views hold the site-wide lifetime: the music page and the root's dock host. */
let lifetimeHolders = 0;
let releaseLifetime: (() => void) | null = null;
/** The newest `t` a holder rendered with, for the toasts. */
let noticeT: PlayerT | null = null;

/** Bind what playback needs for as long as anything holds it, whichever page is showing. */
function installLifetime(): () => void {
    hydrate();
    tryRestore();
    notifier = (n) => {
        // Named `t` so the i18n extractor counts these keys as used.
        const t = noticeT;
        if (!t) return;
        if (n.kind === "skip") toastManager.add({ title: t("music.player.error.skip", { title: n.title ?? "" }), type: "error" });
        else toastManager.add({ title: t("music.player.error.stopped", { count: n.count ?? MAX_FAILURES }), type: "error" });
    };
    const unbindMedia = bindMediaSession();
    mediaMetadata(state().current);
    mediaPlaybackState();
    // Both audio contexts are created and resumed inside the first gesture, so a later
    // automatic start (the next track, a crossfade) is allowed to sound.
    const onGesture = () => {
        if (armed) return;
        armAudio();
    };
    const onHide = () => {
        flushListen();
        saveSession();
    };
    const onVisibility = () => {
        if (document.visibilityState === "hidden") onHide();
    };
    window.addEventListener("pointerdown", onGesture, { once: false, passive: true });
    window.addEventListener("pagehide", onHide);
    document.addEventListener("visibilitychange", onVisibility);
    const saver = setInterval(() => {
        if (wantsPlay()) saveSession();
    }, SESSION_SAVE_MS);
    return () => {
        window.removeEventListener("pointerdown", onGesture);
        window.removeEventListener("pagehide", onHide);
        document.removeEventListener("visibilitychange", onVisibility);
        clearInterval(saver);
        unbindMedia();
        notifier = null;
        saveSession();
    };
}

/**
 * The site-wide half of the player's lifetime: the session restore, the Media
 * Session, gesture arming, saving on hide, the error toasts. Held by the music
 * page and by the root's dock host, so it outlives a navigation away
 * from the page and is released only when neither is mounted. It never
 * pauses: leaving the page keeps the music playing.
 */
export function useMusicSessionLifetime(): void {
    const t: PlayerT = useT("gallery");
    useEffect(() => {
        noticeT = t;
    }, [t]);

    useEffect(() => {
        lifetimeHolders += 1;
        if (lifetimeHolders === 1) releaseLifetime = installLifetime();
        return () => {
            lifetimeHolders -= 1;
            if (lifetimeHolders > 0) return;
            releaseLifetime?.();
            releaseLifetime = null;
        };
    }, []);
}

/** Mount once on the music page: the site-wide lifetime plus the page's keyboard shortcuts. */
export function useMusicPlayerLifetime(): void {
    useMusicSessionLifetime();
    useEffect(() => {
        window.addEventListener("keydown", handleKeydown);
        return () => window.removeEventListener("keydown", handleKeydown);
    }, []);
}

/** Test seam: forget everything the module holds, as a fresh page load would. */
export function resetPlayerForTests(): void {
    clearQueue();
    stopLoopClock();
    stopSleepClock();
    if (saveTimer !== null) clearTimeout(saveTimer);
    saveTimer = null;
    catalog = null;
    pendingSession = undefined;
    hydrated = false;
    engaged = false;
    armed = false;
    decks = null;
    active = 0;
    ctx = null;
    graphBroken = false;
    volumeFixed = null;
    story = null;
    lyricsCache.clear();
    probedLength.clear();
    probeToken = 0;
    playerStore.setState(() => INITIAL);
    playheadStore.setState(() => EMPTY_PLAYHEAD);
}
