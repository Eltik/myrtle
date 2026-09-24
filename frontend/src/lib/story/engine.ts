/**
 * The story engine: a deterministic interpreter over `StoryScript.commands`.
 *
 * Pure TypeScript, no React. `step()` runs commands from the current index
 * until it HALTS at a text line, a decision, or the end, and returns the
 * frames the renderer plays in order (so a blocker fade to black, an image
 * swap and a fade back are three frames, not one final state) plus the audio
 * effects to fire. `goTo()` re-executes from command 0 with a choice map,
 * which is how Back and resume work: the engine has no undo, only replay.
 *
 * Scene semantics are READ out of the client binary and the shipped prefabs,
 * not inferred from the corpus: `docs/story-reader-il2cpp-scene.md`,
 * `docs/story-reader-il2cpp-characters.md` and `docs/story-reader-avg-prefab.md`.
 * Corpus counts (from `docs/story-reader.md`, section 2) say how much each rule
 * is worth, never what the rule is. Everything the engine does not model is
 * consumed without throwing and counted in `unhandledKinds`.
 *
 * WHAT THIS FILE OWNS is the walk and the halt contract: `Halt`, `Effect`,
 * `StepResult`, `Engine` and the command switch inside `build`. The vocabulary
 * that switch writes into lives in `scene.ts`, the readers that turn a wire
 * string into a number or a slot in `args.ts`, and the DOTween curve table in
 * `ease.ts`.
 *
 * The switch itself is ONE function on purpose. Its thirty-eight arms share
 * eleven pieces of walk state (`state`, `pc`, `skipping`, `currentSounds`,
 * `sfxLevels`, the two sequence counters and the four closures `frame`, `hold`,
 * `finish` and `unhandled`), so cutting it into per-kind modules would mean
 * threading a context object through every arm: more surface, the same logic,
 * and a behaviour risk the tests cannot see. It is long because the game has
 * thirty-eight commands, not because two things are tangled in it.
 */
import type { StoryCommand } from "#/types/generated/StoryCommand";
import type { StoryScript } from "#/types/generated/StoryScript";
import type { VideoSources } from "#/types/generated/VideoSources";
import { adaptOf, bool, channel01, key, num, slotOf } from "./args";
import { CANVAS_H, CANVAS_W, LEGACY_SLOT } from "./canvas";
import { EASE_LINEAR, EASE_OUT_CUBIC, easeOf } from "./ease";
import { clamp, clamp01 } from "./num";
import { cloneState, type FocusOut, type Frame, type ImageLayer, initialState, layerFrom, nativeSizeOf, overlayFrom, type SceneState, type Slot } from "./scene";
import { substitute } from "./text";

/**
 * `camerashake` is `DOShakePosition`: duration default -1 meaning TEN SECONDS,
 * strength `(xstrength 1.0, ystrength 0.0)`, vibrato 10, randomness 90,
 * fadeOut FALSE, and `stop=true` resets it. It never blocks.
 */
export interface CameraShake {
    kind: "cameraShake";
    stop: boolean;
    duration: number;
    xStrength: number;
    yStrength: number;
    vibrato: number;
    randomness: number;
    fadeOut: boolean;
}

export type Effect =
    | { kind: "music"; intro?: string; loop?: string; volume: number; crossfade: number }
    | { kind: "stopMusic"; fade: number }
    | { kind: "sound"; url: string; volume: number; loop: boolean; delay: number; channel?: string }
    | { kind: "stopSound"; channel?: string; fade: number }
    | { kind: "musicVolume"; volume: number; fade: number }
    | { kind: "sfxVolume"; volume: number; fade: number; channel?: string }
    | CameraShake;

export type LineSurface = "box" | "subtitle" | "sticker";

/**
 * A `video` halt carries the resolved sources and the command's own args, so
 * the player never has to look the clip up again. An UNRESOLVED reference is
 * not a halt at all: 13 of the 25 `res` values the EN scripts name have no
 * file, and the engine skips those and counts them as `video:missing` rather
 * than stopping the reader on a black rectangle.
 */
export type Halt = { kind: "line"; speaker?: string; text: string; isNarration: boolean; surface: LineSurface } | { kind: "decision"; options: string[]; values: string[] } | { kind: "video"; res: string; sources: VideoSources; args: StoryCommand["args"] } | { kind: "end" };

export interface StepResult {
    halt: Halt;
    timeline: Frame[];
    effects: Effect[];
    /** Ordinal of this halt, 0-based. */
    haltIndex: number;
}

export interface EngineOptions {
    nickname: string;
    /**
     * `AVGController.animateRatio`: the ONE number every duration is multiplied
     * by. Default 1. The game's skip is this ratio at zero, which is why
     * `NeedSkipAnimation` is `MathUtil.IsZero(scaledTime)` and nothing else.
     */
    animateRatio?: number;
    /**
     * `?legacyclamp=1`: restore the pre-parity 1.5 s clamps on transitions and
     * holds and ignore the ratio. Kill switch for the timing change.
     */
    legacyClamp?: boolean;
    /**
     * `?firstright=1`: put the legacy `[character]` pair back the way the
     * reader shipped it, with the FIRST name on the right. Kill switch for the
     * side swap measured in `docs/story-reader-captures.md`, 2.
     */
    firstNameRight?: boolean;
    /**
     * `?video=0` and the "Play cutscene videos" setting. Default on; an
     * explicit `false` is the OFF switch, so a missing option is never read as
     * off. With it off a `[Video]` is skipped and counted as `video:off`, and
     * the reader has exactly the halt count it had before cutscenes shipped.
     */
    videos?: boolean;
}

/** A name a command referenced that the wire `assets` maps have no entry for. */
export interface UnresolvedAsset {
    kind: "background" | "image" | "character" | "music" | "sound" | "video";
    name: string;
}

export interface Engine {
    step(choiceForPendingDecision?: string): StepResult;
    goTo(haltIndex: number, choices: Record<number, string>): StepResult;
    readonly haltIndex: number;
    readonly totalHalts: number;
    readonly unhandledKinds: Record<string, number>;
    /**
     * Names with no entry in `assets`: 17 over the EN corpus (12 characters, 2
     * images, 2 music keys, 1 sound). Recording them is what stops the
     * renderer building a URL out of a name that has none, which would be a
     * request that 404s.
     */
    readonly unresolvedAssets: UnresolvedAsset[];
    /** The chosen value per decision ordinal so far. */
    readonly choices: Record<number, string>;
    readonly state: SceneState;
    readonly atEnd: boolean;
    /** The music the script last asked for (a `music` or `stopMusic` effect), so a replay can restore it. */
    readonly currentMusic: Effect | null;
    /**
     * The LOOPING sounds that are still running at the current halt, keyed by
     * channel, and the script's own level per channel. A replay fires only the
     * last step's effects, so without these a `goTo` (Back, resume, `?halt=`)
     * drops every ambience the script established earlier and keeps whatever
     * channel levels the previous walk happened to leave behind.
     */
    readonly currentSounds: Extract<Effect, { kind: "sound" }>[];
    readonly currentSfxLevels: Record<string, number>;
    /** The live `animateRatio`; a settings change retunes the engine without rebuilding it. */
    animateRatio: number;
    readonly legacyClamp: boolean;
}

/** Only the `?legacyclamp=1` path uses these; the client clamps nothing. */
export const MAX_TRANSITION_SEC = 1.5;
export const MAX_HOLD_SEC = 1.5;

/** The command kinds the engine models. `unhandledKinds` never lists one of these. */
export const HANDLED_KINDS: ReadonlySet<string> = new Set([
    "name",
    "text",
    "narration",
    "multiline",
    "dialog",
    "voicewithin",
    "sticker",
    "stickerclear",
    "subtitle",
    "decision",
    "predicate",
    "character",
    "charslot",
    "image",
    "imagerotate",
    "background",
    "blocker",
    "delay",
    "playmusic",
    "stopmusic",
    "musicvolume",
    "playsound",
    "stopsound",
    "soundvolume",
    "camerashake",
    "cameraeffect",
    "header",
    "imagetween",
    "backgroundtween",
    "largebgtween",
    "characteraction",
    "charactercutin",
    "curtain",
    "focusout",
    "largebg",
    "gridbg",
    "popupdialog",
    "interlude",
    "video",
]);

function build(script: StoryScript, options: EngineOptions, total: number): Engine {
    const commands = script.commands;
    const nickname = options.nickname.trim() === "" ? "Doctor" : options.nickname;
    const legacyClamp = options.legacyClamp === true;
    const firstNameRight = options.firstNameRight === true;
    let animateRatio = Number.isFinite(options.animateRatio) ? Math.max(0, options.animateRatio as number) : 1;
    const unhandledKinds: Record<string, number> = {};
    const unresolvedAssets: UnresolvedAsset[] = [];
    const unresolvedSeen = new Set<string>();
    let pc = 0;
    let state = initialState();
    let haltIndex = -1;
    let choices: Record<number, string> = {};
    let decisionOrdinal = -1;
    let pendingDecision: { values: string[] } | null = null;
    let skipping = false;
    let atEnd = false;
    let currentMusic: Effect | null = null;
    // Live loops keyed by channel; a loop with no channel cannot be addressed
    // by `[stopsound]` and is not replayable, so it is not tracked.
    let currentSounds = new Map<string, Extract<Effect, { kind: "sound" }>>();
    let sfxLevels: Record<string, number> = {};
    let actionSeq = 0;
    let swapSeq = 0;

    /**
     * Every duration in the client is ONE multiply: `CalculateFadetime` loads
     * the argument, reads `AVGController.get_animateRatio` and returns the
     * product. A reverse scan of that getter returns eleven sites and not one
     * carries an `fmin`, an `fmax` or a constant comparison, so the 1.5 s
     * clamps this reader shipped were an invention.
     */
    const scaleSec = (raw: number): number => {
        if (!Number.isFinite(raw) || raw <= 0) return 0;
        return legacyClamp ? Math.min(raw, MAX_TRANSITION_SEC) : raw * animateRatio;
    };

    const unhandled = (kind: string) => {
        unhandledKinds[kind] = (unhandledKinds[kind] ?? 0) + 1;
    };
    /** Record a name the wire maps do not carry, once per kind and name. */
    const unresolved = (kind: UnresolvedAsset["kind"], name: string) => {
        const seen = `${kind}:${name}`;
        if (unresolvedSeen.has(seen)) return;
        unresolvedSeen.add(seen);
        unresolvedAssets.push({ kind, name });
    };
    /**
     * A background or CG layer with its `native` rect attached. Only the
     * SKIPPED `screenadapt` arm reads a size, so a missing one is only a
     * defect there, and it is counted as `imageSize:missing` beside the
     * unhandled kinds rather than thrown.
     */
    const makeLayer = (a: StoryCommand["args"], name: string, url: string, previous?: ImageLayer): ImageLayer => {
        const native = nativeSizeOf(script.assets, key(name));
        if (!native && adaptOf(a.screenadapt) === "native") unhandled("imageSize:missing");
        return layerFrom(a, name, url, native, previous);
    };

    function reset() {
        pc = 0;
        state = initialState();
        haltIndex = -1;
        choices = {};
        decisionOrdinal = -1;
        pendingDecision = null;
        skipping = false;
        atEnd = false;
        currentMusic = null;
        currentSounds = new Map();
        sfxLevels = {};
        actionSeq = 0;
        swapSeq = 0;
    }

    function step(choice?: string): StepResult {
        const timeline: Frame[] = [];
        const effects: Effect[] = [];
        // Push a snapshot of the state as a new frame. `sec` is RAW script
        // seconds; the ratio is applied here so no call site can forget it.
        const frame = (sec: number, blocking: boolean, ease?: string) => {
            timeline.push({ state: cloneState(state), transitionSec: scaleSec(sec), holdSec: 0, blocking, ease });
        };
        const hold = (sec: number) => {
            if (timeline.length === 0) frame(0, true);
            const last = timeline[timeline.length - 1];
            last.holdSec = legacyClamp ? clamp(last.holdSec + sec, 0, MAX_HOLD_SEC) : last.holdSec + scaleSec(sec);
        };
        const finish = (halt: Halt): StepResult => {
            // The halt itself shows or hides the box; the last frame must carry that.
            if (timeline.length === 0 || timeline[timeline.length - 1].state.dialogVisible !== state.dialogVisible) frame(0, true);
            if (halt.kind !== "end") haltIndex += 1;
            // `{@nickname}`, `{@nbs}` and literal `\n` are resolved here so the
            // backlog, the auto-play timer and the text box all read one string.
            const resolved: Halt = halt.kind === "line" ? { ...halt, text: substitute(halt.text, nickname) } : halt;
            return { halt: resolved, timeline, effects, haltIndex };
        };

        if (atEnd) return finish({ kind: "end" });

        if (pendingDecision) {
            const values = pendingDecision.values;
            const chosen = choice !== undefined && values.includes(choice) ? choice : values[0];
            choices[decisionOrdinal] = chosen;
            pendingDecision = null;
        }

        while (pc < commands.length) {
            const cmd = commands[pc];
            pc += 1;
            const a = cmd.args;
            const kind = cmd.kind;

            // Branching runs before the skip check: a predicate can end a skipped block.
            if (kind === "predicate") {
                const chosen = choices[decisionOrdinal];
                if (chosen === undefined) {
                    skipping = false;
                } else {
                    const refs = (a.references ?? "").split(";").map((s) => s.trim());
                    skipping = !refs.includes(chosen);
                }
                continue;
            }
            if (kind === "decision") {
                skipping = false;
                const options = (a.options ?? "").split(";");
                const values = (a.values ?? "").split(";").map((s) => s.trim());
                if (values.length === 0 || (values.length === 1 && values[0] === "")) {
                    // A decision with no values cannot branch: skip it (0 in the two fixtures).
                    continue;
                }
                decisionOrdinal += 1;
                pendingDecision = { values };
                // A decision does NOT touch the box: `_ExecuteDecision` only
                // activates its own panel and the line stays under the options.
                return finish({ kind: "decision", options, values });
            }
            if (skipping) continue;

            switch (kind) {
                case "header":
                    break;
                case "video": {
                    // The command names its clip by the path the CN bundles
                    // used (`video/act38side/PV01.mp4`); the wire is keyed by
                    // that same string, so no case folding happens here.
                    const res = (a.res ?? "").trim();
                    if (options.videos === false) {
                        unhandled("video:off");
                        break;
                    }
                    const sources = res === "" ? undefined : script.assets.videos?.[res];
                    if (!sources || (sources.webmUrl === undefined && sources.mp4Url === undefined)) {
                        unresolved("video", res);
                        unhandled("video:missing");
                        break;
                    }
                    // The clip covers the stage, so the box goes with it.
                    state.dialogVisible = false;
                    return finish({ kind: "video", res, sources, args: a });
                }
                case "name":
                case "multiline": {
                    state.dialogVisible = true;
                    const speaker = (a.name ?? "").trim();
                    // `[name=""]` (the tutorial's opening voice and every unattributed
                    // line) has no plate to draw, so it reads as narration.
                    return finish({ kind: "line", speaker: speaker === "" ? undefined : speaker, text: cmd.text ?? "", isNarration: speaker === "", surface: "box" });
                }
                case "text":
                case "narration":
                    state.dialogVisible = true;
                    return finish({ kind: "line", text: cmd.text ?? "", isNarration: true, surface: "box" });
                case "dialog": {
                    // The discriminator is the TEXT, not the arguments:
                    // `_ExecuteDialog` reads ONE field, the text, and calls
                    // `String.IsNullOrEmpty` on it. So `[dialog(fadetime=1)]`
                    // hides the box exactly like a bare one, and `fadetime` is
                    // never read.
                    const text = (cmd.text ?? "").trim();
                    if (text === "") {
                        state.dialogVisible = false;
                        frame(0, true);
                        break;
                    }
                    state.dialogVisible = true;
                    return finish({ kind: "line", text: cmd.text ?? "", isNarration: false, surface: "box" });
                }
                case "voicewithin": {
                    const text = (cmd.text ?? "").trim();
                    if (text === "") break;
                    state.dialogVisible = true;
                    return finish({ kind: "line", text: cmd.text ?? "", isNarration: false, surface: "box" });
                }
                case "subtitle": {
                    const text = a.text;
                    if (text === undefined) {
                        // Bare `[subtitle]` (2,454 uses) clears the overlay.
                        state.subtitle = undefined;
                        frame(0, true);
                        break;
                    }
                    const overlay = overlayFrom(a, text, nickname);
                    // The executor REJECTS the command outside the canvas.
                    if (overlay.x < 0 || overlay.x > CANVAS_W || overlay.y < 0 || overlay.y > CANVAS_H) {
                        unhandled("subtitle:offcanvas");
                        break;
                    }
                    state.subtitle = overlay;
                    return finish({ kind: "line", text, isNarration: true, surface: "subtitle" });
                }
                case "sticker": {
                    const id = a.id ?? "st";
                    const text = a.text;
                    const exists = state.stickers[id] !== undefined;
                    // `block` DEFAULTS by branch: true for a new id, false for a repeat.
                    const blocking = bool(a.block, !exists);
                    if (text === undefined) {
                        // `[sticker(id=)]` with no text (3,043 uses) removes that sticker.
                        delete state.stickers[id];
                        frame(0, blocking);
                        break;
                    }
                    if (exists && !bool(a.multi, false)) {
                        // An EXISTING id WITHOUT `multi` HIDES the sticker even
                        // when the command carries text. It does not halt: the
                        // repeat branch is non-blocking and there would be
                        // nothing on screen to read.
                        delete state.stickers[id];
                        unhandled("sticker:repeatHides");
                        frame(0, blocking);
                        break;
                    }
                    const prev = state.stickers[id];
                    if (prev) {
                        // `multi=true` appends to the sticker with the same id.
                        prev.text += substitute(text, nickname);
                        return finish({ kind: "line", text, isNarration: true, surface: "sticker" });
                    }
                    state.stickers[id] = overlayFrom(a, text, nickname);
                    return finish({ kind: "line", text, isNarration: true, surface: "sticker" });
                }
                case "stickerclear":
                    state.stickers = {};
                    frame(0, true);
                    break;
                case "character": {
                    // Bare `[character]` (17,409 uses) clears both slots.
                    const name1 = a.name;
                    const name2 = a.name2;
                    const fade = num(a.fadetime, 0);
                    const blocking = bool(a.block, true);
                    state.slots = {};
                    if (name1 === undefined && name2 === undefined) {
                        frame(fade, blocking);
                        break;
                    }
                    // `_ProcessSlotWithParam` reads the focus INT: absent or 0
                    // lights both, 1 lights slot 1 and dims slot 2, 2 is the
                    // reverse, -1 is EXACTLY 1 (`orr w8,focus,#2; cmp w8,#2` is
                    // false for -1), and 3 dims everything. This reader used to
                    // light both on -1 (3,045 lines) and on 3 (127).
                    const focus = Math.trunc(num(a.focus, 0));
                    const lit1 = focus !== 2 && focus !== 3;
                    const lit2 = focus === 0 || focus === 2;
                    const put = (slot: Slot, name: string, lit: boolean) => {
                        const sprite = script.assets.characters[key(name)];
                        if (!sprite) {
                            unresolved("character", name);
                            return;
                        }
                        swapSeq += 1;
                        state.slots[slot] = { sprite, name: key(name), lit, x: 0, y: 0, alpha: 1, scale: 1, pivotX: 0.5, pivotY: 0.5, swap: swapSeq };
                    };
                    if (name1 !== undefined && name2 !== undefined) {
                        // MEASURED (`docs/story-reader-captures.md`, 2): with
                        // Amiya as `name` and Dobermann as `name2`, their ink
                        // centres are canvas x -175.3 and +228.3, so slot 1 is
                        // the LEFT figure at -200 and slot 2 the right at +200.
                        // The reader shipped them mirrored off the
                        // `_GenPosition` read; `?firstright=1` puts that back.
                        put(LEGACY_SLOT[firstNameRight ? 2 : 1], name1, lit1);
                        put(LEGACY_SLOT[firstNameRight ? 1 : 2], name2, lit2);
                    } else {
                        put("m", (name1 ?? name2) as string, lit1);
                    }
                    frame(fade, blocking);
                    break;
                }
                case "charslot": {
                    const slot = slotOf(a.slot);
                    const duration = num(a.duration, 0);
                    const blocking = bool(a.isblock ?? a.block, true);
                    if (slot === undefined && a.slot !== undefined) {
                        // `n`, `all`, `0,0`: 5 uses in the corpus, logged and ignored.
                        unhandled(`charslot:slot=${a.slot}`);
                        break;
                    }
                    if (slot === undefined && a.name === undefined && a.focus === undefined) {
                        // Bare `[charslot]` (32,715 uses) FADES every slot out
                        // over `duration`; with a zero duration it is a cut.
                        state.slots = {};
                        frame(duration, blocking);
                        break;
                    }
                    if (slot !== undefined && a.name !== undefined) {
                        const sprite = script.assets.characters[key(a.name)];
                        if (!sprite) unresolved("character", a.name);
                        else {
                            const name = key(a.name);
                            const prev = state.slots[slot];
                            if (prev && prev.name === name) {
                                prev.sprite = sprite;
                            } else {
                                // A DIFFERENT sprite resets the slot to home,
                                // `localPosition` zero and `localScale` one,
                                // before the back-to-fore crossfade.
                                swapSeq += 1;
                                state.slots[slot] = { sprite, name, lit: prev?.lit ?? true, x: 0, y: 0, alpha: prev?.alpha ?? 1, scale: 1, pivotX: 0.5, pivotY: 0.5, swap: swapSeq };
                            }
                        }
                    }
                    const target = slot !== undefined ? state.slots[slot] : undefined;
                    if (target) {
                        // `posfrom` is set ABSOLUTELY first, in the slot's own
                        // space, which is our zero-length from-frame.
                        const from = a.posfrom !== undefined ? a.posfrom.split(",").map((s) => num(s, 0)) : undefined;
                        const afrom = num(a.afrom, -1);
                        if (from !== undefined || afrom >= 0) {
                            if (from !== undefined) {
                                target.x = from[0] ?? 0;
                                target.y = from[1] ?? 0;
                            }
                            if (afrom >= 0) target.alpha = clamp01(afrom);
                            frame(0, true);
                        }
                        if (a.posto !== undefined) {
                            const [x, y] = a.posto.split(",").map((s) => num(s, 0));
                            target.x = x ?? 0;
                            target.y = y ?? 0;
                        }
                        // `afrom`/`ato` default to -1, which means UNCHANGED.
                        const ato = num(a.ato, -1);
                        if (ato >= 0) target.alpha = clamp01(ato);
                        const action = (a.action ?? "").trim().toLowerCase();
                        if (action === "jump" || action === "shake" || action === "move" || action === "zoom") {
                            if (action === "zoom") {
                                // `poszoom` is a NORMALIZED PIVOT and `CharZoom`
                                // rejects the command outside 0..1 on either axis.
                                const [px, py] = (a.poszoom ?? "0.5,0.5").split(",").map((s) => num(s, Number.NaN));
                                if (!(px >= 0 && px <= 1 && py >= 0 && py <= 1)) {
                                    unhandled("charslot:poszoomOutOfRange");
                                } else {
                                    target.pivotX = px;
                                    target.pivotY = py;
                                    target.scale = num(a.scale, 1);
                                    actionSeq += 1;
                                    target.action = { kind: "zoom", power: 0, times: 0, randomness: 0, sec: scaleSec(duration), seq: actionSeq };
                                }
                            } else {
                                actionSeq += 1;
                                target.action = { kind: action, power: num(a.power, 0), times: Math.trunc(num(a.times, 0)), randomness: num(a.randomness, 10), sec: scaleSec(duration), seq: actionSeq };
                            }
                        } else if (a.action !== undefined && a.action.trim() !== "") {
                            unhandled(`charslot:action=${a.action}`);
                        }
                    }
                    // `_ExecuteCharslot` puts the `all` literal into the focus
                    // array when `focus` is EMPTY, and `_ProcessFocusArray`
                    // clears all three flags before it sets the listed ones, so
                    // naming one slot actively DIMS the other two.
                    const f = (a.focus ?? "").trim().toLowerCase();
                    const lit = new Set<Slot>();
                    let all = false;
                    if (f === "" || f === "all" || f === "a") all = true;
                    else if (f !== "n" && f !== "none") {
                        for (const part of f.split(",")) {
                            const s = slotOf(part);
                            if (s) lit.add(s);
                        }
                    }
                    for (const k of Object.keys(state.slots) as Slot[]) {
                        const st = state.slots[k];
                        if (st) st.lit = all || lit.has(k);
                    }
                    frame(duration, blocking);
                    break;
                }
                case "image": {
                    const fade = num(a.fadetime, 0);
                    const blocking = bool(a.block, true);
                    const name = a.image;
                    if (name === undefined) {
                        state.image = undefined;
                    } else {
                        const url = script.assets.images[key(name)];
                        if (!url) unresolved("image", name);
                        state.image = url ? makeLayer(a, name, url) : undefined;
                    }
                    frame(fade, blocking);
                    break;
                }
                case "imagerotate": {
                    // 19 uses, all `angle=` with a `fadetime` and `block=false`.
                    // `_ExecuteImageRotate` passes ease 1, Linear.
                    if (!state.image) {
                        unhandled("imagerotate:noimage");
                        break;
                    }
                    state.image.rotate = num(a.angle, 0);
                    frame(num(a.fadetime ?? a.duration, 0), bool(a.block, true), EASE_LINEAR);
                    break;
                }
                case "background": {
                    const fade = num(a.fadetime, 0);
                    const blocking = bool(a.block, true);
                    const name = a.image;
                    if (name === undefined) {
                        state.background = undefined;
                    } else {
                        const url = script.assets.backgrounds[key(name)];
                        if (!url) unresolved("background", name);
                        state.background = url ? makeLayer(a, name, url) : undefined;
                    }
                    frame(fade, blocking);
                    break;
                }
                case "blocker": {
                    const fade = num(a.fadetime, 0);
                    const blocking = bool(a.block, true);
                    // Defaults a 1.0, r/g/b 0.0; each `*from` defaults to the
                    // image's CURRENT channel and is written immediately, which
                    // is the zero-length from-frame.
                    const to = { a: channel01(a.a, 1), r: channel01(a.r, 0), g: channel01(a.g, 0), b: channel01(a.b, 0) };
                    if (a.afrom !== undefined || a.rfrom !== undefined || a.gfrom !== undefined || a.bfrom !== undefined) {
                        state.blocker = { a: channel01(a.afrom, state.blocker.a), r: channel01(a.rfrom, state.blocker.r), g: channel01(a.gfrom, state.blocker.g), b: channel01(a.bfrom, state.blocker.b) };
                        frame(0, true);
                    }
                    state.blocker = to;
                    frame(fade, blocking);
                    break;
                }
                case "delay":
                    hold(num(a.time ?? a.delay, 0));
                    break;
                case "playmusic": {
                    const cue = a.key !== undefined ? script.assets.music[key(a.key)] : undefined;
                    const introCue = a.intro !== undefined ? script.assets.music[key(a.intro)] : undefined;
                    // The wire keys a cue by its LOOP name and hangs the intro
                    // url on the same cue, so an `intro=` key with no entry of
                    // its own is resolved whenever the loop cue carries one.
                    if (a.key !== undefined && !cue) unresolved("music", a.key);
                    if (a.intro !== undefined && !introCue && !cue?.intro) unresolved("music", a.intro);
                    if (cue || introCue) {
                        currentMusic = { kind: "music", intro: introCue?.intro ?? cue?.intro, loop: cue?.loop ?? introCue?.loop, volume: clamp01(num(a.volume, 1)), crossfade: num(a.crossfade, 0) };
                        effects.push(currentMusic);
                    }
                    break;
                }
                case "stopmusic":
                    currentMusic = { kind: "stopMusic", fade: num(a.fadetime, 0) };
                    effects.push(currentMusic);
                    break;
                case "musicvolume":
                    effects.push({ kind: "musicVolume", volume: clamp01(num(a.volume, 1)), fade: num(a.fadetime, 0) });
                    break;
                case "playsound": {
                    const url = a.key !== undefined ? script.assets.sounds[key(a.key)] : undefined;
                    if (a.key !== undefined && url === undefined) unresolved("sound", a.key);
                    // Census over 48,144 EN `playsound` lines: key 48,138,
                    // volume 34,356, channel 6,318, loop 4,249 (true 3,251),
                    // delay 2,836, block 211, fadetime 45. `block` and
                    // `fadetime` are NOT modelled: a sound never gates the
                    // reader and the SFX bus has one ramp.
                    if (url) {
                        const e: Extract<Effect, { kind: "sound" }> = { kind: "sound", url, volume: clamp01(num(a.volume, 1)), loop: bool(a.loop, false), delay: scaleSec(num(a.delay, 0)), channel: a.channel ?? a.ch };
                        effects.push(e);
                        // A `channel` is a BUS: the `[playsound]` that opens it
                        // writes its level, and a second sound on a live
                        // channel replaces the first.
                        if (e.channel !== undefined) {
                            sfxLevels[e.channel] = e.volume;
                            if (e.loop) currentSounds.set(e.channel, e);
                            else currentSounds.delete(e.channel);
                        }
                    }
                    break;
                }
                case "stopsound":
                    // 3,796 of 3,799 name a `channel`; 7 name a `key` instead
                    // and those stop every voice, which is what a bare stop does.
                    {
                        const ch = a.channel ?? a.ch;
                        // 2,804 of the 2,937 EN `[stopsound]` carry a
                        // `fadetime`; a hard stop on all of them is a click.
                        effects.push({ kind: "stopSound", channel: ch, fade: scaleSec(num(a.fadetime, 0)) });
                        if (ch === undefined) currentSounds.clear();
                        else currentSounds.delete(ch);
                    }
                    break;
                case "soundvolume":
                    // Per CHANNEL: 1,650 of 1,651 name one, over 150 distinct
                    // channels. A global level was measured silencing every
                    // voice on act29side_level_act29side_10_beg, so the channel
                    // rides on the effect and the audio module keeps one level
                    // per channel.
                    {
                        const level = clamp01(num(a.volume, 1));
                        effects.push({ kind: "sfxVolume", volume: level, fade: num(a.fadetime, 0), channel: a.channel });
                        if (a.channel !== undefined) sfxLevels[a.channel] = level;
                    }
                    break;
                case "camerashake": {
                    // `DOShakePosition(duration, strength (x,y,0), vibrato,
                    // randomness, snapping false, fadeOut false)`. A duration
                    // of -1, which is the default, becomes TEN seconds. It
                    // never blocks, and `stop=true` resets the shake.
                    const stop = bool(a.stop, false);
                    const raw = num(a.duration, -1);
                    effects.push({
                        kind: "cameraShake",
                        stop,
                        duration: stop ? 0 : scaleSec(raw < 0 ? 10 : raw),
                        xStrength: num(a.xstrength, 1),
                        yStrength: num(a.ystrength, 0),
                        vibrato: Math.max(1, Math.trunc(num(a.vibrato, 10))),
                        randomness: num(a.randomness, 90),
                        fadeOut: bool(a.fadeout, false),
                    });
                    break;
                }
                case "cameraeffect": {
                    // Two effects over the corpus: Grayscale 2,480 and
                    // Colorinverse 8. `initamount` starts the ramp elsewhere,
                    // which is one extra zero-length frame, like a blocker's
                    // `afrom`.
                    const effect = (a.effect ?? "").toLowerCase();
                    const field = effect === "grayscale" ? "grayscale" : effect === "colorinverse" ? "colorInverse" : undefined;
                    if (field === undefined) {
                        unhandled(`cameraeffect:${a.effect ?? ""}`);
                        break;
                    }
                    if (a.initamount !== undefined) {
                        state.effects[field] = clamp01(num(a.initamount, 0));
                        frame(0, true);
                    }
                    state.effects[field] = clamp01(num(a.amount, 1));
                    frame(num(a.fadetime, 0), bool(a.block, true));
                    break;
                }
                case "imagetween":
                case "backgroundtween":
                case "largebgtween": {
                    // Both `*from` and `*to` default to the CURRENT value, so a
                    // tween naming only `xTo` FREEZES y and the scales instead
                    // of snapping them to zero or one.
                    const target = kind === "imagetween" ? "image" : "background";
                    if (a.image !== undefined) {
                        const map = target === "image" ? script.assets.images : script.assets.backgrounds;
                        const url = map[key(a.image)];
                        if (url) state[target] = makeLayer(a, a.image, url, state[target]);
                        else unresolved(target === "image" ? "image" : "background", a.image);
                    }
                    const layer = state[target];
                    if (!layer) break;
                    const ease = easeOf(a.ease);
                    const hasFrom = a.xfrom !== undefined || a.yfrom !== undefined || a.xscalefrom !== undefined || a.yscalefrom !== undefined || a.xfromscale !== undefined || a.yfromscale !== undefined;
                    if (hasFrom) {
                        layer.x = num(a.xfrom, layer.x);
                        layer.y = num(a.yfrom, layer.y);
                        layer.xScale = num(a.xscalefrom ?? a.xfromscale, layer.xScale);
                        layer.yScale = num(a.yscalefrom ?? a.yfromscale, layer.yScale);
                        frame(0, true);
                    }
                    const moved = state[target];
                    if (moved) {
                        moved.x = num(a.xto ?? a.x, moved.x);
                        moved.y = num(a.yto ?? a.y, moved.y);
                        moved.xScale = num(a.xscaleto ?? a.xscale, moved.xScale);
                        moved.yScale = num(a.yscaleto ?? a.yscale, moved.yScale);
                    }
                    frame(num(a.duration ?? a.fadetime, 0), bool(a.block, true), ease);
                    break;
                }
                case "characteraction": {
                    // `name` is the SLOT (right 2,347, left 2,303, middle
                    // 2,202, `lfte` 4, a typo for left), not a sprite; `type`
                    // is move 5,283, jump 1,367, shake 94, exit 89, zoom 27.
                    const raw = (a.name ?? "").trim().toLowerCase();
                    const slot = slotOf(raw === "lfte" ? "left" : raw);
                    if (slot === undefined) {
                        unhandled(`characteraction:name=${a.name ?? ""}`);
                        break;
                    }
                    const target = state.slots[slot];
                    if (!target) break;
                    const type = (a.type ?? "").trim().toLowerCase();
                    const sec = num(a.fadetime ?? a.duration ?? a.time, 0);
                    const blocking = bool(a.block ?? a.isblock, true);
                    let ease: string | undefined;
                    switch (type) {
                        case "move":
                            // `MoveChar` is the RELATIVE one on this path:
                            // target = localPosition + (dx, dy).
                            target.x += num(a.xpos, 0);
                            target.y += num(a.ypos, 0);
                            target.action = undefined;
                            ease = EASE_OUT_CUBIC;
                            break;
                        case "zoom":
                            target.scale = num(a.scale ?? a.power, 1);
                            target.action = undefined;
                            ease = EASE_LINEAR;
                            break;
                        case "exit":
                            delete state.slots[slot];
                            break;
                        case "jump":
                        case "shake":
                            actionSeq += 1;
                            target.action = { kind: type, power: num(a.power, 0), times: Math.trunc(num(a.times, 0)), randomness: num(a.randomness, 10), sec: scaleSec(sec), seq: actionSeq };
                            ease = EASE_OUT_CUBIC;
                            break;
                        default:
                            unhandled(`characteraction:type=${a.type ?? ""}`);
                            break;
                    }
                    frame(sec, blocking, ease);
                    break;
                }
                case "charactercutin": {
                    // 611 of 1,007 carry a `name`; the rest are the CLOSE.
                    // `Show` sizes the plate from two ints and NEGATES both
                    // offsets before writing `anchoredPosition`, then fades
                    // alpha 0 to 1. There is no slide.
                    const sec = num(a.fadetime, 0);
                    const blocking = bool(a.block, true);
                    if (a.name === undefined || a.name.trim() === "") {
                        state.cutin = undefined;
                        frame(sec, blocking);
                        break;
                    }
                    const sprite = script.assets.characters[key(a.name)];
                    if (!sprite) {
                        unresolved("character", a.name);
                        break;
                    }
                    const negate = (v: number) => (v === 0 ? 0 : -v);
                    state.cutin = { sprite, name: key(a.name), x: negate(num(a.offsetx, 0)), y: negate(num(a.offsety, 0)), width: num(a.width, 200), height: num(a.height, CANVAS_H) };
                    frame(sec, blocking);
                    break;
                }
                case "curtain": {
                    // 242 of 1,308 are bare and clear it. `fillfrom` defaults
                    // to 1.0 and `fillto` to 0.0; `r`, `g`, `b` and their
                    // `from` twins are READ and DISCARDED, so the colour is the
                    // prefab's and never the script's.
                    if (a.fillto === undefined && a.fillfrom === undefined && a.a === undefined) {
                        state.curtain = undefined;
                        frame(num(a.fadetime, 0), bool(a.block, true));
                        break;
                    }
                    const direction = Math.trunc(num(a.direction, -1));
                    const grad = bool(a.grad, false);
                    if (a.fillfrom !== undefined) {
                        state.curtain = { direction, fill: clamp01(num(a.fillfrom, 1)), grad };
                        frame(0, true);
                    }
                    state.curtain = { direction, fill: clamp01(num(a.fillto ?? a.ato ?? a.a, state.curtain?.fill ?? 0)), grad };
                    frame(num(a.fadetime ?? a.duration, 0), bool(a.block, true));
                    break;
                }
                case "focusout": {
                    // `type` is bg 357, cg 78, char 54, cgitem 19, lbg 8. The
                    // 0..1 amount drives a BLUR (`GetBlitAlphaGhostMatPath`
                    // over a ping-pong downsample chain), not a darken.
                    const t = (a.type ?? "bg").trim().toLowerCase();
                    const field: keyof FocusOut = t === "char" ? "char" : t === "cg" || t === "cgitem" ? "cg" : "bg";
                    if (a.from !== undefined && num(a.from, -1) >= 0) {
                        state.focus[field] = clamp01(num(a.from, 0));
                        frame(0, true);
                    }
                    state.focus[field] = clamp01(num(a.to, 0));
                    frame(num(a.duration, 0), bool(a.block, true));
                    break;
                }
                case "largebg":
                case "gridbg": {
                    // Bare closes it: 69 of 138 largebg and 85 of 186 gridbg.
                    const group = a.imagegroup ?? a.cggroup;
                    const sec = num(a.fadetime, 0);
                    if (group === undefined || group.trim() === "") {
                        state.panels = undefined;
                        frame(sec, true);
                        break;
                    }
                    const names = group
                        .split("/")
                        .map((n) => n.trim())
                        .filter((n) => n !== "");
                    const urls: string[] = [];
                    for (const n of names) {
                        const url = script.assets.backgrounds[key(n)] ?? script.assets.images[key(n)];
                        if (url) urls.push(url);
                        else unresolved("background", n);
                    }
                    if (urls.length !== names.length || urls.length === 0) break;
                    const widths = (a.solidwidth ?? "").split("/").map((w) => num(w, CANVAS_W));
                    while (widths.length < urls.length) widths.push(widths[widths.length - 1] ?? CANVAS_W);
                    const height = num((a.solidheight ?? "").split("/")[0], CANVAS_H);
                    state.panels = { urls, widths, height, rows: kind === "gridbg" ? 2 : 1, x: num(a.x, 0), y: num(a.y, 0) };
                    frame(sec, bool(a.block, true));
                    break;
                }
                case "popupdialog": {
                    // 1,590 of 1,591 carry a `dialoghead`; all 163 distinct
                    // values resolve to an operator avatar. NOT REACHABLE from
                    // the shipped library: every one of the 547 files that use
                    // it is a training or guide script and none is listed in
                    // `story_review_table`.
                    const head = a.dialoghead;
                    if (head === undefined || head.trim() === "") {
                        state.popupHead = undefined;
                        frame(0, true);
                        break;
                    }
                    // `avatars` is newer than the running backend can be, so a
                    // payload without it skips instead of throwing.
                    const url = script.assets.avatars?.[key(head)];
                    if (!url) {
                        unresolved("character", head);
                        break;
                    }
                    state.popupHead = { key: head, url, x: num(a.dialogx, Number.NaN) || undefined, y: num(a.dialogy, Number.NaN) || undefined };
                    frame(0, true);
                    break;
                }
                case "interlude": {
                    // REFUTED: interlude is not a title card. It carries no
                    // text argument at all; its keys are `channel` (1,167),
                    // `switch` (734), `type` (487), `slot` (412), `name` (377)
                    // and mask geometry. What it does is put a masked panel on
                    // a channel and slide it from `pfrom` to `pto`. Only the
                    // panel is modelled; the mask shapes (4 `maskid` values)
                    // are not, and nothing in the script says their geometry.
                    const channel = (a.channel ?? "1").trim();
                    const duration = num(a.duration, 0);
                    const blocking = bool(a.block, true);
                    const closing = bool(a.clear, false) || a.switch?.trim().toLowerCase() === "false";
                    if (closing) {
                        delete state.interludes[channel];
                        frame(duration, blocking);
                        break;
                    }
                    const name = a.name?.trim();
                    if (name === undefined || name === "") break;
                    const url = script.assets.characters[key(name)]?.bodyUrl ?? script.assets.images[key(name)] ?? script.assets.backgrounds[key(name)];
                    if (!url) {
                        unresolved("image", name);
                        break;
                    }
                    const [fx, fy] = (a.pfrom ?? "").split(",").map((v) => num(v, Number.NaN));
                    const [tx, ty] = (a.pto ?? "").split(",").map((v) => num(v, Number.NaN));
                    if (Number.isFinite(fx)) {
                        state.interludes[channel] = { url, name, x: fx, y: Number.isFinite(fy) ? fy : 0 };
                        frame(0, true);
                    }
                    state.interludes[channel] = { url, name, x: Number.isFinite(tx) ? tx : (state.interludes[channel]?.x ?? 0), y: Number.isFinite(ty) ? ty : (state.interludes[channel]?.y ?? 0) };
                    frame(duration, blocking);
                    break;
                }
                default:
                    unhandled(kind);
            }
        }
        atEnd = true;
        // The end card owns the stage; `OnReset` is what snaps the box hidden.
        state.dialogVisible = false;
        return finish({ kind: "end" });
    }

    function goTo(target: number, wanted: Record<number, string>): StepResult {
        reset();
        let result = step();
        while (result.halt.kind !== "end" && result.haltIndex < target) {
            const next = result.halt.kind === "decision" ? wanted[decisionOrdinal] : undefined;
            result = step(next);
        }
        // Replay lands on the target state; the renderer shows only the final frame.
        return { ...result, timeline: result.timeline.slice(-1).map((f) => ({ ...f, transitionSec: 0, holdSec: 0 })) };
    }

    const engine: Engine = {
        step,
        goTo,
        get haltIndex() {
            return haltIndex;
        },
        get totalHalts() {
            return total;
        },
        get unhandledKinds() {
            return unhandledKinds;
        },
        get unresolvedAssets() {
            return unresolvedAssets;
        },
        get choices() {
            return choices;
        },
        get state() {
            return state;
        },
        get atEnd() {
            return atEnd;
        },
        get currentMusic() {
            return currentMusic;
        },
        get currentSounds() {
            return [...currentSounds.values()];
        },
        get currentSfxLevels() {
            return { ...sfxLevels };
        },
        get animateRatio() {
            return animateRatio;
        },
        set animateRatio(next: number) {
            animateRatio = Number.isFinite(next) ? Math.max(0, next) : 1;
        },
        get legacyClamp() {
            return legacyClamp;
        },
    };

    return engine;
}

/**
 * The public constructor: the engine plus `totalHalts`, the halt count on the
 * default-first-choice path, measured by a scratch engine that has no total
 * of its own (that is what stops the recursion).
 */
export function createEngine(script: StoryScript, options: EngineOptions): Engine {
    return build(script, options, countHalts(script, options.videos));
}

/**
 * Halts on the default-first-choice path. `videos` must be the SAME switch the
 * live engine was built with, or the total counts cutscenes the walk will
 * never reach and the scrubber's index space drifts from the engine's.
 */
export function countHalts(script: StoryScript, videos?: boolean): number {
    const probe = build(script, { nickname: "", videos }, 0);
    let n = 0;
    for (let r = probe.step(); r.halt.kind !== "end"; r = probe.step()) n += 1;
    return n;
}
