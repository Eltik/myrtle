import type { CSSProperties } from "react";
import { asset, skinpackFile } from "#/components/operators/detail/impl/assets";
import { env } from "#/env";
import { gamedataPath } from "#/lib/api/gamedata";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import type { ProfileBackgroundKind } from "#/types/generated/ProfileBackgroundKind";
import type { StoryArtKind } from "#/types/generated/StoryArtKind";

/** The kinds the editor's Character art source offers through the tier-list entity picker, in its tab order. The backend's `ProfileBackgroundKind` is the allow-list; the gallery kinds ({@link GALLERY_KINDS}) are sources of their own. */
export const BACKGROUND_KINDS = ["skin", "operator"] as const satisfies readonly ProfileBackgroundKind[];

/**
 * The kinds the art browser's gallery offers, one source each: the Archives gallery, the
 * story scripts' CGs and their scene plates. All three are landscape pictures served as the
 * backend's JPEGs, start centred and offer both crop axes.
 */
export const GALLERY_KINDS = ["archive_pic", "story_cg", "story_scene"] as const satisfies readonly ProfileBackgroundKind[];
export type GalleryKind = (typeof GALLERY_KINDS)[number];

export function isGalleryKind(kind: ProfileBackgroundKind): kind is GalleryKind {
    return (GALLERY_KINDS as readonly ProfileBackgroundKind[]).includes(kind);
}

/**
 * Where the crop of a freshly picked art starts, as a percentage of its height. Both
 * entity kinds are square portraits with the head in the upper third, so the centre (the
 * backend's reading of a missing focus) shows a torso; 25 lands near the face. A gallery
 * picture is a 16:9 scene with no such subject and starts at the centre.
 */
export const DEFAULT_FOCUS_Y = 25;

/** The focus a background without one is drawn at, on either axis: the centre, the backend's reading of a missing focus. */
const FOCUS_CENTRE = 50;

/** The background zoom's range, the backend's `BACKGROUND_SCALE_MIN..=MAX`: 100 is `cover`, 300 three times it. */
export const SCALE_MIN = 100;
export const SCALE_MAX = 300;

const STORY_ART_KIND = { story_cg: "cg", story_scene: "scene" } as const satisfies Record<Exclude<GalleryKind, "archive_pic">, StoryArtKind>;

/** The servers an art file is looked for on: the default one, then CN, which also has every operator the default has not released yet. */
const ART_SERVERS = [undefined, "cn"] as const;

/** `n` rounded and clamped to 0..100, the backend's normalization. */
export function clampFocus(n: number): number {
    return Math.min(100, Math.max(0, Math.round(n)));
}

/**
 * The URLs to try for a background, best first. The header tries each in turn and
 * falls back to its plain look when none loads.
 *
 * The REDUCED variant leads: the game ships `<file>b.png` at 1024 px beside the full
 * art for 469 of 474 outfits and 323 of 381 elite 2 arts on EN (skin `b` mean 1.26 MB
 * against 4.77 MB full; elite 2 `b` mean 1.25 MB against 4.68 MB). The header draws the
 * art at most about 1000 px wide, so the full file buys nothing it can show. An
 * operator's art is elite 2 where they have one, else elite 1 (three-star and lower).
 */
export function backgroundSources(background: Pick<ProfileBackground, "kind" | "id" | "elite">): string[] {
    // The `/cn` form tries CN and then the default server, so it covers a picture only CN
    // lists. A failed render answers with the original PNG, so there is no third URL to try.
    const { kind, id } = background;
    if (isGalleryKind(kind)) return ART_SERVERS.map((server) => galleryArtUrl(kind, id, "header", server));
    const files = artFiles(background);
    return ART_SERVERS.flatMap((server) => files.map((file) => asset(file, server)));
}

/** The asset paths of a background's art, best first, on any one server. An operator chosen at elite 1 draws that art alone; otherwise elite 2, falling back to elite 1. */
function artFiles({ kind, id, elite }: Pick<ProfileBackground, "kind" | "id" | "elite">): string[] {
    if (kind === "skin") {
        // `char_002_amiya@epoque#4` lives at `skinpack/char_002_amiya/char_002_amiya_epoque%234.png`.
        // The folder is the id's own prefix, which is not always the wearer: Amiya's
        // guard and medic outfits file under `char_1001_amiya2` and `char_1037_amiya3`.
        const at = id.indexOf("@");
        if (at <= 0) return [];
        const folder = id.slice(0, at);
        const file = skinpackFile(id);
        return [`/textures/skinpack/${folder}/${file}b.png`, `/textures/skinpack/${folder}/${file}.png`];
    }
    const base = `/textures/chararts/${id}/${id}`;
    const e1 = [`${base}_1b.png`, `${base}_1.png`];
    return elite === 1 ? e1 : [`${base}_2b.png`, `${base}_2.png`, ...e1];
}

/**
 * A gallery picture as the backend's smaller JPEG (`GET /story/gallery/{id}/{size}`):
 * `thumb` is 320 px wide for a browser tile, `header` the full 1600 px. On EN 2026-10-06
 * the PNGs average 2,049,755 bytes; see `gallery.rs` for what the two variants weigh.
 */
export function galleryPictureUrl(id: string, size: "thumb" | "header", server?: string): string {
    return `${env.VITE_BACKEND_URL}/api${gamedataPath(server, `/story/gallery/${encodeURIComponent(id)}/${size}`)}`;
}

/**
 * A story CG or scene plate as the backend's smaller JPEG
 * (`GET /story/art-gallery/{kind}/{id}/{size}`): `thumb` is 320 px wide, `header` 1600 px
 * or the source's width when narrower (every scene plate is 1024x576). On a 41-picture EN
 * sample a CG thumb is 14,614 bytes and its header 192,742 against the PNG's 1,616,954; a
 * scene 13,020 and 90,472 against 782,336.
 */
function storyArtUrl(kind: StoryArtKind, id: string, size: "thumb" | "header", server?: string): string {
    return `${env.VITE_BACKEND_URL}/api${gamedataPath(server, `/story/art-gallery/${kind}/${encodeURIComponent(id)}/${size}`)}`;
}

function galleryArtUrl(kind: GalleryKind, id: string, size: "thumb" | "header", server?: string): string {
    if (kind === "archive_pic") return galleryPictureUrl(id, size, server);
    return storyArtUrl(STORY_ART_KIND[kind], id, size, server);
}

/** The 320 px tile of any gallery picture. */
export function galleryThumbUrl(kind: GalleryKind, id: string): string {
    return galleryArtUrl(kind, id, "thumb");
}

/** The CSS `object-position` for a background: its focus, the centre where it has none. */
export function objectPosition(background: Pick<ProfileBackground, "focus_x" | "focus_y">): string {
    const x = background.focus_x ?? FOCUS_CENTRE;
    const y = background.focus_y ?? FOCUS_CENTRE;
    return `${clampFocus(x)}% ${clampFocus(y)}%`;
}

/** Whether two backgrounds draw the same picture at the same crop, zoom and elite art; `null` is no background. */
export function sameBackground(a: ProfileBackground | null | undefined, b: ProfileBackground | null | undefined): boolean {
    if (!a || !b) return !a && !b;
    return a.kind === b.kind && a.id === b.id && (a.focus_x ?? FOCUS_CENTRE) === (b.focus_x ?? FOCUS_CENTRE) && (a.focus_y ?? FOCUS_CENTRE) === (b.focus_y ?? FOCUS_CENTRE) && (a.scale ?? SCALE_MIN) === (b.scale ?? SCALE_MIN) && (a.elite ?? null) === (b.elite ?? null);
}

/**
 * The background a pick in the art browser makes. Picking the art already shown keeps its
 * crop; a new art starts at {@link DEFAULT_FOCUS_Y}.
 */
export function pickBackground(previous: ProfileBackground | null, kind: ProfileBackgroundKind, id: string): ProfileBackground {
    if (previous && previous.kind === kind && previous.id === id) return previous;
    return isGalleryKind(kind) ? { kind, id } : { kind, id, focus_y: DEFAULT_FOCUS_Y };
}

/**
 * Which crop axes a background is guessed to have from its kind alone, before its art is
 * measured (see {@link croppableAxes}). A portrait crops by height. A gallery picture is
 * 16:9, and the slot's aspect runs from narrower than that (a phone, where the art fills
 * the header above tall text) to well over 2:1 (the right 62% of a desktop header), so
 * `cover` crops its width on one and its height on the other: both are offered. Zoomed in
 * past 100, any art overflows the slot on both axes, so a portrait gets the horizontal
 * axis too.
 */
export function cropAxes(kind: ProfileBackgroundKind, scale?: number): readonly ("x" | "y")[] {
    return isGalleryKind(kind) || zoomOf({ scale }) > SCALE_MIN ? ["x", "y"] : ["y"];
}

export function withFocusX(background: ProfileBackground, x: number): ProfileBackground {
    return { ...background, focus_x: clampFocus(x) };
}

export function withFocusY(background: ProfileBackground, y: number): ProfileBackground {
    return { ...background, focus_y: clampFocus(y) };
}

/** `n` rounded and clamped to {@link SCALE_MIN}..{@link SCALE_MAX}, the backend's normalization. */
export function clampScale(n: number): number {
    return Math.min(SCALE_MAX, Math.max(SCALE_MIN, Math.round(n)));
}

/** A background's zoom: its scale clamped, 100 where it has none. A missing scale is checked explicitly, never as a falsy one. */
export function zoomOf(background: Pick<ProfileBackground, "scale">): number {
    return background.scale === undefined || background.scale === null || !Number.isFinite(background.scale) ? SCALE_MIN : clampScale(background.scale);
}

/** `background` zoomed to `scale`. 100 drops the key, so an unzoomed background stores no `scale`. */
export function withScale(background: ProfileBackground, scale: number): ProfileBackground {
    const { scale: _drop, ...rest } = background;
    const next = clampScale(scale);
    return next === SCALE_MIN ? rest : { ...rest, scale: next };
}

/**
 * The inline style of the header art. Unzoomed (no scale, or 100) it is the
 * `object-position` alone: the kill switch.
 *
 * Zoomed, the `cover` box is scaled by `s = scale / 100` about the focus point,
 * the same percentages as `object-position`. That point of the art stays put
 * (object-position puts the art's p% point on the box's p% point; the transform
 * origin is the box's p% point), so the zoom is about the focus. It always COVERS:
 * a box `[0, W]` scaled by `s >= 1` about `O` in `[0, W]` maps to
 * `[O(1 - s), W + (W - O)(s - 1)]`, which contains `[0, W]` for every focus.
 *
 * From `sm` up the art carries a fade mask on its own box, which the transform
 * would scale with it. The mask is shrunk to `1/s` of the box and placed at the
 * focus percentages, so that after the transform it lands exactly on the
 * unscaled box: the local mask origin `o W (1 - 1/s)` maps to
 * `O + s(o W (1 - 1/s) - O) = 0` with `O = o W`. `no-repeat` hides the zoomed
 * art outside that box, so it never spills under the header's text.
 */
export function artStyle(background: Pick<ProfileBackground, "focus_x" | "focus_y" | "scale">): CSSProperties {
    const position = objectPosition(background);
    const zoom = zoomOf(background);
    if (zoom === SCALE_MIN) return { objectPosition: position };
    const s = zoom / 100;
    const maskSize = `${100 / s}% ${100 / s}%`;
    return {
        objectPosition: position,
        transform: `scale(${s})`,
        transformOrigin: position,
        maskSize,
        WebkitMaskSize: maskSize,
        maskPosition: position,
        WebkitMaskPosition: position,
        maskRepeat: "no-repeat",
        WebkitMaskRepeat: "no-repeat",
    };
}

/** What a pan needs to know about the art: its drawn box before any transform, and the file's own size. */
export interface IArtGeometry {
    boxWidth: number;
    boxHeight: number;
    naturalWidth: number;
    naturalHeight: number;
}

/**
 * The least overflow, in CSS px, that makes an axis croppable: under it the art fits the
 * header's box on that axis and moving its focus moves nothing. The drag pan and the
 * keyboard pan read the same rule, so an arrow key moves exactly when a drag along it does.
 */
const MIN_SLACK_PX = 0.5;

/**
 * How far the drawn art overflows its box on each axis at `scale`, in CSS px: the travel
 * the focus spans from 0 to 100. With `cover` the art draws at its size times the larger
 * of the two box/art ratios, so at scale 100 one axis is exactly the box (0 slack) and the
 * other overflows; zoom `s` multiplies both. `null` before the art has a size.
 *
 * Measured 2026-10-06 at a 1500 x 753 window: the header's art box is 892 x 217 (4.11:1),
 * a 1600 x 900 story CG covers it at 0.5575 as 892 x 501.75, so x slack is 0 and y slack
 * 284.75 px. That is why the horizontal crop of a story CG "did nothing" there: every
 * focus_x drew the same pixels. Scale 101 gives 8.92 px of x slack.
 */
export function artSlack(geometry: IArtGeometry, scale: number | undefined): { x: number; y: number } | null {
    const { boxWidth, boxHeight, naturalWidth, naturalHeight } = geometry;
    if (!(boxWidth > 0 && boxHeight > 0 && naturalWidth > 0 && naturalHeight > 0)) return null;
    const cover = Math.max(boxWidth / naturalWidth, boxHeight / naturalHeight);
    const s = zoomOf({ scale }) / 100;
    return { x: s * naturalWidth * cover - boxWidth, y: s * naturalHeight * cover - boxHeight };
}

/**
 * Which crop axes move the art in the header as it is drawn now. With the art's geometry
 * an axis is croppable when it has {@link MIN_SLACK_PX} of slack at the current zoom, the
 * same rule for every kind; without it (not loaded yet, or no header art) it falls back to
 * {@link cropAxes}, the kind-based guess.
 */
export function croppableAxes(kind: ProfileBackgroundKind, scale: number | undefined, geometry: IArtGeometry | null): { x: boolean; y: boolean } {
    const slack = geometry ? artSlack(geometry, scale) : null;
    if (slack) return { x: slack.x >= MIN_SLACK_PX, y: slack.y >= MIN_SLACK_PX };
    return { x: cropAxes(kind, scale).includes("x"), y: true };
}

/**
 * The background after dragging its art by `(dx, dy)` CSS pixels from `start`,
 * so the art follows the pointer. With `cover` the art draws at `R` (its size
 * times the larger of the two box/art ratios) and the zoom scales that to `sR`;
 * the art's left edge sits at `p (B - sR)` for focus fraction `p` (see
 * {@link artStyle}), so one pixel of drag is `1 / (sR - B)` of focus. An axis the
 * art does not overflow (`sR - B` under half a pixel) does not move. Computed
 * from the drag's start, so rounding never accumulates.
 */
export function panFocus(start: ProfileBackground, dx: number, dy: number, geometry: IArtGeometry): ProfileBackground {
    const slack = artSlack(geometry, start.scale);
    if (!slack) return start;
    const axis = (focus: number | undefined, delta: number, overflow: number): number | undefined => {
        if (overflow < MIN_SLACK_PX || delta === 0) return focus;
        return clampFocus((focus ?? FOCUS_CENTRE) - (delta / overflow) * 100);
    };
    const x = axis(start.focus_x, dx, slack.x);
    const y = axis(start.focus_y, dy, slack.y);
    const next: ProfileBackground = { ...start };
    if (x !== undefined) next.focus_x = x;
    if (y !== undefined) next.focus_y = y;
    return next;
}

/**
 * `background` back to how a fresh pick of the same art starts: the kind's default focus
 * (the centre for a gallery picture, {@link DEFAULT_FOCUS_Y} for a portrait) and no zoom.
 */
export function resetBackground(background: ProfileBackground): ProfileBackground {
    const fresh = pickBackground(null, background.kind, background.id);
    return background.elite === undefined ? fresh : { ...fresh, elite: background.elite };
}

/**
 * The elite art an operator background shows: its stored choice, else elite 2 where the
 * operator has it (`hasElite2`) and elite 1 where it does not. `null` for any other kind.
 */
export function shownElite(background: Pick<ProfileBackground, "kind" | "elite">, hasElite2: boolean): 1 | 2 | null {
    if (background.kind !== "operator") return null;
    if (background.elite === 1) return 1;
    return hasElite2 ? 2 : 1;
}

/**
 * `background` drawing elite `elite` art. The key is stored only when it changes what
 * draws: elite 1 of an operator that has elite 2. Elite 2, or elite 1 of an operator
 * without it, is the default art and drops the key. The crop and zoom are kept.
 */
export function withElite(background: ProfileBackground, elite: 1 | 2, hasElite2: boolean): ProfileBackground {
    const { elite: _drop, ...rest } = background;
    if (background.kind !== "operator") return rest;
    return elite === 1 && hasElite2 ? { ...rest, elite: 1 } : rest;
}

/** One arrow-key pan step, in focus points; Shift takes {@link PAN_STEP_LARGE}. */
const PAN_STEP = 1;
export const PAN_STEP_LARGE = 10;
/** One +/- zoom step, in scale points. */
export const ZOOM_STEP = 10;

const PAN_KEYS: Record<string, [number, number]> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };

/** What a key does to the background: the moved background, or the dead axis it tried to move along (`dead`), or `null` for a key it does not handle. */
export type KeyAdjust = { next: ProfileBackground; dead: null } | { next: null; dead: "x" | "y" } | null;

/**
 * The keyboard's reading of the drag. An arrow moves the PICTURE that way, as a drag
 * does (the art follows the pointer), so ArrowRight lowers `focus_x`: 1 point a press,
 * 10 with Shift. An axis `axes` marks dead moves nothing and answers `dead` instead, so
 * the caller can say why. `+` (or `=`, its unshifted key) and `-` (or `_`) zoom by
 * {@link ZOOM_STEP} about the focus.
 */
export function keyAdjust(background: ProfileBackground, key: string, shift: boolean, axes: { x: boolean; y: boolean }): KeyAdjust {
    if (key === "+" || key === "=") return { next: withScale(background, zoomOf(background) + ZOOM_STEP), dead: null };
    if (key === "-" || key === "_") return { next: withScale(background, zoomOf(background) - ZOOM_STEP), dead: null };
    const arrow = PAN_KEYS[key];
    if (!arrow) return null;
    const step = shift ? PAN_STEP_LARGE : PAN_STEP;
    const [dx, dy] = arrow;
    if (dx !== 0) return axes.x ? { next: withFocusX(background, (background.focus_x ?? FOCUS_CENTRE) - dx * step), dead: null } : { next: null, dead: "x" };
    return axes.y ? { next: withFocusY(background, (background.focus_y ?? FOCUS_CENTRE) - dy * step), dead: null } : { next: null, dead: "y" };
}

/** How far a drag must travel along a dead axis, in screen px, before the editor says why nothing moves. */
export const DEAD_DRAG_PX = 12;

/**
 * The axis a drag of `(dx, dy)` is mostly along when the art has no slack there, so
 * the drag moves nothing that way; `null` while the drag is short or along a live axis.
 */
export function deadDragAxis(dx: number, dy: number, axes: { x: boolean; y: boolean }): "x" | "y" | null {
    const ax = Math.abs(dx);
    const ay = Math.abs(dy);
    if (Math.max(ax, ay) < DEAD_DRAG_PX) return null;
    if (ax >= ay) return axes.x ? null : "x";
    return axes.y ? null : "y";
}

/**
 * The zoom after one wheel event: one 100 px notch is about 15 points of scale. A
 * trackpad pinch arrives as a ctrl-wheel with small deltas, so it is scaled up. A
 * line-mode delta (`deltaMode` 1) counts 16 px a line.
 */
export function wheelScale(scale: number, deltaY: number, deltaMode: number, ctrlKey: boolean): number {
    const pixels = deltaMode === 1 ? deltaY * 16 : deltaY;
    return scale * Math.exp(-pixels * (ctrlKey ? 0.01 : 0.0015));
}

/**
 * Whether the editor's draft differs from what is saved: a new picture, crop or zoom, or
 * a removal. Read by `sameBackground`, so a draft dragged away and back, or zoomed to 100
 * on an art saved with no scale, is clean again and Cancel closes without asking.
 */
export function draftDirty(saved: ProfileBackground | null, draft: ProfileBackground | null): boolean {
    return !sameBackground(saved, draft);
}
