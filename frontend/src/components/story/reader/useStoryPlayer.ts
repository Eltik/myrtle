import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import type { StoryAudio } from "#/lib/story/audio";
import { type CameraShake, createEngine, type Effect, type Engine, type Halt, type StepResult, type UnresolvedAsset } from "#/lib/story/engine";
import { loadProgress, type StoryPosition, saveProgress, withoutPosition, withPosition, withRead } from "#/lib/story/progress";
import type { Frame } from "#/lib/story/scene";
import { plainStoryText, renderLine } from "#/lib/story/text";
import type { StoryScript } from "#/types/generated/StoryScript";
import { firstWords, type HaltSummary } from "./scrub";

export type Phase = "resume" | "title" | "reading" | "end";

export interface BacklogEntry {
    haltIndex: number;
    /** `cutscene` is a video halt, which has no line of its own to log. */
    kind: "line" | "choice" | "cutscene";
    speaker?: string;
    text: string;
    isNarration: boolean;
}

interface Playback {
    frames: Frame[];
    index: number;
}

export interface StoryPlayer {
    phase: Phase;
    halt: Halt | null;
    /** The frame the renderer shows now. */
    frame: Frame | null;
    /** True while the step's timeline is still being played out. */
    playing: boolean;
    haltIndex: number;
    totalHalts: number;
    backlog: BacklogEntry[];
    unhandledKinds: Record<string, number>;
    /** Names the wire `assets` maps have no entry for: each one is a request NOT made. */
    unresolvedAssets: UnresolvedAsset[];
    /** Increments on every halt, so the text box restarts its reveal. */
    revealKey: number;
    /** From the title or resume card: begin at line 0. */
    start(): void;
    /** From the resume card: replay to the saved halt. */
    resume(): void;
    restart(): void;
    /** Skip the timeline, or step to the next halt. Ignored on a decision. */
    advance(): void;
    choose(value: string): void;
    back(): void;
    /** The saved position offered by the resume card, when there is one. */
    savedHalt: number | null;
    /**
     * Every halt in the script, with the first words of its line: what the
     * scrubber's tooltip reads and what a backlog jump lands on. One probe
     * walk per script, memoised.
     */
    haltSummaries: HaltSummary[];
    /** Jump to a halt, replaying the scene from the start with `choices`. */
    jumpTo(target: number): void;
}

interface Options {
    script: StoryScript;
    storyId: string;
    nickname: string;
    audio: StoryAudio;
    /** `?halt=N` from the URL: replay straight to that halt. */
    initialHalt?: number;
    /**
     * `AVGController.animateRatio`: every scene duration is multiplied by it.
     * A settings change retunes the live engine instead of rebuilding it, so
     * the reader keeps its place.
     */
    animateRatio: number;
    /** `?legacyclamp=1`: the pre-parity 1.5 s clamps, for A/B only. */
    legacyClamp?: boolean;
    /** `?firstright=1`: the pre-capture legacy `[character]` sides, first name on the right. */
    firstNameRight?: boolean;
    /**
     * `?video=0` and the "Play cutscene videos" setting. Default on; only an
     * explicit `false` turns cutscenes off. It rebuilds the engine, because
     * the halt COUNT changes with it.
     */
    videos?: boolean;
    /** The cutscene label the log and the scrubber read. */
    cutsceneLabel: string;
    /** A `[camerashake]` reached the renderer. */
    onShake?: (shake: CameraShake) => void;
}

export function useStoryPlayer({ script, storyId, nickname, audio, initialHalt, animateRatio, legacyClamp, firstNameRight, videos, cutsceneLabel, onShake }: Options): StoryPlayer {
    // The ratio is a LIVE property, not a build argument: rebuilding the engine
    // on a settings change would reset the reader to command 0.
    const ratioRef = useRef(animateRatio);
    const engine = useMemo<Engine>(() => createEngine(script, { nickname, animateRatio: ratioRef.current, legacyClamp, firstNameRight, videos }), [script, nickname, legacyClamp, firstNameRight, videos]);
    useEffect(() => {
        engine.animateRatio = animateRatio;
        ratioRef.current = animateRatio;
    }, [engine, animateRatio]);
    const shakeRef = useRef(onShake);
    shakeRef.current = onShake;
    const [halt, setHalt] = useState<Halt | null>(null);
    const [playback, setPlayback] = useState<Playback>({ frames: [], index: 0 });
    const [haltIndex, setHaltIndex] = useState(-1);
    const [backlog, setBacklog] = useState<BacklogEntry[]>([]);
    const [revealKey, setRevealKey] = useState(0);
    // `saved` is localStorage, which the server cannot read. Reading it in the
    // first render made the server send the title card and the client the resume
    // card, which is the hydration mismatch React logged on every title screen.
    // Both now render the title card and the resume offer arrives after mount.
    const [saved, setSaved] = useState<StoryPosition | null>(null);
    const [phase, setPhase] = useState<Phase>(initialHalt !== undefined ? "reading" : "title");
    const timer = useRef<number | null>(null);
    const backlogRef = useRef<BacklogEntry[]>([]);

    const clearTimer = useCallback(() => {
        if (timer.current !== null) {
            window.clearTimeout(timer.current);
            timer.current = null;
        }
    }, []);

    const playFrames = useCallback(
        (frames: Frame[]) => {
            clearTimer();
            setPlayback({ frames, index: 0 });
            const go = (i: number) => {
                if (i >= frames.length - 1) return;
                const f = frames[i];
                const wait = (f.blocking ? f.transitionSec : 0) + f.holdSec;
                timer.current = window.setTimeout(() => {
                    setPlayback({ frames, index: i + 1 });
                    go(i + 1);
                }, wait * 1000);
            };
            go(0);
        },
        [clearTimer],
    );

    // Engine urls are assets-root-relative (`/audio/audio/sound_beta_2/...`);
    // `asset()` is what puts them on the backend under `/api/assets`. Without it
    // every cue 404s against the dev server's own origin.
    const playMusicEffect = useCallback(
        (e: { intro?: string; loop?: string; volume: number; crossfade: number }) => {
            audio.playMusic({ intro: e.intro ? asset(e.intro) : undefined, loop: e.loop ? asset(e.loop) : undefined, volume: e.volume, crossfade: e.crossfade });
        },
        [audio],
    );

    const fire = useCallback(
        (effects: Effect[]) => {
            for (const e of effects) {
                switch (e.kind) {
                    case "music":
                        playMusicEffect(e);
                        break;
                    case "stopMusic":
                        audio.stopMusic(e.fade);
                        break;
                    case "musicVolume":
                        audio.setCueVolume(e.volume, e.fade);
                        break;
                    case "sound":
                        audio.playSound(asset(e.url), { volume: e.volume, loop: e.loop, delaySec: e.delay, channel: e.channel });
                        break;
                    case "stopSound":
                        audio.stopSounds(e.channel, e.fade);
                        break;
                    case "sfxVolume":
                        audio.setSfxCueVolume(e.channel, e.volume, e.fade);
                        break;
                    case "cameraShake":
                        shakeRef.current?.(e);
                        break;
                }
            }
        },
        [audio, playMusicEffect],
    );

    const persist = useCallback(
        (result: StepResult) => {
            let p = loadProgress();
            if (result.halt.kind === "end") {
                p = withRead(withoutPosition(p, storyId), storyId);
                p.last = storyId;
            } else {
                p = withPosition(p, storyId, { halt: result.haltIndex, total: engine.totalHalts, choices: { ...engine.choices } });
            }
            saveProgress(p);
        },
        [engine, storyId],
    );

    const apply = useCallback(
        (result: StepResult, replay: boolean) => {
            startedRef.current = true;
            audio.noteHalt(result.haltIndex);
            setHalt(result.halt);
            setHaltIndex(result.haltIndex);
            setRevealKey((k) => k + 1);
            playFrames(result.timeline);
            if (replay) {
                // A replay carries only its last step's effects, so the AUDIO
                // STATE is re-derived from the engine: the music it last asked
                // for, the script's own per-channel levels, and every loop that
                // is still running at this halt. Without the last two a Back
                // dropped the ambience for good and carried the previous walk's
                // channel levels into the new one.
                const music = engine.currentMusic;
                if (music?.kind === "music") playMusicEffect({ ...music, crossfade: 0.5 });
                else audio.stopMusic(0.5);
                audio.stopSounds();
                audio.resetSfxLevels();
                for (const s of engine.currentSounds) audio.playSound(asset(s.url), { volume: s.volume, loop: s.loop, delaySec: 0, channel: s.channel });
                for (const [channel, level] of Object.entries(engine.currentSfxLevels)) audio.setSfxCueVolume(channel, level, 0);
            } else {
                fire(result.effects);
            }
            const kept = backlogRef.current.filter((e) => e.haltIndex < result.haltIndex);
            if (result.halt.kind === "line") kept.push({ haltIndex: result.haltIndex, kind: "line", speaker: result.halt.speaker, text: result.halt.text, isNarration: result.halt.isNarration });
            if (result.halt.kind === "video") kept.push({ haltIndex: result.haltIndex, kind: "cutscene", text: cutsceneLabel, isNarration: false });
            backlogRef.current = kept;
            setBacklog(kept);
            setPhase(result.halt.kind === "end" ? "end" : "reading");
            persist(result);
        },
        [audio, cutsceneLabel, engine, fire, persist, playFrames, playMusicEffect],
    );

    // A replay rebuilds the backlog from the halts it passes through, so Back
    // and resume both leave a complete log behind the current line.
    const replayTo = useCallback(
        (target: number, choices: Record<number, string>) => {
            const result = engine.goTo(target, choices);
            // The engine's goTo returns only the final halt; walk the path again for the log.
            const log: BacklogEntry[] = [];
            const probe = createEngine(script, { nickname, videos });
            let r = probe.step();
            while (r.halt.kind !== "end" && r.haltIndex <= target) {
                if (r.halt.kind === "line") log.push({ haltIndex: r.haltIndex, kind: "line", speaker: r.halt.speaker, text: r.halt.text, isNarration: r.halt.isNarration });
                if (r.halt.kind === "video") log.push({ haltIndex: r.haltIndex, kind: "cutscene", text: cutsceneLabel, isNarration: false });
                if (r.halt.kind === "decision") {
                    const chosen = choices[Object.keys(probe.choices).length] ?? r.halt.values[0];
                    const at = r.halt.values.indexOf(chosen);
                    log.push({ haltIndex: r.haltIndex, kind: "choice", text: r.halt.options[at >= 0 ? at : 0] ?? chosen, isNarration: false });
                    r = probe.step(chosen);
                } else {
                    r = probe.step();
                }
            }
            backlogRef.current = log.filter((e) => e.haltIndex < target);
            apply(result, true);
        },
        [apply, cutsceneLabel, engine, nickname, script, videos],
    );

    const start = useCallback(() => {
        backlogRef.current = [];
        // goTo(0) is a reset plus one step, so the result carries the first step's effects.
        apply(engine.goTo(0, {}), false);
    }, [apply, engine]);

    const resume = useCallback(() => {
        if (!saved) return start();
        replayTo(saved.halt, saved.choices);
    }, [replayTo, saved, start]);

    const restart = useCallback(() => {
        audio.stopMusic(0.3);
        audio.stopSounds();
        start();
    }, [audio, start]);

    const advance = useCallback(() => {
        if (phase !== "reading" || halt === null) return;
        if (playback.index < playback.frames.length - 1) {
            clearTimer();
            setPlayback((p) => ({ frames: p.frames, index: p.frames.length - 1 }));
            return;
        }
        if (halt.kind === "decision") return;
        apply(engine.step(), false);
    }, [apply, clearTimer, engine, halt, phase, playback]);

    const choose = useCallback(
        (value: string) => {
            if (halt?.kind !== "decision") return;
            const at = halt.values.indexOf(value);
            backlogRef.current = [...backlogRef.current.filter((e) => e.haltIndex < haltIndex), { haltIndex, kind: "choice", text: halt.options[at >= 0 ? at : 0] ?? value, isNarration: false }];
            apply(engine.step(value), false);
        },
        [apply, engine, halt, haltIndex],
    );

    /**
     * One probe walk of the script, for the scrubber and the backlog jumps.
     * A decision takes its FIRST option, which is the same default
     * `replayTo` uses when a walk reaches a decision the reader has not
     * answered, so the two agree on every halt ordinal.
     */
    const haltSummaries = useMemo<HaltSummary[]>(() => {
        const out: HaltSummary[] = [];
        const probe = createEngine(script, { nickname, videos });
        let r = probe.step();
        // `totalHalts` is the ceiling the engine already counts; the guard is a
        // belt on a walk that a malformed predicate could otherwise not end.
        for (let guard = 0; r.halt.kind !== "end" && guard <= probe.totalHalts + 1; guard++) {
            if (r.halt.kind === "line") out.push({ haltIndex: r.haltIndex, kind: "line", speaker: r.halt.speaker, preview: firstWords(plainStoryText(renderLine(r.halt.text, nickname)), 60) });
            if (r.halt.kind === "video") out.push({ haltIndex: r.haltIndex, kind: "video", preview: cutsceneLabel });
            if (r.halt.kind === "decision") {
                out.push({ haltIndex: r.haltIndex, kind: "decision", preview: firstWords(plainStoryText(renderLine(r.halt.options[0] ?? "", nickname))) });
                r = probe.step(r.halt.values[0]);
            } else {
                r = probe.step();
            }
        }
        return out;
    }, [script, nickname, videos, cutsceneLabel]);

    const jumpTo = useCallback(
        (target: number) => {
            if (target < 0) return;
            replayTo(target, { ...engine.choices });
        },
        [engine, replayTo],
    );

    const back = useCallback(() => {
        if (haltIndex <= 0 || phase === "title" || phase === "resume") return;
        const choices = { ...engine.choices };
        replayTo(haltIndex - 1, choices);
    }, [engine, haltIndex, phase, replayTo]);

    // After mount, offer to resume where this story was left. Guarded on the
    // title phase and on nothing having been read yet, so it can never pull a
    // reader that is already going back to a card.
    const startedRef = useRef(false);
    const initialHaltRef = useRef(initialHalt);
    useEffect(() => {
        if (initialHaltRef.current !== undefined) return;
        const p = loadProgress().pos[storyId];
        if (!p || p.halt <= 0) return;
        setSaved(p);
        if (!startedRef.current) setPhase((current) => (current === "title" ? "resume" : current));
    }, [storyId]);

    // `?halt=N`: replay straight there, once per story (the reader remounts per story id).
    const replayRef = useRef(replayTo);
    replayRef.current = replayTo;
    useEffect(() => {
        const target = initialHaltRef.current;
        if (target === undefined) return;
        const p = loadProgress().pos[storyId];
        replayRef.current(Math.max(0, target), p?.choices ?? {});
    }, [storyId]);

    useEffect(() => () => clearTimer(), [clearTimer]);

    // Dev instrument: a browser run reads what the engine did instead of
    // inferring it from the DOM. Removed on unmount so a stale engine cannot
    // answer for the next story.
    useEffect(() => {
        if (!import.meta.env.DEV || typeof window === "undefined") return;
        const w = window as unknown as { __storyEngine?: () => unknown };
        w.__storyEngine = () => ({ storyId, haltIndex: engine.haltIndex, totalHalts: engine.totalHalts, animateRatio: engine.animateRatio, legacyClamp: engine.legacyClamp, unhandledKinds: { ...engine.unhandledKinds }, unresolvedAssets: [...engine.unresolvedAssets] });
        return () => {
            w.__storyEngine = undefined;
        };
    }, [engine, storyId]);

    return {
        phase,
        halt,
        frame: playback.frames[playback.index] ?? null,
        playing: playback.index < playback.frames.length - 1,
        haltIndex,
        totalHalts: engine.totalHalts,
        backlog,
        unhandledKinds: engine.unhandledKinds,
        unresolvedAssets: engine.unresolvedAssets,
        revealKey,
        start,
        resume,
        restart,
        advance,
        choose,
        back,
        savedHalt: saved?.halt ?? null,
        haltSummaries,
        jumpTo,
    };
}
