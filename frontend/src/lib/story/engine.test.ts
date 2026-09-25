/**
 * Engine tests on REAL parser output: the two fixtures under `__fixtures__`
 * are `StoryScript` JSON dumped by `backend/tests/story_real_data_test.rs`
 * (`dump_engine_fixtures_when_asked`). Regenerate them with
 * `cd backend && STORY_FIXTURE_OUT=../frontend/src/lib/story/__fixtures__ cargo test --test story_real_data_test fixtures`.
 */
import { describe, expect, it } from "vitest";
import type { StoryScript } from "#/types/generated/StoryScript";
import welcomeJson from "./__fixtures__/main_0_0_welcome_to_guide.json";
import epilogueJson from "./__fixtures__/main_15_level_main_15-08_end.json";
import { EASE_LINEAR, EASE_OUT_CUBIC } from "./ease";
import { countHalts, createEngine, type Engine, HANDLED_KINDS, MAX_HOLD_SEC, MAX_TRANSITION_SEC, type StepResult } from "./engine";

const welcome = welcomeJson as StoryScript;
const epilogue = epilogueJson as StoryScript;

function runAll(engine: Engine): StepResult[] {
    const out: StepResult[] = [];
    for (let r = engine.step(); r.halt.kind !== "end"; r = engine.step()) {
        out.push(r);
        if (out.length > 5000) throw new Error("runaway");
    }
    return out;
}

function script(commands: Array<{ kind: string; args?: Record<string, string>; text?: string }>, assets: Partial<StoryScript["assets"]> = {}): StoryScript {
    return {
        id: "t",
        name: "t",
        groupId: "g",
        wordCount: 0,
        commands: commands.map((c, i) => ({ kind: c.kind, args: c.args ?? {}, text: c.text, line: i + 1 })),
        assets: { backgrounds: {}, images: {}, characters: {}, music: {}, sounds: {}, avatars: {}, imageSizes: {}, videos: {}, ...assets },
    };
}

const SPRITE_A = { bodyUrl: "/a.png" };
const SPRITE_B = { bodyUrl: "/b.png" };

describe("real fixtures", () => {
    it("welcome_to_guide: 333 commands, 122 halts (116 lines, 6 decisions), nothing handled is listed as unhandled", () => {
        const e = createEngine(welcome, { nickname: "Doctor" });
        expect(welcome.commands).toHaveLength(333);
        const halts = runAll(e);
        expect(halts).toHaveLength(122);
        expect(halts.filter((h) => h.halt.kind === "line")).toHaveLength(116);
        expect(halts.filter((h) => h.halt.kind === "decision")).toHaveLength(6);
        expect(e.totalHalts).toBe(122);
        expect(e.atEnd).toBe(true);
        // imagetween 3 moved out of this list when the tween shipped.
        // `imageSize:missing` is a DIAGNOSTIC, not an unhandled command. The
        // fixture is dumped from a backend that carries `imageSizes` for every
        // referenced name (2,174 of 2,174 over the corpus), so none of its 12
        // `screenadapt`-less layers falls back to the prefab's 1280x720 rect
        // and the diagnostic never fires.
        const { "imageSize:missing": missingSizes, ...kinds } = e.unhandledKinds;
        expect(missingSizes).toBeUndefined();
        expect(kinds).toEqual({ skiptothis: 1, startbattle: 1, tutorial: 1 });
        for (const k of Object.keys(kinds)) expect(HANDLED_KINDS.has(k)).toBe(false);
    });

    it("welcome_to_guide: the fixture's own `imageSizes` reach the layer, so a `screenadapt`-less background is 1500.4 x 844.0 and not 1280x720", () => {
        // The wire entry, not a hand-built one: `bg_indoor_1` ships
        // `1024 x 576` at ppu 68.246445 and `bg_0_babel` `1280 x 720` at ppu
        // 100, so the first is 1500.4 x 844.0 canvas px and the second is the
        // 1280x720 the fallback happens to agree with. This is the assertion
        // that catches `imageSizes` never reaching `createEngine`: a wire read
        // that fails returns the same 1280x720 the fallback does, and only a
        // name whose plate is NOT the canvas tells the two apart.
        expect(welcome.assets.imageSizes.bg_indoor_1).toEqual({ w: 1024, h: 576, ppu: 68.246445 });
        const e = createEngine(welcome, { nickname: "Doctor" });
        const seen = new Map<string, { w: number; h: number }>();
        for (const r of runAll(e)) {
            for (const f of r.timeline) {
                for (const layer of [f.state.background, f.state.image]) {
                    if (layer?.nativeW !== undefined && layer.nativeH !== undefined) seen.set(layer.name, { w: layer.nativeW, h: layer.nativeH });
                }
            }
        }
        const round = (n: string) => [Number((seen.get(n)?.w ?? 0).toFixed(1)), Number((seen.get(n)?.h ?? 0).toFixed(1))];
        expect(round("bg_indoor_1")).toEqual([1500.4, 844.0]);
        expect(round("bg_wild_a")).toEqual([1500.4, 844.0]);
        expect(round("bg_0_babel")).toEqual([1280.0, 720.0]);
        // Every one of the fixture's seven sized names is carried, none dropped.
        expect(seen.size).toBe(7);
    });

    it("15-08_end: 570 commands, 214 halts (167 lines of which 7 stickers, 47 decisions)", () => {
        const e = createEngine(epilogue, { nickname: "Doctor" });
        expect(epilogue.commands).toHaveLength(570);
        const halts = runAll(e);
        expect(halts).toHaveLength(214);
        const lines = halts.filter((h) => h.halt.kind === "line");
        expect(lines).toHaveLength(167);
        expect(lines.filter((h) => h.halt.kind === "line" && h.halt.surface === "sticker")).toHaveLength(7);
        expect(halts.filter((h) => h.halt.kind === "decision")).toHaveLength(47);
        // focusout 4 and imagetween 1 moved out of this list when they shipped.
        expect(e.unhandledKinds).toEqual({ cgitem: 2, hidecgitem: 2 });
        for (const k of Object.keys(e.unhandledKinds)) expect(HANDLED_KINDS.has(k)).toBe(false);
    });

    it("no duration is clamped, every duration is ONE multiply by animateRatio, and `?legacyclamp=1` restores the 1.5 s pair byte for byte", () => {
        // The client scales and never clamps: `CalculateFadetime` is a load, a
        // read of `animateRatio` and an `fmul`, and the eleven call sites carry
        // no `fmin`, `fmax` or constant compare between them.
        for (const s of [welcome, epilogue]) {
            const plain = runAll(createEngine(s, { nickname: "Doctor" }));
            const half = runAll(createEngine(s, { nickname: "Doctor", animateRatio: 0.5 }));
            const skip = runAll(createEngine(s, { nickname: "Doctor", animateRatio: 0 }));
            const legacy = runAll(createEngine(s, { nickname: "Doctor", legacyClamp: true }));
            expect(half).toHaveLength(plain.length);
            for (let i = 0; i < plain.length; i++) {
                for (let f = 0; f < plain[i].timeline.length; f++) {
                    const one = plain[i].timeline[f];
                    expect(one.transitionSec).toBeGreaterThanOrEqual(0);
                    expect(half[i].timeline[f].transitionSec).toBeCloseTo(one.transitionSec / 2, 10);
                    expect(half[i].timeline[f].holdSec).toBeCloseTo(one.holdSec / 2, 10);
                    expect(skip[i].timeline[f].transitionSec).toBe(0);
                    expect(skip[i].timeline[f].holdSec).toBe(0);
                    // The kill switch is the OLD pair, clamp included.
                    expect(legacy[i].timeline[f].transitionSec).toBe(Math.min(one.transitionSec, MAX_TRANSITION_SEC));
                    expect(legacy[i].timeline[f].holdSec).toBeLessThanOrEqual(MAX_HOLD_SEC);
                }
            }
        }
        // Something in each fixture is ABOVE the old clamp, so the switch is not inert by accident.
        const longest = Math.max(...runAll(createEngine(welcome, { nickname: "Doctor" })).flatMap((r) => r.timeline.map((f) => f.transitionSec)));
        expect(longest).toBeGreaterThan(MAX_TRANSITION_SEC);
    });

    it("15-08_end: the third decision diverges on references 1 and 2 and reconverges", () => {
        const e = createEngine(epilogue, { nickname: "Doctor" });
        // Halt 14 is the decision whose predicate blocks list "1" then "2" then "1;2".
        const atDecision = e.goTo(14, {});
        expect(atDecision.halt).toEqual({ kind: "decision", options: ["How are you feeling, Amiya?", "What is my current condition, Amiya?"], values: ["1", "2"] });
        const branchOne = e.goTo(15, { 2: "1" });
        const branchTwo = e.goTo(15, { 2: "2" });
        expect(branchOne.halt.kind).toBe("line");
        expect(branchTwo.halt.kind).toBe("line");
        if (branchOne.halt.kind !== "line" || branchTwo.halt.kind !== "line") throw new Error("unreachable");
        expect(branchOne.halt.text).toBe("I just woke up myself.");
        expect(branchTwo.halt.text).not.toBe(branchOne.halt.text);
        expect(branchTwo.halt.text.length).toBeGreaterThan(0);
        expect(e.choices).toEqual({ 0: "1", 1: "1", 2: "2" });
        // Both branches reconverge on the same line after the `references="1;2"` predicate.
        const afterOne = e.goTo(19, { 2: "1" });
        const afterTwo = e.goTo(19, { 2: "2" });
        expect(afterOne.halt).toEqual(afterTwo.halt);
    });

    it("goTo is deterministic: replaying to a halt yields the same halt and one zero-length frame", () => {
        const e = createEngine(welcome, { nickname: "Doctor" });
        const forward = runAll(e);
        const target = 40;
        const replay = e.goTo(target, {});
        expect(replay.halt).toEqual(forward[target].halt);
        expect(replay.haltIndex).toBe(target);
        expect(replay.timeline).toHaveLength(1);
        expect(replay.timeline[0].transitionSec).toBe(0);
        expect(replay.timeline[0].state).toEqual(forward[target].timeline[forward[target].timeline.length - 1].state);
        // Stepping on from a replay continues the same path.
        expect(e.step().halt).toEqual(forward[target + 1].halt);
    });

    it("the nickname is substituted in halted lines", () => {
        const e = createEngine(epilogue, { nickname: "Kal" });
        const withNick = runAll(e).find((r) => r.halt.kind === "line" && /\bKal\b/.test(r.halt.text) && r.halt.speaker === "Amiya");
        expect(withNick).toBeDefined();
        const raw = epilogue.commands.some((c) => (c.text ?? "").includes("{@nickname}"));
        expect(raw).toBe(true);
    });

    it("welcome: the fade to black, the image swap and the fade back are separate frames, not one final state", () => {
        const e = createEngine(welcome, { nickname: "Doctor" });
        const halts = runAll(e);
        // Script lines 23 to 25: blocker a=1 fadetime=2, which now runs the
        // whole 2 s the script asks for, [Image] clear, then the line "No...".
        const toBlack = halts.find((r) => r.halt.kind === "line" && r.halt.text === "No...");
        expect(toBlack).toBeDefined();
        if (!toBlack) throw new Error("unreachable");
        expect(toBlack.timeline.map((f) => [f.state.blocker.a, f.state.image?.name, f.transitionSec])).toEqual([
            [1, "bg_0_babel", 2],
            [1, undefined, 0],
        ]);
        // Lines 27 to 32: [Dialog] hide, white flash from black over 0.1, grayscale, image bg_0_am, blocker a=0 over 0.3.
        const swap = halts.find((r) => r.timeline.some((f) => f.state.image?.name === "bg_0_am"));
        expect(swap).toBeDefined();
        if (!swap) throw new Error("unreachable");
        // `cameraeffect(initamount=0, amount=1)` on line 29 is TWO frames, the
        // ramp's start and its end, the same shape as a blocker's `afrom`.
        const rows = swap.timeline.map((f) => [f.state.blocker.a, f.state.blocker.r, f.state.image?.name, f.transitionSec, f.state.effects.grayscale]);
        // Channels are 0..1 and CLAMPED: `r=255` beside `a=1` is white, not a
        // signal to divide the whole command by 255.
        expect(rows.slice(0, 7)).toEqual([
            [1, 0, undefined, 0, undefined],
            [0, 0, undefined, 0, undefined],
            [1, 1, undefined, 0.1, undefined],
            [1, 1, undefined, 0, 0],
            [1, 1, undefined, 0, 1],
            [1, 1, "bg_0_am", 0, 1],
            // Line 32 is `[Blocker(a=0, fadetime=0.3)]` with no colour, and
            // r/g/b DEFAULT to 0, so the white flash tweens to transparent
            // black rather than holding the white it faded in from.
            [0, 0, "bg_0_am", 0.3, 1],
        ]);
    });
});

describe("semantics on hand-built scripts", () => {
    it("[dialog] bare hides the box; [dialog] with text is a line", () => {
        const e = createEngine(script([{ kind: "name", args: { name: "A" }, text: "hi" }, { kind: "dialog" }, { kind: "dialog", args: { head: "x" }, text: "spoken" }]), { nickname: "D" });
        const first = e.step();
        expect(first.halt).toEqual({ kind: "line", speaker: "A", text: "hi", isNarration: false, surface: "box" });
        expect(first.timeline[first.timeline.length - 1].state.dialogVisible).toBe(true);
        const second = e.step();
        expect(second.halt).toEqual({ kind: "line", text: "spoken", isNarration: false, surface: "box" });
        expect(second.timeline[0].state.dialogVisible).toBe(false);
        expect(second.timeline[second.timeline.length - 1].state.dialogVisible).toBe(true);
        expect(e.step().halt).toEqual({ kind: "end" });
    });

    it("[character] one name goes to m, the FIRST name to the LEFT slot and name2 to the right; focus -1 is exactly focus 1 and focus 3 dims both", () => {
        const s = script(
            [
                { kind: "character", args: { name: "A" } },
                { kind: "text", text: "1" },
                { kind: "character", args: { name: "A", name2: "B", focus: "2" } },
                { kind: "text", text: "2" },
                { kind: "character", args: { name: "A", name2: "B", focus: "1" } },
                { kind: "text", text: "3" },
                { kind: "character", args: { name: "A", name2: "B", focus: "-1" } },
                { kind: "text", text: "4" },
                { kind: "character" },
                { kind: "text", text: "5" },
            ],
            { characters: { A: SPRITE_A, B: SPRITE_B } },
        );
        const e = createEngine(s, { nickname: "D" });
        const last = () => {
            const r = e.step();
            return r.timeline[r.timeline.length - 1].state.slots;
        };
        // `_ExecuteCharacter` routes one name to slot 3, the middle, and two
        // names to slot 1 then slot 2. MEASURED
        // (`docs/story-reader-captures.md`, 2): with Amiya as `name` and
        // Dobermann as `name2` their ink centres are canvas x -175.3 and
        // +228.3, so slot 1 is the LEFT figure at -200 and slot 2 the right at
        // +200. The reader shipped them mirrored.
        expect(last()).toEqual({ m: expect.objectContaining({ sprite: SPRITE_A, name: "A", lit: true }) });
        expect(last()).toEqual({ l: expect.objectContaining({ name: "A", lit: false }), r: expect.objectContaining({ name: "B", lit: true }) });
        expect(last()).toEqual({ l: expect.objectContaining({ name: "A", lit: true }), r: expect.objectContaining({ name: "B", lit: false }) });
        // focus=-1 is EXACTLY focus=1: slot 2 takes the dim colour.
        expect(last()).toEqual({ l: expect.objectContaining({ name: "A", lit: true }), r: expect.objectContaining({ name: "B", lit: false }) });
        expect(last()).toEqual({});
    });

    it("?firstright=1 puts the pre-capture legacy sides back, first name on the right", () => {
        const s = script(
            [
                { kind: "character", args: { name: "A", name2: "B", focus: "1" } },
                { kind: "text", text: "1" },
            ],
            { characters: { A: SPRITE_A, B: SPRITE_B } },
        );
        const e = createEngine(s, { nickname: "D", firstNameRight: true });
        const r = e.step();
        const slots = r.timeline[r.timeline.length - 1].state.slots;
        expect(slots).toEqual({ r: expect.objectContaining({ name: "A", lit: true }), l: expect.objectContaining({ name: "B", lit: false }) });
    });

    it("[charslot] with no focus lights ALL THREE, a named focus dims the rest, n/none dims all, bare clears, odd slots are logged", () => {
        const s = script(
            [
                { kind: "charslot", args: { slot: "left", name: "A" } },
                { kind: "charslot", args: { slot: "right", name: "B", focus: "r" } },
                { kind: "text", text: "1" },
                { kind: "charslot", args: { slot: "l", focus: "n" } },
                { kind: "text", text: "2" },
                { kind: "charslot", args: { slot: "l", focus: "l,r" } },
                { kind: "text", text: "3" },
                { kind: "charslot", args: { slot: "m", focus: "none" } },
                { kind: "charslot", args: { slot: "l", name: "A#2", posto: "100,-20", ato: "0.5", action: "jump" } },
                { kind: "text", text: "4" },
                { kind: "text", text: "4b" },
                { kind: "charslot", args: { slot: "all", name: "B" } },
                { kind: "charslot", args: { slot: "l", focus: "all" } },
                { kind: "text", text: "5" },
                { kind: "charslot" },
                { kind: "text", text: "6" },
            ],
            { characters: { A: SPRITE_A, "A#2": SPRITE_A, B: SPRITE_B } },
        );
        const e = createEngine(s, { nickname: "D" });
        const last = () => {
            const r = e.step();
            return r.timeline[r.timeline.length - 1].state.slots;
        };
        expect(last()).toEqual({ l: expect.objectContaining({ name: "A", lit: false }), r: expect.objectContaining({ name: "B", lit: true }) });
        expect(last()).toEqual({ l: expect.objectContaining({ lit: false }), r: expect.objectContaining({ lit: false }) });
        expect(last()).toEqual({ l: expect.objectContaining({ lit: true }), r: expect.objectContaining({ lit: true }) });
        // A named placement with NO focus expands to the `all` literal, so both
        // slots light; the offsets and the alpha ride on the placed slot.
        expect(last()).toEqual({ l: expect.objectContaining({ name: "A#2", lit: true, x: 100, y: -20, alpha: 0.5 }), r: expect.objectContaining({ lit: true }) });
        expect(last()?.l?.action).toMatchObject({ kind: "jump", seq: 1 });
        expect(last()).toEqual({ l: expect.objectContaining({ lit: true }), r: expect.objectContaining({ lit: true }) });
        expect(last()).toEqual({});
        expect(e.unhandledKinds).toEqual({ "charslot:slot=all": 1 });
    });

    it("decision then predicates: only the chosen block runs, a full-list predicate reconverges, decisions are remembered by ordinal", () => {
        const s = script([
            { kind: "decision", args: { options: "a;b", values: "1;2" } },
            { kind: "predicate", args: { references: "1" } },
            { kind: "text", text: "one" },
            { kind: "predicate", args: { references: "2" } },
            { kind: "text", text: "two" },
            { kind: "predicate", args: { references: "1;2" } },
            { kind: "text", text: "both" },
            { kind: "decision", args: { options: "c", values: "1" } },
            { kind: "predicate", args: { references: "1" } },
            { kind: "text", text: "after" },
        ]);
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().halt).toMatchObject({ kind: "decision", options: ["a", "b"] });
        expect(e.step("2").halt).toMatchObject({ text: "two" });
        expect(e.step().halt).toMatchObject({ text: "both" });
        expect(e.step().halt).toMatchObject({ kind: "decision", options: ["c"] });
        expect(e.step().halt).toMatchObject({ text: "after" });
        expect(e.step().halt).toEqual({ kind: "end" });
        expect(e.choices).toEqual({ 0: "2", 1: "1" });
        // An unknown choice falls back to the first value.
        expect(e.goTo(1, { 0: "9" }).halt).toMatchObject({ text: "one" });
        expect(countHalts(s)).toBe(5);
    });

    it("blocker: channels CLAMP at 1 on both scales, `afrom` starts a frame at the current colour, delay holds unclamped", () => {
        const s = script([
            { kind: "blocker", args: { a: "1", r: "0", g: "0", b: "0", fadetime: "0.6", block: "true" } },
            { kind: "delay", args: { time: "3" } },
            { kind: "blocker", args: { a: "0.5", afrom: "0", r: "255", g: "255", b: "255", fadetime: "0.2" } },
            { kind: "blocker", args: { a: "0.7", r: "0.95", g: "0.95", b: "0.95", fadetime: "0.02", block: "false" } },
            { kind: "delay", args: { delay: "0.4" } },
            { kind: "text", text: "x" },
        ]);
        const r = createEngine(s, { nickname: "D" }).step();
        const frames = r.timeline;
        // Channels clamp at 1 and the two scales MIX inside one command; the
        // 3 s delay runs its whole 3 s.
        expect(frames.map((f) => [f.state.blocker.a, f.state.blocker.r, f.transitionSec, f.holdSec, f.blocking])).toEqual([
            [1, 0, 0.6, 3, true],
            [0, 0, 0, 0, true],
            [0.5, 1, 0.2, 0, true],
            [0.7, 0.95, 0.02, 0.4, false],
            // The halt shows the box: one more zero-length frame carries `dialogVisible`.
            [0.7, 0.95, 0, 0, true],
        ]);
    });

    it("image and background: set, offsets, bare clears; unknown names clear rather than throw", () => {
        const s = script(
            [{ kind: "background", args: { image: "bg", x: "10", y: "-5", xscale: "1.2", yscale: "1.2", fadetime: "1", screenadapt: "coverall" } }, { kind: "image", args: { image: "cg" } }, { kind: "text", text: "1" }, { kind: "image" }, { kind: "background", args: { image: "missing" } }, { kind: "text", text: "2" }],
            {
                backgrounds: { bg: "/bg.png" },
                images: { cg: "/cg.png" },
            },
        );
        const e = createEngine(s, { nickname: "D" });
        const a = e.step().timeline;
        expect(a[0].state.background).toEqual({ name: "bg", url: "/bg.png", x: 10, y: -5, xScale: 1.2, yScale: 1.2, adapt: "coverall", widthMul: 1, heightMul: 1, rotate: 0 });
        expect(a[0].transitionSec).toBe(1);
        // No `screenadapt` is NATIVE size, not a fill; `width`/`height` are
        // multipliers defaulting to 1, never pixels.
        expect(a[1].state.image).toEqual({ name: "cg", url: "/cg.png", x: 0, y: 0, xScale: 1, yScale: 1, adapt: "native", widthMul: 1, heightMul: 1, rotate: 0 });
        const b = e.step().timeline;
        expect(b[0].state.image).toBeUndefined();
        expect(b[1].state.background).toBeUndefined();
    });

    it("audio commands become effects with resolved urls; unresolved keys are dropped", () => {
        const s = script(
            [
                { kind: "playmusic", args: { intro: "$m_intro", key: "$m_loop", volume: "0.8", crossfade: "1.5" } },
                { kind: "musicvolume", args: { volume: "0.3", fadetime: "2" } },
                { kind: "playsound", args: { key: "$s", volume: "0.5" } },
                { kind: "playsound", args: { key: "$nope" } },
                { kind: "stopsound" },
                { kind: "stopmusic", args: { fadetime: "1" } },
                { kind: "camerashake", args: { duration: "0.5" } },
                { kind: "text", text: "x" },
            ],
            { music: { $m_loop: { intro: "/i.ogg", loop: "/l.ogg" } }, sounds: { $s: "/s.ogg" } },
        );
        expect(createEngine(s, { nickname: "D" }).step().effects).toEqual([
            { kind: "music", intro: "/i.ogg", loop: "/l.ogg", volume: 0.8, crossfade: 1.5 },
            { kind: "musicVolume", volume: 0.3, fade: 2 },
            { kind: "sound", url: "/s.ogg", volume: 0.5, loop: false, delay: 0, channel: undefined },
            { kind: "stopSound", channel: undefined, fade: 0 },
            { kind: "stopMusic", fade: 1 },
            { kind: "cameraShake", stop: false, duration: 0.5, xStrength: 1, yStrength: 0, vibrato: 10, randomness: 90, fadeOut: false },
        ]);
    });

    it("subtitle and sticker with text halt as overlays; bare forms clear; multi appends", () => {
        const s = script([
            { kind: "subtitle", args: { text: "sub", x: "300" } },
            { kind: "subtitle" },
            { kind: "sticker", args: { id: "st1", text: "a", block: "true" } },
            { kind: "sticker", args: { id: "st1", multi: "true", text: "\\nb" } },
            { kind: "sticker", args: { id: "st1" } },
            { kind: "stickerclear" },
            { kind: "text", text: "end" },
        ]);
        const e = createEngine(s, { nickname: "D" });
        const one = e.step();
        expect(one.halt).toEqual({ kind: "line", text: "sub", isNarration: true, surface: "subtitle" });
        expect(one.timeline[one.timeline.length - 1].state.subtitle).toMatchObject({ text: "sub", x: 300 });
        const two = e.step();
        expect(two.halt).toMatchObject({ surface: "sticker", text: "a" });
        expect(two.timeline[0].state.subtitle).toBeUndefined();
        const three = e.step();
        expect(three.halt).toMatchObject({ surface: "sticker", text: "\nb" });
        expect(three.timeline[three.timeline.length - 1].state.stickers.st1.text).toBe("a\nb");
        const four = e.step();
        expect(four.halt).toMatchObject({ text: "end" });
        expect(four.timeline[0].state.stickers).toEqual({});
    });

    it("cameraeffect grayscale and colorinverse are state; every other kind is counted and never throws", () => {
        const s = script([
            { kind: "cameraeffect", args: { effect: "Grayscale", amount: "1", fadetime: "0.5" } },
            { kind: "cameraeffect", args: { effect: "Colorinverse", amount: "0.8", initamount: "0" } },
            { kind: "cameraeffect", args: { effect: "sepia" } },
            { kind: "tutorial", args: {} },
            { kind: "battle" },
            { kind: "wat" },
            { kind: "header", args: { key: "x" }, text: "title" },
            { kind: "text", text: "x" },
        ]);
        const e = createEngine(s, { nickname: "D" });
        const r = e.step();
        expect(r.timeline[0].state.effects.grayscale).toBe(1);
        // `initamount` is the extra zero-length frame the ramp starts from.
        expect(r.timeline[1].state.effects.colorInverse).toBe(0);
        expect(r.timeline[2].state.effects.colorInverse).toBe(0.8);
        expect(e.unhandledKinds).toEqual({ "cameraeffect:sepia": 1, tutorial: 1, battle: 1, wat: 1 });
    });

    it("an empty script ends at once and stepping past the end stays at end", () => {
        const e = createEngine(script([]), { nickname: "D" });
        expect(e.step().halt).toEqual({ kind: "end" });
        expect(e.step().halt).toEqual({ kind: "end" });
        expect(e.totalHalts).toBe(0);
        expect(e.haltIndex).toBe(-1);
    });
});

describe("commands that reached the engine in phase 3", () => {
    it("imagetween and backgroundtween pan and zoom as a from-frame then a to-frame", () => {
        const s = script(
            [
                { kind: "image", args: { image: "cg" } },
                { kind: "imagetween", args: { xfrom: "-100", yfrom: "0", xscalefrom: "1", yscalefrom: "1", xto: "200", yto: "40", xscaleto: "1.4", yscaleto: "1.4", duration: "20", ease: "OutQuad" } },
                { kind: "background", args: { image: "bg" } },
                { kind: "backgroundtween", args: { xto: "60", xscaleto: "1.2", duration: "1", block: "false" } },
                { kind: "text", text: "x" },
            ],
            { images: { cg: "/cg.png" }, backgrounds: { bg: "/bg.png" } },
        );
        const r = createEngine(s, { nickname: "D" }).step();
        const img = r.timeline.map((f) => [f.state.image?.x, f.state.image?.xScale, f.transitionSec, f.ease]);
        // The `*from` values are their own zero-length frame, and `duration=20`
        // runs twenty seconds: there is no clamp anywhere in the client.
        expect(img[1]).toEqual([-100, 1, 0, undefined]);
        expect(img[2]).toEqual([200, 1.4, 20, "cubic-bezier(0.5, 1, 0.89, 1)"]);
        // The last frame is the halt's own (the box appearing); the tween is the one before it.
        const tween = r.timeline[r.timeline.length - 2];
        expect(tween.state.background).toMatchObject({ x: 60, xScale: 1.2 });
        expect(tween.blocking).toBe(false);
    });

    it("characteraction moves, zooms, shakes and exits the slot its `name` addresses", () => {
        const s = script(
            [
                { kind: "charslot", args: { slot: "l", name: "a" } },
                { kind: "characteraction", args: { name: "left", type: "move", xpos: "200", ypos: "-30", fadetime: "0.5" } },
                { kind: "characteraction", args: { name: "lfte", type: "zoom", scale: "1.3" } },
                { kind: "characteraction", args: { name: "left", type: "shake", times: "3" } },
                { kind: "characteraction", args: { name: "left", type: "spin" } },
                { kind: "characteraction", args: { name: "left", type: "exit", fadetime: "0.3" } },
                { kind: "text", text: "x" },
            ],
            { characters: { a: SPRITE_A } },
        );
        const e = createEngine(s, { nickname: "D" });
        const r = e.step();
        const rows = r.timeline.map((f) => [f.state.slots.l?.x, f.state.slots.l?.scale, f.state.slots.l?.action?.kind, f.ease]);
        // `MoveChar` on this path is the RELATIVE one, `localPosition + (dx,dy)`.
        expect(rows[1]).toEqual([200, 1, undefined, EASE_OUT_CUBIC]);
        expect(rows[2]).toEqual([200, 1.3, undefined, EASE_LINEAR]);
        expect(rows[3]).toEqual([200, 1.3, "shake", EASE_OUT_CUBIC]);
        expect(r.timeline[r.timeline.length - 1].state.slots.l).toBeUndefined();
        // An action type the census does not list is counted, never thrown on.
        expect(e.unhandledKinds).toEqual({ "characteraction:type=spin": 1 });
    });

    it("charactercutin shows a sprite plate and the nameless form closes it", () => {
        const s = script(
            [
                { kind: "charactercutin", args: { name: "a", offsetx: "-300", width: "800", fadetime: "0.4", fadestyle: "horiz_expand_center", style: "cutin" } },
                { kind: "text", text: "in" },
                { kind: "charactercutin", args: { widgetid: "1", fadetime: "0.4" } },
                { kind: "text", text: "out" },
            ],
            { characters: { a: SPRITE_A } },
        );
        const e = createEngine(s, { nickname: "D" });
        const inFrame = e.step();
        // `Show` NEGATES both offsets before writing `anchoredPosition`.
        expect(inFrame.timeline[0].state.cutin).toEqual({ sprite: SPRITE_A, name: "a", x: -300, y: 0, width: 800, height: 720 });
        expect(e.step().timeline[0].state.cutin).toBeUndefined();
    });

    it("curtain fills from `fillfrom` to `fillto` and the bare form clears it", () => {
        const s = script([{ kind: "curtain", args: { direction: "4", fillfrom: "0.01", fillto: "1", fadetime: "0.6", grad: "true" } }, { kind: "text", text: "shut" }, { kind: "curtain" }, { kind: "text", text: "open" }]);
        const e = createEngine(s, { nickname: "D" });
        const shut = e.step();
        expect(shut.timeline[0].state.curtain).toEqual({ direction: 4, fill: 0.01, grad: true });
        expect(shut.timeline[1].state.curtain).toEqual({ direction: 4, fill: 1, grad: true });
        expect(shut.timeline[1].transitionSec).toBe(0.6);
        expect(e.step().timeline[0].state.curtain).toBeUndefined();
    });

    it("focusout folds its five types onto bg, cg and char and ramps from `from` to `to`", () => {
        const s = script([
            { kind: "focusout", args: { type: "bg", from: "0", to: "1", duration: "1" } },
            { kind: "focusout", args: { type: "cgitem", to: "0.5" } },
            { kind: "focusout", args: { type: "char", to: "1" } },
            { kind: "focusout", args: { type: "lbg", to: "0" } },
            { kind: "text", text: "x" },
        ]);
        const r = createEngine(s, { nickname: "D" }).step();
        expect(r.timeline[0].state.focus).toEqual({ bg: 0 });
        expect(r.timeline[1].state.focus).toEqual({ bg: 1 });
        const last = r.timeline[r.timeline.length - 1];
        expect(last.state.focus).toEqual({ bg: 0, cg: 0.5, char: 1 });
    });

    it("largebg and gridbg build a panel strip and a 2 by 2 grid; the bare form clears", () => {
        const s = script(
            [
                { kind: "largebg", args: { imagegroup: "p1/p2", solidwidth: "920/920", solidheight: "720", x: "-180" } },
                { kind: "text", text: "wide" },
                { kind: "gridbg", args: { imagegroup: "p1/p2/p1/p2", solidwidth: "1280/1280/1280/1280", solidheight: "720/720/720/720", x: "-640", fadetime: "1" } },
                { kind: "text", text: "grid" },
                { kind: "largebg" },
                { kind: "text", text: "gone" },
            ],
            { backgrounds: { p1: "/p1.png", p2: "/p2.png" } },
        );
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().timeline[0].state.panels).toEqual({ urls: ["/p1.png", "/p2.png"], widths: [920, 920], height: 720, rows: 1, x: -180, y: 0 });
        expect(e.step().timeline[0].state.panels).toMatchObject({ rows: 2, x: -640, urls: ["/p1.png", "/p2.png", "/p1.png", "/p2.png"] });
        expect(e.step().timeline[0].state.panels).toBeUndefined();
    });

    it("popupdialog resolves its head through the avatars map", () => {
        const s = script(
            [
                { kind: "popupdialog", args: { dialoghead: "$avatar_amiya", dialogy: "120" } },
                { kind: "name", args: { name: "Amiya" }, text: "Doctor." },
                { kind: "popupdialog", args: {} },
                { kind: "text", text: "after" },
            ],
            { avatars: { $avatar_amiya: "/av.png" } },
        );
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().timeline[0].state.popupHead).toEqual({ key: "$avatar_amiya", url: "/av.png", x: undefined, y: 120 });
        expect(e.step().timeline[0].state.popupHead).toBeUndefined();
    });

    it("interlude puts a panel on its channel and `clear`/`switch=false` takes it off", () => {
        const s = script(
            [
                { kind: "interlude", args: { channel: "3", type: "3", slot: "m", switch: "false", pfrom: "0,0", pto: "0,0", name: "sprite", duration: "1" } },
                { kind: "text", text: "closed" },
                { kind: "interlude", args: { channel: "3", switch: "true", name: "sprite", pfrom: "-400,0", pto: "0,0", duration: "1" } },
                { kind: "text", text: "open" },
                { kind: "interlude", args: { channel: "3", clear: "true" } },
                { kind: "text", text: "gone" },
            ],
            { characters: { sprite: SPRITE_A } },
        );
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().timeline[0].state.interludes).toEqual({});
        const open = e.step();
        expect(open.timeline[0].state.interludes["3"]).toEqual({ url: "/a.png", name: "sprite", x: -400, y: 0 });
        expect(open.timeline[1].state.interludes["3"]).toEqual({ url: "/a.png", name: "sprite", x: 0, y: 0 });
        expect(e.step().timeline[0].state.interludes).toEqual({});
    });

    it("a name with no entry in `assets` produces NO url and is listed once in unresolvedAssets", () => {
        const s = script([
            { kind: "background", args: { image: "ghost_bg" } },
            { kind: "image", args: { image: "ghost_cg" } },
            { kind: "charslot", args: { slot: "m", name: "ghost_char" } },
            { kind: "character", args: { name: "ghost_char" } },
            { kind: "playmusic", args: { key: "$ghost_loop", intro: "$ghost_intro" } },
            { kind: "playsound", args: { key: "$ghost_sfx" } },
            { kind: "playsound", args: { key: "$ghost_sfx" } },
            { kind: "text", text: "x" },
        ]);
        const e = createEngine(s, { nickname: "D" });
        const r = e.step();
        // Nothing to request: no layer, no slot, no effect.
        const last = r.timeline[r.timeline.length - 1].state;
        expect(last.background).toBeUndefined();
        expect(last.image).toBeUndefined();
        expect(last.slots).toEqual({});
        expect(r.effects).toEqual([]);
        expect(e.unresolvedAssets).toEqual([
            { kind: "background", name: "ghost_bg" },
            { kind: "image", name: "ghost_cg" },
            { kind: "character", name: "ghost_char" },
            { kind: "music", name: "$ghost_loop" },
            { kind: "music", name: "$ghost_intro" },
            { kind: "sound", name: "$ghost_sfx" },
        ]);
    });

    it("a name written with a trailing space still finds its sprite: the wire keys are trimmed", () => {
        // Found in the browser on act42side_level_act42side_06_beg, where 3
        // charslot names carry a trailing space and the map key does not.
        const s = script(
            [
                { kind: "charslot", args: { slot: "m", name: "avg_x_1#1$1 " } },
                { kind: "background", args: { image: " bg_x" } },
                { kind: "playsound", args: { key: "$s " } },
                { kind: "text", text: "x" },
            ],
            { characters: { "avg_x_1#1$1": SPRITE_A }, backgrounds: { bg_x: "/bg.png" }, sounds: { $s: "/s.ogg" } },
        );
        const e = createEngine(s, { nickname: "D" });
        const r = e.step();
        const last = r.timeline[r.timeline.length - 1].state;
        expect(last.slots.m).toMatchObject({ sprite: SPRITE_A, name: "avg_x_1#1$1" });
        expect(last.background?.url).toBe("/bg.png");
        expect(r.effects).toEqual([{ kind: "sound", url: "/s.ogg", volume: 1, loop: false, delay: 0, channel: undefined }]);
        expect(e.unresolvedAssets).toEqual([]);
    });

    it("an `intro=` key with no entry of its own is resolved through the loop cue, not reported", () => {
        // Found in the browser: `$escape_intro` and `$calamity_intro` were
        // reported unresolved on main_0_level_main_00-01_beg while both intro
        // files loaded 200, because the wire keys a cue by its LOOP name.
        const s = script(
            [
                { kind: "playmusic", args: { intro: "$m_intro", key: "$m_loop" } },
                { kind: "text", text: "x" },
            ],
            { music: { $m_loop: { intro: "/i.ogg", loop: "/l.ogg" } } },
        );
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().effects).toEqual([{ kind: "music", intro: "/i.ogg", loop: "/l.ogg", volume: 1, crossfade: 0 }]);
        expect(e.unresolvedAssets).toEqual([]);
    });

    it("playsound carries delay and channel; stopsound and soundvolume address a channel", () => {
        const s = script(
            [
                { kind: "playsound", args: { key: "$s", volume: "0.6", loop: "true", delay: "0.4", channel: "bgs" } },
                { kind: "soundvolume", args: { channel: "bgs", volume: "0.2", fadetime: "2" } },
                { kind: "stopsound", args: { channel: "bgs", fadetime: "2" } },
                { kind: "text", text: "x" },
            ],
            { sounds: { $s: "/s.ogg" } },
        );
        expect(createEngine(s, { nickname: "D" }).step().effects).toEqual([
            { kind: "sound", url: "/s.ogg", volume: 0.6, loop: true, delay: 0.4, channel: "bgs" },
            { kind: "sfxVolume", volume: 0.2, fade: 2, channel: "bgs" },
            { kind: "stopSound", channel: "bgs", fade: 2 },
        ]);
    });
});

/**
 * The parity pass: every case here is a rule READ out of the client binary or
 * the shipped prefabs, and every one of them contradicts something this reader
 * shipped before. The docs are `docs/story-reader-il2cpp-scene.md`,
 * `docs/story-reader-il2cpp-characters.md` and `docs/story-reader-avg-prefab.md`.
 */
describe("parity with the client, read not inferred", () => {
    it("the TEXT is the dialog box's discriminator, and nothing else touches the box", () => {
        const s = script(
            [
                { kind: "name", args: { name: "A" }, text: "hi" },
                // A `dialog` carrying arguments but no text still HIDES: the
                // executor reads one field, the text, with zero TryGetParam calls.
                { kind: "dialog", args: { fadetime: "1", time: "2" } },
                { kind: "blocker", args: { a: "1" } },
                { kind: "delay", args: { time: "0.2" } },
                { kind: "background", args: { image: "bg" } },
                { kind: "image", args: { image: "cg" } },
                { kind: "charslot", args: { slot: "m", name: "A" } },
                { kind: "subtitle", args: { text: "sub" } },
                { kind: "decision", args: { options: "a;b", values: "1;2" } },
                { kind: "text", text: "back" },
            ],
            { backgrounds: { bg: "/bg.png" }, images: { cg: "/cg.png" }, characters: { A: SPRITE_A } },
        );
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().timeline.at(-1)?.state.dialogVisible).toBe(true);
        const hidden = e.step();
        // Every frame between the hide and the subtitle halt keeps the box hidden.
        expect(hidden.timeline.every((f) => f.state.dialogVisible === false)).toBe(true);
        expect(hidden.halt).toMatchObject({ surface: "subtitle" });
        const decision = e.step();
        expect(decision.halt.kind).toBe("decision");
        expect(decision.timeline.at(-1)?.state.dialogVisible).toBe(false);
        expect(e.step("1").timeline.at(-1)?.state.dialogVisible).toBe(true);
    });

    it("a blocker with no arguments is OPAQUE BLACK and `r=128` renders full, not half", () => {
        const s = script([
            { kind: "blocker", args: { fadetime: "0.5" } },
            { kind: "text", text: "1" },
            { kind: "blocker", args: { a: "1", r: "128", g: "0", b: "0" } },
            { kind: "text", text: "2" },
        ]);
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().timeline[0].state.blocker).toEqual({ a: 1, r: 0, g: 0, b: 0 });
        // 178 blockers in the corpus write `r=128`; Unity's Color32 conversion
        // clamps at 1, so that is FULL red and never mid-grey.
        expect(e.step().timeline[0].state.blocker).toEqual({ a: 1, r: 1, g: 0, b: 0 });
    });

    it("screenadapt is read, and `width`/`height` are multipliers rather than pixels", () => {
        const s = script(
            [
                { kind: "background", args: { image: "bg", screenadapt: "showall", width: "1.2", height: "0.8" } },
                { kind: "text", text: "1" },
                { kind: "background", args: { image: "bg", screenadapt: "coverall" } },
                { kind: "text", text: "2" },
                { kind: "background", args: { image: "bg", screenadapt: "nonsense" } },
                { kind: "text", text: "3" },
            ],
            { backgrounds: { bg: "/bg.png" } },
        );
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().timeline[0].state.background).toMatchObject({ adapt: "showall", widthMul: 1.2, heightMul: 0.8 });
        expect(e.step().timeline[0].state.background).toMatchObject({ adapt: "coverall", widthMul: 1, heightMul: 1 });
        expect(e.step().timeline[0].state.background).toMatchObject({ adapt: "native" });
        // Three `native` layers and no `imageSizes` on the wire at all.
        expect(e.unhandledKinds["imageSize:missing"]).toBe(1);
    });

    it("an absent screenadapt draws the texture at `w*100/ppu`, and a missing size is counted rather than guessed", () => {
        // `bg_cher_1` is 1024x576 at ppu 68.25, so the rect is 1500.4 x 844.0
        // canvas px before the script's own 1.1. The game measured 1500.7 x
        // 843.5 (`docs/story-reader-captures.md`, 1); the 1280x720 prefab rect
        // the reader used to draw is out by 17.2%.
        const s = script(
            [
                { kind: "background", args: { image: "bg_cher_1", x: "0", y: "20", xscale: "1.1", yscale: "1.1", fadetime: "1" } },
                { kind: "text", text: "1" },
                { kind: "background", args: { image: "nosize" } },
                { kind: "text", text: "2" },
                // A size IS on the wire, but this arm adapts, so it never reads it.
                { kind: "background", args: { image: "bg_cher_1", screenadapt: "coverall" } },
                { kind: "text", text: "3" },
            ],
            { backgrounds: { bg_cher_1: "/bg_cher_1.png", nosize: "/nosize.png" }, imageSizes: { bg_cher_1: { w: 1024, h: 576, ppu: 68.25 } } } as Partial<StoryScript["assets"]>,
        );
        const e = createEngine(s, { nickname: "D" });
        const first = e.step().timeline[0].state.background;
        expect(Number((first?.nativeW ?? 0).toFixed(1))).toBe(1500.4);
        expect(Number((first?.nativeH ?? 0).toFixed(1))).toBe(844.0);
        // `x=0, y=20` and the 1.1 scale channel are untouched by the sizing.
        expect(first).toMatchObject({ x: 0, y: 20, xScale: 1.1, yScale: 1.1, adapt: "native" });
        expect(e.unhandledKinds["imageSize:missing"]).toBeUndefined();
        // No entry for this name: the layer keeps no rect and the renderer
        // falls back to the 1280x720 that shipped, counted once.
        const second = e.step().timeline[0].state.background;
        expect(second?.nativeW).toBeUndefined();
        expect(e.unhandledKinds["imageSize:missing"]).toBe(1);
        const third = e.step().timeline[0].state.background;
        expect(third).toMatchObject({ adapt: "coverall" });
        expect(e.unhandledKinds["imageSize:missing"]).toBe(1);
    });

    it("camerashake defaults to TEN seconds with strength (1,0), and `stop=true` resets it without blocking", () => {
        const s = script([
            { kind: "camerashake", args: {} },
            { kind: "camerashake", args: { duration: "-1" } },
            { kind: "camerashake", args: { stop: "true" } },
            { kind: "camerashake", args: { duration: "0.3", xstrength: "30", ystrength: "30", vibrato: "30", randomness: "90", fadeout: "true" } },
            { kind: "text", text: "x" },
        ]);
        const r = createEngine(s, { nickname: "D" }).step();
        expect(r.effects[0]).toEqual({ kind: "cameraShake", stop: false, duration: 10, xStrength: 1, yStrength: 0, vibrato: 10, randomness: 90, fadeOut: false });
        expect(r.effects[1]).toMatchObject({ duration: 10 });
        expect(r.effects[2]).toMatchObject({ stop: true, duration: 0 });
        expect(r.effects[3]).toEqual({ kind: "cameraShake", stop: false, duration: 0.3, xStrength: 30, yStrength: 30, vibrato: 30, randomness: 90, fadeOut: true });
        // A shake is never a frame: it cannot gate the reader.
        expect(r.timeline).toHaveLength(1);
    });

    it("subtitle geometry: top-left origin, width clamped to 1280 - x, size 24, and an off-canvas command is dropped", () => {
        const s = script([
            { kind: "subtitle", args: { text: "a", x: "300", y: "370", width: "700", size: "24", alignment: "center", delay: "0.04" } },
            { kind: "subtitle", args: { text: "b", x: "900" } },
            { kind: "subtitle", args: { text: "c", x: "1400", y: "10" } },
            { kind: "text", text: "x" },
        ]);
        const e = createEngine(s, { nickname: "D" });
        expect(e.step().timeline.at(-1)?.state.subtitle).toEqual({ text: "a", x: 300, y: 370, width: 700, size: 24, alignment: "center" });
        // The default width is 1280 CLAMPED to what is left of the canvas.
        expect(e.step().timeline.at(-1)?.state.subtitle).toEqual({ text: "b", x: 900, y: 0, width: 380, size: 24, alignment: "left" });
        const dropped = e.step();
        expect(dropped.halt).toMatchObject({ text: "x" });
        expect(dropped.timeline.at(-1)?.state.subtitle).toMatchObject({ text: "b" });
        expect(e.unhandledKinds).toEqual({ "subtitle:offcanvas": 1 });
    });

    it("a repeated sticker id without `multi` HIDES it, and `block` defaults true for a new id and false for a repeat", () => {
        const s = script([
            { kind: "sticker", args: { id: "st1", text: "first" } },
            { kind: "sticker", args: { id: "st1", text: "second" } },
            { kind: "sticker", args: { id: "st2", text: "other" } },
            { kind: "text", text: "x" },
        ]);
        const e = createEngine(s, { nickname: "D" });
        const first = e.step();
        expect(first.halt).toMatchObject({ surface: "sticker", text: "first" });
        expect(first.timeline.at(-1)?.blocking).toBe(true);
        // The repeat removes it instead of replacing it, and does not halt.
        const second = e.step();
        expect(second.halt).toMatchObject({ surface: "sticker", text: "other" });
        expect(second.timeline[0].state.stickers).toEqual({});
        expect(second.timeline[0].blocking).toBe(false);
        expect(e.unhandledKinds).toEqual({ "sticker:repeatHides": 1 });
    });

    it("charslot: posfrom is its own frame, afrom/ato -1 mean unchanged, a sprite change resets to home, poszoom outside 0..1 is dropped", () => {
        const s = script(
            [
                { kind: "charslot", args: { slot: "l", name: "A", posfrom: "-400,0", posto: "0,0", afrom: "0", ato: "1", duration: "0.5" } },
                { kind: "text", text: "1" },
                { kind: "charslot", args: { slot: "l", name: "A", posto: "50,10" } },
                { kind: "text", text: "2" },
                { kind: "charslot", args: { slot: "l", name: "B", duration: "0.4" } },
                { kind: "text", text: "3" },
                { kind: "charslot", args: { slot: "l", name: "B", action: "zoom", poszoom: "0.5,0.5", scale: "0.8" } },
                { kind: "text", text: "4" },
                { kind: "charslot", args: { slot: "l", name: "B", action: "zoom", poszoom: "2,0.5", scale: "0.8" } },
                { kind: "text", text: "5" },
            ],
            { characters: { A: SPRITE_A, B: SPRITE_B } },
        );
        const e = createEngine(s, { nickname: "D" });
        const enter = e.step();
        expect(enter.timeline[0].state.slots.l).toMatchObject({ x: -400, alpha: 0 });
        expect(enter.timeline[0].transitionSec).toBe(0);
        expect(enter.timeline[1].state.slots.l).toMatchObject({ x: 0, alpha: 1 });
        // No `ato` at all leaves the alpha where it was: the default is -1, unchanged.
        expect(e.step().timeline[0].state.slots.l).toMatchObject({ x: 50, y: 10, alpha: 1 });
        // A DIFFERENT sprite resets localPosition and localScale to home.
        const swapped = e.step().timeline[0].state.slots.l;
        expect(swapped).toMatchObject({ name: "B", x: 0, y: 0, scale: 1 });
        expect(e.step().timeline[0].state.slots.l).toMatchObject({ pivotX: 0.5, pivotY: 0.5, scale: 0.8 });
        // `CharZoom` rejects the command when either component leaves 0..1.
        expect(e.step().timeline[0].state.slots.l).toMatchObject({ scale: 0.8 });
        expect(e.unhandledKinds).toEqual({ "charslot:poszoomOutOfRange": 1 });
    });

    it('imagerotate turns the CG on ease Linear, and `ease="1"` names Linear rather than no curve', () => {
        const s = script(
            [
                { kind: "image", args: { image: "cg" } },
                { kind: "imagerotate", args: { angle: "-60", fadetime: "0.3", block: "false" } },
                { kind: "imagetween", args: { xto: "10", duration: "1", ease: "1" } },
                { kind: "imagetween", args: { xto: "20", duration: "1", ease: "9" } },
                { kind: "text", text: "x" },
            ],
            { images: { cg: "/cg.png" } },
        );
        const r = createEngine(s, { nickname: "D" }).step();
        expect(r.timeline[1].state.image).toMatchObject({ rotate: -60 });
        expect(r.timeline[1].ease).toBe(EASE_LINEAR);
        expect(r.timeline[1].blocking).toBe(false);
        expect(r.timeline[2].ease).toBe(EASE_LINEAR);
        expect(r.timeline[3].ease).toBe(EASE_OUT_CUBIC);
    });

    it("a tween naming only `xTo` freezes the other channels instead of snapping them", () => {
        const s = script(
            [
                { kind: "background", args: { image: "bg", x: "40", y: "60", xscale: "1.3", yscale: "1.3" } },
                { kind: "backgroundtween", args: { xto: "-100", duration: "2" } },
                { kind: "text", text: "x" },
            ],
            { backgrounds: { bg: "/bg.png" } },
        );
        const r = createEngine(s, { nickname: "D" }).step();
        expect(r.timeline[1].state.background).toMatchObject({ x: -100, y: 60, xScale: 1.3, yScale: 1.3 });
    });

    it("the ratio reaches `[delay]`, a sound's delay and a charslot action, and the engine retunes without a rebuild", () => {
        const s = script(
            [
                { kind: "charslot", args: { slot: "m", name: "A", action: "jump", power: "40", times: "2", duration: "0.8" } },
                { kind: "delay", args: { time: "2" } },
                { kind: "playsound", args: { key: "$s", delay: "1.2" } },
                { kind: "text", text: "x" },
            ],
            { characters: { A: SPRITE_A }, sounds: { $s: "/s.ogg" } },
        );
        const e = createEngine(s, { nickname: "D", animateRatio: 0.5 });
        const r = e.step();
        expect(r.timeline[0].state.slots.m?.action).toMatchObject({ kind: "jump", power: 40, times: 2, sec: 0.4 });
        expect(r.timeline[0].holdSec).toBe(1);
        expect(r.effects[0]).toMatchObject({ delay: 0.6 });
        expect(e.animateRatio).toBe(0.5);
        e.animateRatio = 2;
        const again = createEngine(s, { nickname: "D" });
        again.animateRatio = 2;
        expect(again.step().timeline[0].holdSec).toBe(4);
    });
});

describe("cutscenes", () => {
    const CLIP = { webmUrl: "/video/main_10/main_10_enter.webm", mp4Url: "/video/main_10/main_10_enter.mp4" };
    const withClip = () =>
        script(
            [
                { kind: "video", args: { res: "video/main_10/main_10_enter.mp4" } },
                { kind: "text", text: "after" },
            ],
            {
                videos: { "video/main_10/main_10_enter.mp4": CLIP },
            },
        );

    it("a resolved [Video] is a halt carrying its sources and its own args", () => {
        const e = createEngine(withClip(), { nickname: "Doctor" });
        const halts = runAll(e);
        expect(halts).toHaveLength(2);
        const first = halts[0].halt;
        expect(first.kind).toBe("video");
        if (first.kind !== "video") throw new Error("not a video halt");
        expect(first.res).toBe("video/main_10/main_10_enter.mp4");
        expect(first.sources).toEqual(CLIP);
        expect(first.args.res).toBe("video/main_10/main_10_enter.mp4");
        // The clip covers the stage, so the box is down on its frame.
        expect(halts[0].timeline[halts[0].timeline.length - 1].state.dialogVisible).toBe(false);
        expect(halts[1].halt.kind).toBe("line");
        expect(e.unhandledKinds).toEqual({});
        expect(e.totalHalts).toBe(2);
    });

    it("an unresolved [Video] is NOT a halt: it is skipped and counted", () => {
        // 13 of the 25 `res` values the EN scripts name have no file.
        const e = createEngine(
            script([
                { kind: "video", args: { res: "video/act15side/IW01.mp4" } },
                { kind: "text", text: "after" },
            ]),
            { nickname: "Doctor" },
        );
        const halts = runAll(e);
        expect(halts).toHaveLength(1);
        expect(halts[0].halt.kind).toBe("line");
        expect(e.unhandledKinds).toEqual({ "video:missing": 1 });
        expect(e.unresolvedAssets).toEqual([{ kind: "video", name: "video/act15side/IW01.mp4" }]);
        expect(e.totalHalts).toBe(1);
    });

    it("videos: false is the kill switch, and the halt count is the one without cutscenes", () => {
        const e = createEngine(withClip(), { nickname: "Doctor", videos: false });
        const halts = runAll(e);
        expect(halts).toHaveLength(1);
        expect(halts[0].halt.kind).toBe("line");
        expect(e.unhandledKinds).toEqual({ "video:off": 1 });
        expect(e.totalHalts).toBe(1);
        // The switch is an explicit false, never a missing option: an engine
        // built without it plays the clip.
        expect(createEngine(withClip(), { nickname: "Doctor" }).totalHalts).toBe(2);
        expect(countHalts(withClip(), false)).toBe(1);
        expect(countHalts(withClip())).toBe(2);
    });

    it('"video" is a handled kind, so only the two diagnostics ever list it', () => {
        expect(HANDLED_KINDS.has("video")).toBe(true);
    });
});
