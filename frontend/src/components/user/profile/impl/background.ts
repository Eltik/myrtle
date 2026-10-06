import type { CSSProperties } from "react";
import { asset, skinpackFile } from "#/components/operators/detail/impl/assets";
import { env } from "#/env";
import { gamedataPath } from "#/lib/api/gamedata";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import type { ProfileBackgroundKind } from "#/types/generated/ProfileBackgroundKind";
import type { StoryArtKind } from "#/types/generated/StoryArtKind";

/** The kinds the picker's character-art tab offers through the tier-list entity picker, in its tab order. The backend's `ProfileBackgroundKind` is the allow-list; the gallery kinds ({@link GALLERY_KINDS}) have a tab of their own. */
export const BACKGROUND_KINDS = ["skin", "operator"] as const satisfies readonly ProfileBackgroundKind[];

/**
 * The kinds the Gallery tab offers, one source pill each: the Archives gallery, the story
 * scripts' CGs and their scene plates. All three are landscape pictures served as the
 * backend's JPEGs, start centred and offer both crop axes.
 */
export const GALLERY_KINDS = ["archive_pic", "story_cg", "story_scene"] as const satisfies readonly ProfileBackgroundKind[];
export type GalleryKind = (typeof GALLERY_KINDS)[number];

/** Whether a background kind is one of the Gallery tab's landscape pictures. */
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

/** The background zoom's range, the backend's `BACKGROUND_SCALE_MIN..=MAX`: 100 is `cover`, 300 three times it. */
export const SCALE_MIN = 100;
export const SCALE_MAX = 300;

/** The story-art route's kind for each story Gallery kind. */
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
export function backgroundSources(background: Pick<ProfileBackground, "kind" | "id">): string[] {
    // The `/cn` form tries CN and then the default server, so it covers a picture only CN
    // lists. A failed render answers with the original PNG, so there is no third URL to try.
    const { kind, id } = background;
    if (kind === "archive_pic") return ART_SERVERS.map((server) => galleryPictureUrl(id, "header", server));
    if (kind === "story_cg" || kind === "story_scene") return ART_SERVERS.map((server) => storyArtUrl(STORY_ART_KIND[kind], id, "header", server));
    const files = artFiles(background);
    return ART_SERVERS.flatMap((server) => files.map((file) => asset(file, server)));
}

/** The asset paths of a background's art, best first, on any one server. */
function artFiles({ kind, id }: Pick<ProfileBackground, "kind" | "id">): string[] {
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
    return [`${base}_2b.png`, `${base}_2.png`, `${base}_1b.png`, `${base}_1.png`];
}

/**
 * A gallery picture as the backend's smaller JPEG (`GET /story/gallery/{id}/{size}`):
 * `thumb` is 320 px wide for a picker tile, `header` the full 1600 px. On EN 2026-10-06
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
export function storyArtUrl(kind: StoryArtKind, id: string, size: "thumb" | "header", server?: string): string {
    return `${env.VITE_BACKEND_URL}/api${gamedataPath(server, `/story/art-gallery/${kind}/${encodeURIComponent(id)}/${size}`)}`;
}

/** The 320 px tile of any Gallery-tab picture. */
export function galleryThumbUrl(kind: GalleryKind, id: string): string {
    if (kind === "archive_pic") return galleryPictureUrl(id, "thumb");
    return storyArtUrl(STORY_ART_KIND[kind], id, "thumb");
}

/** The CSS `object-position` for a background: its focus, the centre where it has none. */
export function objectPosition(background: Pick<ProfileBackground, "focus_x" | "focus_y">): string {
    const x = background.focus_x ?? 50;
    const y = background.focus_y ?? 50;
    return `${clampFocus(x)}% ${clampFocus(y)}%`;
}

/** Whether two backgrounds draw the same picture at the same crop and zoom; `null` is no background. */
export function sameBackground(a: ProfileBackground | null | undefined, b: ProfileBackground | null | undefined): boolean {
    if (!a || !b) return !a && !b;
    return a.kind === b.kind && a.id === b.id && (a.focus_x ?? 50) === (b.focus_x ?? 50) && (a.focus_y ?? 50) === (b.focus_y ?? 50) && (a.scale ?? SCALE_MIN) === (b.scale ?? SCALE_MIN);
}

/**
 * The background a pick in the picker makes. Picking the art already shown keeps its
 * crop; a new art starts at {@link DEFAULT_FOCUS_Y}.
 */
export function pickBackground(previous: ProfileBackground | null, kind: ProfileBackgroundKind, id: string): ProfileBackground {
    if (previous && previous.kind === kind && previous.id === id) return previous;
    return isGalleryKind(kind) ? { kind, id } : { kind, id, focus_y: DEFAULT_FOCUS_Y };
}

/**
 * Which crop sliders a background offers. The portrait kinds keep the one height
 * slider they had. A gallery picture is 16:9, and the slot's aspect runs from
 * narrower than that (a phone, where the art fills the header above tall text) to
 * well over 2:1 (the right 62% of a desktop header), so `cover` crops its width on
 * one and its height on the other: both are offered. Zoomed in past 100, any art
 * overflows the slot on both axes, so a portrait gets the horizontal slider too.
 */
export function cropAxes(kind: ProfileBackgroundKind, scale?: number): readonly ("x" | "y")[] {
    return isGalleryKind(kind) || zoomOf({ scale }) > SCALE_MIN ? ["x", "y"] : ["y"];
}

/** `background` with its horizontal crop moved to `x`. */
export function withFocusX(background: ProfileBackground, x: number): ProfileBackground {
    return { ...background, focus_x: clampFocus(x) };
}

/** `background` with its vertical crop moved to `y`. */
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

/** `background` zoomed to `scale`. 100 drops the key, so an unzoomed background keeps the shape it had before zoom existed. */
export function withScale(background: ProfileBackground, scale: number): ProfileBackground {
    const { scale: _drop, ...rest } = background;
    const next = clampScale(scale);
    return next === SCALE_MIN ? rest : { ...rest, scale: next };
}

/**
 * The inline style of the header art. Unzoomed (no scale, or 100) it is the
 * `object-position` alone, exactly as before zoom existed: the kill switch.
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
 * header's box on that axis and moving its focus moves nothing. The pan and the crop
 * sliders read the same rule, so a slider is enabled exactly when a drag along it moves.
 */
export const MIN_SLACK_PX = 0.5;

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
        return clampFocus((focus ?? 50) - (delta / overflow) * 100);
    };
    const x = axis(start.focus_x, dx, slack.x);
    const y = axis(start.focus_y, dy, slack.y);
    const next: ProfileBackground = { ...start };
    if (x !== undefined) next.focus_x = x;
    if (y !== undefined) next.focus_y = y;
    return next;
}
