/**
 * The AVG canvas, as the client actually draws it.
 *
 * Measured 2026-09-23 against EN 36.7.22 on a 2340x1080 screen
 * (`docs/story-reader-captures.md`, 0): `[HEADER(... fit_mode="BLACK_MASK")]`
 * is not decoration, it IS the canvas. The whole scene is drawn into a centred
 * 1920x1080 box with 210 px of pure black each side, non-black columns exactly
 * 210 through 2129, and two independent character measurements pin the scale
 * at 1.5 device px per canvas px, which is 1280 canvas px across that 1920.
 * Every `HEADER` in the EN corpus carries the mask: 2,497 `BLACK_MASK` plus 4
 * lowercase `black_mask`, zero other values, over 3,189 files with a `HEADER`.
 *
 * So the canvas is ALWAYS 1280x720 and the box it fills is the largest 16:9
 * rectangle that fits the stage, centred. The reference resolution and
 * `MatchWidthOrHeight` 0.0 in `docs/story-reader-avg-prefab.md` are still what
 * the CanvasScaler ships; they match on the width of the MASK, not of the
 * screen, so the "extra height split top and bottom" case never arises.
 *
 * Every script coordinate is a pixel in this space with the origin at the
 * canvas CENTRE and y UP, except the subtitle and sticker pair, which the
 * executor anchors top-left with y downward
 * (`docs/story-reader-il2cpp-scene.md`, 7).
 *
 * `--story-cpx` is one canvas pixel as a CSS length: 1280 canvas px is the
 * BOX's width, so one canvas px is `min(width/1280, height*16/9/1280)` against
 * a stage that declares `container-type: size`. Using container units rather
 * than a measured scale keeps the mapping exact at every viewport with no JS.
 */

export const CANVAS_W = 1280;
export const CANVAS_H = 720;

/**
 * One canvas pixel against the stage container, as a fallback for the property
 * the stage writes: the largest 16:9 box divided by 1280. `100/1280` is
 * 0.078125cqw on the width side and `1600/(9*1280)` is 0.1388889cqh on the
 * height side.
 */
export const CPX_BOX = "min(0.078125cqw, 0.1388889cqh)";

/**
 * `?canvas=stretch`: the pre-capture mapping, where the canvas width was the
 * whole STAGE width at every aspect. Kept as a kill switch so the earlier
 * sprite and layer rects can be reproduced side by side.
 */
export const CPX_STRETCH = "0.078125cqw";

/** One canvas pixel in CSS, against the stage container. */
export const CPX = `var(--story-cpx, ${CPX_BOX})`;

/** `n` canvas pixels as a CSS length. */
export function cpx(n: number): string {
    return `calc(${n} * ${CPX})`;
}

/**
 * Character slots, from the prefab: `slot_left` anchoredPosition (-200,0),
 * `slot_middle` (0,0), `slot_right` (+200,0), all anchored (0.5,0), that is at
 * the canvas BOTTOM.
 */
export const SLOT_X: Record<"l" | "m" | "r", number> = { l: -200, m: 0, r: 200 };

/**
 * The body plate: the rect a character's sprite is drawn into, in canvas
 * pixels, centred `x`,`y` above the SLOT origin at the canvas bottom with y up.
 *
 * The prefab's own template is 1024 square at (0,203), but the template is NOT
 * what the client draws. Measured in one frame (`docs/story-reader-captures.md`,
 * 2 and 3): Amiya's plate is 1091.7 canvas px against her bundle's own root
 * RectTransform of 1090 (0.16%), Dobermann's is 958.9 and `avg_npc_935_1`'s is
 * 1094.5, three different sizes no shared template can produce. The character's
 * OWN prefab rect wins, and it rides the wire as `CharacterSprite.plate`.
 */
export interface BodyPlate {
    x: number;
    y: number;
    w: number;
    h: number;
}

/** The slot template, used when the wire carries no `plate` for a sprite. */
export const BODY_PLATE: BodyPlate = { x: 0, y: 203, w: 1024, h: 1024 };

/** The template plate, canvas px. */
export const BODY_PX = BODY_PLATE.w;
/** The template body centre above the slot origin, canvas px. */
export const BODY_RISE = BODY_PLATE.y;
/** How far the template body's bottom edge falls below the canvas bottom: 1024/2 - 203. */
export const BODY_DROP = BODY_PX / 2 - BODY_RISE;

/**
 * The legacy `[character]` path places by slot INDEX. MEASURED
 * (`docs/story-reader-captures.md`, 2): with `name="char_002_amiya_1#7",
 * name2="char_130_doberm_ex"`, Amiya's ink centre is canvas x -175.3 and
 * Dobermann's +228.3, either side of the slot pair -200 and +200, so the FIRST
 * name is the LEFT figure and `name2` the right. The reader shipped them
 * mirrored, off the `_GenPosition` read in
 * `docs/story-reader-il2cpp-characters.md`, 2.
 */
export const LEGACY_SLOT: Record<1 | 2 | 3, "l" | "m" | "r"> = { 1: "l", 2: "r", 3: "m" };

/**
 * The game's own text box, from the prefab: the scroll view runs the full
 * canvas width and stops 47.152 canvas px above the bottom, its content is
 * inset 60, the name column is 237.761 wide and the message column starts at
 * x 283.200 and runs 924.598 wide. Our reader keeps its own styled box; these
 * are what its DEFAULT margins and max width map to.
 */
export const BOX_BOTTOM = 47.152099609375;
export const BOX_INSET = 60;
export const BOX_MESSAGE_X = 283.20001220703125;
export const BOX_MESSAGE_W = 924.597900390625;
export const BOX_NAME_W = 237.76119995117188;
export const BOX_LINE_H = 56.737701416015625;

/**
 * The two margin sliders are percentages that land on `padding`, and a
 * percentage padding resolves against the containing block's WIDTH, so BOTH
 * defaults are canvas x fractions: the box's 60 px content inset and the
 * 47.152 px the scroll view stops above the canvas bottom.
 */
export const DEFAULT_SIDE_MARGIN = Number(((BOX_INSET / CANVAS_W) * 100).toFixed(4));
export const DEFAULT_BOTTOM_MARGIN = Number(((BOX_BOTTOM / CANVAS_W) * 100).toFixed(4));
