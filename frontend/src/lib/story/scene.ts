/**
 * THE PICTURE THE ENGINE DRAWS, and the arithmetic that places it.
 *
 * `engine.ts` owns the command switch; this file owns the vocabulary that
 * switch writes into, so the interpreter reads top to bottom instead of opening
 * with three hundred lines of interface. Every field here was READ out of the
 * client binary and the shipped prefabs, never inferred from the corpus:
 * `docs/story-reader-il2cpp-scene.md`, `docs/story-reader-il2cpp-characters.md`
 * and `docs/story-reader-avg-prefab.md`. The measured number that justifies a
 * unit or a default stays attached to the field it justifies.
 *
 * Nothing here is stateful and nothing here imports the engine, which is what
 * lets the renderer (`components/story/reader/Stage.tsx`) take these types
 * without pulling the interpreter in behind them.
 */
import type { CharacterSprite } from "#/types/generated/CharacterSprite";
import type { StoryCommand } from "#/types/generated/StoryCommand";
import type { StoryScript } from "#/types/generated/StoryScript";
import { adaptOf, num } from "./args";
import { CANVAS_W } from "./canvas";
import { substitute } from "./text";

export type Slot = "l" | "m" | "r";

/**
 * A `[charslot]` action. `power` is DOTween's strength or jump power, `times`
 * its vibrato or number of jumps, and `randomness` its randomness, all read in
 * `docs/story-reader-il2cpp-characters.md`, 4. `seq` increments per command so
 * that re-issuing the same action replays it.
 */
export interface SlotAction {
    kind: "jump" | "shake" | "move" | "zoom";
    power: number;
    times: number;
    randomness: number;
    /** Scaled seconds the action runs. */
    sec: number;
    seq: number;
}

export interface SlotState {
    sprite: CharacterSprite;
    /** The raw script name (`avg_1037_amiya3_1#8$2`), the key into `assets.characters`. */
    name: string;
    lit: boolean;
    /** Slot-local absolute offsets in canvas pixels, from `posfrom`/`posto`; (0,0) is the slot's home. */
    x: number;
    y: number;
    /** 0..1 from `ato`; `afrom`/`ato` default to -1, which means UNCHANGED. */
    alpha: number;
    /** `poszoom` scale about a normalized pivot. */
    scale: number;
    /** `poszoom`, a 0..1 pivot; the command is dropped outside that range. */
    pivotX: number;
    pivotY: number;
    action?: SlotAction;
    /** Bumps when this slot takes a DIFFERENT sprite, which is a crossfade and a reset to home. */
    swap: number;
}

/**
 * `screenadapt`, read at `_LoadImage` (`docs/story-reader-il2cpp-scene.md`, 4).
 * Absent or unrecognised means NO adaptation: the sprite keeps its native size
 * in canvas pixels. EN writes only `coverall` (16,879) and `showall` (3,249),
 * which is every background and image command in the corpus.
 */
export type ScreenAdapt = "coverall" | "showall" | "fill" | "native";

export interface ImageLayer {
    name: string;
    url: string;
    /** Canvas pixels, centre origin, straight into `localPosition`. */
    x: number;
    y: number;
    xScale: number;
    yScale: number;
    adapt: ScreenAdapt;
    /** `width`/`height` are MULTIPLIERS on the adapted size, default 1.0, not pixels. */
    widthMul: number;
    heightMul: number;
    /** `imagerotate` angle in degrees. */
    rotate: number;
    /**
     * The `native` arm's own rect in CANVAS pixels, `texturePx * 100 / ppu`,
     * from `StoryAssets.imageSizes`. Absent means the wire carried no size for
     * this name and the renderer falls back to the prefab's 1280x720 reference
     * rect, which is the pre-capture reading (`docs/story-reader-captures.md`, 1).
     */
    nativeW?: number;
    nativeH?: number;
}

/**
 * `largebg` (138 uses) and `gridbg` (186) draw a background out of SEVERAL
 * panels: `imagegroup="bg_beach_1/bg_beach_2"` with `solidwidth="920/920"`,
 * scrolled by `x`. The panels butt left to right and `x` moves the CONTAINER
 * in the same canvas pixels as every other offset, so `solidwidth="920/920"`
 * is an 1840 px strip and `x=-720` pans it 720 px left.
 */
export interface PanelLayer {
    urls: string[];
    /** Panel widths in canvas pixels, one per url. */
    widths: number[];
    /** Panel height in canvas pixels. */
    height: number;
    /** 2 for a gridbg, 1 for a single-row largebg strip. */
    rows: number;
    x: number;
    y: number;
}

/** `charactercutin`: a sized sprite plate that fades alpha 0 to 1, with no slide. */
export interface Cutin {
    sprite: CharacterSprite;
    name: string;
    /** Canvas pixels; `Show` NEGATES both offsets before writing `anchoredPosition`. */
    x: number;
    y: number;
    /** Plate size in canvas pixels; `width=200` is every use that carries one. */
    width: number;
    height: number;
}

/**
 * `curtain`: a solid fill wiping across the stage. `fillfrom` and `fillto`
 * are the covered fraction, `direction` the edge it grows from. The colour is
 * NOT script-controlled: the executor reads `r`, `g`, `b` and their `from`
 * twins and discards all six.
 */
export interface Curtain {
    direction: number;
    fill: number;
    grad: boolean;
}

/** `interlude`: one masked panel per `channel`. */
export interface InterludePanel {
    url: string;
    name: string;
    /** Canvas-pixel offsets; `pfrom`/`pto` are "x,y" pairs. */
    x: number;
    y: number;
}

/** Blocker channels are 0..1, CLAMPED, never divided by 255. */
export interface Blocker {
    a: number;
    r: number;
    g: number;
    b: number;
}

export type OverlayAlign = "left" | "center" | "right";

/**
 * A subtitle or sticker. `x`/`y` are canvas pixels with the origin TOP-LEFT
 * and y DOWNWARD (`anchoredPosition = (x, -y)`), `width` defaults to 1280 and
 * is clamped to `1280 - x`, `size` is a raw point size defaulting to 24 at
 * canvas scale, and `alignment` is upper-left, -centre or -right.
 */
export interface Overlay {
    text: string;
    x: number;
    y: number;
    alignment: OverlayAlign;
    size: number;
    width: number;
}

/** `focusout` targets, folded from its 5 `type` values. It is a BLUR, not a darken. */
export interface FocusOut {
    /** `type=bg` 357 uses and `type=lbg` 8. */
    bg?: number;
    /** `type=cg` 78 uses and `type=cgitem` 19. */
    cg?: number;
    /** `type=char` 54 uses. */
    char?: number;
}

export interface SceneState {
    background?: ImageLayer;
    /** `largebg`/`gridbg`, drawn instead of `background` while it is set. */
    panels?: PanelLayer;
    image?: ImageLayer;
    blocker: Blocker;
    slots: Partial<Record<Slot, SlotState>>;
    /**
     * The text box. Only `dialog`, `multiline` and `aside` (our `name`, `text`
     * and `narration`) touch it, plus the reset that snaps it hidden: a reverse
     * scan of 2.57M call sites finds SIX setters of `set_isHidden` and no other
     * command is among them.
     */
    dialogVisible: boolean;
    subtitle?: Overlay;
    stickers: Record<string, Overlay>;
    effects: { grayscale?: number; colorInverse?: number };
    curtain?: Curtain;
    cutin?: Cutin;
    /** 0 is in focus, 1 fully blurred. */
    focus: FocusOut;
    /** `popupdialog`: the head that rides beside the next line. */
    popupHead?: { key: string; url: string; x?: number; y?: number };
    /** `interlude` panels, keyed by `channel`. */
    interludes: Record<string, InterludePanel>;
}

export interface Frame {
    state: SceneState;
    /** The tween's curve as a CSS timing function, from the DOTween `Ease` enum or a name. */
    ease?: string;
    /** Seconds the renderer transitions into this frame's state, already scaled by `animateRatio`. */
    transitionSec: number;
    /** Seconds the renderer holds after the transition, already scaled by `animateRatio`. */
    holdSec: number;
    /** False when the script marked the transition `block=false`: the player does not wait for it. */
    blocking: boolean;
}

export function initialState(): SceneState {
    return { blocker: { a: 0, r: 0, g: 0, b: 0 }, slots: {}, dialogVisible: false, stickers: {}, effects: {}, focus: {}, interludes: {} };
}

export function cloneState(s: SceneState): SceneState {
    return {
        background: s.background ? { ...s.background } : undefined,
        image: s.image ? { ...s.image } : undefined,
        blocker: { ...s.blocker },
        slots: Object.fromEntries(Object.entries(s.slots).map(([k, v]) => [k, { ...v, action: v.action ? { ...v.action } : undefined }])) as Partial<Record<Slot, SlotState>>,
        dialogVisible: s.dialogVisible,
        subtitle: s.subtitle ? { ...s.subtitle } : undefined,
        stickers: Object.fromEntries(Object.entries(s.stickers).map(([k, v]) => [k, { ...v }])),
        effects: { ...s.effects },
        panels: s.panels ? { ...s.panels, urls: [...s.panels.urls], widths: [...s.panels.widths] } : undefined,
        curtain: s.curtain ? { ...s.curtain } : undefined,
        cutin: s.cutin ? { ...s.cutin } : undefined,
        focus: { ...s.focus },
        popupHead: s.popupHead ? { ...s.popupHead } : undefined,
        interludes: Object.fromEntries(Object.entries(s.interludes).map(([k, v]) => [k, { ...v }])),
    };
}

/**
 * `StoryAssets.imageSizes`, read DEFENSIVELY: a backend older than the binding
 * sends none at all, and then every `native` layer keeps the pre-capture
 * 1280x720 rect. `w` and `h` are texture pixels and `ppu` the sprite's
 * pixels-per-unit; the canvas rect is `px * 100 / ppu`, which for `bg_cher_1`
 * (1024x576, ppu 68.25) is 1500.4 x 844.0 canvas px against the 1500.7 x 843.5
 * the game measured (`docs/story-reader-captures.md`, 1).
 */
export interface ImageNativeSize {
    w: number;
    h: number;
}

export function nativeSizeOf(assets: StoryScript["assets"], name: string): ImageNativeSize | null {
    const map = (assets as { imageSizes?: unknown }).imageSizes;
    if (!map || typeof map !== "object") return null;
    const raw = (map as Record<string, unknown>)[name] as { w?: unknown; h?: unknown; ppu?: unknown } | undefined;
    if (!raw || typeof raw !== "object") return null;
    const n = (v: unknown): number => (typeof v === "number" && Number.isFinite(v) ? v : Number.NaN);
    const w = n(raw.w);
    const h = n(raw.h);
    const ppu = n(raw.ppu);
    if (!(w > 0) || !(h > 0) || !(ppu > 0)) return null;
    return { w: (w * 100) / ppu, h: (h * 100) / ppu };
}

export function layerFrom(args: StoryCommand["args"], name: string, url: string, native: ImageNativeSize | null, previous?: ImageLayer): ImageLayer {
    return {
        name,
        url,
        x: num(args.x, 0),
        y: num(args.y, 0),
        xScale: num(args.xscale, 1),
        yScale: num(args.yscale, 1),
        adapt: adaptOf(args.screenadapt),
        widthMul: num(args.width, 1),
        heightMul: num(args.height, 1),
        rotate: previous?.rotate ?? 0,
        ...(native ? { nativeW: native.w, nativeH: native.h } : {}),
    };
}

function alignOf(value: string | undefined): OverlayAlign {
    const v = (value ?? "").trim().toLowerCase();
    return v === "center" || v === "centre" || v === "1" ? "center" : v === "right" || v === "2" ? "right" : "left";
}

/**
 * Subtitle and sticker geometry, read at `_ExecuteSubtitle`: origin top-left
 * with y downward, `width` default 1280.0 clamped to `1280 - x`, `size` a raw
 * point size default 24, alignment a three-way compare. `delay` is NOT read by
 * the executor, so there is no per-character reveal on these surfaces.
 */
export function overlayFrom(args: StoryCommand["args"], text: string, nickname: string): Overlay {
    const x = num(args.x, 0);
    const y = num(args.y, 0);
    return { text: substitute(text, nickname), x, y, width: Math.min(num(args.width, CANVAS_W), CANVAS_W - x), size: num(args.size, 24), alignment: alignOf(args.alignment) };
}
