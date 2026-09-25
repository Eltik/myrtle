/**
 * The settings object is the reader's only persisted configuration, and every
 * field is read back from a string a user can hand-edit. These cases pin the
 * fields the responsive pass added: a margin out of range clamps instead of
 * throwing the text box off the stage, and a stored object written before the
 * fields existed still loads with the defaults that reproduce the old layout.
 */
import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { DEFAULT_BOTTOM_MARGIN, DEFAULT_SIDE_MARGIN } from "./canvas";
import {
    applyReadingStyle,
    BOX_PRESET_POSITIONS,
    boxPositionFromDrag,
    CUTSCENE_PLAYERS,
    clampBoxPosition,
    coerceSettings,
    coerceSpeakerTint,
    DEFAULT_NICKNAME,
    DEFAULT_SETTINGS,
    matchReadingStyle,
    NICKNAME_MAX,
    parseSettings,
    presetForPosition,
    READING_PRESETS,
    READING_STYLE_SETTINGS,
    readingStyleOf,
    resolveNickname,
    SETTINGS_KEY,
    SPEAKER_TINTS,
    sliderValue,
    withVolume,
} from "./settings";

describe("story settings", () => {
    it("the text box margins default to the game's own canvas numbers", () => {
        // The prefab's box insets its content 60 canvas px and stops 47.152 px
        // above the canvas bottom. Both sliders land on `padding`, and a
        // percentage padding resolves against the containing block's WIDTH, so
        // both defaults are fractions of 1280.
        expect(DEFAULT_SETTINGS.sideMargin).toBe(DEFAULT_SIDE_MARGIN);
        expect(DEFAULT_SETTINGS.bottomMargin).toBe(DEFAULT_BOTTOM_MARGIN);
        expect(DEFAULT_SIDE_MARGIN).toBe(4.6875);
        expect(DEFAULT_BOTTOM_MARGIN).toBe(3.6838);
        expect(DEFAULT_SETTINGS.lightBox).toBe(false);
    });

    it("fills the new fields from a settings object saved before they existed", () => {
        const old = { v: 1, cps: 40, textSize: 100, lineWidth: 70, autoPace: 1, minLineSec: 1.5, musicVolume: 0.6, sfxVolume: 0.8, muted: false, progressBar: true, nickname: "Doctor", style: "default" };
        const s = coerceSettings(old);
        expect(s.sideMargin).toBe(DEFAULT_SIDE_MARGIN);
        expect(s.bottomMargin).toBe(DEFAULT_BOTTOM_MARGIN);
        expect(s.lightBox).toBe(false);
        // A settings object with no `animateRatio` reads as the scripted speed,
        // never as zero: `Number(undefined)` is NaN and the clamp falls back.
        expect(s.animateRatio).toBe(1);
    });

    it("animateRatio clamps to 0..3 and a missing value is 1, never a falsy 0", () => {
        expect(coerceSettings({ animateRatio: 0 }).animateRatio).toBe(0);
        expect(coerceSettings({ animateRatio: 9 }).animateRatio).toBe(3);
        expect(coerceSettings({ animateRatio: -2 }).animateRatio).toBe(0);
        expect(coerceSettings({ animateRatio: "1.5" }).animateRatio).toBe(1.5);
        expect(coerceSettings({ animateRatio: null }).animateRatio).toBe(1);
        expect(coerceSettings({}).animateRatio).toBe(1);
    });

    it("clamps the margins to 0..20 and refuses a non-boolean light box", () => {
        const s = coerceSettings({ sideMargin: 999, bottomMargin: -12, lightBox: "yes" });
        expect(s.sideMargin).toBe(20);
        expect(s.bottomMargin).toBe(0);
        expect(s.lightBox).toBe(false);
    });

    it("keeps a light box that was explicitly turned on", () => {
        expect(parseSettings(JSON.stringify({ lightBox: true, sideMargin: 0 })).lightBox).toBe(true);
        expect(parseSettings(JSON.stringify({ lightBox: true, sideMargin: 0 })).sideMargin).toBe(0);
    });

    it("the letterbox defaults OFF, survives a round trip and refuses a non-boolean", () => {
        // The trade: the client paints the stage outside its 16:9 box black,
        // and the user asked for the reader to span the full width, so the
        // default extends the background across it instead. This switch takes
        // the client's own black back.
        expect(DEFAULT_SETTINGS.letterbox).toBe(false);
        expect(coerceSettings({}).letterbox).toBe(false);
        expect(coerceSettings({ letterbox: "yes" }).letterbox).toBe(false);
        expect(parseSettings(JSON.stringify({ letterbox: true })).letterbox).toBe(true);
    });
});

describe("withVolume, the toolbar's volume popover", () => {
    it("a drag while muted unmutes and takes the level", () => {
        const muted = { ...DEFAULT_SETTINGS, muted: true, musicVolume: 0.6 };
        expect(withVolume(muted, "musicVolume", [0.3])).toEqual({ ...muted, musicVolume: 0.3, muted: false });
        // Even a drag that lands on the same level unmutes: the reader touched the control.
        expect(withVolume(muted, "sfxVolume", [muted.sfxVolume]).muted).toBe(false);
    });

    it("clamps to 0..1 and keeps the current level for a missing value", () => {
        const s = { ...DEFAULT_SETTINGS, musicVolume: 0.6 };
        expect(withVolume(s, "musicVolume", [1.4]).musicVolume).toBe(1);
        expect(withVolume(s, "musicVolume", [-0.2]).musicVolume).toBe(0);
        expect(withVolume(s, "musicVolume", []).musicVolume).toBe(0.6);
        expect(withVolume(s, "musicVolume", null).musicVolume).toBe(0.6);
    });

    it("an unmuted no-op hands back the same document, so nothing is written", () => {
        const s = { ...DEFAULT_SETTINGS, musicVolume: 0.6, muted: false };
        expect(withVolume(s, "musicVolume", [0.6])).toBe(s);
    });

    it("zero is a real zero and persists as one", () => {
        expect(withVolume({ ...DEFAULT_SETTINGS, sfxVolume: 0.8 }, "sfxVolume", [0]).sfxVolume).toBe(0);
    });
});

describe("a volume the reader keeps", () => {
    it("never reads a missing slider value as zero, which for music is silence", () => {
        // `v[0] ?? 0` was the shipped reading and it persisted SILENCE.
        expect(sliderValue([], 0.4)).toBe(0.4);
        expect(sliderValue(undefined, 0.4)).toBe(0.4);
        expect(sliderValue(null, 0.4)).toBe(0.4);
        expect(sliderValue([Number.NaN], 0.4)).toBe(0.4);
        // A real zero is still a real zero: the user is allowed to mute by slider.
        expect(sliderValue([0], 0.4)).toBe(0);
        expect(sliderValue([0.15], 0.4)).toBe(0.15);
        expect(sliderValue(0.15, 0.4)).toBe(0.15);
    });

    it("writes localStorage once per change, outside the state updater", () => {
        window.localStorage.removeItem(SETTINGS_KEY);
        let writes = 0;
        const real = Storage.prototype.setItem;
        Storage.prototype.setItem = function (k: string, v: string) {
            if (k === SETTINGS_KEY) writes += 1;
            return real.call(this, k, v);
        };
        try {
            const { result } = renderHook(() => useLocalStorageState(SETTINGS_KEY, DEFAULT_SETTINGS, { parse: (raw) => coerceSettings(JSON.parse(raw)) }));
            act(() => result.current[1]({ ...DEFAULT_SETTINGS, musicVolume: 0.15 }));
            expect(writes).toBe(1);
            expect(parseSettings(window.localStorage.getItem(SETTINGS_KEY) ?? "")).toMatchObject({ musicVolume: 0.15 });
            // A second change reads the value the FIRST one resolved to, not the render closure's.
            act(() => result.current[1]((prev) => ({ ...prev, sfxVolume: 0.25 })));
            expect(writes).toBe(2);
            const stored = parseSettings(window.localStorage.getItem(SETTINGS_KEY) ?? "");
            expect(stored.musicVolume).toBe(0.15);
            expect(stored.sfxVolume).toBe(0.25);
        } finally {
            Storage.prototype.setItem = real;
        }
    });

    it("has written the new volume by the time the slider's handler returns", () => {
        window.localStorage.removeItem(SETTINGS_KEY);
        const { result } = renderHook(() => useLocalStorageState(SETTINGS_KEY, DEFAULT_SETTINGS, { parse: (raw) => coerceSettings(JSON.parse(raw)) }));
        // No `act`: this is what a pointermove on the slider does, and the write
        // must NOT be waiting inside a state updater React has not run yet.
        result.current[1]({ ...DEFAULT_SETTINGS, musicVolume: 0.15 });
        expect(window.localStorage.getItem(SETTINGS_KEY)).not.toBeNull();
        expect(parseSettings(window.localStorage.getItem(SETTINGS_KEY) ?? "").musicVolume).toBe(0.15);
    });

    it("round-trips a non-default volume through the stored string", () => {
        const stored = JSON.stringify({ ...DEFAULT_SETTINGS, musicVolume: 0.15, sfxVolume: 0.25 });
        const back = parseSettings(stored);
        expect(back.musicVolume).toBe(0.15);
        expect(back.sfxVolume).toBe(0.25);
        // A settings object written before these fields existed keeps the defaults, never 0.
        expect(coerceSettings({ cps: 35 }).musicVolume).toBe(DEFAULT_SETTINGS.musicVolume);
    });
});

describe("the new reader fields", () => {
    it("defaults to the kill switches: preset font, no colour, box bottom-centre, pills that stay", () => {
        expect(DEFAULT_SETTINGS.font).toBe("preset");
        expect(DEFAULT_SETTINGS.customFontName).toBe("");
        expect(DEFAULT_SETTINGS.textColor).toBe("");
        expect(DEFAULT_SETTINGS.boxX).toBe(0);
        expect(DEFAULT_SETTINGS.boxY).toBe(0);
        expect(DEFAULT_SETTINGS.autoHideToolbar).toBe(false);
        expect(DEFAULT_SETTINGS.toolbarHidden).toBe(false);
        expect(DEFAULT_SETTINGS.toolbarIdleSec).toBe(2.5);
        expect(DEFAULT_SETTINGS.v).toBe(2);
    });

    it("the idle wait clamps to 1..15 seconds and junk reads as the 2.5 s default", () => {
        expect(coerceSettings({ toolbarIdleSec: 8 }).toolbarIdleSec).toBe(8);
        expect(coerceSettings({ toolbarIdleSec: 0.2 }).toolbarIdleSec).toBe(1);
        expect(coerceSettings({ toolbarIdleSec: 900 }).toolbarIdleSec).toBe(15);
        expect(coerceSettings({ toolbarIdleSec: "soon" }).toolbarIdleSec).toBe(2.5);
        expect(coerceSettings({ toolbarIdleSec: null }).toolbarIdleSec).toBe(2.5);
        expect(coerceSettings({}).toolbarIdleSec).toBe(2.5);
    });

    it("a collapsed toolbar SURVIVES a reload, because it was asked for on purpose", () => {
        expect(parseSettings(JSON.stringify({ toolbarHidden: true })).toolbarHidden).toBe(true);
        expect(coerceSettings({ toolbarHidden: "yes" }).toolbarHidden).toBe(false);
        // A v1 object has no such field and reads as a toolbar that is there.
        expect(coerceSettings({ v: 1 }).toolbarHidden).toBe(false);
    });

    it("coerces an unknown font, a junk colour and an out-of-range position back into range", () => {
        const s = coerceSettings({ font: "comic", textColor: "javascript:x", boxX: 900, boxY: -4, autoHideToolbar: "yes" });
        expect(s.font).toBe("preset");
        expect(s.textColor).toBe("");
        expect(s.boxX).toBe(1);
        expect(s.boxY).toBe(0);
        expect(s.autoHideToolbar).toBe(false);
    });

    it("THE v1 -> v2 MIGRATION: a stored auto-hide is dropped, never inverted", () => {
        // `alwaysShowToolbar` was OFF by default, so it is the settings of a
        // reader who never touched the switch. Inverting it would turn the
        // auto-hide on for every one of them; the field is dropped instead and
        // the pills stay, which is what the client does.
        expect(coerceSettings({ v: 1, alwaysShowToolbar: false }).autoHideToolbar).toBe(false);
        expect(coerceSettings({ v: 1, alwaysShowToolbar: true }).autoHideToolbar).toBe(false);
        // And the object is rewritten as v2, so the drop happens once.
        expect(coerceSettings({ v: 1, alwaysShowToolbar: true }).v).toBe(2);
        // Only an explicit new field turns the fade back on.
        expect(coerceSettings({ v: 2, autoHideToolbar: true }).autoHideToolbar).toBe(true);
        // Everything else the v1 object carried survives the migration.
        const migrated = coerceSettings({ v: 1, alwaysShowToolbar: true, nickname: "Kal'tsit", cps: 90, boxY: 0.5 });
        expect(migrated.nickname).toBe("Kal'tsit");
        expect(migrated.cps).toBe(90);
        expect(migrated.boxY).toBe(0.5);
    });

    it("reads a settings object written before the second axis existed as the same lift", () => {
        // `boxLift` was a 0..40 percentage of the stage height; `boxY` is that
        // percentage over its own ceiling, so 20 is half the travel and 0 is
        // still the shipped bottom.
        expect(coerceSettings({ boxLift: 20 }).boxY).toBeCloseTo(0.5, 10);
        expect(coerceSettings({ boxLift: 40 }).boxY).toBe(1);
        expect(coerceSettings({ boxLift: 0 }).boxY).toBe(0);
        expect(coerceSettings({ boxLift: 20 }).boxX).toBe(0);
        // An object carrying BOTH reads the new field, never the legacy one.
        expect(coerceSettings({ boxLift: 40, boxY: 0.25 }).boxY).toBe(0.25);
    });

    it("keeps a swatch id and a hex colour", () => {
        expect(coerceSettings({ textColor: "s5" }).textColor).toBe("s5");
        expect(coerceSettings({ textColor: "#AABBCC" }).textColor).toBe("#AABBCC");
    });
});

describe("clampBoxPosition", () => {
    it("holds x to -1..1 and y to 0..1", () => {
        expect(clampBoxPosition({ x: -9, y: 9 })).toEqual({ x: -1, y: 1 });
        expect(clampBoxPosition({ x: 0.33, y: 0.66 })).toEqual({ x: 0.33, y: 0.66 });
    });

    it("falls back to the default rather than persisting NaN, which is the coercion trap", () => {
        expect(clampBoxPosition({ x: Number.NaN, y: undefined })).toEqual({ x: 0, y: 0 });
        expect(clampBoxPosition({})).toEqual({ x: 0, y: 0 });
    });
});

describe("the position presets", () => {
    it("maps the five presets onto the corners the names promise", () => {
        expect(BOX_PRESET_POSITIONS.bottomCentre).toEqual({ x: 0, y: 0 });
        expect(BOX_PRESET_POSITIONS.bottomLeft).toEqual({ x: -1, y: 0 });
        expect(BOX_PRESET_POSITIONS.bottomRight).toEqual({ x: 1, y: 0 });
        expect(BOX_PRESET_POSITIONS.centre).toEqual({ x: 0, y: 0.5 });
        expect(BOX_PRESET_POSITIONS.topCentre).toEqual({ x: 0, y: 1 });
    });

    it("names the preset a position IS, and nothing for a position between them", () => {
        expect(presetForPosition({ x: 0, y: 0 })).toBe("bottomCentre");
        expect(presetForPosition({ x: 1, y: 0 })).toBe("bottomRight");
        expect(presetForPosition({ x: 0.4, y: 0.2 })).toBeNull();
    });

    it("bottomCentre IS the default, which is what makes Reset the kill switch", () => {
        expect(BOX_PRESET_POSITIONS.bottomCentre).toEqual({ x: DEFAULT_SETTINGS.boxX, y: DEFAULT_SETTINGS.boxY });
    });
});

describe("boxPositionFromDrag", () => {
    const travel = { x: 554, y: 334.4 };

    it("raises the box when the pointer goes UP, which is a negative delta", () => {
        expect(boxPositionFromDrag({ x: 0, y: 0 }, 0, -167.2, travel).y).toBeCloseTo(0.5, 10);
    });

    it("moves it across by the pointer's own pixels: half the free room is the full half-span", () => {
        expect(boxPositionFromDrag({ x: 0, y: 0 }, 277, 0, travel).x).toBeCloseTo(1, 10);
        expect(boxPositionFromDrag({ x: 0, y: 0 }, -138.5, 0, travel).x).toBeCloseTo(-0.5, 10);
    });

    it("clamps at both ends on both axes", () => {
        expect(boxPositionFromDrag({ x: 0, y: 0 }, 5000, 5000, travel)).toEqual({ x: 1, y: 0 });
        expect(boxPositionFromDrag({ x: 0, y: 0 }, -5000, -5000, travel)).toEqual({ x: -1, y: 1 });
    });

    it("keeps the axis rather than persisting NaN when its travel is zero or missing", () => {
        // A box the width slider has already filled the stage with has no
        // horizontal room, and must not jump when it is dragged sideways.
        expect(boxPositionFromDrag({ x: 0.4, y: 0.2 }, 300, 0, { x: 0, y: 334.4 })).toEqual({ x: 0.4, y: 0.2 });
        expect(boxPositionFromDrag({ x: 0.4, y: 0.2 }, 0, -100, { x: 554, y: 0 })).toEqual({ x: 0.4, y: 0.2 });
        expect(boxPositionFromDrag({ x: 0.4, y: 0.2 }, Number.NaN, Number.NaN, travel)).toEqual({ x: 0.4, y: 0.2 });
    });
});

describe("the cutscene player setting", () => {
    // Priced before it was defaulted: the full player is a lazy chunk of
    // 88.26 kB gzipped plus 10.66 kB of stylesheet, fetched only by a reader
    // who actually reaches a clip, and the reader's own route chunk moves
    // 27.61 -> 28.29 kB gzipped for the switch.
    it("defaults to the full player", () => {
        expect(DEFAULT_SETTINGS.cutscenePlayer).toBe("vidstack");
    });

    it("offers three layers and nothing else", () => {
        expect(CUTSCENE_PLAYERS).toEqual(["simple", "native", "vidstack"]);
    });

    it("keeps each of the three stored choices", () => {
        for (const layer of CUTSCENE_PLAYERS) expect(coerceSettings({ cutscenePlayer: layer })).toMatchObject({ cutscenePlayer: layer });
    });

    // A settings object predates the field by every version before this one, so
    // the ABSENT case is the common one and it has to land on the default
    // rather than on a layer nobody picked.
    it("coerces a missing field, junk and a non-string to the default", () => {
        expect(coerceSettings({}).cutscenePlayer).toBe(DEFAULT_SETTINGS.cutscenePlayer);
        expect(coerceSettings({ cutscenePlayer: "plyr" }).cutscenePlayer).toBe(DEFAULT_SETTINGS.cutscenePlayer);
        expect(coerceSettings({ cutscenePlayer: 2 }).cutscenePlayer).toBe(DEFAULT_SETTINGS.cutscenePlayer);
        expect(coerceSettings({ cutscenePlayer: null }).cutscenePlayer).toBe(DEFAULT_SETTINGS.cutscenePlayer);
    });

    it("leaves the cutscene toggle alone: the layer says HOW a clip plays, never whether", () => {
        expect(coerceSettings({ cutscenePlayer: "simple" }).playVideos).toBe(true);
        expect(coerceSettings({ cutscenePlayer: "vidstack", playVideos: false }).playVideos).toBe(false);
    });
});

describe("the speaker colour settings", () => {
    it("defaults to off, which is the hashed plate and the reader's own text colour, exactly what the reader shipped", () => {
        expect(DEFAULT_SETTINGS.speakerTint).toBe("off");
        expect(coerceSettings({}).speakerTint).toBe("off");
    });

    it("offers three modes and nothing else", () => {
        expect(SPEAKER_TINTS).toEqual(["off", "text", "name"]);
    });

    it("keeps each of the three stored choices", () => {
        for (const mode of SPEAKER_TINTS) expect(coerceSettings({ speakerTint: mode })).toMatchObject({ speakerTint: mode });
    });

    // The two-control shape this replaced: `hash` | `sprite` beside a
    // `colorDialogue` switch. The three states a reader could be in map one to
    // one onto the three the select offers, so nobody loses their choice.
    it("migrates the sprite plate with the dialogue switch ON to text and name", () => {
        expect(coerceSpeakerTint({ speakerTint: "sprite", colorDialogue: true })).toBe("text");
        expect(coerceSettings({ speakerTint: "sprite", colorDialogue: true })).toMatchObject({ speakerTint: "text" });
    });

    it("migrates the sprite plate with the switch off, or absent, to name only", () => {
        expect(coerceSpeakerTint({ speakerTint: "sprite", colorDialogue: false })).toBe("name");
        expect(coerceSpeakerTint({ speakerTint: "sprite" })).toBe("name");
        expect(coerceSettings({ speakerTint: "sprite", colorDialogue: false })).toMatchObject({ speakerTint: "name" });
    });

    it("migrates the hashed plate to off, with or without the switch", () => {
        expect(coerceSpeakerTint({ speakerTint: "hash" })).toBe("off");
        expect(coerceSpeakerTint({ speakerTint: "hash", colorDialogue: true })).toBe("off");
    });

    it("coerces junk, a missing field and a non-boolean switch to off", () => {
        expect(coerceSpeakerTint({ speakerTint: "rainbow" })).toBe("off");
        expect(coerceSpeakerTint({})).toBe("off");
        expect(coerceSpeakerTint({ speakerTint: 7 })).toBe("off");
        expect(coerceSpeakerTint({ speakerTint: "sprite", colorDialogue: "yes" })).toBe("name");
        expect(coerceSettings({ speakerTint: "rainbow", colorDialogue: "yes" })).toMatchObject({ speakerTint: "off" });
    });

    it("drops the retired switch instead of carrying it forward", () => {
        expect(coerceSettings({ speakerTint: "sprite", colorDialogue: true })).not.toHaveProperty("colorDialogue");
    });

    it("lands a settings object written before this pass on off, so nothing changes under an old store", () => {
        expect(coerceSettings({ v: 2, cps: 40 })).toMatchObject({ speakerTint: "off" });
    });
});

/**
 * A reading style is a BUNDLE of settings, not a second rendering path: the
 * select writes five fields and reads them back, so these pin the round trip
 * in both directions and the one value that has to stay inert.
 */
describe("reading styles", () => {
    it("restores the shipped reader exactly on Default, which is the kill switch", () => {
        const edited = applyReadingStyle({ ...DEFAULT_SETTINGS, font: "mono", textSize: 180, lineWidth: 110, lineHeight: 2.1, letterSpacing: 0.1 }, "default");
        expect(edited.font).toBe(DEFAULT_SETTINGS.font);
        expect(edited.textSize).toBe(DEFAULT_SETTINGS.textSize);
        expect(edited.lineWidth).toBe(DEFAULT_SETTINGS.lineWidth);
        expect(edited.lineHeight).toBe(DEFAULT_SETTINGS.lineHeight);
        expect(edited.letterSpacing).toBe(DEFAULT_SETTINGS.letterSpacing);
        expect(edited.style).toBe("default");
    });

    it("writes each preset's whole bundle and reads the same name back", () => {
        for (const name of READING_PRESETS) {
            const next = applyReadingStyle(DEFAULT_SETTINGS, name);
            expect(next).toMatchObject(READING_STYLE_SETTINGS[name]);
            expect(matchReadingStyle(next)).toBe(name);
            expect(readingStyleOf(next)).toBe(name);
        }
    });

    it("puts OpenDyslexic, a taller line and wide tracking on the dyslexia preset", () => {
        const next = applyReadingStyle(DEFAULT_SETTINGS, "dyslexia");
        expect(next.font).toBe("dyslexic");
        expect(next.lineHeight).toBe(1.9);
        expect(next.letterSpacing).toBe(0.06);
    });

    it("leaves the box position and every unrelated setting where the reader put them", () => {
        const mine = { ...DEFAULT_SETTINGS, boxX: -1, boxY: 0.5, cps: 90, muted: true, nickname: "Kal'tsit" };
        const next = applyReadingStyle(mine, "large");
        expect(next.boxX).toBe(-1);
        expect(next.boxY).toBe(0.5);
        expect(next.cps).toBe(90);
        expect(next.muted).toBe(true);
        expect(next.nickname).toBe("Kal'tsit");
    });

    it("reads as Custom once ANY one of the five is edited by hand", () => {
        const base = applyReadingStyle(DEFAULT_SETTINGS, "dyslexia");
        for (const edit of [{ font: "mono" as const }, { textSize: 120 }, { lineWidth: 61 }, { lineHeight: 1.85 }, { letterSpacing: 0.05 }]) {
            expect(readingStyleOf({ ...base, ...edit })).toBe("custom");
        }
    });

    it("reads a preset's name back when the five are set to its values one at a time", () => {
        const b = READING_STYLE_SETTINGS.comfortable;
        expect(readingStyleOf({ ...DEFAULT_SETTINGS, ...b })).toBe("comfortable");
    });

    it("carries a stored style written before line height existed at the size it rendered at", () => {
        // The old object named the style and nothing else; reading it back at
        // the DEFAULT line height would have flattened a dyslexia reader.
        const old = coerceSettings({ v: 2, style: "dyslexia", textSize: 100 });
        expect(old.lineHeight).toBe(READING_STYLE_SETTINGS.dyslexia.lineHeight);
        expect(old.letterSpacing).toBe(READING_STYLE_SETTINGS.dyslexia.letterSpacing);
        // It reads as Custom, because the size and width it stored are not the
        // preset's: the RENDER is unchanged, the label is not, and the stored
        // name is reconciled rather than left to name a preset it is not.
        expect(readingStyleOf(old)).toBe("custom");
        expect(old.style).toBe("custom");
    });

    it("keeps an unknown stored style off the bundle and on the defaults", () => {
        const junk = coerceSettings({ v: 2, style: "neon" });
        expect(junk.style).toBe("default");
        expect(junk.lineHeight).toBe(DEFAULT_SETTINGS.lineHeight);
    });

    it("clamps a hand-edited line height and tracking instead of rendering a collapsed box", () => {
        expect(coerceSettings({ lineHeight: 99, letterSpacing: 9 })).toMatchObject({ lineHeight: 2.2, letterSpacing: 0.12 });
        expect(coerceSettings({ lineHeight: -3, letterSpacing: -3 })).toMatchObject({ lineHeight: 1.2, letterSpacing: 0 });
        expect(coerceSettings({ lineHeight: "tall" })).toMatchObject({ lineHeight: DEFAULT_SETTINGS.lineHeight });
    });

    it("reads a stored Custom whose five values ARE the defaults back as Default, because that is what it is", () => {
        // `custom` is a derived state, never a stored fact: an object carrying
        // the name and none of the fields is the shipped reader, and calling it
        // Custom would print a state the reader is not in.
        expect(coerceSettings({ v: 2, style: "custom" }).style).toBe("default");
    });

    it("reconciles a stored name that a hand-edit has made false, in BOTH directions", () => {
        // The defect this pins: a stored `style: dyslexia` beside a text size
        // the preset never writes kept naming the preset in the select.
        const lying = coerceSettings({ ...applyReadingStyle(DEFAULT_SETTINGS, "dyslexia"), textSize: 140 });
        expect(lying.style).toBe("custom");
        // And the other way: a stored `custom` whose five values ARE a preset's
        // reads as that preset rather than staying stuck on Custom.
        const honest = coerceSettings({ ...applyReadingStyle(DEFAULT_SETTINGS, "large"), style: "custom" });
        expect(honest.style).toBe("large");
    });
});

describe("resolveNickname", () => {
    it("falls back to Doctor on an empty field, on spaces alone, and on a name trimmed to nothing", () => {
        expect(resolveNickname("")).toBe(DEFAULT_NICKNAME);
        expect(resolveNickname("   ")).toBe(DEFAULT_NICKNAME);
        expect(resolveNickname("\t\n ")).toBe(DEFAULT_NICKNAME);
    });

    it("trims the name it is given and keeps the inner spaces", () => {
        expect(resolveNickname("  Amiya  ")).toBe("Amiya");
        expect(resolveNickname(" Doctor Kal'tsit ")).toBe("Doctor Kal'tsit");
    });

    it("caps at 24 characters and does not leave a trailing space behind the cut", () => {
        const long = "A".repeat(40);
        expect(resolveNickname(long)).toHaveLength(NICKNAME_MAX);
        expect(resolveNickname(`${"B".repeat(NICKNAME_MAX)} tail`)).toBe("B".repeat(NICKNAME_MAX));
        // The cut lands on a space: the trailing one goes rather than being substituted into the line.
        expect(resolveNickname(`${"C".repeat(23)} D`)).toBe("C".repeat(23));
    });

    it("stores an emptied field as Doctor rather than as a blank", () => {
        expect(coerceSettings({ nickname: "" }).nickname).toBe(DEFAULT_NICKNAME);
        expect(coerceSettings({ nickname: "  Priestess  " }).nickname).toBe("Priestess");
        expect(coerceSettings({ nickname: 42 }).nickname).toBe(DEFAULT_NICKNAME);
    });
});
