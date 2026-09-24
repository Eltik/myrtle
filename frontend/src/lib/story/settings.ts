/**
 * Reader settings: one versioned object under `myrtle.story.settings`. Every
 * field is coerced on read so a hand-edited or stale value degrades to its
 * default rather than breaking the reader.
 */

import { STORY_FONTS, type StoryFont } from "#/components/story/reader/fonts";
import { readStoredValue, useLocalStorageState, writeStoredValue } from "#/hooks/use-local-storage-state";
import { DEFAULT_BOTTOM_MARGIN, DEFAULT_SIDE_MARGIN } from "./canvas";
import { clamp } from "./num";

export const SETTINGS_KEY = "myrtle.story.settings";

/** The four presets a reader can CHOOSE. `custom` is not one of them: it is what the select reads once a preset's own settings have been edited. */
export const READING_PRESETS = ["default", "comfortable", "large", "dyslexia"] as const;
export type ReadingPreset = (typeof READING_PRESETS)[number];

export const READING_STYLES = [...READING_PRESETS, "custom"] as const;
export type ReadingStyle = (typeof READING_STYLES)[number];

/**
 * Where the speaker's own colour goes: nowhere, on the line and the plate, or
 * on the plate alone.
 *
 * `off` is the DEFAULT and the kill switch: the djb2 hue over the speaker's
 * name that the reader shipped with, seven curated tints per surface, and a
 * line in the reader's own text colour. The other two SAMPLE the body PNG of
 * the lit sprite on stage and ink that character's own dominant colour, which
 * is what the user asked for: the skin table's colour list is per OUTFIT and
 * says nothing about the sprite actually on screen, so the pixels are the
 * source.
 *
 * `text` is the one the user asked for by name, "I wanted it to change the
 * text colour, not the speaker colour": the DIALOGUE LINE takes the ink and
 * the plate takes it with her, because a line in one colour under a plate in
 * another reads as two speakers. `name` is the plate alone, which is the
 * smaller change and what the first pass shipped as `sprite`.
 */
export const SPEAKER_TINTS = ["off", "text", "name"] as const;
export type SpeakerTint = (typeof SPEAKER_TINTS)[number];

/**
 * The stored `speakerTint`, INCLUDING the two-control shape this replaced.
 *
 * The first pass stored `"hash" | "sprite"` beside a `colorDialogue` switch,
 * so the three states a reader could actually be in map one to one onto the
 * three the select now offers: `sprite` with the switch ON was the line and
 * the plate (`text`), `sprite` with it off was the plate alone (`name`), and
 * `hash` was neither (`off`). A missing field, an unknown string and a
 * `colorDialogue` that is not a boolean all land on `off`, which is the
 * shipped default, so junk can never hand a reader a colour they did not ask
 * for.
 */
export function coerceSpeakerTint(raw: { speakerTint?: unknown; colorDialogue?: unknown }): SpeakerTint {
    const v = String(raw.speakerTint);
    if ((SPEAKER_TINTS as readonly string[]).includes(v)) return v as SpeakerTint;
    if (v === "sprite") return raw.colorDialogue === true ? "text" : "name";
    return "off";
}

/**
 * WHICH controls a cutscene carries, and it is three layers over the same clip.
 *
 * `simple` is the layer the reader shipped with: a bare `<video>`, no controls
 * at all, and one Skip pill. It is the KILL SWITCH for both of the others and
 * it is byte-for-byte the element that shipped, so picking it restores the
 * reader that existed before this setting.
 *
 * `native` is that same element with the browser's own controls attribute, so
 * scrubbing, volume, fullscreen and picture-in-picture are whatever Chrome,
 * Safari or Firefox each offer; nothing is downloaded to get them.
 *
 * `vidstack` is the full player, lazily loaded, and it costs a chunk the
 * reader does not otherwise fetch.
 */
export const CUTSCENE_PLAYERS = ["simple", "native", "vidstack"] as const;
export type CutscenePlayer = (typeof CUTSCENE_PLAYERS)[number];

export interface StorySettings {
    v: 2;
    /** Characters per second for the typewriter reveal, 10..120. */
    cps: number;
    /** Text size in percent of the base, 70..200. */
    textSize: number;
    /** Line width in `ch`, 40..120. */
    lineWidth: number;
    /** Text box inset from the stage sides, percent of stage width, 0..20. */
    sideMargin: number;
    /** Text box inset from the stage bottom; a percentage padding, so also percent of stage WIDTH, 0..20. */
    bottomMargin: number;
    /** Light reading surface: the text box flips to paper, the site theme does not. */
    lightBox: boolean;
    /**
     * The stage OUTSIDE the 16:9 canvas box. The client paints it pure black
     * (`fit_mode="BLACK_MASK"`, `docs/story-reader-captures.md`, 0); our
     * default instead extends the background across it blurred and darkened,
     * so a wide screen sees a continuation. This switch takes the client's
     * black back, and `?mask=1` does the same for one run.
     */
    letterbox: boolean;
    /**
     * `AVGController.animateRatio`: the ONE multiplier on every scene duration,
     * 0..3, default 1. This is the game's own speed control and it is SEPARATE
     * from the auto-play pace below, which decides how long a finished line
     * waits before the reader advances itself.
     */
    animateRatio: number;
    /** Auto-play pace multiplier, 0.5..3. */
    autoPace: number;
    /** Minimum seconds a line stays before auto-play advances, 0..10. */
    minLineSec: number;
    musicVolume: number;
    sfxVolume: number;
    muted: boolean;
    progressBar: boolean;
    /**
     * "Play cutscene videos": a `[Video]` halts the reader and plays its clip.
     * ON by default, which is what the game does. Off skips every cutscene,
     * and so does `?video=0`, which is the URL half of the same switch.
     */
    playVideos: boolean;
    /**
     * The controls a playing cutscene carries. Read only while a clip is on
     * screen, so changing it costs nothing to a reader who never meets one.
     */
    cutscenePlayer: CutscenePlayer;
    /**
     * The Doctor's name, substituted into every `{@nickname}` in the corpus.
     * The DEFAULT is the literal "Doctor", not the signed-in account's own
     * nickname: the reader runs signed out, and an account name would make the
     * same line read differently for two people looking at the same link.
     * Read through `resolveNickname`, so a field emptied while the dialog is
     * open falls straight back to "Doctor" rather than printing nothing.
     */
    nickname: string;
    /**
     * The reading style the settings currently ARE. A preset name while every
     * field in its bundle still matches, `custom` the moment one of them is
     * edited; nothing reads it but the select, because the render reads the
     * bundle fields themselves.
     */
    style: ReadingStyle;
    /** Line height for the dialogue, 1.2..2.2. Written by the reading style. */
    lineHeight: number;
    /** Tracking in `em`, 0..0.12. Written by the reading style; the dyslexia preset is the one that needs it. */
    letterSpacing: number;
    /**
     * The font FAMILY override, and one of the fields a reading style writes.
     * `preset` is the reader's own default face; everything else replaces the
     * family only, never the style's size, line height or tracking.
     */
    font: StoryFont;
    /** The uploaded face's file name, for the settings row. The bytes live in IndexedDB. */
    customFontName: string;
    /**
     * Dialogue and narration colour: `""` (the default and the kill switch, the
     * box keeps its own colour), a curated swatch id `s0`..`s7`, or a `#rrggbb`
     * from the native colour input. Never applied to the speaker plate.
     */
    textColor: string;
    /**
     * Where the speaker's colour goes. `off` is the shipped behaviour.
     * NARRATION keeps the text colour in every mode: it has no speaker, and
     * tinting it prints a voice on a line nobody is saying.
     */
    speakerTint: SpeakerTint;
    /**
     * Where the text box sits on the stage, on TWO axes.
     *
     * `boxY` is the lift, 0..1 of `MAX_BOX_LIFT` percent of the STAGE HEIGHT:
     * 0 is the default and resolves to `bottom: 0%`, which is the `bottom-0`
     * the box shipped with, and 1 lifts it 40% of the stage. `boxX` is the
     * horizontal shift over the free space the box does not occupy, -1 flush
     * against the left side margin, 0 centred (the shipped position, and the
     * only value that keeps the flex centring the box shipped with) and 1
     * flush right. The side and bottom margin sliders are untouched and keep
     * their own defaults.
     */
    boxY: number;
    boxX: number;
    /**
     * The toolbar's auto-hide, now OPT-IN: with it on the pills and the
     * scrubber fade after `CHROME_IDLE_MS` of no pointer and no key, and any
     * movement or key brings them back. Off by default, because the client
     * never hides its own story chrome (`docs/story-reader.md`, the reader
     * chrome paragraphs). This is the inverse of the `v: 1` field
     * `alwaysShowToolbar`, whose value is deliberately NOT carried over.
     */
    autoHideToolbar: boolean;
    /**
     * How long the auto-hide waits, in seconds, 1..15. Read through
     * `chromeIdleMs`, which clamps it again, so a junk value can never resolve
     * to a bar that vanishes the instant it is drawn.
     */
    toolbarIdleSec: number;
    /**
     * "Hide toolbar", the chevron at the end of a pill and `T`: the pills and
     * the scrubber go and the TEXT BOX STAYS. Persisted, unlike theater mode,
     * because a reader who collapses the toolbar means it for the next story
     * too; a handle at the top edge brings it back.
     */
    toolbarHidden: boolean;
}

/** The Doctor's name when the reader has not set one, and the cap on the one they do. */
export const DEFAULT_NICKNAME = "Doctor";
export const NICKNAME_MAX = 24;

/**
 * The name the engine actually substitutes. Trimmed, capped, and an EMPTY
 * field falls back to "Doctor" rather than printing a blank where a name goes:
 * the fallback is here and not only in `coerceSettings`, so clearing the field
 * with the dialog open reads back as "Doctor" on the very next line instead of
 * on the next page load.
 */
export function resolveNickname(raw: string): string {
    const trimmed = raw.trim().slice(0, NICKNAME_MAX).trim();
    return trimmed === "" ? DEFAULT_NICKNAME : trimmed;
}

export const DEFAULT_SETTINGS: StorySettings = {
    v: 2,
    cps: 40,
    textSize: 100,
    lineWidth: 70,
    sideMargin: DEFAULT_SIDE_MARGIN,
    bottomMargin: DEFAULT_BOTTOM_MARGIN,
    lightBox: false,
    letterbox: false,
    animateRatio: 1,
    autoPace: 1,
    minLineSec: 1.5,
    musicVolume: 0.6,
    sfxVolume: 0.8,
    muted: false,
    progressBar: true,
    playVideos: true,
    cutscenePlayer: "vidstack",
    nickname: DEFAULT_NICKNAME,
    style: "default",
    lineHeight: 1.55,
    letterSpacing: 0,
    font: "preset",
    customFontName: "",
    textColor: "",
    speakerTint: "off",
    boxY: 0,
    boxX: 0,
    autoHideToolbar: false,
    toolbarIdleSec: 2.5,
    toolbarHidden: false,
};

/**
 * The five fields a reading style OWNS. A preset is a bundle of settings the
 * reader already had, not a second rendering path: choosing one writes these
 * five and the text box reads them the way it reads every other setting, so
 * "Dyslexia friendly" and the same five values set by hand are the same
 * reader.
 */
export interface ReadingStyleBundle {
    font: StoryFont;
    textSize: number;
    lineWidth: number;
    lineHeight: number;
    letterSpacing: number;
}

/**
 * The four presets. `default` is the KILL SWITCH and it is verified inert:
 * every field in it is `DEFAULT_SETTINGS`'s own value, so picking it restores
 * exactly the reader that shipped (font `preset`, 100%, 70ch, line height
 * 1.55, no tracking).
 *
 * The values sit on the sliders' own steps (5 for size and width, 0.05 for
 * line height), so the first drag after a preset moves by one step instead of
 * snapping off an in-between number.
 *
 * "Dyslexia friendly" is the one preset that brings its own TYPEFACE:
 * OpenDyslexic, weighted bottoms and distinct ascenders, under the SIL Open
 * Font License 1.1. The rest is what the research asks for around it, a taller
 * line and wide tracking, which a face alone does not give.
 */
export const READING_STYLE_SETTINGS: Record<ReadingPreset, ReadingStyleBundle> = {
    default: { font: "preset", textSize: 100, lineWidth: 70, lineHeight: 1.55, letterSpacing: 0 },
    comfortable: { font: "display", textSize: 110, lineWidth: 65, lineHeight: 1.7, letterSpacing: 0.005 },
    large: { font: "preset", textSize: 130, lineWidth: 55, lineHeight: 1.65, letterSpacing: 0.01 },
    dyslexia: { font: "dyslexic", textSize: 115, lineWidth: 60, lineHeight: 1.9, letterSpacing: 0.06 },
};

/** The settings a preset writes, leaving every other field (the box position included) where the reader put it. */
export function applyReadingStyle(settings: StorySettings, preset: ReadingPreset): StorySettings {
    return { ...settings, ...READING_STYLE_SETTINGS[preset], style: preset };
}

/**
 * The preset a settings object IS, or null when no preset's whole bundle
 * matches. Line height and tracking are compared with a tolerance because they
 * come back off a slider and out of JSON; the other three are exact.
 */
export function matchReadingStyle(settings: Pick<StorySettings, keyof ReadingStyleBundle>): ReadingPreset | null {
    for (const name of READING_PRESETS) {
        const b = READING_STYLE_SETTINGS[name];
        if (settings.font !== b.font || settings.textSize !== b.textSize || settings.lineWidth !== b.lineWidth) continue;
        if (Math.abs(settings.lineHeight - b.lineHeight) > 1e-9 || Math.abs(settings.letterSpacing - b.letterSpacing) > 1e-9) continue;
        return name;
    }
    return null;
}

/** What the select shows: the preset the values match, `custom` otherwise. */
export function readingStyleOf(settings: Pick<StorySettings, keyof ReadingStyleBundle>): ReadingStyle {
    return matchReadingStyle(settings) ?? "custom";
}

/** The bundle a stored `style` names, for the fields that object predates. An unknown or absent style has none. */
function storedBundle(r: Record<string, unknown>): ReadingStyleBundle | null {
    const name = String(r.style);
    return (READING_PRESETS as readonly string[]).includes(name) ? READING_STYLE_SETTINGS[name as ReadingPreset] : null;
}

function clampNumber(v: unknown, min: number, max: number, fallback: number): number {
    const n = typeof v === "number" ? v : typeof v === "string" ? Number(v) : Number.NaN;
    if (!Number.isFinite(n)) return fallback;
    return clamp(n, min, max);
}

/**
 * Read a settings object back, EVERY field coerced.
 *
 * The reading style is the one field that is not read at face value. It names
 * a bundle of five other fields, so a stored `style` that disagrees with them
 * is a lie a hand-edit, an older shape or another tab can all write; the name
 * is reconciled against the values here and the select is handed `custom`
 * rather than a preset whose numbers are not on screen.
 */
export function coerceSettings(raw: unknown): StorySettings {
    if (typeof raw !== "object" || raw === null || Array.isArray(raw)) return { ...DEFAULT_SETTINGS };
    const r = raw as Record<string, unknown>;
    const d = DEFAULT_SETTINGS;
    const out: StorySettings = {
        v: 2,
        cps: clampNumber(r.cps, 10, 120, d.cps),
        textSize: clampNumber(r.textSize, 70, 200, d.textSize),
        lineWidth: clampNumber(r.lineWidth, 40, 120, d.lineWidth),
        sideMargin: clampNumber(r.sideMargin, 0, 20, d.sideMargin),
        bottomMargin: clampNumber(r.bottomMargin, 0, 20, d.bottomMargin),
        lightBox: typeof r.lightBox === "boolean" ? r.lightBox : d.lightBox,
        letterbox: typeof r.letterbox === "boolean" ? r.letterbox : d.letterbox,
        animateRatio: clampNumber(r.animateRatio, 0, 3, d.animateRatio),
        autoPace: clampNumber(r.autoPace, 0.5, 3, d.autoPace),
        minLineSec: clampNumber(r.minLineSec, 0, 10, d.minLineSec),
        musicVolume: clampNumber(r.musicVolume, 0, 1, d.musicVolume),
        sfxVolume: clampNumber(r.sfxVolume, 0, 1, d.sfxVolume),
        muted: typeof r.muted === "boolean" ? r.muted : d.muted,
        progressBar: typeof r.progressBar === "boolean" ? r.progressBar : d.progressBar,
        playVideos: typeof r.playVideos === "boolean" ? r.playVideos : d.playVideos,
        cutscenePlayer: (CUTSCENE_PLAYERS as readonly string[]).includes(String(r.cutscenePlayer)) ? (r.cutscenePlayer as CutscenePlayer) : d.cutscenePlayer,
        nickname: typeof r.nickname === "string" ? resolveNickname(r.nickname) : d.nickname,
        style: (READING_STYLES as readonly string[]).includes(String(r.style)) ? (r.style as ReadingStyle) : d.style,
        // A settings object written before the bundle carries a style name and
        // no line height, and it must read back at the size the style RENDERED
        // at, not at the default: the values come from that style's bundle,
        // which is the same number the old render-time table used.
        lineHeight: clampNumber(r.lineHeight, 1.2, 2.2, storedBundle(r)?.lineHeight ?? d.lineHeight),
        letterSpacing: clampNumber(r.letterSpacing, 0, 0.12, storedBundle(r)?.letterSpacing ?? d.letterSpacing),
        font: (STORY_FONTS as readonly string[]).includes(String(r.font)) ? (r.font as StoryFont) : d.font,
        customFontName: typeof r.customFontName === "string" ? r.customFontName.slice(0, 120) : d.customFontName,
        textColor: typeof r.textColor === "string" && /^(s[0-7]|#[0-9a-fA-F]{3}|#[0-9a-fA-F]{6})$/.test(r.textColor) ? r.textColor : d.textColor,
        speakerTint: coerceSpeakerTint(r),
        // A settings object written before the two-axis position carries
        // `boxLift`, a 0..40 percentage, and nothing else; it reads back as the
        // same lift rather than as a reset to the bottom.
        boxY: clampNumber(r.boxY, 0, 1, r.boxY === undefined ? clampNumber(r.boxLift, 0, MAX_BOX_LIFT, 0) / MAX_BOX_LIFT : d.boxY),
        boxX: clampNumber(r.boxX, -1, 1, d.boxX),
        // THE `v: 1` -> `v: 2` MIGRATION, and it is a drop, not an inversion.
        // `alwaysShowToolbar` was OFF by default, so inverting it would hand
        // every existing reader an auto-hide they never chose; a stored object
        // without `autoHideToolbar` therefore lands on the new default and the
        // bar stays on screen. Only an explicit `autoHideToolbar` is read.
        autoHideToolbar: typeof r.autoHideToolbar === "boolean" ? r.autoHideToolbar : d.autoHideToolbar,
        toolbarIdleSec: clampNumber(r.toolbarIdleSec, 1, 15, d.toolbarIdleSec),
        toolbarHidden: typeof r.toolbarHidden === "boolean" ? r.toolbarHidden : d.toolbarHidden,
    };
    return { ...out, style: readingStyleOf(out) };
}

export function parseSettings(raw: string): StorySettings {
    try {
        return coerceSettings(JSON.parse(raw));
    } catch {
        return { ...DEFAULT_SETTINGS };
    }
}

/**
 * THE DOCUMENT HAS ONE READ PATH AND ONE WRITE PATH, and these are them.
 *
 * The reader holds the settings in React ({@link useStorySettings}) and the
 * library's music channel holds them in a module singleton, because a store
 * that survives the sheet closing cannot be a hook. Both go through the three
 * functions here, so the coercion, the JSON encoding and the SSR guard are
 * written once. The two never run at once: the reader and the library are
 * different routes, and each side reads the document fresh on mount.
 */
export function loadSettings(): StorySettings {
    return readStoredValue(SETTINGS_KEY, DEFAULT_SETTINGS, parseSettings);
}

export function saveSettings(next: StorySettings): void {
    writeStoredValue(SETTINGS_KEY, next);
}

/** Read, patch, write, and hand back what was written. The caller gets the whole document, never just its patch. */
export function updateSettings(patch: Partial<StorySettings>): StorySettings {
    const next = { ...loadSettings(), ...patch };
    saveSettings(next);
    return next;
}

/** The reader's copy of the document, persisted per browser. */
export function useStorySettings(): [StorySettings, (next: StorySettings) => void] {
    return useLocalStorageState<StorySettings>(SETTINGS_KEY, DEFAULT_SETTINGS, { parse: parseSettings });
}

/**
 * What a slider's `onValueChange` actually means. Base UI hands back an array,
 * and `v[0] ?? 0` is the coercion trap house rule 9 names: a MISSING value is
 * not zero, and for `musicVolume` zero is SILENCE that then persists. A value
 * that is not a finite number keeps `current` instead.
 */
export function sliderValue(v: number | readonly number[] | null | undefined, current: number): number {
    const n = Array.isArray(v) ? v[0] : (v as number | null | undefined);
    return typeof n === "number" && Number.isFinite(n) ? n : current;
}

/** Seconds auto-play waits after the reveal: `max(minLineSec, chars / cps * pace)`. */
export function autoPlayDelaySec(chars: number, s: StorySettings): number {
    return Math.max(s.minLineSec, (chars / s.cps) * s.autoPace);
}

/** The lift ceiling: 40% of the stage height at `boxY` 1. */
export const MAX_BOX_LIFT = 40;

/** The text box's place on the stage, both axes normalised. */
export interface BoxPosition {
    /** -1 flush against the left side margin, 0 centred, 1 flush right. */
    x: number;
    /** 0 the shipped bottom margin, 1 lifted `MAX_BOX_LIFT` percent of the stage height. */
    y: number;
}

/** Clamp a position to the box's own travel. A non-finite axis keeps the default, never NaN. */
export function clampBoxPosition(p: { x?: unknown; y?: unknown }): BoxPosition {
    return { x: clampNumber(p.x, -1, 1, 0), y: clampNumber(p.y, 0, 1, 0) };
}

export const BOX_PRESETS = ["bottomCentre", "bottomLeft", "bottomRight", "centre", "topCentre"] as const;
export type BoxPreset = (typeof BOX_PRESETS)[number];

/**
 * The five presets the settings pad offers. `bottomCentre` is the DEFAULT
 * position, which is why it is first and why it is the one Reset writes.
 */
export const BOX_PRESET_POSITIONS: Record<BoxPreset, BoxPosition> = {
    bottomCentre: { x: 0, y: 0 },
    bottomLeft: { x: -1, y: 0 },
    bottomRight: { x: 1, y: 0 },
    centre: { x: 0, y: 0.5 },
    topCentre: { x: 0, y: 1 },
};

/** The preset a position IS, or null when it sits between them. */
export function presetForPosition(p: BoxPosition): BoxPreset | null {
    for (const name of BOX_PRESETS) {
        const at = BOX_PRESET_POSITIONS[name];
        if (Math.abs(at.x - p.x) < 1e-9 && Math.abs(at.y - p.y) < 1e-9) return name;
    }
    return null;
}

/** One arrow-key nudge: 1% of each axis's own span, so 0.02 across x and 0.01 up y. */
export const BOX_NUDGE_X = 0.02;
export const BOX_NUDGE_Y = 0.01;

/**
 * Dragging the box. `travel` is the free room the box has in CSS pixels: `x`
 * the whole width it can slide across (the wrapper's content box minus the
 * box's own width, which the x span of 2 covers end to end) and `y` the whole
 * height the lift can climb (`MAX_BOX_LIFT` percent of the stage).
 *
 * `deltaY` is pointer-down minus pointer-now, so dragging UP is a NEGATIVE
 * delta and must RAISE the box; the sign is flipped here rather than at each
 * call site. A zero, negative or missing travel keeps THAT axis instead of
 * dividing by zero and persisting NaN, which is the coercion trap: a box the
 * text has filled has zero horizontal travel and must not jump.
 */
export function boxPositionFromDrag(start: BoxPosition, deltaX: number, deltaY: number, travel: { x: number; y: number }): BoxPosition {
    const movedX = Number.isFinite(travel.x) && travel.x > 0 && Number.isFinite(deltaX) ? (2 * deltaX) / travel.x : 0;
    const movedY = Number.isFinite(travel.y) && travel.y > 0 && Number.isFinite(deltaY) ? -deltaY / travel.y : 0;
    return clampBoxPosition({ x: start.x + movedX, y: start.y + movedY });
}
